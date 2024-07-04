// Public domain, 2017-01-23, James Bendig.

#![allow(unused_variables)]

#[macro_use]
extern crate fix_rs;

use std::time::Duration;

use fix_rs::dictionary::field_types::other::EncryptMethod;
use fix_rs::dictionary::messages::{
    BusinessMessageReject, Heartbeat, Logon, Logout, Reject, ResendRequest, SequenceReset,
    TestRequest,
};
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt::engine::{Engine, EngineEvent};
use fix_rs::message_version::MessageVersion;

fn main() {
    //List only the messages we need. The define_dictionary!() macro creates the following for us:
    //  fn build_dictionary(): A function that turns the listed messages into instructions used for
    //                         parsing FIX messages.
    //  enum MessageEnum:      An enum of the listed messages so Rust's match statement can be used to
    //                         make sure all messages are being handled.
    //  fn message_to_enum():  A function used to convert a FIXTMessage into a MessageEnum. Since
    //                         the Engine returns FIXTMessages, this function is a convenient way
    //                         to match and get concrete types.
    define_dictionary!(
        BusinessMessageReject,
        Heartbeat,
        Logon,
        Logout,
        Reject,
        ResendRequest,
        SequenceReset,
        TestRequest,
    );

    //Create an Engine which is used for initiating FIX connections.
    let max_message_size = 4096; //The maximum message size allowed to be received in bytes.
    let mut client = Engine::new(build_dictionary(), max_message_size).unwrap();

    //Initiate a connection to a FIX engine. The connection_id is used to interact with this
    //connection.
    let fix_version = FIXVersion::FIXT_1_1; //Communicate with protocol version FIXT.1.1.
    let message_version = MessageVersion::FIX50SP2; //Default to FIX 5.0. SP2 for outgoing messages.
    let sender_comp_id = b"Client"; //SenderCompID sent in every FIX message.
    let target_comp_id = b"Exchange"; //TargetCompID sent in every FIX message.
    let addr = "127.0.0.1:7001"; //IP and port to connect to.
    let connection_id = client
        .add_connection(
            fix_version,
            message_version,
            sender_comp_id,
            target_comp_id,
            addr,
        )
        .unwrap();

    //Poll client for new events. Events include new messages, connection status updates, errors,
    //etc.
    let timeout_duration = Duration::from_secs(120); //Optional.
    while let Some(event) = client.poll(timeout_duration) {
        match event {
            //Connection was able to open TCP stream to server.
            EngineEvent::ConnectionSucceeded(connection_id) => {
                println!("({})Connection succeeded", connection_id);

                //Start logon process.
                let mut logon_message = Logon::new();
                logon_message.encrypt_method = EncryptMethod::None;
                logon_message.heart_bt_int = 5;
                logon_message.default_appl_ver_id = message_version;
                client.send_message(connection_id, logon_message);
            }
            //Connection could not open TCP stream to server.
            EngineEvent::ConnectionFailed(connection_id, err) => {
                println!("({})Connection failed: {}", connection_id, err);
                break;
            }
            //Connection to server was closed either using the Engine::logout() function, logout
            //request by server, or an unrecoverable error.
            EngineEvent::ConnectionTerminated(connection_id, reason) => {
                println!("({})Connection terminated: {:?}", connection_id, reason);
                break;
            }
            //Connection completed the logon process and is free to communicate.
            EngineEvent::SessionEstablished(connection_id) => {
                println!("({})Session established", connection_id);

                //Start sending messages here.
            }
            //Connection received a new message.
            EngineEvent::MessageReceived(connection_id, message) => {
                //Handle the received message. Must be one of the messages listed in the
                //define_dictionary!() macro above. In this case, these are all administrative
                //messages that are handled by the Engine automatically but are passed along here
                //for logging purposes.
                match message_to_enum(message) {
                    MessageEnum::BusinessMessageReject(message) => {}
                    MessageEnum::Heartbeat(message) => {}
                    MessageEnum::Logon(message) => {}
                    MessageEnum::Logout(message) => {}
                    MessageEnum::Reject(message) => {}
                    MessageEnum::ResendRequest(message) => {}
                    MessageEnum::SequenceReset(message) => {}
                    MessageEnum::TestRequest(message) => {}
                };
            }
            //Connection received a message that could not be parsed correctly.
            EngineEvent::MessageReceivedGarbled(connection_id, parse_error) => {
                println!(
                    "({})Could not parse message: {}",
                    connection_id, parse_error
                );
            }
            //Connection received a message with a MsgSeqNum matching another message that was
            //already received.
            EngineEvent::MessageReceivedDuplicate(connection_id, message) => {
                println!(
                    "({})Received message with duplicate MsgSeqNum: {}",
                    connection_id,
                    message.msg_seq_num()
                );
            }
            //Connection received a message that doesn't follow session rules and was rejected. No
            //further action is necessary but it might be worth logging.
            EngineEvent::MessageRejected(connection_id, message) => {
                println!("({})Message was rejected", connection_id);
            }
            //Connected received a ResendRequest message for the messages in
            //[range.start,range.end).
            EngineEvent::ResendRequested(connection_id, range) => {
                println!(
                    "({})Received ResendRequest for messages where {} <= MsgSeqNum < {}",
                    connection_id, range.start, range.end
                );
            }
            //Connection received a SequenceReset-Reset message where NewSeqNo is set to the same
            //number as the expected MsgSeqNum.
            EngineEvent::SequenceResetResetHasNoEffect(connection_id) => {
                println!(
                    "({})Received SequenceReset-Reset with no effect",
                    connection_id
                );
            }
            //Connection received a SequenceReset-Reset message where NewSeqNo is set to an already
            //seen MsgSeqNum.
            EngineEvent::SequenceResetResetInThePast(connection_id) => {
                println!(
                    "({})Received SequenceReset-Reset where NoSeqNo is in the past",
                    connection_id
                );
            }
            //Internal error setting up Engine (before any connections were added).
            EngineEvent::FatalError(_, _) => {
                println!("Could not setup Engine.");
                break;
            }
            //The following events are not used for client connections.
            EngineEvent::ConnectionDropped(_, _)
            | EngineEvent::ConnectionAccepted(_, _, _)
            | EngineEvent::ConnectionLoggingOn(_, _, _)
            | EngineEvent::ListenerFailed(_, _)
            | EngineEvent::ListenerAcceptFailed(_, _) => {}
        }
    }
}
