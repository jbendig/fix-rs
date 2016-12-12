// Copyright 2016 James Bendig. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under:
//   the MIT license
//     <LICENSE-MIT or https://opensource.org/licenses/MIT>
//   or the Apache License, Version 2.0
//     <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0>,
// at your option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(const_fn)]
#![allow(non_snake_case)]

///! The following test cases are based on the tests listed in the FIXT 1.1 spec.

extern crate chrono;
#[macro_use]
extern crate fix_rs;
extern crate mio;

use mio::tcp::Shutdown;
use std::any::Any;
use std::collections::HashMap;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[macro_use]
mod common;
use common::{TestServer,new_logon_message,recv_bytes_with_timeout};
use fix_rs::dictionary::fields::{TestReqID,HeartBtInt,BeginSeqNo,EndSeqNo,SideField,OrigSendingTime,NoHops,HopCompID};
use fix_rs::dictionary::messages::{Logon,Logout,NewOrderSingle,ResendRequest,TestRequest,Heartbeat,SequenceReset,Reject,BusinessMessageReject};
use fix_rs::field_type::{CharFieldType,NoneFieldType,Side,StringFieldType};
use fix_rs::field::Field;
use fix_rs::fix::ParseError;
use fix_rs::fixt::client::{Client,ClientEvent,ConnectionTerminatedReason};
use fix_rs::fixt::message::FIXTMessage;
use fix_rs::message::{NOT_REQUIRED,REQUIRED,MessageDetails};

fn is_logon_valid(message: &Logon) -> bool {
    //TODO: Confirm Logon message is valid.
    true
}

#[test]
fn test_1B() {
    define_dictionary!(
        Logout : Logout,
        Logon : Logon,
        NewOrderSingle : NewOrderSingle,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
    );

    fn do_logon<F>(server_response_func: F) -> (TestServer,Client,usize,Logon)
        where F: Fn(&mut TestServer,Logon) {
        let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

        let logon_message = new_logon_message();
        client.send_message(connection_id,Box::new(logon_message.clone()));

        let message = test_server.recv_message::<Logon>();
        server_response_func(&mut test_server,message.clone());

        (test_server,client,connection_id,logon_message)
    }

    //a, b and c. Handle a simple logon exchange.
    {
        let (_,mut client,connection_id,logon_message) = do_logon(|mut test_server,message| {
            assert!(is_logon_valid(&message));
            test_server.send_message(message);
        });

        client_poll_event!(client,ClientEvent::SessionEstablished(session_connection_id) => {
            assert_eq!(session_connection_id,connection_id);
        });

        //Make sure message received is identical to the one sent. Sending time is tested
        //separately because Client changes this field before it's sent.
        let mut message = client_poll_message!(client,connection_id,Logon);
        assert!((logon_message.sending_time - message.sending_time).num_milliseconds() < 50);
        message.sending_time = logon_message.sending_time;
        assert_eq!(message.sender_comp_id,logon_message.target_comp_id);
        message.sender_comp_id = logon_message.sender_comp_id.clone();
        assert_eq!(message.target_comp_id,logon_message.sender_comp_id);
        message.target_comp_id = logon_message.target_comp_id.clone();
        assert!(message == logon_message);
    }

    //c. Handle receiving a valid Logon with too high of MsgSeqNum.
    {
        let (mut test_server,mut client,connection_id,_) = do_logon(|mut test_server,mut message| {
            assert!(is_logon_valid(&message));
            message.msg_seq_num = 9;
            test_server.send_message(message);
        });

        client_poll_event!(client,ClientEvent::SessionEstablished(session_connection_id) => {
            assert_eq!(session_connection_id,connection_id);
        });

        //Confirm client sent a ResendRequest with high MsgSeqNum.
        let message = test_server.recv_message::<ResendRequest>();
        assert_eq!(message.begin_seq_no,1);
        assert!(message.end_seq_no == 0 || message.end_seq_no == 8);

        //Gap fill up to the Logon message.
        let mut message = new_fixt_message!(SequenceReset);
        message.gap_fill_flag = true;
        message.new_seq_no = 10;
        message.msg_seq_num = 1;
        test_server.send_message(message);

        //Confirm client received Logon message.
        let _ = client_poll_message!(client,connection_id,Logon);
    }

    //d. Handle receiving an invalid Logon.
    {
        let (mut test_server,mut client,connection_id,_) = do_logon(|mut test_server,mut message| {
            message.heart_bt_int = -1;
            test_server.send_message(message);
        });

        //Confirm the client sent a Logout message.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,"HeartBtInt cannot be negative");

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::LogonHeartBtIntNegativeError = reason { true } else { false });
        });
    }

    //e. Handle receiving any message other than a Logon.
    {
        let (mut test_server,mut client,connection_id,_) = do_logon(|mut test_server,_| {
            let mut new_order_single = new_fixt_message!(NewOrderSingle);
            new_order_single.cl_ord_id = String::from("0");
            new_order_single.symbol = String::from("TEST");
            new_order_single.security_id = String::from("0");
            new_order_single.security_id_source = String::from("0");
            new_order_single.side = Side::Buy;
            new_order_single.transact_time = String::from("time");
            new_order_single.order_qty = String::from("1");
            new_order_single.ord_type = String::from("test");
            test_server.send_message(new_order_single);
        });

        //Confirm the client sent a Logout message.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,"First message not a logon");

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::LogonNotFirstMessageError = reason { true } else { false });
        });
    }
}

#[test]
fn test_2B() {
    fn garbled_test_requests() -> Vec<&'static [u8]> {
        //According to FIXT version 1.1, page 40:
        //A garbled message is when:
        //  - BeginString is not the first tag in a message or is not formatted correctly.
        //  - BodyLength is not the second tag in a message or the message does not match the byte
        //    count.
        //  - MsgType is not the third tag in a message.
        //  - Checksum is not the last tag or the checksum is incorrect.
        //TODO: Also, if a MsgSeqNum is EVER missing, a logout message should be sent and the
        //connection should be terminated.

        //TODO: This is the reference message: 8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01

        //TODO: Probably need to check for appropriate ParseError for each otherwise we don't know
        //if it's working correctly.
        let mut result: Vec<&'static [u8]> = Vec::new();
        //result.push(b"9=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01"); //BeginString is not the first tag. //TODO: This should be supported in theory but the error reporting might be a big performance hit...investigate later.
        //result.push(b"8=FIXWRONG\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01"); //BeginString has an invalid version. //TODO: Support once we actually manage protocol version numbers.
        result.push(b"8=FIX.4.2\x0149=TEST\x019=38\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01"); //BodyLength is not the second tag.
        result.push(b"8=FIX.4.2\x019=39\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01"); //BodyLength has too long of byte count.
        result.push(b"8=FIX.4.2\x019=37\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01"); //BodyLength has too short of byte count.
        result.push(b"8=FIX.4.2\x019=38\x0149=TEST\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01"); //MsgType is not the third tag.
        result.push(b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x0110=204\x01112=1\x01"); //Checksum is not the last tag.
        result.push(b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=255\x01"); //Checksum is incorrect.
        result.push(b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=25\x01"); //Checksum is two numbers instead of three.
        result.push(b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=2\x01"); //Checksum is one number instead of three.
        result.push(b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=\x01"); //Checksum is zero numbers instead of three.
        result.push(b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=2555"); //Checksum is missing SOH delimiter at end.

        result
    }

    define_dictionary!(
        Logout : Logout,
        Logon : Logon,
        ResendRequest : ResendRequest,
        TestRequest : TestRequest,
        Heartbeat : Heartbeat,
        SequenceReset : SequenceReset,
        Reject : Reject,
    );

    //a. Basic conversation should be numbered correctly and all responses should be accepted as
    //numbered correctly.
    //Client: (-> Send, <- Recv)
    //-> Logon
    //<- Logon
    //-> TestRequest
    //<- Heartbeat
    //-> Logout
    //<- Logout
    {
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        let mut message = new_fixt_message!(TestRequest);
        message.test_req_id = String::from("1");
        client.send_message(connection_id,Box::new(message));
        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,2);

        let mut hb_message = new_fixt_message!(Heartbeat);
        hb_message.msg_seq_num = 2;
        hb_message.test_req_id = message.test_req_id;
        test_server.send_message(hb_message);
        let message = client_poll_message!(client,connection_id,Heartbeat);
        assert_eq!(message.msg_seq_num,2);

        let message = new_fixt_message!(Logout);
        client.send_message(connection_id,Box::new(message));
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.msg_seq_num,3);

        test_server.send_message(message);
        let message = client_poll_message!(client,connection_id,Logout);
        assert_eq!(message.msg_seq_num,3);
    }

    //b. Having the server send a message with a MsgSeqNum higher than expected sometime after
    //Logon should cause the client to send a ResendRequest.
    {
        //Connect and logon.
        let (mut test_server,_client,_) = TestServer::setup_and_logon(build_dictionary());

        //Server sends TestRequest with high MsgSeqNum.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 25;
        message.test_req_id = String::from("1");
        test_server.send_message(message);

        //Client should automatically send a ResendRequest.
        let message = test_server.recv_message::<ResendRequest>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.begin_seq_no,2);
        assert!(message.end_seq_no == 0 || message.end_seq_no == 25);
    }

    //c. Having the server send a message with a MsgSeqNum lower than expected sometime after Logon
    //should cause the client to send a Logout with an appropriate text message. Then the client
    //should disconnect and prompt the user of the error.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Server sends TestRequest with low MsgSeqNum.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 1;
        message.test_req_id = String::from("1");
        test_server.send_message(message);

        //Client should automatically send a Logout with an appropriate text message.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,"MsgSeqNum too low, expecting 2 but received 1");

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError = reason { true } else { false });
        });
    }

    //TODO: Handle this with the SeqReset-Reset exception too.

    //d. Logon, send the different types of garbled messages, then a valid message and make sure
    //MsgSeqNum is what's expected.
    for garbled_test_request in garbled_test_requests() {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send garbled message.
        let bytes_written = test_server.stream.write(garbled_test_request).unwrap();
        assert_eq!(bytes_written,garbled_test_request.len());

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,_) => {
            assert_eq!(connection_id,gm_connection_id);
        });

        //Send valid message.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("1");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,2);
    }

    //e. Logon, send message with PossDupFlag set to Y, MsgSeqNum lower than expected, and:
    //      1. OrigSendingTime < SendingTime
    //      2. OrigSendingTime == SendingTime
    //And for each type handle:
    //      1. MsgSeqNum not already received => Process as normal.
    //      2. MsgSeqNum has already been received => Ignore message,
    let mut orig_sending_time_setup_fns = Vec::<Box<FnMut(&mut TestRequest)>>::new();
    orig_sending_time_setup_fns.push(Box::new(|message| { message.orig_sending_time = message.sending_time - chrono::Duration::seconds(1); }));
    orig_sending_time_setup_fns.push(Box::new(|message| { message.orig_sending_time = message.sending_time; }));
    for mut orig_sending_time_setup_fn in orig_sending_time_setup_fns {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send message with high MsgSeqNum to client.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 9;
        message.test_req_id = String::from("1");
        test_server.send_message(message);

        let message = test_server.recv_message::<ResendRequest>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.begin_seq_no,2);
        assert!(message.end_seq_no == 0 || message.end_seq_no == 9);

        //Respond with a gap fill so we can send a message with PossDupFlag set and a low MsgSeqNum
        //afterwards.
        let mut message = new_fixt_message!(SequenceReset);
        message.gap_fill_flag = true;
        message.new_seq_no = 9;
        message.msg_seq_num = 2;
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,SequenceReset);
        assert_eq!(message.gap_fill_flag,true);
        assert_eq!(message.new_seq_no,9);
        assert_eq!(message.msg_seq_num,2);

        /* TODO: There seems to be conflicting information about whether the MsgSeqNum that was gap
         * filled should be considered received or still outstanding. If it's still outstanding,
         * then we are breaking the strict ordered processing of messages.
        //Send TestRequest with OrigSendingTime <= SendingTime for MsgSeqNum not already received.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("2");
        message.poss_dup_flag = true;
        orig_sending_time_setup_fn(&mut message);
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,"2");
        assert_eq!(message.poss_dup_flag,true);

        //Send the same TestRequest but now MsgSeqNum has already been received. The message should
        //be ignored.
        test_server.send_message(message);
        client_poll_event!(client,ClientEvent::MessageReceivedDuplicate(msg_connection_id,duplicate_message) => {
            assert_eq!(msg_connection_id,connection_id);

            let message = duplicate_message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,2);
            assert_eq!(message.test_req_id,"2");
            assert_eq!(message.poss_dup_flag,true);
        });
        */
    }

    //f. Similar to (e.) above except OrigSendingTime is greater than SendingTime and MsgSeqNum is
    //as expected. Client should send a Reject but otherwise increment inbound MsgSeqNum as normal.
    //Client should also report the error.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send TestRequest with OrigSendingTime > SendingTime.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("2");
        message.poss_dup_flag = true;
        message.orig_sending_time = message.sending_time + chrono::Duration::seconds(1);
        test_server.send_message(message);

        //Server should receive Reject with an appropriate reason.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.ref_seq_num,2);
        assert_eq!(message.session_reject_reason,"10");
        assert_eq!(message.text,"SendingTime accuracy problem");

        client_poll_event!(client,ClientEvent::MessageRejected(msg_connection_id,rejected_message) => {
            assert_eq!(msg_connection_id,connection_id);

            let message = rejected_message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,2);
            assert_eq!(message.test_req_id,"2");
            assert_eq!(message.poss_dup_flag,true);
        });
    }

    //g. Similar to (f.) except OrigSendingTime is not specified. Client should respond with a
    //Reject and increment the inbound MsgSeqNum just like when any required field is missing.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send TestRequest without OrigSendingTime.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("2");
        message.poss_dup_flag = true;
        test_server.send_message(message);

        //Server should receive Reject with an appropriate reason.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.ref_seq_num,2);
        assert_eq!(message.session_reject_reason,"1");
        assert_eq!(message.text,"Conditionally required tag missing");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);

            match parse_error {
                ParseError::MissingConditionallyRequiredTag(tag,message) => {
                    assert_eq!(tag,OrigSendingTime::tag());

                    let message = message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
                    assert_eq!(message.msg_seq_num,2);
                    assert_eq!(message.test_req_id,"2");
                    assert_eq!(message.poss_dup_flag,true);
                },
                _ => panic!("Wrong parse error"),
            };
        });
    }

    //h., i.: TODO: BeginStr should match value in specified testing profile. Otherwise, Logout.
    //j., k.: TODO: SenderCompID and TargetCompID should match values in specified testing
    //              profile. Otherwise, reject and Logout.
    //l., m.: TODO: BodyLength must be correct. Otherwise, ignore and issue warning.
    //n., o.: TODO: SendingTime must be within 2 minutes of current (atomic click-based) time.
    //              Otherwise, Reject and Logout.
    //p., q.: TODO: MsgType should be valid. Otherwise, send Reject, increment inbound MsgSeqNum
    //              and issue warning.
    //r:      TODO: MsgType is valid but unsupported. Send Reject (< FIX 4.2) or Business Message
    //              Reject (>= FIX 4.2), increment inbound MsgSeqNum, and issue warning.
    //s., t.: TODO: BeginString, BodyLength, and MsgType should be first three fields. Otherwise,
    //              ignore and issue warning.
}

#[test]
fn test_3B() {
    //This is covered by 2B.d.
}

#[test]
fn test_4B() {
    define_dictionary!(
        Logon : Logon,
        TestRequest : TestRequest,
        Heartbeat : Heartbeat,
    );

    //a. Make sure a Heartbeat message is sent automatically after no data is sent after
    //HeartBeatInt seconds.
    {
        //Connect and logon.
        let (mut test_server,_client,_) = TestServer::setup_and_logon(build_dictionary());

        //Sleep until Heartbeat is triggered.
        thread::sleep(Duration::from_millis(5500));

        //Make sure Heartbeat was sent by client.
        let _ = test_server.recv_message::<Heartbeat>();
    }

    //a. Similar to above but make sure a Heartbeat is not sent after HeartBeatInt seconds when
    //data is sent before HeartBeatInt seconds.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Sleep for half the Heartbeat session.
        thread::sleep(Duration::from_millis(2500));

        //Send message to reset Client's output heartbeat.
        let mut message = new_fixt_message!(TestRequest);
        message.test_req_id = String::from("1");
        client.send_message(connection_id,Box::new(message));
        let _ = test_server.recv_message::<TestRequest>();

        //Sleep a little bit and make sure clienent sends a TestRequest because it didn't receive
        //anything.
        thread::sleep(Duration::from_millis(1000));
        let _ = test_server.recv_message::<TestRequest>();

        //Sleep a little longer than the original heartbeat session.
        thread::sleep(Duration::from_millis(2000));

        //Make sure Heartbeat was NOT sent by client.
        assert!(test_server.try_recv_fixt_message(Duration::from_secs(1)).is_none());
    }

    //b. Reply to TestRequest with a Heartbeat with Test Request matching TestReqID.
    {
        //Connect and logon.
        let (mut test_server,_client,_connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send TestRequest.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        //Make sure client responds with Heartbeat.
        let message = test_server.recv_message::<Heartbeat>();
        assert_eq!(message.test_req_id,"test_id");
    }
}

#[test]
fn test_5B() {
    //Receive a valid Heartbeat message.

    define_dictionary!(
        Logon : Logon,
        Heartbeat : Heartbeat,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Server sends Heartbeat to client.
    let mut hb_message = new_fixt_message!(Heartbeat);
    hb_message.msg_seq_num = 2;
    test_server.send_message(hb_message);

    //Client should accept heartbeat message as normal.
    let message = client_poll_message!(client,connection_id,Heartbeat);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.test_req_id,"");
}

#[test]
fn test_6B() {
    define_dictionary!(
        Logon : Logon,
        TestRequest : TestRequest,
        Heartbeat : Heartbeat,
    );

    //When no data is sent from server to client for HeartBeatInt + "some reasonable period of
    //time", client should send a TestRequest. The server should respond with a matching TestReqID
    //and the client should make this confirmation.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Sleep until TestRequest is triggered.
        thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

        //Ignore HeartBeat because Client didn't send anything for HeartBeatInt seconds.
        let message = test_server.recv_message::<Heartbeat>();
        assert_eq!(message.msg_seq_num,2);

        //Confirm Client sent TestRequest.
        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,3);
        let test_req_id = message.test_req_id;

        //Reply with Heartbeat message and make sure client think it's correct.
        let mut hb_message = new_fixt_message!(Heartbeat);
        hb_message.msg_seq_num = 2;
        hb_message.test_req_id = test_req_id.clone();
        test_server.send_message(hb_message);

        let message = client_poll_message!(client,connection_id,Heartbeat);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,test_req_id);
    }

    //Same as above but do not respond to TestRequest so client should consider the connection
    //lost.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Sleep until TestRequest is triggered.
        thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

        //Ignore HeartBeat because Client didn't send anything for HeartBeatInt seconds.
        let _ = test_server.recv_message::<Heartbeat>();

        //Confirm Client sent TestRequest.
        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,3);

        //Sleep until disconnect.
        thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::TestRequestNotRespondedError = reason { true } else { false });
        });
    }
}

#[test]
fn test_7B() {
    //Client should handle Reject messages just like any other message. Specifically, The inbound
    //MsgSeqNum should be incremented.

    define_dictionary!(
        Logon : Logon,
        TestRequest : TestRequest,
        Reject : Reject,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Send Reject to client.
    let mut message = new_fixt_message!(Reject);
    message.msg_seq_num = 2;
    message.ref_seq_num = 2;
    test_server.send_message(message);

    //Confirm client received Reject.
    let message = client_poll_message!(client,connection_id,Reject);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,2);

    //Confirm MsgSeqNum was incremented by sending another message and making sure it's expected.
    let mut message = new_fixt_message!(TestRequest);
    message.msg_seq_num = 3;
    message.test_req_id = String::from("test_id");
    test_server.send_message(message);

    let message = client_poll_message!(client,connection_id,TestRequest);
    assert_eq!(message.msg_seq_num,3);
}

#[test]
fn test_8B() {
    //Wait for Client to send two non-Logon administrative messages. Send a ResendRequest to client
    //and expect it to respond with a SequenceReset-GapFill.
    //TODO: Need to handle ResendRequest on non-administrative messages too.

    define_dictionary!(
        Logon : Logon,
        TestRequest : TestRequest,
        Heartbeat : Heartbeat,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
    );

    //Connect and logon.
    let (mut test_server,_client,_) = TestServer::setup_and_logon(build_dictionary());

    //Sleep until TestRequest and Heartbeat are triggered.
    thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

    let _ = test_server.recv_message::<Heartbeat>();
    let _ = test_server.recv_message::<TestRequest>();

    //Send ResendRequest to client.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 2;
    message.begin_seq_no = 2;
    message.end_seq_no = 3;
    test_server.send_message(message);

    //Make sure client responds with an appropriate SequenceReset-GapFill.
    let message = test_server.recv_message::<SequenceReset>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.gap_fill_flag,true);
    assert_eq!(message.new_seq_no,4);
}

#[test]
fn test_9B() {
    //TODO: Not really sure the best way to test this yet.
}

#[test]
fn test_10B() {
    define_dictionary!(
        Logon : Logon,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
        TestRequest : TestRequest,
        Logout : Logout,
        Reject : Reject,
    );

    //a. Send SequenceReset-GapFill to client with NewSeqNo > MsgSeqNum > expected inbound sequence
    //number. Client should respond wth ResendRequest for between expected inbound sequence number
    //received MsgSeqNum.
    {
        //Connect.
        let (mut test_server,_client,_) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > MsgSeqNum > expected inbound sequence number.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 10;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        test_server.send_message(message);
        
        //Confirm client responds with appropriate ResendRequest.
        let message = test_server.recv_message::<ResendRequest>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.begin_seq_no,2);
        assert!(message.end_seq_no == 9 || message.end_seq_no == 0);
    }

    //b. Same as above except MsgSeqNum == expected inbound sequence number. Client should change
    //the next expected inbound sequence number to NewSeqNo.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > MsgSeqNum == expected inbound sequence number.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 2;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        test_server.send_message(message);

        //Confirm client is not buffering the SequenceReset.
        let message = client_poll_message!(client,connection_id,SequenceReset);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.gap_fill_flag,true);
        assert_eq!(message.new_seq_no,15);

        //Send a new message to make sure the expected inbound sequence number was changed.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 15;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,15);
        assert_eq!(message.test_req_id,"test_id");
    }

    //c. Same as above except MsgSeqNum < expected inbound sequence number and PossDupFlag set to
    //"Y". The client should ignore the message.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > expected inbound sequence number > MsgSeqNum.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 1;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        message.poss_dup_flag = true;
        message.orig_sending_time = message.sending_time;
        test_server.send_message(message);

        //Confirm client ignored the message.
        client_poll_event!(client,ClientEvent::MessageReceivedDuplicate(msg_connection_id,duplicate_message) => {
            assert_eq!(msg_connection_id,connection_id);

            let message = duplicate_message.as_any().downcast_ref::<SequenceReset>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,1);
            assert_eq!(message.gap_fill_flag,true);
            assert_eq!(message.new_seq_no,15);
            assert_eq!(message.poss_dup_flag,true);
        });

        //Send a new message to make sure the expected inbound sequence number was NOT changed.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,"test_id");
    }

    //d. Same as above except PossDupFlag is not set. The client should send a Logout with an
    //appropriate reason, disconnect, and then issue an error.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > expected inbound sequence number > MsgSeqNum.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 1;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        test_server.send_message(message);

        //Confirm client sent Logout with an appropriate reason.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,"MsgSeqNum too low, expecting 2 but received 1");

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError = reason { true } else { false });
        });
    }

    //e. Send SequenceReset-GapFill to client with NewSeqNo <= MsgSeqNum == expected inbound
    //sequence number. Client should respond with Reject containing an appropriate message.
    for new_seq_no in 1..3 {
        use std::fmt::Write;

        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo <= MsgSeqNum == expected inbound sequence
        //number.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 2;
        message.gap_fill_flag = true;
        message.new_seq_no = new_seq_no;
        test_server.send_message(message);

        //Make sure client sends Reject with an appropriate message as response.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.ref_seq_num,2);
        let mut expected_error_text = String::new();
        let _ = write!(&mut expected_error_text,"Attempt to lower sequence number, invalid value NewSeqNo={}",new_seq_no);
        assert_eq!(message.text,expected_error_text);
        assert_eq!(message.session_reject_reason,"5");

        client_poll_event!(client,ClientEvent::MessageRejected(msg_connection_id,rejected_message) => {
            assert_eq!(msg_connection_id,connection_id);

            let message = rejected_message.as_any().downcast_ref::<SequenceReset>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,2);
            assert_eq!(message.gap_fill_flag,true);
            assert_eq!(message.new_seq_no,new_seq_no);
        });
    }
}

#[test]
fn test_11B() {
    define_dictionary!(
        Logon : Logon,
        SequenceReset : SequenceReset,
        ResendRequest : ResendRequest,
        TestRequest : TestRequest,
        Logout : Logout,
        Reject : Reject,
    );

    //Try a few msg_seq_nums to make sure they are ignored.
    let msg_seq_nums: Vec<u64> = vec![1,2,800,2000000];

    //a. Client receives SequenceReset-Reset message with NewSeqNo > inbound expected sequence
    //number. Client should ignore MsgSeqNum of received message and set inbound expected sequence
    //number to NewSeqNo.
    for msg_seq_num in msg_seq_nums.clone() { 
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 99999;
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,99999);

        //Make sure client accepts a new message with the new sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 99999;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,99999);
        assert_eq!(message.test_req_id,"test_id");
    }

    //a. Same as (a) except confirming that buffered messages are discarded.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Create a message that client will be forced to buffer.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 10;
        message.test_req_id = String::from("buffer_me");
        test_server.send_message(message);
        let _ = test_server.recv_message::<ResendRequest>();

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 99999;
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,99999);

        //Make sure old TestRequest is never accepted and client accepts a new message with the new
        //sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 99999;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,99999);
        assert_eq!(message.test_req_id,"test_id");
    }

    //b. Same as (a) except NewSeqNo == inbound expected sequence number. Client should ignore
    //MsgSeqNum of received message and issue a warning.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 2;
        test_server.send_message(message);

        //Make sure client issued a warning.
        client_poll_event!(client,ClientEvent::SequenceResetResetHasNoEffect(warning_connection_id) => {
            assert_eq!(warning_connection_id,connection_id);
        });

        //Make sure client accepted the message.
        let message = client_poll_message!(client,connection_id,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,2);

        //Make sure client did not change inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,"test_id");
    }

    //c. Same as (a) except NewSeqNo < inbound expected sequence number. Client should ignore
    //MsgSeqNum. Client should respond with Reject message containing an appropriate reason. Client
    //should not change inbound expected sequence number. Client should issue an error.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 1;
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::SequenceResetResetInThePast(warning_connection_id) => {
            assert_eq!(warning_connection_id,connection_id);
        });
        
        //Make sure client accepted the message.
        let message = client_poll_message!(client,connection_id,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,1);

        //Make sure client replied with Reject message.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.ref_seq_num,msg_seq_num);
        assert_eq!(message.session_reject_reason,"5");
        assert_eq!(message.text,"Attempt to lower sequence number, invalid value NewSeqNo=1");

        //Make sure client did not change inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,"test_id");
    }
}

#[test]
fn test_12B() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
    );

    //Client should be able to initiate a Logout via the API. The client should wait for a Logout
    //message and then disconnect.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Begin Logout.
        client.logout(connection_id);

        //Have server respond to Logout.
        let mut message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,"");
        message.session_status = String::from("4"); //Session logout complete"
        test_server.send_message(message);

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::ClientRequested = reason { true } else { false });
        });
    }

    //Same as above except the server never sends the Logout message. Then the client should
    //disconnect automatically after 10 seconds and issue a warning.
    {
        //Connect and Logon.
        let (test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Begin Logout.
        client.logout(connection_id);

        //Make sure socket isn't closed immediatelly.
        assert!(!test_server.is_stream_closed(Duration::from_secs(1)));

        //Wait for Logout to timeout.
        thread::sleep(Duration::from_millis(9500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::LogoutNoResponseError = reason { true } else { false });
        });
    }
}

#[test]
fn test_13B() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
    );

    //a. Client receives Logout response in response to its Logout message and then should
    //disconnect immediately. This is already covered by 12B above.
    
    //b. Client receives a Logout message without sending a Logout message first. Client should
    //respond with a Logout message and wait for server to disconnect.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send Logout to client.
        let mut message = new_fixt_message!(Logout);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Client should respond with a response Logout message.
        let _ = client_poll_message!(client,connection_id,Logout);
        let _ = test_server.recv_message::<Logout>();

        //Server disconnects and client should acknowledge that the connection has been closed.
        //Since client isn't sending anymore data in this state, it can take up to 10 seconds to
        //notice the shutdown.
        let _ = test_server.stream.shutdown(Shutdown::Both);
        thread::sleep(Duration::from_secs(6)); //6 seconds + the duration in client_poll_event!() >= 10 seconds
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::ServerRequested = reason { true } else { false });
        });
    }

    //b. Same as above except the server does not disconnect so the client should disconnect after
    //10 seconds and issue an error.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send Logout to client.
        let mut message = new_fixt_message!(Logout);
        message.msg_seq_num = 2;
        test_server.send_message(message);
        let _ = client_poll_message!(client,connection_id,Logout);

        //Client should respond with a response Logout message.
        let _ = test_server.recv_message::<Logout>();

        //Wait around a little bit and make sure client doesn't disconnect instantly.
        thread::sleep(Duration::from_secs(5));
        assert!(!test_server.is_stream_closed(Duration::from_millis(100)));

        //Wait around a little over the full 10 seconds and make sure client does force a
        //disconnect.
        thread::sleep(Duration::from_millis(5500));
        assert!(recv_bytes_with_timeout(&mut test_server.stream,Duration::from_secs(1)).is_none()); //Client should have stopped sending TestRequests and Heartbeats!
        assert!(test_server.is_stream_closed(Duration::from_secs(1)));
        client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
            assert_eq!(terminated_connection_id,connection_id);
            assert!(if let ConnectionTerminatedReason::LogoutNoHangUpError = reason { true } else { false });
        });
    }
}

#[test]
fn test_14B() {
    define_dictionary!(
        Logon : Logon,
        TestRequest : TestRequest,
        ResendRequest : ResendRequest,
        Reject : Reject,
        BusinessMessageReject : BusinessMessageReject,
    );

    define_fields!(
        BeginSeqNoString: StringFieldType = BeginSeqNo::tag(),
        TestReqIDEmpty: NoneFieldType = TestReqID::tag(),
        UndefinedField: StringFieldType = b"9999999",
    );

    define_fixt_message!(TestRequestWithUndefinedField: b"1" => {
        REQUIRED, test_req_id: TestReqID,
        REQUIRED, undefined: UndefinedField,
    });

    define_fixt_message!(TestRequestWithNotRequiredField: b"1" => {
        NOT_REQUIRED, test_req_id: TestReqID,
    });

    define_fixt_message!(TestRequestWithWrongField: b"1" => {
        REQUIRED, test_req_id: TestReqID,
        REQUIRED, heart_bt_int: HeartBtInt,
    });

    define_fixt_message!(TestRequestWithEmptyField: b"1" => {
        REQUIRED, test_req_id: TestReqIDEmpty,
    });

    define_fixt_message!(ResendRequestWithStringBeginSeqNo: b"2" => {
        REQUIRED, begin_seq_no: BeginSeqNoString,
        REQUIRED, end_seq_no: EndSeqNo,
    });

    define_fixt_message!(TestRequestWithDuplicateField: b"1" => {
        REQUIRED, test_req_id_1: TestReqID,
        REQUIRED, test_req_id_2: TestReqID,
    });

    fn do_garbled_test_with_dict<F: Fn(&mut TestServer,&mut Client,usize),TestRequestResponse: FIXTMessage + Any + Clone>(session_reject_reason: &'static str,ref_tag_id: &'static [u8],test_func: F,dict: HashMap<&'static [u8],Box<FIXTMessage + Send>>) {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(dict);

        //Perform test by sending message and making sure client reacts correctly.
        test_func(&mut test_server,&mut client,connection_id);

        //Make sure client responds with an appropriate Reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason,String::from(session_reject_reason));
        assert_eq!(message.ref_tag_id,String::from_utf8_lossy(ref_tag_id));

        //Make sure client incremented inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequestResponse);
        assert_eq!(message.msg_seq_num(),3);
    }

    fn do_garbled_test<F: Fn(&mut TestServer,&mut Client,usize)>(session_reject_reason: &'static str,ref_tag_id: &'static [u8],test_func: F) {
        do_garbled_test_with_dict::<F,TestRequest>(session_reject_reason,ref_tag_id,test_func,build_dictionary());
    }

    //a. Send message with tag not defined in spec (tag shouldn't be allowed in any message).
    //Client should respond with Reject, increment inbound sequence number, and issue an error.
    do_garbled_test("0",UndefinedField::tag(),|test_server,client,connection_id| {
        //Send message with undefined tag.
        let mut message = new_fixt_message!(TestRequestWithUndefinedField);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        message.undefined = String::from("undefined");
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::UnknownTag(tag) => assert_eq!(tag,UndefinedField::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //b. Send message with a required field missing. Client should respond with Reject, increment
    //inbound sequence number, and issue an error.
    do_garbled_test("1",TestReqID::tag(),|test_server,client,connection_id| {
        //Send message with missing required tag.
        let mut message = new_fixt_message!(TestRequestWithNotRequiredField);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::MissingRequiredTag(tag,message) => {
                    assert_eq!(tag,TestReqID::tag());
                    assert_eq!(message.msg_seq_num(),2);
                },
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //c. Send message with defined field but not for the message type. Client should respond with
    //Reject, increment inbound sequence number, and issue an error.
    do_garbled_test("2",HeartBtInt::tag(),|test_server,client,connection_id| {
        //Send message with wrong tag for message.
        let mut message = new_fixt_message!(TestRequestWithWrongField);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        message.heart_bt_int = 5;
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::UnexpectedTag(tag) => assert_eq!(tag,HeartBtInt::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //d. Send message with a tag containing no value. Client should respond with Reject, increment
    //inbound sequence number, and issue an error.
    do_garbled_test("4",TestReqIDEmpty::tag(),|test_server,client,connection_id| {
        //Send message with valid tag but empty field for message.
        let mut message = new_fixt_message!(TestRequestWithEmptyField);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::NoValueAfterTag(tag) => assert_eq!(tag,TestReqIDEmpty::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //e. Send message with an incorrect value (not in an enumerated set) for a field. Client should
    //respond with Reject, increment inbound sequence number, and issue an error.
    {
        define_fixt_message!(TestRequestWithEnumeratedField: b"1" => {
            REQUIRED, test_req_id: TestReqID,
            NOT_REQUIRED, enumerated_field: SideField,
        });

        define_fields!(
            SideChar: CharFieldType = SideField::tag(),
        );

        define_fixt_message!(TestRequestWithIncorrectField: b"1" => {
            REQUIRED, test_req_id: TestReqID,
            NOT_REQUIRED, enumerated_field: SideChar,
        });

        define_dictionary!(
            Logon : Logon,
            TestRequestWithEnumeratedField : TestRequestWithEnumeratedField,
            ResendRequest : ResendRequest,
            Reject : Reject,
        );

        do_garbled_test_with_dict::<_,TestRequestWithEnumeratedField>("5",SideField::tag(),|test_server,client,connection_id| {
            //Send message with incorrect value.
            let mut message = new_fixt_message!(TestRequestWithIncorrectField);
            message.test_req_id = String::from("test_id");
            message.enumerated_field = b'Z';
            test_server.send_message(message);

            //Make sure client issued an error.
            client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
                assert_eq!(gm_connection_id,connection_id);
                match parse_error {
                    ParseError::OutOfRangeTag(tag) => assert_eq!(tag,SideField::tag()),
                    _ => panic!("Wrong parse error"),
                };
            });
        },build_dictionary());
    }

    //f. Send message with an incorrect data format for a field. Client should respond with Reject,
    //increment inbound sequence number, and issue an error.
    do_garbled_test("6",BeginSeqNoString::tag(),|test_server,client,connection_id| {
        //Send message with incorrect value.
        let mut message = new_fixt_message!(ResendRequestWithStringBeginSeqNo);
        message.msg_seq_num = 2;
        message.begin_seq_no = String::from("-1");
        message.end_seq_no = 0;
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::WrongFormatTag(tag) => assert_eq!(tag,BeginSeqNoString::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //g. Send message with one of the header fields after after the body fields or the trailer
    //fields are not at the end. Client should respond with Reject, increment inbound sequence
    //number, and issue an error.
    /* TODO: This test conflicts with the fact that these issues indicate that the message is
     * garbled. Page 26 of FIXT Version 1.1 clearly states that garbled messages should be outright
     * ignored.
    for message_bytes in vec![
        b"49=TEST\x018=FIX.4.2\x019=38\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01", //BeginStr is not the first tag.
        b"8=FIX.4.2\x0149=TEST\x019=38\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01", //BodyLength is not the second tag.
        b"8=FIX.4.2\x019=38\x0149=TEST\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01", //MsgType is not the third tag.
        b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x0110=204\x01112=1\x01" //Checksum is not the last tag.
    ] {
        do_garbled_test("14",|test_server,client,connection_id| {
            //Send message.
            let bytes_written = test_server.stream.write(message_bytes).unwrap();
            assert_eq!(bytes_written,message_bytes.len());

            //Make sure client issued an error.
            client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
                assert_eq!(gm_connection_id,connection_id);
                match parse_error {
                    ParseError::BeginStrNotFirstTag => {},
                    ParseError::BodyLengthNotSecondTag => {},
                    ParseError::MsgTypeNotThirdTag => {},
                    ParseError::ChecksumNotLastTag => {},
                    _ => panic!("Wrong parse error"),
                };
            });
        });
    }
    */

    //h. Send message with a tag duplicated outside of an appropriate repeating group. Client
    //should respond with Reject, increment inbound sequence number, and issue an error.
    do_garbled_test("13",TestReqID::tag(),|test_server,client,connection_id| {
        //Send message with duplicate tag.
        let mut message = new_fixt_message!(TestRequestWithDuplicateField);
        message.msg_seq_num = 2;
        message.test_req_id_1 = String::from("test_id_1");
        message.test_req_id_2 = String::from("test_id_2");
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::DuplicateTag(tag) => assert_eq!(tag,TestReqID::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //i. Send message with repeating groups that don't match the specified count. Client should
    //respond with Reject, increment inbound sequence number, and issue an error.
    {
        let mut messages_bytes: Vec<(&'static [u8],&'static [u8])> = Vec::new();
        messages_bytes.push((NoHops::tag(),b"8=FIX.4.2\x019=999\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01627=2\x01112=1\x0110=204\x01")); //Claim two groups but have zero.
        messages_bytes.push((TestReqID::tag(),b"8=FIX.4.2\x019=999\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01627=2\x01628=1\x01112=1\x0110=204\x01")); //Claim two groups but have one.
        messages_bytes.push((HopCompID::tag(),b"8=FIX.4.2\x019=999\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01627=2\x01628=1\x01628=2\x01628=3\x01112=1\x0110=204\x01")); //Claim two groups but have three.
        for (ref_tag_id,message_bytes) in messages_bytes {
            do_garbled_test("16",ref_tag_id,|test_server,client,connection_id| {
                //Send message.
                let bytes_written = test_server.stream.write(message_bytes).unwrap();
                assert_eq!(bytes_written,message_bytes.len());

                //Make sure client issued an error.
                client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
                    assert_eq!(gm_connection_id,connection_id);
                    match parse_error {
                        ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,ref_tag_id),
                        ParseError::RepeatingGroupTagWithNoRepeatingGroup(tag) => assert_eq!(tag,ref_tag_id),
                        ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(tag) => assert_eq!(tag,ref_tag_id),
                        _ => panic!("Wrong parse error"),
                    };
                });
            });
        }
    }

    //j. Send message with repeating groups but the first required field is not at the beginning of
    //the repeating group. Client should Reject, increment inbound sequence number, and issue an
    //error.
    //Skipping because there is nothing about there being a required field order in repeating
    //groups (excluding the first field) according to page 21 of FIX Version 5.0 Service Pack 2
    //Volume 1.

    //k. Send message with SOH character in value of non-data field.
    //Skipping because it isn't practical to test for. For example, in a string field, the string
    //would be cut short and the remaining part would represent the next tag. This next tag then
    //would likely be wrong and trigger an error.

    //l. Send message when application-level processing is not available.
    //Skipping because engine can't run without application-level processing available.

    //m. Send message with a conditionally required field missing. Client should respond with
    //Business Message Reject referencing the conditionally missing field, increment MsgSeqNum,
    //and issue an error.
    //Note: FIXT 1.1 implies responding with a Business Message Reject but page 106 of FIX Version
    //5.0 Service Pack 2 Volume 1 says Business Message Reject must only be used for a certain set
    //of messages.
    //TODO: Once we support business messages, in addition to the test below, test that they use a
    //BusinessMessageReject instead.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send message with conditionally required field (OrigSendingTime) missing.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = String::from("test_id");
        message.poss_dup_flag = true;
        test_server.send_message(message);

        //Make sure client issued an error.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(gm_connection_id,parse_error) => {
            assert_eq!(gm_connection_id,connection_id);
            match parse_error {
                ParseError::MissingConditionallyRequiredTag(tag,_) => assert_eq!(tag,OrigSendingTime::tag()),
                _ => panic!("Wrong parse error"),
            };
        });

        //Make sure client responds with an appropriate Reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason,String::from("1"));
        assert_eq!(message.ref_msg_type,String::from_utf8_lossy(<TestRequest as MessageDetails>::msg_type()));
        assert_eq!(message.ref_tag_id,String::from_utf8_lossy(OrigSendingTime::tag()));

        //Make sure client incremented inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = String::from("test_id");
        test_server.send_message(message);

        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num(),3);
    }

    //n. Send message with field appearing in both cleartext and encrypted section with different
    //values.
    //Skipping because engine doesn't support encryption yet.
}

#[test]
fn test_15B() {
    //This is covered by parser tests.
}

#[test]
fn test_16B() {
    //TODO: API does not support queuing while disconnected at the moment. Might be too low of
    //level.
    unimplemented!();
}

#[test]
fn test_17B() {
    //Encryption is not supported for now.
}

#[test]
fn test_18B() {
    //Third party addressing is not supported for now.
}

#[test]
fn test_19B() {
    //TODO: This is Message specific functionality that has to wait until we support non-admin
    //messages.
    unimplemented!();
}

#[test]
fn test_20B() {
    define_dictionary!(
        Logon : Logon,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
        TestRequest : TestRequest,
    );

    //Client sends a ResendRequest and then receives a ResendRequest. The client should resend
    //requested messages and then send a new ResendRequest for the remaining missing messages.

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Have client send a few messages to server without server acknowledging them.
    for x in 2..6 {
        let mut message = new_fixt_message!(TestRequest);
        message.test_req_id = x.to_string();
        client.send_message(connection_id,Box::new(message));

        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,x);
    }

    //Trigger client to send a ResendRequest.
    let mut message = new_fixt_message!(TestRequest);
    message.msg_seq_num = 10;
    message.test_req_id = String::from("10");
    test_server.send_message(message);
    
    let message = test_server.recv_message::<ResendRequest>();
    assert_eq!(message.msg_seq_num,6);
    assert_eq!(message.begin_seq_no,2);
    assert!(message.end_seq_no == 9 || message.end_seq_no == 0);

    //Have server send its own ResendRequest.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 11;
    message.begin_seq_no = 2;
    message.end_seq_no = 5;
    test_server.send_message(message);

    //Make sure client complies with ResendRequest.
    let message = test_server.recv_message::<SequenceReset>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.gap_fill_flag,true);
    assert_eq!(message.new_seq_no,6);

    //Make sure client sends a new ResendRequest for still missing messages.
    let message = test_server.recv_message::<ResendRequest>();
    assert_eq!(message.msg_seq_num,7);
    assert_eq!(message.begin_seq_no,2);
    assert!(message.end_seq_no == 9 || message.end_seq_no == 0);
}
