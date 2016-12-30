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
use common::{TestServer,new_logon_message};
use fix_rs::dictionary::field_types::other::SessionRejectReason;
use fix_rs::dictionary::messages::{Heartbeat,Logon,Logout,Reject,ResendRequest,SequenceReset,TestRequest};
use fix_rs::fixt::client::{ClientEvent,ConnectionTerminatedReason};
use fix_rs::fixt::tests::{INBOUND_MESSAGES_BUFFER_LEN_MAX,INBOUND_BYTES_BUFFER_CAPACITY};
use fix_rs::fixt::message::FIXTMessage;
use fix_rs::message::Message;

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
    client.send_message(connection_id,Box::new(message));
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
    client.send_message(connection_id,Box::new(message));
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

        test_request_message.read(&mut bytes);
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
