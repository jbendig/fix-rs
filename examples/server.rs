// Public domain, 2017-02-21, James Bendig.

#![allow(unused_variables)]

#[macro_use]
extern crate fix_rs;

use std::time::Duration;

use fix_rs::dictionary::messages::{BusinessMessageReject,Heartbeat,Logon,Logout,Reject,ResendRequest,SequenceReset,TestRequest};
use fix_rs::fixt::engine::{Engine,EngineEvent,ResendResponse};

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

    //Create an Engine which is used for accepting FIX connections.
    let max_message_size = 4096; //The maximum message size allowed to be received in bytes.
    let mut server = Engine::new(build_dictionary(),max_message_size).unwrap();

    //Setup a listener to accept new connections. The listener_id is used to interact with this
    //listener.
    let sender_comp_id = b"Server"; //SenderCompID sent in every FIX message.
    let addr = "127.0.0.1:7001"; //IP and port to listen on.
    let listener_id = server.add_listener(sender_comp_id,addr);

    //Poll server for new events. Events include new connections, connection status updates,
    //received messages, errors, etc.
    let timeout_duration = Duration::from_secs(120); //Optional
    while let Some(event) = server.poll(timeout_duration) {
        match event {
            //Listener accepted a new connection and is awaiting a Logon message.
            EngineEvent::ConnectionAccepted(listener_id,connection_id,addr) => {
                println!("({},{})New connection accepted: {}",listener_id,connection_id,addr);

                //The connection can be rejected here if the addr is blacklisted/not whitelisted or
                //if over capacity.
                //listener.reject_new_connection(connection_id,None);
            },
            //Listener accepted a new connection but doesn't have the capacity to manage it so it
            //was immediately dropped.
            EngineEvent::ConnectionDropped(listener_id,addr) => {
                println!("({})New connection was dropped: {}",listener_id,addr);
            },
            //Connection sent a Logon message and is awaiting approval or rejection.
            EngineEvent::ConnectionLoggingOn(listener_id,connection_id,logon) => {
                if logon.username == b"some_user" &&
                   logon.password == b"some_password" {
                    let mut response_logon = Logon::new();
                    response_logon.encrypt_method = logon.encrypt_method.clone();
                    response_logon.heart_bt_int = logon.heart_bt_int.clone();
                    response_logon.default_appl_ver_id = logon.default_appl_ver_id;
                    server.approve_new_connection(connection_id,Box::new(response_logon),None);

                    //Store connection_id somewhere to interact with this connection in the future.
                }
                else {
                    server.reject_new_connection(connection_id,Some(b"Invalid username and/or password".to_vec()));
                }
            },
            //Listener could not be setup because of a lack of resources.
            EngineEvent::ListenerFailed(listener_id,err) => {
                println!("({})Listener failed: {:?}",listener_id,err);
                break;
            },
            //Listener encountered a socket error while trying to accept a new connection.
            EngineEvent::ListenerAcceptFailed(listener_id,err) => {
                println!("({})Listener accept failed: {:?}",listener_id,err);
                break;
            }
            //Connection to server was closed either using the Engine::logout() function, logout
            //request by client, or an unrecoverable error.
            EngineEvent::ConnectionTerminated(connection_id,reason) => {
                println!("({})Connection terminated: {:?}",connection_id,reason);
            },
            //Connection received a new message.
            EngineEvent::MessageReceived(connection_id,message) => {
                //Handle the received message. Must be one of the messages listed in the
                //define_dictionary!() macro above. In this case, these are all administrative
                //messages that are handled by the Engine automatically but are passed along here
                //for logging purposes.
                match message_to_enum(message) {
                    MessageEnum::BusinessMessageReject(message) => {},
                    MessageEnum::Heartbeat(message) => {},
                    MessageEnum::Logon(message) => {},
                    MessageEnum::Logout(message) => {},
                    MessageEnum::Reject(message) => {},
                    MessageEnum::ResendRequest(message) => {},
                    MessageEnum::SequenceReset(message) => {},
                    MessageEnum::TestRequest(message) => {},
                };
            },
            //Connection received a message that could not be parsed correctly.
            EngineEvent::MessageReceivedGarbled(connection_id,parse_error) => {
                println!("({})Could not parse message: {}",connection_id,parse_error);
            },
            //Connection received a message with a MsgSeqNum matching another message that was
            //already received.
            EngineEvent::MessageReceivedDuplicate(connection_id,message) => {
                println!("({})Received message with duplicate MsgSeqNum: {}",connection_id,message.msg_seq_num());
            },
            //Connection received a message that doesn't follow session rules and was rejected. No
            //further action is necessary but it might be worth logging.
            EngineEvent::MessageRejected(connection_id,message) => {
                println!("({})Message was rejected",connection_id);
            },
            //Connected received a ResendRequest message for the messages in
            //[range.start,range.end).
            EngineEvent::ResendRequested(connection_id,range) => {
                //Messages should be added to response in order. Administrative messages should be
                //filled with a ResendResponse::Gap. Business messages should each be added as a
                //ResendResponse::Message.
                let mut response = Vec::new();
                response.push(ResendResponse::Gap(range));
                server.send_resend_response(connection_id,response);
            },
            //Connection received a SequenceReset-Reset message where NewSeqNo is set to the same
            //number as the expected MsgSeqNum.
            EngineEvent::SequenceResetResetHasNoEffect(connection_id) => {
                println!("({})Received SequenceReset-Reset with no effect",connection_id);
            },
            //Connection received a SequenceReset-Reset message where NewSeqNo is set to an already
            //seen MsgSeqNum.
            EngineEvent::SequenceResetResetInThePast(connection_id) => {
                println!("({})Received SequenceReset-Reset where NoSeqNo is in the past",connection_id);
            },
            //Internal error setting up Engine (before any listeners were added).
            EngineEvent::FatalError(_,_) => {
                println!("Could not setup Engine.");
                break;
            },
            //The following events are not used for connections accepted using a listener.
            //Connection was able to open TCP stream to server.
            EngineEvent::ConnectionSucceeded(_) |
            EngineEvent::ConnectionFailed(_,_) |
            EngineEvent::SessionEstablished(_) => {},
        }
    }
}

