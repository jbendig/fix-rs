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

#[macro_use]
extern crate fix_rs;
#[macro_use]
extern crate fix_rs_macros;
extern crate mio;
extern crate phf;

use mio::tcp::Shutdown;
use std::io::Write;
use std::thread;
use std::time::{Duration,Instant};
use std::sync::{Arc,Mutex};
use std::sync::atomic::{AtomicBool,Ordering};

#[macro_use]
mod common;
use common::{SERVER_SENDER_COMP_ID,SERVER_TARGET_COMP_ID,TestStream,new_logon_message};
use fix_rs::byte_buffer::ByteBuffer;
use fix_rs::dictionary::field_types::other::{MsgDirection,SessionRejectReason};
use fix_rs::dictionary::fields::{MsgTypeGrp,SenderCompID,TargetCompID,Text};
use fix_rs::dictionary::messages::{Heartbeat,Logon,Logout,Reject,ResendRequest,SequenceReset,TestRequest};
use fix_rs::field::Field;
use fix_rs::field_tag::{self,FieldTag};
use fix_rs::fix::ParseError;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt;
use fix_rs::fixt::engine::{EngineEvent,ConnectionTerminatedReason,ResendResponse};
use fix_rs::fixt::tests::{AUTO_DISCONNECT_AFTER_INBOUND_RESEND_REQUEST_LOOP_COUNT,INBOUND_MESSAGES_BUFFER_LEN_MAX,INBOUND_BYTES_BUFFER_CAPACITY};
use fix_rs::fixt::message::FIXTMessage;
use fix_rs::message::{self,NOT_REQUIRED,REQUIRED,Message};
use fix_rs::message_version::{self,MessageVersion};

fn serialize_and_append_message<T: FIXTMessage>(message: &T,fix_version: FIXVersion,message_version: MessageVersion,buffer: &mut Vec<u8>) {
    let mut bytes = ByteBuffer::new();
    message.read(fix_version,message_version,&mut bytes);

    buffer.extend_from_slice(bytes.bytes());
}

#[test]
fn test_recv_resend_request_invalid_end_seq_no() {
    define_dictionary!(
        Logon,
        ResendRequest,
        Reject,
    );

    //Connect and Logon.
    let (mut test_server,_client,_) = TestStream::setup_test_server_and_logon(build_dictionary());

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
        Logon,
        Logout,
    );

    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    //Send Logout immediately.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 1;
    test_server.send_message(message);

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

#[test]
fn test_recv_logout_with_high_msg_seq_num() {
    define_dictionary!(
        Logon,
        Logout,
        ResendRequest,
        SequenceReset,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

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
    let _ = engine_poll_message!(client,connection,SequenceReset);

    //Make sure client responds with Logout now that it's caught up.
    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.msg_seq_num,3);

    //Close connection and make sure client notifies that connection closed cleanly.
    let _ = test_server.stream.shutdown(Shutdown::Both);
    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::RemoteRequested = reason { true } else { false });
    });
}

#[test]
fn test_recv_logout_with_high_msg_seq_num_and_no_reply() {
    define_dictionary!(
        Logon,
        Logout,
        ResendRequest,
        SequenceReset,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

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
    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::RemoteRequested = reason { true } else { false });
    });
}

#[test]
fn test_recv_logout_send_logout_recv_resend_request() {
    define_dictionary!(
        Heartbeat,
        Logon,
        Logout,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Send Logout to client.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 2;
    test_server.send_message(message);

    //Make sure client responds with Logout.
    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.msg_seq_num,2);
    let _ = engine_poll_message!(client,connection,Logout);

    //Ask client for missing messages even though they already responded to Logout. This should
    //cancel the logout when done before the timeout.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 3;
    message.begin_seq_no = 2;
    message.end_seq_no = 0;
    test_server.send_message(message);

    //Handle the resend request.
    engine_gap_fill_resend_request!(client,connection,2..3);
    let _ = engine_poll_message!(client,connection,ResendRequest);

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
    let _ = engine_poll_message!(client,connection,Logout);

    //Close connection and make sure client notifies that connection closed cleanly.
    let _ = test_server.stream.shutdown(Shutdown::Both);
    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::RemoteRequested = reason { true } else { false });
    });
}

#[test]
fn test_send_logout_and_recv_resend_request() {
    define_dictionary!(
        Heartbeat,
        Logon,
        Logout,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Wait around for a Heartbeat and TestRequest. Ignore these so we can send a valid
    //ResendRequest below.
    thread::sleep(Duration::from_millis(5500));
    let _ = test_server.recv_message::<Heartbeat>();
    let _ = test_server.recv_message::<TestRequest>();

    //Begin Logout.
    client.logout(connection);
    let _ = test_server.recv_message::<Logout>();

    //Have server send a ResendRequest.
    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = 2;
    message.begin_seq_no = 2;
    message.end_seq_no = 0;
    test_server.send_message(message);

    engine_gap_fill_resend_request!(client,connection,2..5);
    let _ = engine_poll_message!(client,connection,ResendRequest);

    //Make sure client still responds to ResendRequest while logging out.
    let message = test_server.recv_message::<SequenceReset>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.new_seq_no,5);

    //Respond to logout and make sure client still logs out cleanly.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 3;
    test_server.send_message(message);

    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::LocalRequested = reason { true } else { false });
    });
}

#[test]
fn test_send_logout_and_recv_logout_with_high_msg_seq_num() {
    define_dictionary!(
        Heartbeat,
        Logon,
        Logout,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Begin Logout.
    client.logout(connection);
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
    let _ = engine_poll_message!(client,connection,SequenceReset);

    //Make sure client automatically attempts to logout again after being caught up.
    let _ = test_server.recv_message::<Logout>();

    //Finish logging out cleanly.
    let mut message = new_fixt_message!(Logout);
    message.msg_seq_num = 16;
    test_server.send_message(message);

    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::LocalRequested = reason { true } else { false });
    });
}

#[test]
fn test_send_logout_and_recv_logout_with_high_msg_seq_num_and_no_reply() {
    define_dictionary!(
        Heartbeat,
        Logon,
        Logout,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Connect and Logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Begin Logout.
    client.logout(connection);
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
    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::LogoutNoResponseError = reason { true } else { false });
    });
}

#[test]
fn test_wrong_sender_comp_id_in_logon_response() {
    define_dictionary!(
        Logon,
        Logout,
        Reject,
    );

    //Connect and attempt logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    let message = new_logon_message();
    client.send_message(connection,message);
    let _ = test_server.recv_message::<Logon>();

    //Respond with a logon messaging containing the wrong SenderCompID.
    let mut message = new_logon_message();
    message.sender_comp_id = b"unknown".to_vec();
    test_server.send_message(message);

    //Confirm client sends Reject, Logout, and then disconnects.
    let message = test_server.recv_message::<Reject>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,1);
    assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::CompIDProblem);
    assert_eq!(message.text,b"CompID problem".to_vec());

    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.text,b"SenderCompID is wrong".to_vec());

    engine_poll_event!(client,EngineEvent::MessageRejected(msg_connection,rejected_message) => {
        assert_eq!(msg_connection,connection);

        let message = rejected_message.as_any().downcast_ref::<Logon>().expect("Not expected message type").clone();
        assert_eq!(message.msg_seq_num,1);
        assert_eq!(message.sender_comp_id,b"unknown".to_vec());
    });

    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::SenderCompIDWrongError = reason { true } else { false });
    });
}

#[test]
fn test_wrong_target_comp_id_in_logon_response() {
    define_dictionary!(
        Logon,
        Logout,
        Reject,
    );

    //Connect and attempt logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    let message = new_logon_message();
    client.send_message(connection,message);
    let _ = test_server.recv_message::<Logon>();

    //Respond with a logon messaging containing the wrong TargetCompID.
    let mut message = new_logon_message();
    message.target_comp_id = b"unknown".to_vec();
    test_server.send_message(message);

    //Confirm client sends Reject, Logout, and then disconnects.
    let message = test_server.recv_message::<Reject>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.ref_seq_num,1);
    assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::CompIDProblem);
    assert_eq!(message.text,b"CompID problem".to_vec());

    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.text,b"TargetCompID is wrong".to_vec());

    engine_poll_event!(client,EngineEvent::MessageRejected(msg_connection,rejected_message) => {
        assert_eq!(msg_connection,connection);

        let message = rejected_message.as_any().downcast_ref::<Logon>().expect("Not expected message type").clone();
        assert_eq!(message.msg_seq_num,1);
        assert_eq!(message.target_comp_id,b"unknown".to_vec());
    });

    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
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
        Logon,
        Heartbeat,
        TestRequest,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Send INBOUND_MESSAGES_BUFFER_LEN_MAX + 1 TestRequests (hopefully) merged into a single TCP
    //frame.
    let mut bytes = Vec::new();
    for x in 0..INBOUND_MESSAGES_BUFFER_LEN_MAX + 1 {
        let mut test_request_message = new_fixt_message!(TestRequest);
        test_request_message.msg_seq_num = (x + 2) as u64;
        test_request_message.test_req_id = b"test".to_vec();

        serialize_and_append_message(&test_request_message,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    }
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    assert!(bytes.len() < INBOUND_BYTES_BUFFER_CAPACITY); //Make sure client thread can theoretically store all of the messages in a single recv().
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges messages as normal.
    for x in 0..INBOUND_MESSAGES_BUFFER_LEN_MAX + 1 {
        let message = engine_poll_message!(client,connection,TestRequest);
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
        Logon,
        Reject,
        TestMessage,
    );

    //FIXT.1.1: Make sure SenderCompID has to be the fourth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50,build_dictionary());

        //Accept when SenderCompID is the fourth tag.
        let target_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=236\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);

        //Reject when SenderCompID is the fifth tag.
        let sender_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0156=TEST\x0149=TX\x0134=3\x0152=20170105-01:01:01\x0110=012\x01";
        let bytes_written = test_server.stream.write(sender_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,sender_comp_id_fifth_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,b"SenderCompID must be the 4th tag".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::SenderCompIDNotFourthTag = parse_error { true } else { false });
        });

        //Reject when SenderCompID is missing.
        let missing_sender_comp_id_tag_message = b"8=FIXT.1.1\x019=50\x0135=9999\x0156=TEST\x0134=10\x0152=20170105-01:01:01\x0110=086\x01";
        let bytes_written = test_server.stream.write(missing_sender_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_sender_comp_id_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,b"SenderCompID must be the 4th tag".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::SenderCompIDNotFourthTag = parse_error { true } else { false });
        });
    }

    //FIX.4.0: Make sure SenderCompID does not have to be the fourth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon_with_ver(FIXVersion::FIX_4_0,MessageVersion::FIX40,build_dictionary());

        //Accept when SenderCompID is the fourth tag.
        let target_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=154\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);

        //Accept when SenderCompID is the fifth tag.
        let sender_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0156=TEST\x0149=TX\x0134=3\x0152=20170105-01:01:01\x0110=155\x01";
        let bytes_written = test_server.stream.write(sender_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,sender_comp_id_fifth_tag_message.len());

        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.sender_comp_id,SERVER_SENDER_COMP_ID);

        //Reject when SenderCompID is missing.
        let missing_sender_comp_id_tag_message = b"8=FIX.4.0\x019=42\x0135=9999\x0156=TEST\x0134=4\x0152=20170105-01:01:01\x0110=063\x01";
        let bytes_written = test_server.stream.write(missing_sender_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_sender_comp_id_tag_message.len());

       let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.text,b"Required tag missing".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
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
        Logon,
        Reject,
        TestMessage,
    );

    //FIXT.1.1: Make sure TargetCompID has to be the fifth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50,build_dictionary());

        //Accept when TargetCompID is the fifth tag.
        let target_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=236\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);

        //Reject when TargetCompID is the sixth tag.
        let target_comp_id_sixth_tag_message = b"8=FIXT.1.1\x019=48\x0135=9999\x0149=TX\x0134=3\x0156=TEST\x0152=20170105-01:01:01\x0110=237\x01";
        let bytes_written = test_server.stream.write(target_comp_id_sixth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_sixth_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,b"TargetCompID must be the 5th tag".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::TargetCompIDNotFifthTag = parse_error { true } else { false });
        });

        //Reject when TargetCompID is missing.
        let missing_target_comp_id_tag_message = b"8=FIXT.1.1\x019=59\x0135=9999\x0149=TX\x0134=3\x0152=20170105-01:01:01\x0110=086\x01";
        let bytes_written = test_server.stream.write(missing_target_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_target_comp_id_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.session_reject_reason.expect("SessionRejectReason must be provided"),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,b"TargetCompID must be the 5th tag".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::TargetCompIDNotFifthTag = parse_error { true } else { false });
        });
    }

    //FIX.4.0: Make sure TargetCompID does not have to be the fifth field.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon_with_ver(FIXVersion::FIX_4_0,MessageVersion::FIX40,build_dictionary());

        //Accept when TargetCompID is the fifth tag.
        let target_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0149=TX\x0156=TEST\x0134=2\x0152=20170105-01:01:01\x0110=154\x01";
        let bytes_written = test_server.stream.write(target_comp_id_fifth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_fifth_tag_message.len());

        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);

        //Accept when TargetCompID is the sixth tag.
        let target_comp_id_sixth_tag_message = b"8=FIX.4.0\x019=48\x0135=9999\x0149=TX\x0134=3\x0156=TEST\x0152=20170105-01:01:01\x0110=155\x01";
        let bytes_written = test_server.stream.write(target_comp_id_sixth_tag_message).unwrap();
        assert_eq!(bytes_written,target_comp_id_sixth_tag_message.len());

        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.msg_seq_num,3);
        assert_eq!(message.target_comp_id,SERVER_TARGET_COMP_ID);

        //Reject when TargetCompID is missing.
        let missing_target_comp_id_tag_message = b"8=FIX.4.0\x019=40\x0135=9999\x0149=TX\x0134=4\x0152=20170105-01:01:01\x0110=171\x01";
        let bytes_written = test_server.stream.write(missing_target_comp_id_tag_message).unwrap();
        assert_eq!(bytes_written,missing_target_comp_id_tag_message.len());

        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.text,b"Required tag missing".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
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
        Logon,
        TestMessage,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX40,build_dictionary());

    //Make sure DefaultApplVerID is respected for sent messages.
    {
        //Make client send a TestMessage.
        let mut message = new_fixt_message!(TestMessage);
        message.text = b"text".to_vec();
        client.send_message(connection,message);

        //Confirm text field was excluded by server due to requiring >= FIX50 but default is FIX40.
        let message = test_server.recv_message::<TestMessage>();
        assert_eq!(message.text.len(),0);
    }

    //Make sure DefaultApplVerID is respected for received messages.
    {
        //Make server send a TestMessage.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 2;
        message.text = b"text".to_vec();
        test_server.send_message(message);

        //Confirm text field was excluded by client due to requiring >= FIX50 but default is FIX40.
        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.text.len(),0);

        //Make sever send a TestMessage again but force the text field to be sent.
        let mut message = new_fixt_message!(TestMessage2);
        message.msg_seq_num = 3;
        message.text = b"text".to_vec();
        test_server.send_message(message);

        //Make sure message is considered invalid.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::UnknownTag(ref tag) = parse_error { *tag == FieldTag(58) } else { false });
        });
    }
}

#[test]
fn test_appl_ver_id() {
    define_fixt_message!(TestMessage: b"9999" => {
        REQUIRED, text: Text [FIX50..],
    });

    define_dictionary!(
        Logon,
        Reject,
        TestMessage,
    );

    //Make sure when ApplVerID is specified after the sixth field, Engine responds with an
    //appropriate Reject message and notification.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send TestMessage with ApplVerID field as the seventh tag.
        let appl_ver_id_seventh_tag_message = b"8=FIXT.1.1\x019=44\x0135=9999\x0149=SERVER\x0156=CLIENT\x0134=2\x011128=9\x0110=000\x01";
        let bytes_written = test_server.stream.write(appl_ver_id_seventh_tag_message).unwrap();
        assert_eq!(bytes_written,appl_ver_id_seventh_tag_message.len());

        //Make sure Engine responds with an appropriate reject.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,b"ApplVerID must be the 6th tag if specified".to_vec());

        //Make sure Engine indicates that it rejected the message.
        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::ApplVerIDNotSixthTag = parse_error { true } else { false });
        });
    }

    //Make sure ApplVerID overrides the default message version set in the initial Logon message.
    {
        //Connect and logon.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Send TestMessage with ApplVerID < FIX50 and without text field.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 2;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        test_server.send_message_with_ver(FIXVersion::FIXT_1_1,message.appl_ver_id.unwrap(),message);

        //Confirm Engine accepted message correctly.
        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.appl_ver_id,Some(MessageVersion::FIX40));
        assert_eq!(message.text.len(),0);

        //Send TestMessage with ApplVerID < FIX50 and with text field.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 3;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        message.text = b"text".to_vec();
        test_server.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message); //Force text field to be included.

        //Confirm Engine rejected message because text field is unsupported for this version.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::TagNotDefinedForThisMessageType);
        assert_eq!(message.text,b"Tag not defined for this message type".to_vec());

        engine_poll_event!(client,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::UnexpectedTag(ref tag) = parse_error { *tag == Text::tag()  } else { false });
        });
    }
}

#[test]
fn test_message_type_default_application_version() {
    define_fixt_message!(TestMessage: b"9999" => {
        REQUIRED, text: Text [FIX50SP1..],
    });

    define_dictionary!(
        Logon,
        Reject,
        TestMessage,
    );

    //Connect.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    //Logon.
    let mut logon_message = new_logon_message();
    logon_message.default_appl_ver_id = MessageVersion::FIX50;
    client.send_message_box_with_message_version(connection,MessageVersion::FIX50SP2,Box::new(logon_message));
    let message = test_server.recv_message::<Logon>();
    assert_eq!(message.msg_seq_num,1);

    let mut response_message = new_fixt_message!(Logon);
    response_message.encrypt_method = message.encrypt_method;
    response_message.heart_bt_int = message.heart_bt_int;
    response_message.default_appl_ver_id = message.default_appl_ver_id;
    let mut msg_type_grp = MsgTypeGrp::new();
    msg_type_grp.ref_msg_type = TestMessage::msg_type().to_vec();
    msg_type_grp.ref_appl_ver_id = Some(MessageVersion::FIX50SP1);
    msg_type_grp.msg_direction = MsgDirection::Send;
    msg_type_grp.default_ver_indicator = true;
    response_message.no_msg_types.push(Box::new(msg_type_grp));
    test_server.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,response_message);
    engine_poll_event!(client,EngineEvent::SessionEstablished(_) => {});
    let message = engine_poll_message!(client,connection,Logon);
    assert_eq!(message.msg_seq_num,1);

    //Make sure specifying a message type specific default application version overrides the
    //default message version.
    {
        //Send TestMessage text field.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 2;
        message.text = b"test".to_vec();
        test_server.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP1,message);

        //Confirm Engine accepted message correctly.
        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.meta.unwrap().message_version,MessageVersion::FIX50SP1); //Set by parser what it parsed message as.
        assert_eq!(message.text,b"test");
    }

    //Make sure ApplVerID overrides the message type specific default application version.
    {
        //Send TestMessage with explicit ApplVerID < FIX50 and without text field.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 3;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        test_server.send_message_with_ver(FIXVersion::FIXT_1_1,message.appl_ver_id.unwrap(),message);

        //Confirm Engine accepted message correctly.
        let message = engine_poll_message!(client,connection,TestMessage);
        assert_eq!(message.meta.unwrap().message_version,MessageVersion::FIX40);
        assert_eq!(message.text.len(),0);
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
        Logon,
        Heartbeat,
        TestRequest,
    );

    //Connect to server.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    //Have client send Logon.
    client.send_message_box(connection,Box::new(new_logon_message()));
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
    test_request_message.test_req_id = b"test".to_vec();

    let mut bytes = Vec::new();
    serialize_and_append_message(&logon_message,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    serialize_and_append_message(&test_request_message,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges both as normal.
    engine_poll_event!(client,EngineEvent::SessionEstablished(_) => {});
    let message = engine_poll_message!(client,connection,Logon);
    assert_eq!(message.msg_seq_num,1);
    let message = engine_poll_message!(client,connection,TestRequest);
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
        Logon,
        Logout,
        Reject,
        TestMessage,
    );

    //Connect to server.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    //Have client send Logon.
    let mut logon_message = new_logon_message();
    logon_message.default_appl_ver_id = MessageVersion::FIX50SP2;
    client.send_message_box(connection,Box::new(logon_message));
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
    test_message.text = b"test".to_vec();

    let mut bytes = Vec::new();
    serialize_and_append_message(&logon_message,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    serialize_and_append_message(&test_message,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges Logon as normal.
    engine_poll_event!(client,EngineEvent::SessionEstablished(_) => {});
    let message = engine_poll_message!(client,connection,Logon);
    assert_eq!(message.msg_seq_num,1);

    //Make sure client applies DefaultApplVerID version to TestMessage so that the Text field is
    //parsed.
    let message = engine_poll_message!(client,connection,TestMessage);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.text,b"test".to_vec());
}

#[test]
fn test_logout_and_terminate_wrong_versioned_test_request_immediately_after_logon() {
    //This is very similar to test_respond_to_test_request_immediately_after_logon() above except
    //it makes sure using the wrong FIX version follows the typical Logout and disconnect as
    //expected.

    define_dictionary!(
        Logon,
        Logout,
        TestRequest,
    );

    //Connect to server.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

    //Have client send Logon.
    client.send_message_box(connection,Box::new(new_logon_message()));
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
    test_request_message.test_req_id = b"test".to_vec();

    let mut bytes = Vec::new();
    serialize_and_append_message(&logon_message,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);
    serialize_and_append_message(&test_request_message,FIXVersion::FIX_4_2,MessageVersion::FIX42,&mut bytes);
    assert!(bytes.len() < 1400); //Make sure the serialized body is reasonably likely to fit within the MTU.
    let bytes_written = test_server.stream.write(&bytes).unwrap();
    assert_eq!(bytes_written,bytes.len());

    //Make sure client acknowledges Logon as normal.
    engine_poll_event!(client,EngineEvent::SessionEstablished(_) => {});
    let message = engine_poll_message!(client,connection,Logon);
    assert_eq!(message.msg_seq_num,1);

    //Make sure Engine sends Logout and then disconnects.
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

#[test]
fn test_max_message_size() {
    const MAX_MESSAGE_SIZE: u64 = 4096;

    define_fixt_message!(TestMessage: b"9999" => {
        REQUIRED, text: Text [FIX40..],
    });

    define_dictionary!(
        Logon,
        Logout,
        Reject,
        TestMessage,
    );

    fn message_length<T: Message>(message: &T) -> u64 {
        let mut buffer = ByteBuffer::new();
        message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut buffer) as u64
    }

    //Make sure exceeding the MaxMessageSize in messages after Logon results in a Reject message.
    {
        //Connect to server.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

        //Have client send Logon.
        let mut message = new_logon_message();
        message.max_message_size = MAX_MESSAGE_SIZE;
        client.send_message_box(connection,Box::new(message));
        let message = test_server.recv_message::<Logon>();
        assert_eq!(message.msg_seq_num,1);
        assert_eq!(message.max_message_size,MAX_MESSAGE_SIZE);

        //Acknowledge Logon.
        let mut response_message = new_fixt_message!(Logon);
        response_message.encrypt_method = message.encrypt_method;
        response_message.heart_bt_int = message.heart_bt_int;
        response_message.default_appl_ver_id = message.default_appl_ver_id;
        test_server.send_message(response_message);
        engine_poll_event!(client,EngineEvent::SessionEstablished(_) => {});
        let message = engine_poll_message!(client,connection,Logon);
        assert_eq!(message.msg_seq_num,1);

        //Try and send Engine a message exceeding MAX_MESSAGE_SIZE.
        let mut message = new_fixt_message!(TestMessage);
        message.msg_seq_num = 2;
        let current_message_len = message_length(&message);
        for _ in 0..(MAX_MESSAGE_SIZE - current_message_len) + 1 {
            message.text.push(b'A');
        }
        test_server.send_message(message);

        //Make sure Engine rejected the message.
        let message = test_server.recv_message::<Reject>();
        assert_eq!(message.msg_seq_num,2);
        assert_eq!(message.ref_seq_num,2);
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::Other);
        let mut expected_error_text = b"Message size exceeds MaxMessageSize=".to_vec();
        expected_error_text.extend_from_slice(MAX_MESSAGE_SIZE.to_string().as_bytes());
        assert_eq!(message.text,expected_error_text);
    }

    //Make sure exceeding the MaxMessageSize in the Logon response results in the Engine just
    //disconnecting.
    {
        //Connect to server.
        let (mut test_server,mut client,connection) = TestStream::setup_test_server(build_dictionary());

        //Have client send Logon.
        let mut message = new_logon_message();
        message.max_message_size = MAX_MESSAGE_SIZE;
        client.send_message_box(connection,Box::new(message));
        let message = test_server.recv_message::<Logon>();
        assert_eq!(message.msg_seq_num,1);
        assert_eq!(message.max_message_size,MAX_MESSAGE_SIZE);

        //Respond with Logon message that exceeds MAX_MESSAGE_SIZE.
        let mut response_message = new_fixt_message!(Logon);
        response_message.encrypt_method = message.encrypt_method.clone();
        response_message.heart_bt_int = message.heart_bt_int;
        response_message.default_appl_ver_id = message.default_appl_ver_id;
        while message_length(&response_message) <= MAX_MESSAGE_SIZE {
            let mut msg_type_grp = MsgTypeGrp::new();
            msg_type_grp.ref_msg_type = b"L".to_vec();
            msg_type_grp.ref_appl_ver_id = Some(MessageVersion::FIX50SP1);
            msg_type_grp.msg_direction = MsgDirection::Send;
            response_message.no_msg_types.push(Box::new(msg_type_grp));
        }
        test_server.send_message(response_message);

        //Make sure Engine just disconnects.
        engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogonParseError(parse_error) = reason {
                if let ParseError::MessageSizeTooBig = parse_error { true } else { false }
            }
            else {
                false
            });
        });
    }
}

#[test]
fn test_block_read_when_write_blocks() {

    define_dictionary!(
        Logon,
        Heartbeat,
        Reject,
        ResendRequest,
        TestRequest,
    );

    //Send a bunch of messages to Engine without reading the responses. Engine should stop reading
    //until it can write again.
    {
        //Connect and Logon.
        let (mut test_server,client,_) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Run a background thread to drain client events until they stop. The stopping indicates the
        //client has stopped accepting new messages.
        let client = Arc::new(Mutex::new(client)); //Keep client around even after thread ends.
        let client_clone = client.clone(); //Clone to be passed to thread.
        let thread_running = Arc::new(AtomicBool::new(true));
        let thread_running_clone = thread_running.clone();
        let thread_handle = thread::spawn(move || {
            let mut client = client_clone.lock().unwrap();
            while let Some(event) = client.poll(Duration::from_secs(2)) {
                match event {
                    EngineEvent::ConnectionTerminated(_,_) => panic!("Engine should not have terminated connection yet."),
                    _ => {},
                }
            }

            thread_running_clone.store(false,Ordering::Relaxed);
        });

        //Flood client with TestRequest messages until thread notifies that messages are being
        //blocked.
        let mut outbound_msg_seq_num = 2;
        let now = Instant::now();
        let mut stop_writing = false;
        loop {
            if !thread_running.load(Ordering::Relaxed) {
                thread_handle.join().expect("Thread must be stopped.");
                break;
            }
            else if now.elapsed() > Duration::from_secs(15) {
                panic!("Engine never blocked receiving of new messages.");
            }

            if !stop_writing {
                let mut message = new_fixt_message!(TestRequest);
                message.msg_seq_num = outbound_msg_seq_num;
                message.test_req_id = b"test".to_vec();
                if let Err(bytes_not_written) = test_server.send_message_with_timeout(message,Duration::from_millis(10)) {
                    //Stop writing new messages because TCP indicated that the other side is
                    //congested.
                    stop_writing = true;

                    if bytes_not_written > 0 {
                        continue;
                    }
                }

                outbound_msg_seq_num += 1;
            }
        }

        //Drain server's read buffer.
        loop {
            let message = test_server.recv_message::<Heartbeat>();
            if message.msg_seq_num == outbound_msg_seq_num - 1 {
                break;
            }
        }

        //Send gibberish that will force an incomplete message to be discarded.
        let _ = test_server.stream.write(b"\x0110=000\x01=000");

        //Make sure messages continue to flow again.
        let mut message = new_fixt_message!(TestRequest);
        message.msg_seq_num = outbound_msg_seq_num + 1;
        message.test_req_id = b"final".to_vec();
        test_server.send_message(message);

        let message = test_server.recv_fixt_message();
        let message = match message_to_enum(message) {
            MessageEnum::Heartbeat(message) => message,
            _ => Box::from(test_server.recv_message::<Heartbeat>()),
        };
        assert_eq!(message.test_req_id,b"final");
    }

    //Same as above but never drain the server's read buffer so the connection must eventually be
    //dropped.
    {
        //Connect and Logon.
        let (mut test_server,mut client,_) = TestStream::setup_test_server_and_logon(build_dictionary());

        //Flood client with TestRequest messages until Engine drops the connection.
        let mut outbound_msg_seq_num = 2;
        let now = Instant::now();
        let mut stop_writing = false;
        loop {
            if now.elapsed() > Duration::from_secs(30) {
                panic!("Engine never disconnected.");
            }

            if let Some(EngineEvent::ConnectionTerminated(_,reason)) = client.poll(Duration::from_millis(0)) {
                assert!(if let ConnectionTerminatedReason::SocketNotWritableTimeoutError = reason { true } else { false });
                assert!(test_server.is_stream_closed(Duration::from_secs(3)));

                //Success! Engine disconnected.
                break;
            }

            if !stop_writing {
                let mut message = new_fixt_message!(TestRequest);
                message.msg_seq_num = outbound_msg_seq_num;
                message.test_req_id = b"test".to_vec();
                if let Err(_) = test_server.send_message_with_timeout(message,Duration::from_millis(10)) {
                    stop_writing = true;
                }

                outbound_msg_seq_num += 1;
            }
        }
    }
}

#[test]
fn test_inbound_resend_loop_detection() {
    define_dictionary!(
        Logon,
        Logout,
        Heartbeat,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Connect and logon.
    let (mut test_server,mut client,connection) = TestStream::setup_test_server_and_logon(build_dictionary());

    //Have server send TestRequest so Engine responds with a Heartbeat.
    let mut message = new_fixt_message!(TestRequest);
    message.msg_seq_num = 2;
    message.test_req_id = b"test".to_vec();
    test_server.send_message(message);
    engine_poll_message!(client,connection,TestRequest);
    let message = test_server.recv_message::<Heartbeat>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.test_req_id,b"test");

    //Have server ignore the Heartbeat response by sending ResendRequest a few times. The client
    //should eventually logout and disconnect.
    const BASE_MSG_SEQ_NUM: u64 = 3;
    for x in 0..AUTO_DISCONNECT_AFTER_INBOUND_RESEND_REQUEST_LOOP_COUNT {
        let mut message = new_fixt_message!(ResendRequest);
        message.msg_seq_num = BASE_MSG_SEQ_NUM + x;
        message.begin_seq_no = 2;
        message.end_seq_no = 0;
        test_server.send_message(message);

        engine_gap_fill_resend_request!(client,connection,2..3);
        let _ = engine_poll_message!(client,connection,ResendRequest);

        let message = test_server.recv_message::<SequenceReset>();
        assert_eq!(message.gap_fill_flag,true);
        assert_eq!(message.new_seq_no,3);
        assert_eq!(message.msg_seq_num,2);
    }

    let mut message = new_fixt_message!(ResendRequest);
    message.msg_seq_num = BASE_MSG_SEQ_NUM + AUTO_DISCONNECT_AFTER_INBOUND_RESEND_REQUEST_LOOP_COUNT;
    message.begin_seq_no = 2;
    message.end_seq_no = 0;
    test_server.send_message(message);

    let message = test_server.recv_message::<Logout>();
    assert_eq!(message.text,b"Detected ResendRequest loop for BeginSeqNo 2".to_vec());

    engine_poll_event!(client,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::InboundResendRequestLoopError = reason { true } else { false });
    });
    assert!(test_server.is_stream_closed(Duration::from_secs(3)));
}
