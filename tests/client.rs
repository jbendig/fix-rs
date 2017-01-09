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

#[macro_use]
extern crate fix_rs;
extern crate mio;

use mio::tcp::Shutdown;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[macro_use]
mod common;
use common::{SERVER_SENDER_COMP_ID,SERVER_TARGET_COMP_ID,TestServer,new_logon_message};
use fix_rs::dictionary::field_types::other::SessionRejectReason;
use fix_rs::dictionary::fields::{SenderCompID,TargetCompID,Text};
use fix_rs::dictionary::messages::{Heartbeat,Logon,Logout,Reject,ResendRequest,SequenceReset,TestRequest};
use fix_rs::field::Field;
use fix_rs::fix::ParseError;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt::client::{ClientEvent,ConnectionTerminatedReason};
use fix_rs::fixt::tests::{INBOUND_MESSAGES_BUFFER_LEN_MAX,INBOUND_BYTES_BUFFER_CAPACITY};
use fix_rs::fixt::message::FIXTMessage;
use fix_rs::message::{NOT_REQUIRED,REQUIRED,Message};
use fix_rs::message_version::MessageVersion;

#[test]
fn test_recv_resend_request_invalid_end_seq_no() {
    define_dictionary!(
        Logon : Logon,
        ResendRequest : ResendRequest,
        Reject : Reject,
    );

    //Connect and Logon.
    let (mut test_server,_client,_) = TestServer::setup_and_logon(build_dictionary());

    //Send ResendRequest to client with EndSeqNo < BeginSeqNo.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 5;
    message.begin_seq_no = 2;
    message.end_seq_no = 1;
    test_server.send_message(message);

    //Make sure client responds with an appropriate Reject.
    let message = test_server.recv_message::<Reject>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,5);
    assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::ValueIsIncorrectForThisTag);
}

#[test]
fn test_send_logout_before_logon() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
    );

    let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

    //Send Logout immediately.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 1;
    test_server.send_message(message);

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

#[test]
fn test_recv_logout_with_high_msg_seq_num() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Send Logout with a high MsgSeqNum
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 15;
    test_server.send_message(message);

    //Make sure client tries to retrieve the missing messages.
    let message = test_server.recv_message::<ResendRequest>();
    assert_eq!(message.begin_seq_no,2);
    assert!(message.end_seq_no == 0 || message.end_seq_no == 14);

    //Respond with gap-fill.
    let mut message = new_fixt_message!(SequenceReset);
    message.gap_fill_flag = true;
    message.new_seq_no = 15;
    message.msg_seq_num = 2;
    test_server.send_message(message);
    let _ = client_poll_message!(client,connection_id,SequenceReset);

    //Make sure client responds with Logout now that it's caught up.
    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.msg_seq_num,3);

    //Close connection and make sure client notifies that connection closed cleanly.
    let _ = test_server.stream.shutdown(Shutdown::Both);
    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::ServerRequested = reason { true } else { false });
    });
}

#[test]
fn test_recv_logout_with_high_msg_seq_num_and_no_reply() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Send Logout with a high MsgSeqNum
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 15;
    test_server.send_message(message);

    //Make sure client tries to retrieve the missing messages.
    let message = test_server.recv_message::<ResendRequest>();
    assert_eq!(message.begin_seq_no,2);
    assert!(message.end_seq_no == 0 || message.end_seq_no == 14);

    //Wait around without replying to ResendRequest.
    thread::sleep(Duration::from_millis(10500));

    //Make sure client responds with Logout even though it didn't get caught up.
    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.msg_seq_num,3);

    //Close connection and make sure client notifies that connection closed cleanly.
    let _ = test_server.stream.shutdown(Shutdown::Both);
    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::ServerRequested = reason { true } else { false });
    });
}

#[test]
fn test_recv_logout_send_logout_recv_resend_request() {
    define_dictionary!(
        Heartbeat : Heartbeat,
        Logon : Logon,
        Logout : Logout,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
        TestRequest : TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Send Logout to client.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 2;
    test_server.send_message(message);

    //Make sure client responds with Logout.
    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.msg_seq_num,2);
    let _ = client_poll_message!(client,connection_id,Logout);

    //Ask client for missing messages even though they already responded to Logout. This should
    //cancel the logout when done before the timeout.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 3;
    message.begin_seq_no = 2;
    message.end_seq_no = 0;
    test_server.send_message(message);
    let _ = client_poll_message!(client,connection_id,ResendRequest);

    //Make sure ResendRequest is responded to.
    let message = test_server.recv_message::<SequenceReset>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.new_seq_no,3);

    //Wait around to make sure the server requested logout was cancelled.
    thread::sleep(Duration::from_millis(5500));
    let _ = test_server.recv_message::<Heartbeat>();
    let _ = test_server.recv_message::<TestRequest>();

    //Try and logout again but cleanly this time.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 4;
    test_server.send_message(message);
    let _ = client_poll_message!(client,connection_id,Logout);

    //Close connection and make sure client notifies that connection closed cleanly.
    let _ = test_server.stream.shutdown(Shutdown::Both);
    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::ServerRequested = reason { true } else { false });
    });
}

#[test]
fn test_send_logout_and_recv_resend_request() {
    define_dictionary!(
        Heartbeat : Heartbeat,
        Logon : Logon,
        Logout : Logout,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
        TestRequest : TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Wait around for a Heartbeat and TestRequest. Ignore these so we can send a valid
    //ResendRequest below.
    thread::sleep(Duration::from_millis(5500));
    let _ = test_server.recv_message::<Heartbeat>();
    let _ = test_server.recv_message::<TestRequest>();

    //Begin Logout.
    client.logout(connection_id);
    let _ = test_server.recv_message::<Logout>();

    //Have server send a ResendRequest.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 2;
    message.begin_seq_no = 2;
    message.end_seq_no = 0;
    test_server.send_message(message);
    let _ = client_poll_message!(client,connection_id,ResendRequest);

    //Make sure client still responds to ResendRequest while logging out.
    let message = test_server.recv_message::<SequenceReset>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.new_seq_no,5);

    //Respond to logout and make sure client still logs out cleanly.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 3;
    test_server.send_message(message);

    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::ClientRequested = reason { true } else { false });
    });
}

#[test]
fn test_send_logout_and_recv_logout_with_high_msg_seq_num() {
    define_dictionary!(
        Heartbeat : Heartbeat,
        Logon : Logon,
        Logout : Logout,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
        TestRequest : TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Begin Logout.
    client.logout(connection_id);
    let _ = test_server.recv_message::<Logout>();

    //Respond with Logout containing high MsgSeqNum.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 15;
    test_server.send_message(message);

    //Make sure client requests missing messages.
    let message = test_server.recv_message::<ResendRequest>();
    assert_eq!(message.msg_seq_num,3);
    assert_eq!(message.begin_seq_no,2);
    assert!(message.end_seq_no == 0 || message.end_seq_no == 15);

    //Tell client about missing messages.
    let mut message = new_fixt_message!(SequenceReset);
    message.gap_fill_flag = true;
    message.msg_seq_num = 2;
    message.new_seq_no = 16;
    test_server.send_message(message);
    let _ = client_poll_message!(client,connection_id,SequenceReset);

    //Make sure client automatically attempts to logout again after being caught up.
    let _ = test_server.recv_message::<Logout>();

    //Finish logging out cleanly.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 16;
    test_server.send_message(message);

    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::ClientRequested = reason { true } else { false });
    });
}

#[test]
fn test_send_logout_and_recv_logout_with_high_msg_seq_num_and_no_reply() {
    define_dictionary!(
        Heartbeat : Heartbeat,
        Logon : Logon,
        Logout : Logout,
        ResendRequest : ResendRequest,
        SequenceReset : SequenceReset,
        TestRequest : TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Begin Logout.
    client.logout(connection_id);
    let _ = test_server.recv_message::<Logout>();

    //Respond with Logout containing high MsgSeqNum.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 15;
    test_server.send_message(message);

    //Make sure client requests missing messages.
    let message = test_server.recv_message::<ResendRequest>();
    assert_eq!(message.msg_seq_num,3);
    assert_eq!(message.begin_seq_no,2);
    assert!(message.end_seq_no == 0 || message.end_seq_no == 15);

    //Wait around without replying to ResendRequest.
    thread::sleep(Duration::from_millis(10500));

    //Make sure client disconnects instead of retrying the logout process. If the other end sends a
    //logout with an expected MsgSeqNum, then we saw a later MsgSeqNum once already and something
    //has gone terribly wrong. If the other sends a further out MsgSeqNum but won't reply to
    //ResendRequest then we're just going to keep looping.
    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::LogoutNoResponseError = reason { true } else { false });
    });
}

#[test]
fn test_wrong_sender_comp_id_in_logon_response() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
        Reject : Reject,
    );

    //Connect and attempt logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

    let message = new_logon_message();
    client.send_message(connection_id,message);
    let _ = test_server.recv_message::<Logon>();

    //Respond with a logon messaging containing the wrong SenderCompID.
    let mut message = new_logon_message();
    message.sender_comp_id = String::from("unknown");
    test_server.send_message(message);

    //Confirm client sends Reject, Logout, and then disconnects.
    let message = test_server.recv_message::<Reject>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,1);
    assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::CompIDProblem);
    assert_eq!(message.text,"CompID problem");

    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.text,"SenderCompID is wrong");

    client_poll_event!(client,ClientEvent::MessageRejected(msg_connection_id,rejected_message) => {
        assert_eq!(msg_connection_id,connection_id);

        let message = rejected_message.as_any().downcast_ref::<Logon>().expect("Not expected message type").clone();
        assert_eq!(message.msg_seq_num,1);
        assert_eq!(message.sender_comp_id,"unknown");
    });

    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::SenderCompIDWrongError = reason { true } else { false });
    });
}

#[test]
fn test_wrong_target_comp_id_in_logon_response() {
    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
        Reject : Reject,
    );

    //Connect and attempt logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

    let message = new_logon_message();
    client.send_message(connection_id,message);
    let _ = test_server.recv_message::<Logon>();

    //Respond with a logon messaging containing the wrong TargetCompID.
    let mut message = new_logon_message();
    message.target_comp_id = String::from("unknown");
    test_server.send_message(message);

    //Confirm client sends Reject, Logout, and then disconnects.
    let message = test_server.recv_message::<Reject>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,1);
    assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::CompIDProblem);
    assert_eq!(message.text,"CompID problem");

    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.text,"TargetCompID is wrong");

    client_poll_event!(client,ClientEvent::MessageRejected(msg_connection_id,rejected_message) => {
        assert_eq!(msg_connection_id,connection_id);

        let message = rejected_message.as_any().downcast_ref::<Logon>().expect("Not expected message type").clone();
        assert_eq!(message.msg_seq_num,1);
        assert_eq!(message.target_comp_id,"unknown");
    });

    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
        assert!(if let ConnectionTerminatedReason::TargetCompIDWrongError = reason { true } else { false });
    });
}

#[test]
fn test_overflowing_inbound_messages_buffer_does_resume() {
    //To prevent the client thread from stalling when receiving messages faster than they can be
    //parsed, it will automatically stop receiving bytes and parsing them into messages once
    //INBOUND_MESSAGES_BUFFER_LEN_MAX messages have been parsed. This test makes sure the client
    //thread resumes parsing bytes that have already been read in but not parsed without waiting
    //for a new network notification.

    define_dictionary!(
        Logon : Logon,
        Heartbeat : Heartbeat,
        TestRequest : TestRequest,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

    //Send INBOUND_MESSAGES_BUFFER_LEN_MAX + 1 TestRequests (hopefully) merged into a single TCP
    //frame.
    let mut bytes = Vec::new();
    for x in 0..INBOUND_MESSAGES_BUFFER_LEN_MAX + 1 {
        let mut test_request_message = new_fixt_message!(TestRequest);
        test_request_message.msg_seq_num = (x + 2) as u64;
        test_request_message.test_req_id = String::from("test");

        test_request_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    }
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    assert!(bytes.len() < INBOUND_BYTES_BUFFER_CAPACITY); //Make sure client thread can theoretically store all of the messages in a single recv().
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges messages as normal.
    for x in 0..INBOUND_MESSAGES_BUFFER_LEN_MAX + 1 {
        let message = client_poll_message!(client,connection_id,TestRequest);
        assert_eq!(message.msg_seq_num,(x + 2) as u64);

        let message = test_server.recv_message::<Heartbeat>();
        assert_eq!(message.msg_seq_num,(x + 2) as u64);
    }
}

#[test]
fn test_sender_comp_id() {
    define_fixt_message!(TestMessage: b"9999" => {
        NOT_REQUIRED, text: Text [FIX50..],
    });

    define_dictionary!(
        Logon : Logon,
        Reject : Reject,
        TestMessage : TestMessage,
    );

    //FIXT.1.1: Make sure SenderCompID has to be the fourth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50,build_dictionary());

        //Accept when SenderCompID is the fourth tag.
        let target_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=236\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);

        //Reject when SenderCompID is the fifth tag.
        let sender_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0156=TEST\x0149=TX\x0134=3\x0152=20170105-01:01:01\x0110=012\x01";
        let bytes_written = test_server.stream.write(sender_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,sender_comp_id_fifth_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,"SenderCompID must be the 4th tag");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::SenderCompIDNotFourthTag = parse_error { true } else { false });
        });

        //Reject when SenderCompID is missing.
        let missing_sender_comp_id_tag_message = b"8=FIXT.1.1\x019=50\x0135=9999\x0156=TEST\x0134=10\x0152=20170105-01:01:01\x0110=086\x01";
        let bytes_written = test_server.stream.write(missing_sender_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_sender_comp_id_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,"SenderCompID must be the 4th tag");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::SenderCompIDNotFourthTag = parse_error { true } else { false });
        });
    }

    //FIX.4.0: Make sure SenderCompID does not have to be the fourth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon_with_ver(FIXVersion::FIX_4_0,MessageVersion::FIX40,build_dictionary());

        //Accept when SenderCompID is the fourth tag.
        let target_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=154\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);

        //Accept when SenderCompID is the fifth tag.
        let sender_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0156=TEST\x0149=TX\x0134=3\x0152=20170105-01:01:01\x0110=155\x01";
        let bytes_written = test_server.stream.write(sender_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,sender_comp_id_fifth_tag_message.len());

        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);

        //Reject when SenderCompID is missing.
        let missing_sender_comp_id_tag_message = b"8=FIX.4.0\x019=42\x0135=9999\x0156=TEST\x0134=4\x0152=20170105-01:01:01\x0110=063\x01";
        let bytes_written = test_server.stream.write(missing_sender_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_sender_comp_id_tag_message.len());

       let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.text,"Required tag missing");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::MissingRequiredTag(ref tag,_) = parse_error { *tag == SenderCompID::tag() } else { false });
        });
    }
}

#[test]
fn test_target_comp_id() {
    define_fixt_message!(TestMessage: b"9999" => {
        NOT_REQUIRED, text: Text [FIX50..],
    });

    define_dictionary!(
        Logon : Logon,
        Reject : Reject,
        TestMessage : TestMessage,
    );

    //FIXT.1.1: Make sure TargetCompID has to be the fifth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50,build_dictionary());

        //Accept when TargetCompID is the fifth tag.
        let target_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=236\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);

        //Reject when TargetCompID is the sixth tag.
        let target_comp_id_sixth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0149=TX\x0134=3\x0156=TEST\x0152=20170105-01:01:01\x0110=237\x01";
        let bytes_written = test_server.stream.write(target_comp_id_sixth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_sixth_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,"TargetCompID must be the 5th tag");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::TargetCompIDNotFifthTag = parse_error { true } else { false });
        });

        //Reject when TargetCompID is missing.
        let missing_target_comp_id_tag_message = b"8=FIXT.1.1\x019=59\x0135=9999\x0149=TX\x0134=3\x0152=20170105-01:01:01\x0110=086\x01";
        let bytes_written = test_server.stream.write(missing_target_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_target_comp_id_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,"TargetCompID must be the 5th tag");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::TargetCompIDNotFifthTag = parse_error { true } else { false });
        });
    }

    //FIX.4.0: Make sure TargetCompID does not have to be the fifth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon_with_ver(FIXVersion::FIX_4_0,MessageVersion::FIX40,build_dictionary());

        //Accept when TargetCompID is the fifth tag.
        let target_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=154\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);

        //Accept when TargetCompID is the sixth tag.
        let target_comp_id_sixth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0149=TX\x0134=3\x0156=TEST\x0152=20170105-01:01:01\x0110=155\x01";
        let bytes_written = test_server.stream.write(target_comp_id_sixth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_sixth_tag_message.len());

        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);

        //Reject when TargetCompID is missing.
        let missing_target_comp_id_tag_message = b"8=FIX.4.0\x019=40\x0135=9999\x0149=TX\x0134=4\x0152=20170105-01:01:01\x0110=171\x01";
        let bytes_written = test_server.stream.write(missing_target_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_target_comp_id_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.text,"Required tag missing");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::MissingRequiredTag(ref tag,_) = parse_error { *tag == TargetCompID::tag() } else { false });
        });
    }
}

#[test]
fn test_default_appl_ver_id() {
    define_fixt_message!(TestMessage: b"9999" => {
        REQUIRED, text: Text [FIX50..],
    });

    define_fixt_message!(TestMessage2: b"9999" => {
        REQUIRED, text: Text [FIX40..],
    });

    define_dictionary!(
        Logon : Logon,
        TestMessage : TestMessage,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX40,build_dictionary());

    //Make sure DefaultApplVerID is respected for sent messages.
    {
        //Make client send a TestMessage.
        let mut message = new_fixt_message!(TestMessage);
        message.text = String::from("text");
        client.send_message(connection_id,message);

        //Confirm text field was excluded by server due to requiring >= FIX50 but default is FIX40.
        let message = test_server.recv_message::<TestMessage>();
        assert_eq!(message.text.len(),0);
    }

    //Make sure DefaultApplVerID is respected for received messages.
    {
        //Make server send a TestMessage.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 2;
        message.text = String::from("text");
        test_server.send_message(message);

        //Confirm text field was excluded by client due to requiring >= FIX50 but default is FIX40.
        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.text.len(),0);

        //Make sever send a TestMessage again but force the text field to be sent.
        let mut message = new_fixt_message!(TestMessage2);
        message.msg_seq_num = 3;
        message.text = String::from("text");
        test_server.send_message(message);

        //Make sure message is considered invalid.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::UnknownTag(ref tag) = parse_error { *tag == b"58" } else { false });
        });
    }
}

#[test]
fn test_appl_ver_id() {
    define_fixt_message!(TestMessage: b"9999" => {
        REQUIRED, text: Text [FIX50..],
    });

    define_dictionary!(
        Logon : Logon,
        Reject : Reject,
        TestMessage : TestMessage,
    );

    //Make sure when ApplVerID is specified after the sixth field, Client responds with an
    //appropriate Reject message and notification.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send TestMessage with ApplVerID field as the seventh tag.
        let appl_ver_id_seventh_tag_message = b"8=FIXT.1.1\x019=44\x0135=9999\x0149=SERVER\x0156=CLIENT\x0134=2\x011128=9\x0110=000\x01";
        let bytes_written = test_server.stream.write(appl_ver_id_seventh_tag_message).unwrap();
        assert_eq!(bytes_written,appl_ver_id_seventh_tag_message.len());

        //Make sure Client responds with an appropriate reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,"ApplVerID must be the 6th tag if specified");

        //Make sure Client indicates that it rejected the message.
        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::ApplVerIDNotSixthTag = parse_error { true } else { false });
        });
    }

    //Make sure ApplVerID overrides the default message version set in the initial Logon message.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection_id) = TestServer::setup_and_logon(build_dictionary());

        //Send TestMessage with ApplVerID < FIX50 and without text field.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 2;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        test_server.send_message_with_ver(FIXVersion::FIXT_1_1,message.appl_ver_id.unwrap(),message);

        //Confirm Client accepted message correctly.
        let message = client_poll_message!(client,connection_id,TestMessage);
        assert_eq!(message.appl_ver_id,Some(MessageVersion::FIX40));
        assert_eq!(message.text.len(),0);

        //Send TestMessage with ApplVerID < FIX50 and with text field.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 3;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        message.text = String::from("text");
        test_server.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message); //Force text field to be included.

        //Confirm Client rejected message because text field is unsupported for this version.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::TagNotDefinedForThisMessageType);
        assert_eq!(message.text,"Tag not defined for this message type");

        client_poll_event!(client,ClientEvent::MessageReceivedGarbled(msg_connection_id,parse_error) => {
            assert_eq!(msg_connection_id,connection_id);
            assert!(if let ParseError::UnexpectedTag(ref tag) = parse_error { *tag == Text::tag()  } else { false });
        });
    }
}

#[test]
fn test_respond_to_test_request_immediately_after_logon() {
    //Special processing is required to adjust which messages are acceptable after Logon is
    //received. But the IO processing is level based so the event loop might not be notified of
    //remaining data. This test makes sure the remaining data is processed immediately. In
    //practice, the worst case scenario is some type of timeout would trigger a Heartbeat or a
    //TestRequest that would cause the remaining data to be read.

    define_dictionary!(
        Logon : Logon,
        Heartbeat : Heartbeat,
        TestRequest : TestRequest,
    );

    //Connect to server.
    let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

    //Have client send Logon.
    client.send_message_box(connection_id,Box::new(new_logon_message()));
    let message = test_server.recv_message::<Logon>();
    assert_eq!(message.msg_seq_num,1);

    //Respond with Logon and TestRequest (hopefully) merged into a single TCP packet.
    let mut logon_message = new_fixt_message!(Logon);
    logon_message.msg_seq_num = 1;
    logon_message.encrypt_method = message.encrypt_method;
    logon_message.heart_bt_int = message.heart_bt_int;
    logon_message.default_appl_ver_id = message.default_appl_ver_id;

    let mut test_request_message = new_fixt_message!(TestRequest);
    test_request_message.msg_seq_num = 2;
    test_request_message.test_req_id = String::from("test");

    let mut bytes = Vec::new();
    logon_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    test_request_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges both as normal.
    client_poll_event!(client,ClientEvent::SessionEstablished(_) => {});
    let message = client_poll_message!(client,connection_id,Logon);
    assert_eq!(message.msg_seq_num,1);
    let message = client_poll_message!(client,connection_id,TestRequest);
    assert_eq!(message.msg_seq_num,2);

    let message = test_server.recv_message::<Heartbeat>();
    assert_eq!(message.msg_seq_num,2);
}

#[test]
fn test_respect_default_appl_ver_id_in_test_request_immediately_after_logon() {
    //This is very similar to test_respond_to_test_request_immediately_after_logon() above except
    //it makes sure the DefaultApplVerID is used correctly for the message right after Logon.

    define_fixt_message!(TestMessage: b"9999" => {
        REQUIRED, text: Text [FIX50SP2..],
    });

    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
        Reject : Reject,
        TestMessage : TestMessage,
    );

    //Connect to server.
    let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

    //Have client send Logon.
    let mut logon_message = new_logon_message();
    logon_message.default_appl_ver_id = MessageVersion::FIX50SP2;
    client.send_message_box(connection_id,Box::new(logon_message));
    let message = test_server.recv_message::<Logon>();
    assert_eq!(message.msg_seq_num,1);

    //Respond with Logon and TestMessage (hopefully) merged into a single TCP packet.
    let mut logon_message = new_fixt_message!(Logon);
    logon_message.msg_seq_num = 1;
    logon_message.encrypt_method = message.encrypt_method;
    logon_message.heart_bt_int = message.heart_bt_int;
    logon_message.default_appl_ver_id = message.default_appl_ver_id;

    let mut test_message = new_fixt_message!(TestMessage);
    test_message.msg_seq_num = 2;
    test_message.text = String::from("test");

    let mut bytes = Vec::new();
    logon_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    test_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges Logon as normal.
    client_poll_event!(client,ClientEvent::SessionEstablished(_) => {});
    let message = client_poll_message!(client,connection_id,Logon);
    assert_eq!(message.msg_seq_num,1);

    //Make sure client applies DefaultApplVerID version to TestMessage so that the Text field is
    //parsed.
    let message = client_poll_message!(client,connection_id,TestMessage);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.text,String::from("test"));
}

#[test]
fn test_logout_and_terminate_wrong_versioned_test_request_immediately_after_logon() {
    //This is very similar to test_respond_to_test_request_immediately_after_logon() above except
    //it makes sure using the wrong FIX version follows the typical Logout and disconnect as
    //expected.

    define_dictionary!(
        Logon : Logon,
        Logout : Logout,
        TestRequest : TestRequest,
    );

    //Connect to server.
    let (mut test_server,mut client,connection_id) = TestServer::setup(build_dictionary());

    //Have client send Logon.
    client.send_message_box(connection_id,Box::new(new_logon_message()));
    let message = test_server.recv_message::<Logon>();
    assert_eq!(message.msg_seq_num,1);

    //Respond with Logon and TestRequest (hopefully) merged into a single TCP packet.
    let mut logon_message = new_fixt_message!(Logon);
    logon_message.msg_seq_num = 1;
    logon_message.encrypt_method = message.encrypt_method;
    logon_message.heart_bt_int = message.heart_bt_int;
    logon_message.default_appl_ver_id = message.default_appl_ver_id;

    let mut test_request_message = new_fixt_message!(TestRequest);
    test_request_message.msg_seq_num = 2;
    test_request_message.test_req_id = String::from("test");

    let mut bytes = Vec::new();
    logon_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    test_request_message.read(FIXVersion::FIX_4_2,MessageVersion::FIX42,&mut bytes);
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges Logon as normal.
    client_poll_event!(client,ClientEvent::SessionEstablished(_) => {});
    let message = client_poll_message!(client,connection_id,Logon);
    assert_eq!(message.msg_seq_num,1);

    //Make sure Client sends Logout and then disconnects.
    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.text,"BeginStr is wrong, expected 'FIXT.1.1' but received 'FIX.4.2'");

    client_poll_event!(client,ClientEvent::ConnectionTerminated(terminated_connection_id,reason) => {
        assert_eq!(terminated_connection_id,connection_id);
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
