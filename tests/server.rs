// Copyright 2017 James Bendig. See the COPYRIGHT file at the top-level
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

use mio::tcp::Shutdown;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[macro_use]
mod common;
use common::{CLIENT_SENDER_COMP_ID,CLIENT_TARGET_COMP_ID,TestStream,new_logon_message};
use fix_rs::dictionary::field_types::other::{MsgDirection,SessionRejectReason};
use fix_rs::dictionary::fields::{MsgTypeGrp,Text};
use fix_rs::dictionary::messages::{Heartbeat,Logon,Logout,Reject,TestRequest};
use fix_rs::field::Field;
use fix_rs::field_tag::{self,FieldTag};
use fix_rs::fix::ParseError;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt;
use fix_rs::fixt::engine::{EngineEvent,ConnectionTerminatedReason};
use fix_rs::fixt::message::FIXTMessage;
use fix_rs::fixt::tests::{AUTO_DISCONNECT_AFTER_NO_LOGON_RECEIVED_SECONDS};
use fix_rs::message::{self,REQUIRED};
use fix_rs::message_version::{self,MessageVersion};

#[test]
fn test_wrong_target_comp_id_in_logon() {
    //The engine should automatically disconnect the connection when receiving a logon message with
    //the wrong target_comp_id. No reason to hand this off to authentication.

    define_dictionary!(
        Logon,
    );

    let (mut test_client,mut engine,_,connection) = TestStream::setup_test_client(build_dictionary());

    let mut logon_message = new_logon_message();
    logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
    logon_message.target_comp_id = Vec::new();
    test_client.send_message(logon_message.clone());

    engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::TargetCompIDWrongError = reason { true } else { false });
    });

    //Make sure engine did not send any data.
    assert!(test_client.try_recv_fixt_message(Duration::from_secs(1)).is_none());

    //Confirm the client socket disconnected.
    assert!(test_client.is_stream_closed(Duration::from_secs(5)));
}

#[test]
fn test_logon_all_fix_versions() {
    define_dictionary!(
        Logon,
    );

    for fix_version in FIXVersion::all() {
        let (mut test_client,mut engine,listener,connection) = TestStream::setup_test_client(build_dictionary());

        let mut logon_message = new_logon_message();
        logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
        logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
        logon_message.default_appl_ver_id = fix_version.max_message_version();
        test_client.send_message_with_ver(fix_version,logon_message.default_appl_ver_id,logon_message);

        engine_poll_event!(engine,EngineEvent::ConnectionLoggingOn(some_listener,some_connection,logon_message) => {
            assert_eq!(some_listener,listener);
            assert_eq!(some_connection,connection);
            assert_eq!(logon_message.msg_seq_num,1);

            let mut response_message = new_fixt_message!(Logon);
            response_message.encrypt_method = logon_message.encrypt_method.clone();
            response_message.heart_bt_int = logon_message.heart_bt_int;
            response_message.default_appl_ver_id = logon_message.default_appl_ver_id;
            engine.approve_new_connection(connection,Box::new(response_message),None);
        });
    }
}

#[test]
fn test_approve_already_approved_connection_does_nothing() {
    define_dictionary!(
        Logon,
    );

    let (mut test_client,mut engine,_,connection) = TestStream::setup_test_client_and_logon(build_dictionary());

    let logon_message = new_fixt_message!(Logon);
    engine.approve_new_connection(connection,Box::new(logon_message),None);

    assert!(test_client.try_recv_fixt_message(Duration::from_secs(1)).is_none());
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
    let (mut test_client,mut engine,_,connection) = TestStream::setup_test_client_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX40,build_dictionary());

    //Make sure DefaultApplVerID is respected for sent messages.
    {
        //Make engine send a TestMessage.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage);
        message.msg_seq_num = 2;
        message.text = b"text".to_vec();
        engine.send_message(connection,message);

        //Confirm text field was excluded by engine due to requiring >= FIX50 but default is FIX40.
        let message = test_client.recv_message::<TestMessage>();
        assert_eq!(message.text.len(),0);
    }

    //Make sure DefaultApplVerID is respected for received messages.
    {
        //Make client send a TestMessage.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage);
        message.msg_seq_num = 2;
        message.text = b"text".to_vec();
        test_client.send_message(message);

        //Confirm text field was excluded by engine due to requiring >= FIX50 but default is FIX40.
        let message = engine_poll_message!(engine,connection,TestMessage);
        assert_eq!(message.text.len(),0);

        //Make client send a TestMessage again but force the text field to be sent.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage2);
        message.msg_seq_num = 3;
        message.text = b"text".to_vec();
        test_client.send_message(message);

        //Make sure message is considered invalid.
        engine_poll_event!(engine,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
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
        let (mut test_client,mut engine,_,connection) = TestStream::setup_test_client_and_logon(build_dictionary());

        //Send TestMessage with ApplVerID field as the seventh tag.
        let appl_ver_id_seventh_tag_message = b"8=FIXT.1.1\x019=44\x0135=9999\x0149=SERVER\x0156=CLIENT\x0134=2\x011128=9\x0110=000\x01";
        let bytes_written = test_client.stream.write(appl_ver_id_seventh_tag_message).unwrap();
        assert_eq!(bytes_written,appl_ver_id_seventh_tag_message.len());

        //Make sure Engine responds with an appropriate reject.
        let message = test_client.recv_message::<Reject>();
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder);
        assert_eq!(message.text,b"ApplVerID must be the 6th tag if specified".to_vec());

        //Make sure Engine indicates that it rejected the message.
        engine_poll_event!(engine,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
            assert_eq!(msg_connection,connection);
            assert!(if let ParseError::ApplVerIDNotSixthTag = parse_error { true } else { false });
        });
    }

    //Make sure ApplVerID overrides the default message version set in the initial Logon message.
    {
        //Connect and logon.
        let (mut test_client,mut engine,_,connection) = TestStream::setup_test_client_and_logon(build_dictionary());

        //Send TestMessage with ApplVerID < FIX50 and without text field.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage);
        message.msg_seq_num = 2;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        test_client.send_message_with_ver(FIXVersion::FIXT_1_1,message.appl_ver_id.unwrap(),message);

        //Confirm Engine accepted message correctly.
        let message = engine_poll_message!(engine,connection,TestMessage);
        assert_eq!(message.appl_ver_id,Some(MessageVersion::FIX40));
        assert_eq!(message.text.len(),0);

        //Send TestMessage with ApplVerID < FIX50 and with text field.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage);
        message.msg_seq_num = 3;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        message.text = b"text".to_vec();
        test_client.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message); //Force text field to be included.

        //Confirm Engine rejected message because text field is unsupported for this version.
        let message = test_client.recv_message::<Reject>();
        assert_eq!(message.session_reject_reason.unwrap(),SessionRejectReason::TagNotDefinedForThisMessageType);
        assert_eq!(message.text,b"Tag not defined for this message type".to_vec());

        engine_poll_event!(engine,EngineEvent::MessageReceivedGarbled(msg_connection,parse_error) => {
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
    let (mut test_client,mut engine,listener,connection) = TestStream::setup_test_client(build_dictionary());

    //Logon.
    let mut logon_message = new_logon_message();
    logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
    logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
    logon_message.default_appl_ver_id = MessageVersion::FIX50;

    let mut msg_type_grp = MsgTypeGrp::new();
    msg_type_grp.ref_msg_type = TestMessage::msg_type().to_vec();
    msg_type_grp.ref_appl_ver_id = Some(MessageVersion::FIX50SP1);
    msg_type_grp.msg_direction = MsgDirection::Send;
    msg_type_grp.default_ver_indicator = true;
    logon_message.no_msg_types.push(Box::new(msg_type_grp));

    test_client.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,logon_message);

    engine_poll_event!(engine,EngineEvent::ConnectionLoggingOn(some_listener,some_connection,logon_message) => {
        assert_eq!(some_listener,listener);
        assert_eq!(some_connection,connection);

        let mut response_message = new_fixt_message!(Logon);
        response_message.encrypt_method = logon_message.encrypt_method.clone();
        response_message.heart_bt_int = logon_message.heart_bt_int;
        response_message.default_appl_ver_id = logon_message.default_appl_ver_id;

        engine.approve_new_connection(connection,Box::new(response_message),None);
    });

    let message = test_client.recv_message::<Logon>();
    assert_eq!(message.msg_seq_num,1);

    //Make sure specifying a message type specific default application version overrides the
    //default message version.
    {
        //Send TestMessage text field.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage);
        message.msg_seq_num = 2;
        message.text = b"test".to_vec();
        test_client.send_message_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP1,message);

        //Confirm Engine accepted message correctly.
        let message = engine_poll_message!(engine,connection,TestMessage);
        assert_eq!(message.meta.unwrap().message_version,MessageVersion::FIX50SP1); //Set by parser what it parsed message as.
        assert_eq!(message.text,b"test");
    }

    //Make sure ApplVerID overrides the message type specific default application version.
    {
        //Send TestMessage with explicit ApplVerID < FIX50 and without text field.
        let mut message = new_fixt_message!(FROM_CLIENT TestMessage);
        message.msg_seq_num = 3;
        message.appl_ver_id = Some(MessageVersion::FIX40);
        test_client.send_message_with_ver(FIXVersion::FIXT_1_1,message.appl_ver_id.unwrap(),message);

        //Confirm Engine accepted message correctly.
        let message = engine_poll_message!(engine,connection,TestMessage);
        assert_eq!(message.meta.unwrap().message_version,MessageVersion::FIX40);
        assert_eq!(message.text.len(),0);
    }

}

#[test]
fn test_block_read_while_approving_logon() {
    define_dictionary!(
        Heartbeat,
        Logon,
        TestRequest,
    );

    let (mut test_client,mut engine,listener,connection) = TestStream::setup_test_client(build_dictionary());

    let mut logon_message = new_logon_message();
    logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
    logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
    test_client.send_message(logon_message.clone());

    engine_poll_event!(engine,EngineEvent::ConnectionLoggingOn(some_listener,some_connection,_) => {
        assert_eq!(some_listener,listener);
        assert_eq!(some_connection,connection);
    });

    //Send TestRequest.
    let mut message = new_fixt_message!(FROM_CLIENT TestRequest);
    message.msg_seq_num = 2;
    message.test_req_id = b"test".to_vec();
    test_client.send_message(message);

    //Confirm message does not generate an event and is not replied to with a Heartbeat.
    engine_poll_no_event!(engine);
    assert!(test_client.try_recv_fixt_message(Duration::from_secs(1)).is_none());

    //Approve connection.
    let mut response_message = new_fixt_message!(Logon);
    response_message.encrypt_method = logon_message.encrypt_method.clone();
    response_message.heart_bt_int = logon_message.heart_bt_int.clone();
    response_message.default_appl_ver_id = logon_message.default_appl_ver_id;
    engine.approve_new_connection(connection,Box::new(response_message),None);
    let _ = test_client.recv_message::<Logon>();

    //Confirm TestRequest now generates an event and is replied to with a Heartbeat.
    let message = engine_poll_message!(engine,connection,TestRequest);
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.test_req_id,b"test");
    let message = test_client.recv_message::<Heartbeat>();
    assert_eq!(message.msg_seq_num,2);
    assert_eq!(message.test_req_id,b"test");
}

#[test]
fn test_auto_disconnect_after_no_logon() {
    define_dictionary!(
        Logon,
    );

    let (test_client,mut engine,_,connection) = TestStream::setup_test_client(build_dictionary());

    //Wait for auto-disconnect.
    thread::sleep(Duration::from_secs(AUTO_DISCONNECT_AFTER_NO_LOGON_RECEIVED_SECONDS));

    //Confirm connection was terminated.
    engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::LogonNeverReceivedError = reason { true } else { false });
    });

    //Confirm socket was closed.
    assert!(test_client.is_stream_closed(Duration::from_secs(1)));
}

#[test]
fn test_connection_terminated_when_disconnected_with_no_logon() {
    define_dictionary!(
        Logon,
    );

    let (test_client,mut engine,_,connection) = TestStream::setup_test_client(build_dictionary());

    //Disconnect.
    let _ = test_client.stream.shutdown(Shutdown::Both);

    //Confirm connection was terminated.
    engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::SocketWriteError(_) = reason { true } else { false });
    });
}

#[test]
fn test_connection_terminated_while_approving_logon() {
    define_dictionary!(
        Logon,
    );

    let (mut test_client,mut engine,listener,connection) = TestStream::setup_test_client(build_dictionary());

    //Send Logon.
    let mut logon_message = new_logon_message();
    logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
    logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
    test_client.send_message(logon_message.clone());

    engine_poll_event!(engine,EngineEvent::ConnectionLoggingOn(some_listener,some_connection,_) => {
        assert_eq!(some_listener,listener);
        assert_eq!(some_connection,connection);
    });

    //Disconnect.
    test_client.stream.shutdown(Shutdown::Both);

    //Confirm connection was terminated.
    engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
        assert_eq!(terminated_connection,connection);
        assert!(if let ConnectionTerminatedReason::SocketWriteError(_) = reason { true } else { false });
    });
}

#[test]
fn test_heart_bt_int() {
    define_dictionary!(
        Heartbeat,
        Logon,
        Logout,
        TestRequest,
    );

    //Make sure logging in with a negative heart_bt_int is rejected.
    {
        let (mut test_client,mut engine,_,connection) = TestStream::setup_test_client(build_dictionary());

        let mut logon_message = new_logon_message();
        logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
        logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
        logon_message.heart_bt_int = -1;
        test_client.send_message(logon_message);

        let message = test_client.recv_message::<Logout>();
        assert_eq!(message.text,b"HeartBtInt cannot be negative".to_vec());

        thread::sleep(Duration::from_millis(500));
        assert!(test_client.is_stream_closed(Duration::from_secs(5)));

        engine_poll_event!(engine,EngineEvent::ConnectionTerminated(terminated_connection,reason) => {
            assert_eq!(terminated_connection,connection);
            assert!(if let ConnectionTerminatedReason::LogonHeartBtIntNegativeError = reason { true } else { false });
        });
    }

    //Make sure requested heart_bt_int is respected.
    for heart_bt_int in vec![3,7] {
        let (mut test_client,mut engine,listener,connection) = TestStream::setup_test_client(build_dictionary());

        let mut logon_message = new_logon_message();
        logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
        logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
        logon_message.heart_bt_int = heart_bt_int;
        test_client.send_message(logon_message);

        engine_poll_event!(engine,EngineEvent::ConnectionLoggingOn(some_listener,some_connection,logon_message) => {
            assert_eq!(some_listener,listener);
            assert_eq!(some_connection,connection);
            assert_eq!(logon_message.msg_seq_num,1);

            let mut response_message = new_fixt_message!(Logon);
            response_message.encrypt_method = logon_message.encrypt_method.clone();
            response_message.heart_bt_int = logon_message.heart_bt_int;
            response_message.default_appl_ver_id = logon_message.default_appl_ver_id;
            engine.approve_new_connection(connection,Box::new(response_message),None);
        });

        let _ = test_client.recv_message::<Logon>();

        //Wait a moment and make sure no Heartbeat messages have been sent.
        thread::sleep(Duration::from_millis(heart_bt_int as u64 * 1000 / 2));
        assert!(test_client.try_recv_fixt_message(Duration::from_secs(1)).is_none());

        //Wait another moment and make sure a Heartbeat message has been sent.
        thread::sleep(Duration::from_millis(heart_bt_int as u64 * 1000 / 2 + 200));
        let _ = test_client.recv_message::<Heartbeat>();

        //Make sure a TestRequest is also sent after the Heartbeat.
        let _ = test_client.recv_message::<TestRequest>();
    }
}
