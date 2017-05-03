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

#![feature(attr_literals)]
#![feature(const_fn)]
#![allow(non_snake_case)]

///! The following test cases are based on the tests listed in the FIXT 1.1 spec.

extern crate chrono;
#[macro_use]
extern crate fix_rs;
#[macro_use]
extern crate fix_rs_macros;
extern crate mio;
extern crate phf;

use mio::tcp::Shutdown;
use std::any::Any;
use std::collections::HashMap;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[macro_use]
mod common;
use common::{CLIENT_SENDER_COMP_ID,CLIENT_TARGET_COMP_ID,SERVER_SENDER_COMP_ID,SERVER_TARGET_COMP_ID,TestStream,new_logon_message,recv_bytes_with_timeout,send_message};
use fix_rs::dictionary::standard_msg_types;
use fix_rs::dictionary::field_types::generic::{CharFieldType,NoneFieldType,StringFieldType};
use fix_rs::dictionary::field_types::other::{BusinessRejectReason,OrdType,SecurityIDSource,SessionRejectReason,Side};
use fix_rs::dictionary::fields::{TestReqID,HeartBtInt,EndSeqNo,SideField,OrigSendingTime,NoHops,HopCompID};
use fix_rs::dictionary::messages::{Logon,Logout,NewOrderSingle,ResendRequest,TestRequest,Heartbeat,SequenceReset,Reject,BusinessMessageReject};
use fix_rs::field::Field;
use fix_rs::fix::ParseError;
use fix_rs::field_tag::{self,FieldTag};
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt;
use fix_rs::fixt::engine::{Engine,EngineEvent,Connection,ConnectionTerminatedReason,Listener,ResendResponse};
use fix_rs::fixt::message::{BuildFIXTMessage,FIXTMessage};
use fix_rs::message::{self,NOT_REQUIRED,REQUIRED,MessageDetails};
use fix_rs::message_version::{self,MessageVersion};

fn is_logon_valid(message: &Logon) -> bool {
    //TODO: Confirm Logon message is valid.
    true
}

#[test]
fn test_1B() {
    define_dictionary!(
        Logout,
        Logon,
        NewOrderSingle,
        ResendRequest,
        SequenceReset,
    );

    fn do_logon<F>(server_response_func: F) -> (TestStream,Engine,Connection,Logon)
        where F: Fn(&mut TestStream,Logon) {
        let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

        let logon_message = new_logon_message();
        client.send_message(connection,logon_message.clone());

        let message = test_server.recv_message::<Logon>();
        server_response_func(&mut test_server,message.clone());

        (test_server,client,connection,logon_message)
    }

    //a, b and c. Handle a simple logon exchange.
    {
        let (_,mut client,connection,logon_message) = do_logon(|mut test_server,message| {
            assert!(is_logon_valid(&message));

            let mut response_message = new_fixt_message!(Logon);
            response_message.encrypt_method = message.encrypt_method;
            response_message.heart_bt_int = message.heart_bt_int;
            response_message.default_appl_ver_id = message.default_appl_ver_id;
            test_server.send_message(response_message);
        });

        engine_poll_event!(client,EngineEvent::SessionEstablished(session_connection) => {
            assert_eq!(session_connection,connection);
        });

        //Make sure message received is identical to the one sent. Sending time is tested
        //separately because Engine changes this field before it's sent.
        let mut message = engine_poll_message!(client,connection,Logon);
        assert!(logon_message.sending_time.signed_duration_since(message.sending_time).num_milliseconds() < 50);
        message.sending_time = logon_message.sending_time;
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);
        message.sender_comp_id = logon_message.sender_comp_id.clone();
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);
        message.target_comp_id = logon_message.target_comp_id.clone();
        assert_eq!(message.meta.as_ref().unwrap().message_version,MessageVersion::FIX50SP2);
        assert!(message == logon_message);
    }

    //c. Handle receiving a valid Logon with too high of MsgSeqNum.
    {
        let (mut test_server,mut client,connection,_) = do_logon(|mut test_server,message| {
            assert!(is_logon_valid(&message));

            let mut response_message = new_fixt_message!(Logon);
            response_message.msg_seq_num = 9;
            response_message.encrypt_method = message.encrypt_method;
            response_message.heart_bt_int = message.heart_bt_int;
            response_message.default_appl_ver_id = message.default_appl_ver_id;
            test_server.send_message(response_message);
        });

        engine_poll_event!(client,EngineEvent::SessionEstablished(session_connection) => {
            assert_eq!(session_connection,connection);
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
        let _ = engine_poll_message!(client,connection,Logon);
    }

    //d. Handle receiving an invalid Logon.
    {
        let (mut test_server,mut client,connection,_) = do_logon(|mut test_server,message| {
            let mut response_message = new_fixt_message!(Logon);
            response_message.encrypt_method = message.encrypt_method;
            response_message.heart_bt_int = -1;
            response_message.default_appl_ver_id = message.default_appl_ver_id;
            test_server.send_message(response_message);
        });

        //Confirm the client sent a Logout message.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,b"HeartBtInt cannot be negative".to_vec());

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogonHeartBtIntNegativeError = reason { true } else { false });
        });
    }

    //e. Handle receiving any message other than a Logon.
    {
        let (mut test_server,mut client,connection,_) = do_logon(|mut test_server,_| {
            let mut new_order_single = new_fixt_message!(NewOrderSingle);
            new_order_single.cl_ord_id = b"0".to_vec();
            new_order_single.symbol = b"TEST".to_vec();
            new_order_single.security_id = b"0".to_vec();
            new_order_single.security_id_source = Some(SecurityIDSource::CUSIP);
            new_order_single.side = Side::Buy;
            new_order_single.transact_time = new_order_single.sending_time;
            new_order_single.order_qty = b"1".to_vec();
            new_order_single.ord_type = OrdType::Market;
            test_server.send_message(new_order_single);
        });

        //Confirm the client sent a Logout message.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,b"First message not a logon".to_vec());

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogonNotFirstMessageError = reason { true } else { false });
        });
    }
}

#[test]
fn test_1S() {
    define_dictionary!(
        Logout,
        Logon,
        ResendRequest,
        SequenceReset,
    );

    fn do_logon<F1,F2>(logon_message_setup_func: F1,server_response_func: F2) -> (TestStream,Engine,Listener,Connection,Logon)
        where F1: Fn(&mut Logon),
              F2: Fn(&mut Engine,Connection,Box<Logon>) {
        let (mut test_client,mut engine,listener,connection) = TestStream::setup_test_client(build_dictionary());

        let mut logon_message = new_logon_message();
        logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
        logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
        logon_message_setup_func(&mut logon_message);
        test_client.send_message(logon_message.clone());

        engine_poll_event!(engine,EngineEvent::ConnectionLoggingOn(some_listener,some_connection,logon_message) => {
            assert_eq!(some_listener,listener);
            assert_eq!(some_connection,connection);

            server_response_func(&mut engine,some_connection,logon_message);
        });


        (test_client,engine,listener,connection,logon_message)
    }

    //a. Receive a valid Logon message, respond with a Logon message, and if received Logon message
    //has too high of MsgSeqNum, send a ResendRequest.
    {
        let (mut test_client,_engine,_listener,_connection,initial_logon_message) = do_logon(|_|{},|mut engine,connection,logon_message| {
            let mut response_message = new_fixt_message!(Logon);
            response_message.encrypt_method = logon_message.encrypt_method.clone();
            response_message.heart_bt_int = logon_message.heart_bt_int.clone();
            response_message.default_appl_ver_id = logon_message.default_appl_ver_id;
            engine.approve_new_connection(connection,Box::new(response_message),None);
        });

        let message = test_client.recv_message::<Logon>();
        assert_eq!(message.msg_seq_num,1);
        assert!(initial_logon_message.sending_time.signed_duration_since(message.sending_time).num_milliseconds() < 50);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);
        assert_eq!(message.meta.unwrap().message_version,MessageVersion::FIX50SP2);
    }

    //a. continued. Same as above except MsgSeqNum is too high so server should send a
    //ResendRequest after approving logon.
    {
        let (mut test_client,_engine,_listener,_connection,_initial_logon_message) = do_logon(|logon_message: &mut Logon| {
            logon_message.msg_seq_num = 45;
        },
        |mut engine,connection,logon_message| {
            let mut response_message = new_fixt_message!(Logon);
            response_message.encrypt_method = logon_message.encrypt_method.clone();
            response_message.heart_bt_int = logon_message.heart_bt_int.clone();
            response_message.default_appl_ver_id = logon_message.default_appl_ver_id;
            engine.approve_new_connection(connection,Box::new(response_message),20);
        });

        let message = test_client.recv_message::<Logon>();
        assert_eq!(message.msg_seq_num,1);

        let message = test_client.recv_message::<ResendRequest>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.begin_seq_no,20);
        assert!(message.end_seq_no == 0 || message.end_seq_no == 45);
    }

    //a. continued. Same as above except MsgSeqNum is too low so server should ___TODO___
    //TODO: Handle this case.

    //b. Logon message received using the same identity as an existing condition should generate an
    //error and disconnect.
    //This should probably be done by users of the library unless we provide some authentication
    //system.

    //c. Logon message received using an unknown identity should generate an error and disconnect.
    //Just like above, this should probably be done by the library user.

    //d. Client sends an invalid Logon message, so listener should issue an error, respond with
    //Logout message, and then disconnect.
    {
        let (mut test_client,mut engine,_listener,connection,_initial_logon_message) = do_logon(|logon_message: &mut Logon| {
            //Have client send an invalid Logon message.
            logon_message.username = b"Username".to_vec();
            logon_message.password = b"password".to_vec()
        },
        |mut engine,connection,_| {
            engine.reject_new_connection(connection,Some(b"Invalid username and/or password".to_vec()));
        });

        //Confirm engine responded with an appropriate Logout message.
        let message = test_client.recv_message::<Logout>();
        assert_eq!(message.text,b"Invalid username and/or password".to_vec());

        //Give engine thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_client.is_stream_closed(Duration::from_secs(5)));

        //Confirm listener notified that it disconnected..
        engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogonRejectedError = reason { true } else { false });
        });
    }

    //?. Client's first message is not a Logon message, so listener should issue an error and then
    //disconnect.
    {
        let (mut test_client,mut engine,_listener,connection) = TestStream::setup_test_client(build_dictionary());

        //Send a non-Logon message.
        let mut message = new_fixt_message!(FROM_CLIENT ResendRequest);
        message.begin_seq_no = 1;
        message.end_seq_no = 0;
        test_client.send_message(message);

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_client.is_stream_closed(Duration::from_secs(5)));

        //Confirm listener notified that it disconnected.
        engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
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
        Logout,
        Logon,
        ResendRequest,
        TestRequest,
        Heartbeat,
        SequenceReset,
        Reject,
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
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        let mut message = new_fixt_message!(TestRequest);
        message.test_req_id = b"1".to_vec();
        client.send_message(connection,message);
        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,2);

        let mut hb_message = new_fixt_message!(Heartbeat);
        hb_message.msg_seq_num = 2;
        hb_message.test_req_id = message.test_req_id;
        test_server.send_message(hb_message);
        let message = engine_poll_message!(client,connection,Heartbeat);
        assert_eq!(message.msg_seq_num,2);

        let message = new_fixt_message!(Logout);
        client.send_message(connection,message);
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.msg_seq_num,3);

        let mut message = new_fixt_message!(Logout);
        message.msg_seq_num = 3;
        test_server.send_message(message);
        let message = engine_poll_message!(client,connection,Logout);
        assert_eq!(message.msg_seq_num,3);
    }

    //b. Having the server send a message with a MsgSeqNum higher than expected sometime after
    //Logon should cause the client to send a ResendRequest.
    {
        //Connect and logon.
        let (mut test_server,_client,_) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Server sends TestRequest with high MsgSeqNum.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 25;
        message.test_req_id = b"1".to_vec();
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
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Server sends TestRequest with low MsgSeqNum.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 1;
        message.test_req_id = b"1".to_vec();
        test_server.send_message(message);

        //Client should automatically send a Logout with an appropriate text message.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,b"MsgSeqNum too low, expected 2 but received 1".to_vec());

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError = reason { true } else { false });
        });
    }

    //TODO: Handle this with the SeqReset-Reset exception too.

    //d. Logon, send the different types of garbled messages, then a valid message and make sure
    //MsgSeqNum is what's expected.
    for garbled_test_request in garbled_test_requests() {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send garbled message.
        let bytes_written = test_server.stream.write(garbled_test_request).unwrap();
        assert_eq!(bytes_written,garbled_test_request.len());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,_) => {
            assert_eq!(connection,gm_connection);
        });

        //Send valid message.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = b"1".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,3);
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
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send message with high MsgSeqNum to client.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 9;
        message.test_req_id = b"1".to_vec();
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

        let message = engine_poll_message!(client,connection,SequenceReset);
        assert_eq!(message.gap_fill_flag,true);
        assert_eq!(message.new_seq_no,9);
        assert_eq!(message.msg_seq_num,2);

        /* TODO: There seems to be conflicting information about whether the MsgSeqNum that was gap
         * filled should be considered received or still outstanding. If it's still outstanding,
         * then we are breaking the strict ordered processing of messages.
        //Send TestRequest with OrigSendingTime <= SendingTime for MsgSeqNum not already received.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"2".to_vec();
        message.poss_dup_flag = true;
        orig_sending_time_setup_fn(&mut message);
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,b"2".to_vec());
        assert_eq!(message.poss_dup_flag,true);

        //Send the same TestRequest but now MsgSeqNum has already been received. The message should
        //be ignored.
        test_server.send_message(message);
        engine_poll_event!(client,EngineEvent::MessageReceivedDuplicate(msg_connection,duplicate_message) => {
            assert_eq!(msg_connection,connection);

            let message = duplicate_message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,2);
            assert_eq!(message.test_req_id,b"2".to_vec());
            assert_eq!(message.poss_dup_flag,true);
        });
        */
    }

    //f. Similar to (e.) above except OrigSendingTime is greater than SendingTime and MsgSeqNum is
    //as expected. Engine should send a Reject but otherwise increment inbound MsgSeqNum as normal.
    //Engine should also report the error.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send TestRequest with OrigSendingTime > SendingTime.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"2".to_vec();
        message.poss_dup_flag = true;
        message.orig_sending_time = message.sending_time + chrono::Duration::seconds(1);
        test_server.send_message(message);

        //Server should receive Reject with an appropriate reason.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.ref_seq_num,2);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::SendingTimeAccuracyProblem);
        assert_eq!(message.text,b"SendingTime accuracy problem".to_vec());

        engine_poll_event!(client,EngineEvent::MessageRejected(msg_connection,rejected_message) => {
            assert_eq!(msg_connection,connection);

            let message = rejected_message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,2);
            assert_eq!(message.test_req_id,b"2".to_vec());
            assert_eq!(message.poss_dup_flag,true);
        });
    }

    //g. Similar to (f.) except OrigSendingTime is not specified. Engine should respond with a
    //Reject and increment the inbound MsgSeqNum just like when any required field is missing.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send TestRequest without OrigSendingTime.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"2".to_vec();
        message.poss_dup_flag = true;
        test_server.send_message(message);

        //Server should receive Reject with an appropriate reason.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.ref_seq_num,2);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::RequiredTagMissing);
        assert_eq!(message.text,b"Conditionally required tag missing".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);

            match parse_error {
                ParseError::MissingConditionallyRequiredTag(tag,message) => {
                    assert_eq!(tag,OrigSendingTime::tag());

                    let message = message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
                    assert_eq!(message.msg_seq_num,2);
                    assert_eq!(message.test_req_id,b"2".to_vec());
                    assert_eq!(message.poss_dup_flag,true);
                },
                _ => panic!("Wrong parse error"),
            };
        });
    }

    //h. BeginStr should match specified value. This test is basically performed in most other
    //tests so it's skipped here.

    //i. If BeginStr does not match the specified value, Engine should respond with a Logout
    //referencing the incorrect BeginStr value, disconnect, and issue an error.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,build_dictionary());

        //Send TestRequest with wrong BeginStr.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"2".to_vec();
        send_message(&mut test_server.stream,FIXVersion::FIX_4_2,MessageVersion::FIX42,Box::new(message));

        //Engine should send Logout and then disconnect.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,b"BeginStr is wrong, expected 'FIXT.1.1' but received 'FIX.4.2'".to_vec());

        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(
                if let ConnectionTerminatedReason::BeginStrWrongError{received,expected} = reason {
                    assert_eq!(received,FIXVersion::FIX_4_2);
                    assert_eq!(expected,FIXVersion::FIXT_1_1);
                    true
                }
                else {
                    false
                }
            );
        });
    }

    //j. and k. SenderCompID and TargetCompID should match specified values. Otherwise, Engine
    //should respond with a Reject and then Logout.
    {
        //SenderCompID and TargetCompID correct followed by SenderCompID being wrong.
        {
            //Connect and logon.
            let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

            //Send TestRequest without correct SenderCompID and TargetCompID.
            let mut message = new_fixt_message!(TestRequest);
            message.msg_seq_num = 2;
            message.test_req_id = b"1".to_vec();
            message.sender_comp_id = SERVER_SENDER_COMP_ID.to_vec();
            message.target_comp_id = SERVER_TARGET_COMP_ID.to_vec();
            test_server.send_message(message);

            let message = engine_poll_message!(client,connection,TestRequest);
            assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);
            assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);
            let _ = test_server.recv_message::<Heartbeat>();

            //Send TestRequest with wrong SenderCompID.
            let mut message = new_fixt_message!(TestRequest);
            message.msg_seq_num = 3;
            message.test_req_id = b"2".to_vec();
            message.sender_comp_id = b"unknown".to_vec();
            message.target_comp_id = SERVER_SENDER_COMP_ID.to_vec();
            test_server.send_message(message);

            //Engine should send Reject, Logout, and then disconnect.
            let message = test_server.recv_message::<Reject>();
            assert_eq!(message.msg_seq_num,3);
            assert_eq!(message.ref_seq_num,3);
            assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::CompIDProblem);
            assert_eq!(message.text,b"CompID problem".to_vec());

            let message = test_server.recv_message::<Logout>();
            assert_eq!(message.text,b"SenderCompID is wrong".to_vec());

            engine_poll_event!(client,EngineEvent::MessageRejected(msg_connection,rejected_message) => {
                assert_eq!(msg_connection,connection);

                let _ = rejected_message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
            });

            engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
                assert_eq!(terminated_connection,connection);
                assert!(if let ConnectionTerminatedReason::SenderCompIDWrongError = reason { true } else { false });
            });
        }

        //TargetCompID being wrong.
        {
            //Connect and logon.
            let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

            //Send TestRequest with wrong TargetCompID.
            let mut message = new_fixt_message!(TestRequest);
            message.msg_seq_num = 2;
            message.test_req_id = b"2".to_vec();
            message.sender_comp_id = SERVER_SENDER_COMP_ID.to_vec();
            message.target_comp_id = b"unknown".to_vec();
            test_server.send_message(message);

            //Engine should send Reject, Logout, and then disconnect.
            let message = test_server.recv_message::<Reject>();
            assert_eq!(message.msg_seq_num,2);
            assert_eq!(message.ref_seq_num,2);
            assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::CompIDProblem);
            assert_eq!(message.text,b"CompID problem".to_vec());

            let message = test_server.recv_message::<Logout>();
            assert_eq!(message.text,b"TargetCompID is wrong".to_vec());

            engine_poll_event!(client,EngineEvent::MessageRejected(msg_connection,rejected_message) => {
                assert_eq!(msg_connection,connection);

                let _ = rejected_message.as_any().downcast_ref::<TestRequest>().expect("Not expected message type").clone();
            });

            engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
                assert_eq!(terminated_connection,connection);
                assert!(if let ConnectionTerminatedReason::TargetCompIDWrongError = reason { true } else { false });
            });
        }
    }

    //p. MsgType should match those defined in spec or as part of a user defined list. This test is
    //basically performed in most other tests so it's skipped here.

    //q. If MsgType is not defined in the spec or as part of a user defined list, Engine should
    //respond with a Reject, increment the inbound expected MsgSeqNum, and issue a warning.
    {
        define_dictionary!(
            Logon,
            Reject,
            TestRequest,
        );

        define_fixt_message!(MessageWithInvalidMsgType: b"99999" => {
            NOT_REQUIRED, test_req_id: TestReqID [FIX40..],
        });

        let message = new_fixt_message!(MessageWithInvalidMsgType);
        let invalid_msg_type = <MessageWithInvalidMsgType as FIXTMessage>::msg_type(&message);

        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send message with invalid MsgType.
        let mut message = new_fixt_message!(MessageWithInvalidMsgType);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Confirm Engine responds with an appropriate Reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.ref_msg_type,invalid_msg_type);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::InvalidMsgType);
        assert_eq!(message.text,b"Invalid MsgType".to_vec());

        //Confirm Engine issued warning.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::MsgTypeUnknown(_) = parse_error { true } else { false });
        });

        //Confirm Engine incremented expected inbound MsgSeqNum.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,3);
    }

    //r. If MsgType is part of spec but unsupported, Engine should respond with a
    //BusinessMessageReject, increment the inbound expected MsgSeqNum, and issue a warning.
    //TODO: Send Reject (< FIX 4.2) or Business Message Reject (>= FIX 4.2)
    {
        define_dictionary!(
            BusinessMessageReject,
            Logon,
            TestRequest,
        );

        define_fixt_message!(MessageWithUnsupportedMsgType: b"AA" => {
            NOT_REQUIRED, test_req_id: TestReqID [FIX40..],
        });

        //Confirm message type is standard but not supported.
        let message = new_fixt_message!(MessageWithUnsupportedMsgType);
        let unsupported_msg_type = <MessageWithUnsupportedMsgType as FIXTMessage>::msg_type(&message);
        assert!(standard_msg_types().contains(unsupported_msg_type));
        assert!(!build_dictionary().contains_key(unsupported_msg_type));

        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send message with unsupported MsgType.
        let mut message = new_fixt_message!(MessageWithUnsupportedMsgType);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Confirm Engine responds with an appropriate BusinessMessageReject.
        let message = test_server.recv_message::<BusinessMessageReject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.ref_msg_type,unsupported_msg_type);
        assert_eq!(message.business_reject_reason,BusinessRejectReason::UnsupportedMessageType);
        assert_eq!(message.text,b"Unsupported Message Type".to_vec());

        //Confirm Engine issued warning.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::MsgTypeUnknown(_) = parse_error { true } else { false });
        });

        //Confirm Engine incremented expected inbound MsgSeqNum.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,3);
    }

    //l., m.: TODO: BodyLength must be correct. Otherwise, ignore and issue warning.
    //n., o.: TODO: SendingTime must be within 2 minutes of current (atomic click-based) time.
    //              Otherwise, Reject and Logout. Note: This is probably International Atomic Time
    //              (TAI) instead of UTC.
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
        Logon,
        TestRequest,
        Heartbeat,
    );

    //a. Make sure a Heartbeat message is sent automatically after no data is sent after
    //HeartBeatInt seconds.
    {
        //Connect and logon.
        let (mut test_server,_client,_) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Sleep until Heartbeat is triggered.
        thread::sleep(Duration::from_millis(5500));

        //Make sure Heartbeat was sent by client.
        let _ = test_server.recv_message::<Heartbeat>();
    }

    //a. Similar to above but make sure a Heartbeat is not sent after HeartBeatInt seconds when
    //data is sent before HeartBeatInt seconds.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Sleep for half the Heartbeat session.
        thread::sleep(Duration::from_millis(2500));

        //Send message to reset Engine's output heartbeat.
        let mut message = new_fixt_message!(TestRequest);
        message.test_req_id = b"1".to_vec();
        client.send_message(connection,message);
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
        let (mut test_server,_client,_connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send TestRequest.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        //Make sure client responds with Heartbeat.
        let message = test_server.recv_message::<Heartbeat>();
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }
}

#[test]
fn test_5B() {
    //Receive a valid Heartbeat message.

    define_dictionary!(
        Logon,
        Heartbeat,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Server sends Heartbeat to client.
    let mut hb_message = new_fixt_message!(Heartbeat);
    hb_message.msg_seq_num = 2;
    test_server.send_message(hb_message);

    //Engine should accept heartbeat message as normal.
    let message = engine_poll_message!(client,connection,Heartbeat);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.test_req_id,b"".to_vec());
}

#[test]
fn test_6B() {
    define_dictionary!(
        Logon,
        TestRequest,
        Heartbeat,
    );

    //When no data is sent from server to client for HeartBeatInt + "some reasonable period of
    //time", client should send a TestRequest. The server should respond with a matching TestReqID
    //and the client should make this confirmation.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Sleep until TestRequest is triggered.
        thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

        //Ignore HeartBeat because Engine didn't send anything for HeartBeatInt seconds.
        let message = test_server.recv_message::<Heartbeat>();
        assert_eq!(message.msg_seq_num,2);

        //Confirm Engine sent TestRequest.
        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,3);
        let test_req_id = message.test_req_id;

        //Reply with Heartbeat message and make sure client think it's correct.
        let mut hb_message = new_fixt_message!(Heartbeat);
        hb_message.msg_seq_num = 2;
        hb_message.test_req_id = test_req_id.clone();
        test_server.send_message(hb_message);

        let message = engine_poll_message!(client,connection,Heartbeat);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,test_req_id);
    }

    //Same as above but do not respond to TestRequest so client should consider the connection
    //lost.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Sleep until TestRequest is triggered.
        thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

        //Ignore HeartBeat because Engine didn't send anything for HeartBeatInt seconds.
        let _ = test_server.recv_message::<Heartbeat>();

        //Confirm Engine sent TestRequest.
        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,3);

        //Sleep until disconnect.
        thread::sleep(Duration::from_millis(6000)); //1.2 * HeartBeatInt as stated.

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::TestRequestNotRespondedError = reason { true } else { false });
        });
    }
}

#[test]
fn test_7B() {
    //Engine should handle Reject messages just like any other message. Specifically, The inbound
    //MsgSeqNum should be incremented.

    define_dictionary!(
        Logon,
        TestRequest,
        Reject,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Send Reject to client.
    let mut message = new_fixt_message!(Reject);
    message.msg_seq_num = 2;
    message.ref_seq_num = 2;
    test_server.send_message(message);

    //Confirm client received Reject.
    let message = engine_poll_message!(client,connection,Reject);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,2);

    //Confirm MsgSeqNum was incremented by sending another message and making sure it's expected.
    let mut message = new_fixt_message!(TestRequest);
    message.msg_seq_num = 3;
    message.test_req_id = b"test_id".to_vec();
    test_server.send_message(message);

    let message = engine_poll_message!(client,connection,TestRequest);
    assert_eq!(message.msg_seq_num,3);
}

#[test]
fn test_8B() {
    //Wait for Engine to send two non-Logon administrative messages. Send a ResendRequest to client
    //and expect it to respond with a SequenceReset-GapFill.
    //TODO: Need to handle ResendRequest on non-administrative messages too.

    define_dictionary!(
        Logon,
        TestRequest,
        Heartbeat,
        ResendRequest,
        SequenceReset,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

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

    //Respond to ResendRequest with a gap fill.
    engine_gap_fill_resend_request!(client,connection,2..4);

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
        Logon,
        ResendRequest,
        SequenceReset,
        TestRequest,
        Logout,
        Reject,
    );

    //a. Send SequenceReset-GapFill to client with NewSeqNo > MsgSeqNum > expected inbound sequence
    //number. Engine should respond wth ResendRequest for between expected inbound sequence number
    //received MsgSeqNum.
    {
        //Connect.
        let (mut test_server,_client,_) = TestStream::setup_test_server_and_logon(build_dictionary());

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

    //b. Same as above except MsgSeqNum == expected inbound sequence number. Engine should change
    //the next expected inbound sequence number to NewSeqNo.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > MsgSeqNum == expected inbound sequence number.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 2;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        test_server.send_message(message);

        //Confirm client is not buffering the SequenceReset.
        let message = engine_poll_message!(client,connection,SequenceReset);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.gap_fill_flag,true);
        assert_eq!(message.new_seq_no,15);

        //Send a new message to make sure the expected inbound sequence number was changed.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 15;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,15);
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }

    //c. Same as above except MsgSeqNum < expected inbound sequence number and PossDupFlag set to
    //"Y". The client should ignore the message.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > expected inbound sequence number > MsgSeqNum.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 1;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        message.poss_dup_flag = true;
        message.orig_sending_time = message.sending_time;
        test_server.send_message(message);

        //Confirm client ignored the message.
        engine_poll_event!(client,EngineEvent::MessageReceivedDuplicate(msg_connection,duplicate_message) => {
            assert_eq!(msg_connection,connection);

            let message = duplicate_message.as_any().downcast_ref::<SequenceReset>().expect("Not expected message type").clone();
            assert_eq!(message.msg_seq_num,1);
            assert_eq!(message.gap_fill_flag,true);
            assert_eq!(message.new_seq_no,15);
            assert_eq!(message.poss_dup_flag,true);
        });

        //Send a new message to make sure the expected inbound sequence number was NOT changed.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }

    //d. Same as above except PossDupFlag is not set. The client should send a Logout with an
    //appropriate reason, disconnect, and then issue an error.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send SequenceReset-GapFill with NewSeqNo > expected inbound sequence number > MsgSeqNum.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = 1;
        message.gap_fill_flag = true;
        message.new_seq_no = 15;
        test_server.send_message(message);

        //Confirm client sent Logout with an appropriate reason.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,b"MsgSeqNum too low, expected 2 but received 1".to_vec());

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError = reason { true } else { false });
        });
    }

    //e. Send SequenceReset-GapFill to client with NewSeqNo <= MsgSeqNum == expected inbound
    //sequence number. Engine should respond with Reject containing an appropriate message.
    for new_seq_no in 1..3 {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

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
        let mut expected_error_text = b"Attempt to lower sequence number, invalid value NewSeqNo=".to_vec();
        expected_error_text.extend_from_slice(new_seq_no.to_string().as_bytes());
        assert_eq!(message.text,expected_error_text);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::ValueIsIncorrectForThisTag);

        engine_poll_event!(client,EngineEvent::MessageRejected(msg_connection,rejected_message) => {
            assert_eq!(msg_connection,connection);

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
        Logon,
        SequenceReset,
        ResendRequest,
        TestRequest,
        Logout,
        Reject,
    );

    //Try a few msg_seq_nums to make sure they are ignored.
    let msg_seq_nums: Vec<u64> = vec![1,2,800,2000000];

    //a. Engine receives SequenceReset-Reset message with NewSeqNo > inbound expected sequence
    //number. Engine should ignore MsgSeqNum of received message and set inbound expected sequence
    //number to NewSeqNo.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 99999;
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,99999);

        //Make sure client accepts a new message with the new sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 99999;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,99999);
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }

    //a. Same as (a) except confirming that buffered messages are discarded.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Create a message that client will be forced to buffer.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 10;
        message.test_req_id = b"buffer_me".to_vec();
        test_server.send_message(message);
        let _ = test_server.recv_message::<ResendRequest>();

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 99999;
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,99999);

        //Make sure old TestRequest is never accepted and client accepts a new message with the new
        //sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 99999;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,99999);
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }

    //b. Same as (a) except NewSeqNo == inbound expected sequence number. Engine should ignore
    //MsgSeqNum of received message and issue a warning.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 2;
        test_server.send_message(message);

        //Make sure client issued a warning.
        engine_poll_event!(client,EngineEvent::SequenceResetResetHasNoEffect(warning_connection) => {
            assert_eq!(warning_connection,connection);
        });

        //Make sure client accepted the message.
        let message = engine_poll_message!(client,connection,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,2);

        //Make sure client did not change inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }

    //c. Same as (a) except NewSeqNo < inbound expected sequence number. Engine should ignore
    //MsgSeqNum. Engine should respond with Reject message containing an appropriate reason. Engine
    //should not change inbound expected sequence number. Engine should issue an error.
    for msg_seq_num in msg_seq_nums.clone() {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send SequenceReset-Reset to client.
        let mut message = new_fixt_message!(SequenceReset);
        message.msg_seq_num = msg_seq_num;
        message.new_seq_no = 1;
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::SequenceResetResetInThePast(warning_connection) => {
            assert_eq!(warning_connection,connection);
        });

        //Make sure client accepted the message.
        let message = engine_poll_message!(client,connection,SequenceReset);
        assert_eq!(message.msg_seq_num,msg_seq_num);
        assert_eq!(message.gap_fill_flag,false);
        assert_eq!(message.new_seq_no,1);

        //Make sure client replied with Reject message.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.ref_seq_num,msg_seq_num);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::ValueIsIncorrectForThisTag);
        assert_eq!(message.text,b"Attempt to lower sequence number, invalid value NewSeqNo=1".to_vec());

        //Make sure client did not change inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.test_req_id,b"test_id".to_vec());
    }
}

#[test]
fn test_12B() {
    define_dictionary!(
        Logon,
        Logout,
    );

    //Engine should be able to initiate a Logout via the API. The client should wait for a Logout
    //message and then disconnect.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Begin Logout.
        client.logout(connection);

        //Have server respond to Logout.
        let message = test_server.recv_message::<Logout>();
        assert_eq!(message.text,b"".to_vec());

        let mut message = new_fixt_message!(Logout);
        message.msg_seq_num = 2;
        message.session_status = b"4".to_vec(); //Session logout complete.
        test_server.send_message(message);

        //Give client thread a chance to disconnect.
        thread::sleep(Duration::from_millis(500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LocalRequested = reason { true } else { false });
        });
    }

    //Same as above except the server never sends the Logout message. Then the client should
    //disconnect automatically after 10 seconds and issue a warning.
    {
        //Connect and Logon.
        let (test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Begin Logout.
        client.logout(connection);

        //Make sure socket isn't closed immediatelly.
        assert!(!test_server.is_stream_closed(Duration::from_secs(1)));

        //Wait for Logout to timeout.
        thread::sleep(Duration::from_millis(9500));

        //Confirm the client socket disconnected.
        assert!(test_server.is_stream_closed(Duration::from_secs(5)));

        //Confirm client notified that it disconnected.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogoutNoResponseError = reason { true } else { false });
        });
    }
}

#[test]
fn test_13B() {
    define_dictionary!(
        Logon,
        Logout,
    );

    //a. Engine receives Logout response in response to its Logout message and then should
    //disconnect immediately. This is already covered by 12B above.

    //b. Engine receives a Logout message without sending a Logout message first. Engine should
    //respond with a Logout message and wait for server to disconnect.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send Logout to client.
        let mut message = new_fixt_message!(Logout);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Engine should respond with a response Logout message.
        let _ = engine_poll_message!(client,connection,Logout);
        let _ = test_server.recv_message::<Logout>();

        //Server disconnects and client should acknowledge that the connection has been closed.
        //Since client isn't sending anymore data in this state, it can take up to 10 seconds to
        //notice the shutdown.
        let _ = test_server.stream.shutdown(Shutdown::Both);
        thread::sleep(Duration::from_secs(6)); //6 seconds + the duration in engine_poll_event!() >= 10 seconds
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::RemoteRequested = reason { true } else { false });
        });
    }

    //b. Same as above except the server does not disconnect so the client should disconnect after
    //10 seconds and issue an error.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send Logout to client.
        let mut message = new_fixt_message!(Logout);
        message.msg_seq_num = 2;
        test_server.send_message(message);
        let _ = engine_poll_message!(client,connection,Logout);

        //Engine should respond with a response Logout message.
        let _ = test_server.recv_message::<Logout>();

        //Wait around a little bit and make sure client doesn't disconnect instantly.
        thread::sleep(Duration::from_secs(5));
        assert!(!test_server.is_stream_closed(Duration::from_millis(100)));

        //Wait around a little over the full 10 seconds and make sure client does force a
        //disconnect.
        thread::sleep(Duration::from_millis(5500));
        assert!(recv_bytes_with_timeout(&mut test_server.stream,Duration::from_secs(1)).is_none()); //Engine should have stopped sending TestRequests and Heartbeats!
        assert!(test_server.is_stream_closed(Duration::from_secs(1)));
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogoutNoHangUpError = reason { true } else { false });
        });
    }
}

#[test]
fn test_14B() {
    define_dictionary!(
        Logon,
        TestRequest,
        ResendRequest,
        Reject,
        BusinessMessageReject,
    );

    define_fields!(
        BeginSeqNoString: StringFieldType = 7,
        TestReqIDEmpty: NoneFieldType = 112,
        UndefinedField: StringFieldType = 9999999,
    );

    define_fixt_message!(TestRequestWithUndefinedField: b"1" => {
        REQUIRED, test_req_id: TestReqID [FIX40..],
        REQUIRED, undefined: UndefinedField [FIX40..],
    });

    define_fixt_message!(TestRequestWithNotRequiredField: b"1" => {
        NOT_REQUIRED, test_req_id: TestReqID [FIX40..],
    });

    define_fixt_message!(TestRequestWithWrongField: b"1" => {
        REQUIRED, test_req_id: TestReqID [FIX40..],
        REQUIRED, heart_bt_int: HeartBtInt [FIX40..],
    });

    define_fixt_message!(TestRequestWithEmptyField: b"1" => {
        REQUIRED, test_req_id: TestReqIDEmpty [FIX40..],
    });

    define_fixt_message!(ResendRequestWithStringBeginSeqNo: b"2" => {
        REQUIRED, begin_seq_no: BeginSeqNoString [FIX40..],
        REQUIRED, end_seq_no: EndSeqNo [FIX40..],
    });

    define_fixt_message!(TestRequestWithDuplicateField: b"1" => {
        REQUIRED, test_req_id_1: TestReqID [FIX40..],
        REQUIRED, test_req_id_2: TestReqID [FIX40..],
    });

    fn do_garbled_test_with_dict<F: Fn(&mut TestStream,&mut Engine,Connection),TestRequestResponse: FIXTMessage + Any + Clone>(session_reject_reason: SessionRejectReason,ref_tag_id: &'static [u8],test_func: F,dict: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(dict);

        //Perform test by sending message and making sure client reacts correctly.
        test_func(&mut test_server,&mut client,connection);

        //Make sure client responds with an appropriate Reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason.unwrap(),session_reject_reason);
        assert_eq!(message.ref_tag_id,ref_tag_id);

        //Make sure client incremented inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequestResponse);
        assert_eq!(message.msg_seq_num(),3);
    }

    fn do_garbled_test<F: Fn(&mut TestStream,&mut Engine,Connection)>(session_reject_reason: SessionRejectReason,ref_tag_id: &'static [u8],test_func: F) {
        do_garbled_test_with_dict::<F,TestRequest>(session_reject_reason,ref_tag_id,test_func,build_dictionary());
    }

    //a. Send message with tag not defined in spec (tag shouldn't be allowed in any message).
    //Engine should respond with Reject, increment inbound sequence number, and issue an error.
    do_garbled_test(SessionRejectReason::InvalidTagNumber,UndefinedField::tag_bytes(),|test_server,client,connection| {
        //Send message with undefined tag.
        let mut message = new_fixt_message!(TestRequestWithUndefinedField);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        message.undefined = b"undefined".to_vec();
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::UnknownTag(tag) => assert_eq!(tag,UndefinedField::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //b. Send message with a required field missing. Engine should respond with Reject, increment
    //inbound sequence number, and issue an error.
    do_garbled_test(SessionRejectReason::RequiredTagMissing,TestReqID::tag_bytes(),|test_server,client,connection| {
        //Send message with missing required tag.
        let mut message = new_fixt_message!(TestRequestWithNotRequiredField);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::MissingRequiredTag(tag,message) => {
                    assert_eq!(tag,TestReqID::tag());
                    assert_eq!(message.msg_seq_num(),2);
                },
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //c. Send message with defined field but not for the message type. Engine should respond with
    //Reject, increment inbound sequence number, and issue an error.
    do_garbled_test(SessionRejectReason::TagNotDefinedForThisMessageType,HeartBtInt::tag_bytes(),|test_server,client,connection| {
        //Send message with wrong tag for message.
        let mut message = new_fixt_message!(TestRequestWithWrongField);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        message.heart_bt_int = 5;
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::UnexpectedTag(tag) => assert_eq!(tag,HeartBtInt::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //d. Send message with a tag containing no value. Engine should respond with Reject, increment
    //inbound sequence number, and issue an error.
    do_garbled_test(SessionRejectReason::TagSpecifiedWithoutAValue,TestReqIDEmpty::tag_bytes(),|test_server,client,connection| {
        //Send message with valid tag but empty field for message.
        let mut message = new_fixt_message!(TestRequestWithEmptyField);
        message.msg_seq_num = 2;
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::NoValueAfterTag(tag) => assert_eq!(tag,TestReqIDEmpty::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //e. Send message with an incorrect value (not in an enumerated set) for a field. Engine should
    //respond with Reject, increment inbound sequence number, and issue an error.
    {
        define_fixt_message!(TestRequestWithEnumeratedField: b"1" => {
            REQUIRED, test_req_id: TestReqID [FIX40..],
            NOT_REQUIRED, enumerated_field: SideField [FIX40..],
        });

        define_fields!(
            SideChar: CharFieldType = 54,
        );

        define_fixt_message!(TestRequestWithIncorrectField: b"1" => {
            REQUIRED, test_req_id: TestReqID [FIX40..],
            NOT_REQUIRED, enumerated_field: SideChar [FIX40..],
        });

        define_dictionary!(
            Logon,
            TestRequestWithEnumeratedField,
            ResendRequest,
            Reject,
        );

        do_garbled_test_with_dict::<_,TestRequestWithEnumeratedField>(SessionRejectReason::ValueIsIncorrectForThisTag,SideField::tag_bytes(),|test_server,client,connection| {
            //Send message with incorrect value.
            let mut message = new_fixt_message!(TestRequestWithIncorrectField);
            message.test_req_id = b"test_id".to_vec();
            message.enumerated_field = b'Z';
            test_server.send_message(message);

            //Make sure client issued an error.
            engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
                assert_eq!(gm_connection,connection);
                match parse_error {
                    ParseError::OutOfRangeTag(tag) => assert_eq!(tag,SideField::tag()),
                    _ => panic!("Wrong parse error"),
                };
            });
        },build_dictionary());
    }

    //f. Send message with an incorrect data format for a field. Engine should respond with Reject,
    //increment inbound sequence number, and issue an error.
    do_garbled_test(SessionRejectReason::IncorrectDataFormatForValue,BeginSeqNoString::tag_bytes(),|test_server,client,connection| {
        //Send message with incorrect value.
        let mut message = new_fixt_message!(ResendRequestWithStringBeginSeqNo);
        message.msg_seq_num = 2;
        message.begin_seq_no = b"-1".to_vec();
        message.end_seq_no = 0;
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::WrongFormatTag(tag) => assert_eq!(tag,BeginSeqNoString::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //g. Send message with one of the header fields after after the body fields or the trailer
    //fields are not at the end. Engine should respond with Reject, increment inbound sequence
    //number, and issue an error.
    //This test seems to somewhat contradict page 26 of FIXT Version 1.1 that states a garbled
    //message should be ignored. We are opting to respond with a Reject whenever possible to ease
    //debugging since this shouldn't occur in a live system.
    for message_bytes in vec![
        b"49=TEST\x018=FIX.4.2\x019=38\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01", //BeginStr is not the first tag.
        b"8=FIX.4.2\x0149=TEST\x019=38\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01", //BodyLength is not the second tag.
        b"8=FIX.4.2\x019=38\x0149=TEST\x0135=1\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01112=1\x0110=204\x01", //MsgType is not the third tag.
        b"8=FIX.4.2\x019=38\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x0110=204\x01112=1\x01" //Checksum is not the last tag.
    ] {
        do_garbled_test(SessionRejectReason::TagSpecifiedOutOfRequiredOrder,b"",|test_server,client,connection| {
            //Send message.
            let bytes_written = test_server.stream.write(message_bytes).unwrap();
            assert_eq!(bytes_written,message_bytes.len());

            //Make sure client issued an error.
            engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
                assert_eq!(gm_connection,connection);
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

    //h. Send message with a tag duplicated outside of an appropriate repeating group. Engine
    //should respond with Reject, increment inbound sequence number, and issue an error.
    do_garbled_test(SessionRejectReason::TagAppearsMoreThanOnce,TestReqID::tag_bytes(),|test_server,client,connection| {
        //Send message with duplicate tag.
        let mut message = new_fixt_message!(TestRequestWithDuplicateField);
        message.msg_seq_num = 2;
        message.test_req_id_1 = b"test_id_1".to_vec();
        message.test_req_id_2 = b"test_id_2".to_vec();
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::DuplicateTag(tag) => assert_eq!(tag,TestReqID::tag()),
                _ => panic!("Wrong parse error"),
            };
        });
    });

    //i. Send message with repeating groups that don't match the specified count. Engine should
    //respond with Reject, increment inbound sequence number, and issue an error.
    {
        let mut messages_bytes: Vec<(FieldTag,&'static[u8],&'static [u8])> = Vec::new();
        messages_bytes.push((NoHops::tag(),NoHops::tag_bytes(),b"8=FIX.4.3\x019=999\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01627=2\x01112=1\x0110=204\x01")); //Claim two groups but have zero.
        messages_bytes.push((TestReqID::tag(),TestReqID::tag_bytes(),b"8=FIX.4.3\x019=999\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01627=2\x01628=1\x01112=1\x0110=204\x01")); //Claim two groups but have one.
        messages_bytes.push((HopCompID::tag(),HopCompID::tag_bytes(),b"8=FIX.4.3\x019=999\x0135=1\x0149=TEST\x0156=TX\x0134=1\x0152=20090107-18:15:16\x01627=2\x01628=1\x01628=2\x01628=3\x01112=1\x0110=204\x01")); //Claim two groups but have three.
        for (ref_tag_id,ref_tag_id_bytes,message_bytes) in messages_bytes {
            do_garbled_test(SessionRejectReason::IncorrectNumInGroupCountForRepeatingGroup,ref_tag_id_bytes,|test_server,client,connection| {
                //Send message.
                let bytes_written = test_server.stream.write(message_bytes).unwrap();
                assert_eq!(bytes_written,message_bytes.len());

                //Make sure client issued an error.
                engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
                    assert_eq!(gm_connection,connection);
                    match parse_error {
                        ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,ref_tag_id),
                        ParseError::RepeatingGroupTagWithNoRepeatingGroup(tag) => assert_eq!(tag,ref_tag_id),
                        ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(tag) => assert_eq!(tag,ref_tag_id),
                        _ => panic!("Wrong parse error: {}",&parse_error),
                    };
                });
            });
        }
    }

    //j. Send message with repeating groups but the first required field is not at the beginning of
    //the repeating group. Engine should Reject, increment inbound sequence number, and issue an
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

    //m. Send message with a conditionally required field missing. Engine should respond with
    //Business Message Reject referencing the conditionally missing field, increment MsgSeqNum,
    //and issue an error.
    //Note: FIXT 1.1 implies responding with a Business Message Reject but page 106 of FIX Version
    //5.0 Service Pack 2 Volume 1 says Business Message Reject must only be used for a certain set
    //of messages.
    //TODO: Once we support business messages, in addition to the test below, test that they use a
    //BusinessMessageReject instead.
    {
        //Connect and Logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send message with conditionally required field (OrigSendingTime) missing.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 2;
        message.test_req_id = b"test_id".to_vec();
        message.poss_dup_flag = true;
        test_server.send_message(message);

        //Make sure client issued an error.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(gm_connection,parse_error) => {
            assert_eq!(gm_connection,connection);
            match parse_error {
                ParseError::MissingConditionallyRequiredTag(tag,_) => assert_eq!(tag,OrigSendingTime::tag()),
                _ => panic!("Wrong parse error"),
            };
        });

        //Make sure client responds with an appropriate Reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::RequiredTagMissing);
        assert_eq!(message.ref_msg_type,<TestRequest as MessageDetails>::msg_type());
        assert_eq!(message.ref_tag_id,OrigSendingTime::tag_bytes());

        //Make sure client incremented inbound sequence number.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = 3;
        message.test_req_id = b"test_id".to_vec();
        test_server.send_message(message);

        let message = engine_poll_message!(client,connection,TestRequest);
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
#[ignore]
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
#[ignore]
fn test_19B() {
    //TODO: This is Message specific functionality that has to wait until we support non-admin
    //messages.
    unimplemented!();
}

#[test]
fn test_20B() {
    define_dictionary!(
        Logon,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Client sends a ResendRequest and then receives a ResendRequest. The client should resend
    //requested messages and then send a new ResendRequest for the remaining missing messages.

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Have client send a few messages to server without server acknowledging them.
    for x in 2..6 {
        let mut message = new_fixt_message!(TestRequest);
        message.test_req_id = x.to_string().as_bytes().to_vec();
        client.send_message(connection,message);

        let message = test_server.recv_message::<TestRequest>();
        assert_eq!(message.msg_seq_num,x);
    }

    //Trigger client to send a ResendRequest.
    let mut message = new_fixt_message!(TestRequest);
    message.msg_seq_num = 10;
    message.test_req_id = b"10".to_vec();
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

    //Reply to ResendRequest with a gap fill.
    engine_gap_fill_resend_request!(client,connection,2..6);

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
