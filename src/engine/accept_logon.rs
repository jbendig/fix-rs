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

use futures::{future, Future};
use futures::sync::mpsc::UnboundedSender;
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio_core::net::TcpStream;

use dictionary::administrative_msg_types;
use dictionary::field_types::other::MsgDirection;
use dictionary::messages::Logon;
use fixt::engine::{Connection, Listener};
use engine::constants::NO_INBOUND_TIMEOUT_PADDING_MS;
use engine::event_engine::EngineEvent;
use engine::new_incoming_connection::NewIncomingConnection;
use engine::read_message::{ConnectionReadMessage, ReadMessage};
use fixt::message::BuildFIXTMessage;
use message_version::MessageVersion;
use token_generator::TokenGenerator;

//TODO: Setup timers to auto-disconnect if Logon is not received.

fn allocate_token(listener: Listener,token_generator: Arc<Mutex<TokenGenerator>>,tx: UnboundedSender<EngineEvent>,address: SocketAddr) -> impl Future<Item=Token,Error=io::Error> {
    let token = match token_generator.lock().unwrap().create() {
        Some(token) => token,
        None => {
            tx.send(EngineEvent::ConnectionDropped(listener,address)).unwrap();
            return future::err(io::Error::new(io::ErrorKind::Other,"No available tokens")); //TODO: Should standardize this message.
        }
    };

    //Let engine know about the connection and have a chance to reject it
    //before remote sends a Logon message.
    tx.send(EngineEvent::ConnectionAccepted(listener,Connection(token.0),address.clone())).unwrap();

    future::ok(token)
}

pub fn accept_logon(listener: Listener,
                    token_generator: Arc<Mutex<TokenGenerator>>,
                    tx: UnboundedSender<EngineEvent>,
                    new_incoming_connections: Rc<RefCell<HashMap<usize,NewIncomingConnection>>>,
                    message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
                    max_message_size: u64,
                    socket: TcpStream,
                    address: SocketAddr) -> impl Future<Item=()> {
    let allocate_token = allocate_token(listener,token_generator,tx.clone(),address);

    let approve_logon = allocate_token.and_then(move |token| {
        let read_message = ReadMessage::new(socket,message_dictionary,max_message_size);
        read_message.and_then(move |(mut read_messages,write_messages,message)| {
            if let ConnectionReadMessage::Message(message) = message {
                //TODO: Need to check if target_comp_id is correct before accepting this message.
                if let Some(message) = message.as_any().downcast_ref::<Logon>() {
                    let fix_version = message.meta.as_ref().expect("Meta should be set by parser").begin_string;

                    let (outbound_heartbeat_timeout_duration, inbound_testrequest_timeout_duration) = if message.heart_bt_int > 0 {
                        (Some(Duration::from_secs(message.heart_bt_int as u64)),
                         Some(Duration::from_millis(message.heart_bt_int as u64 * 1000 + NO_INBOUND_TIMEOUT_PADDING_MS)))
                    }
                    else if message.heart_bt_int < 0 {
                        //TODO: Begin logout process here because HeartBtInt cannot be negative.
                        unimplemented!();
                    }
                    else {
                        (None, None)
                    };

                    read_messages.parser.set_default_message_version(message.default_appl_ver_id);

                    //Make parser use the max supported message version for the selected FIX protocol
                    //by default. This must be done before the part below so defaults in the Logon
                    //message can't maliciously overwrite them.
                    read_messages.parser.clear_default_message_type_versions();
                    for msg_type in administrative_msg_types() {
                        read_messages.parser.set_default_message_type_version(msg_type,fix_version.max_message_version());
                    }

                    //Make parser use the Message Type Default Application Version if specified.
                    for msg_type in &message.no_msg_types {
                        if msg_type.default_ver_indicator && msg_type.msg_direction == MsgDirection::Send && msg_type.ref_appl_ver_id.is_some() {
                            read_messages.parser.set_default_message_type_version(&msg_type.ref_msg_type[..],msg_type.ref_appl_ver_id.unwrap());
                        }
                    }

                    let new_incoming_connection = NewIncomingConnection {
                        token: token,
                        fix_version: fix_version,
                        default_message_version: MessageVersion::FIX50SP2, //TODO: Do not hard code this.
                        outbound_messages: write_messages,
                        outbound_heartbeat_timeout_duration: outbound_heartbeat_timeout_duration,
                        inbound_messages: read_messages,
                        inbound_msg_seq_num: message.msg_seq_num + 1,
                        inbound_testrequest_timeout_duration: inbound_testrequest_timeout_duration,
                        sender_comp_id: b"TODO".to_vec(), //TODO: Do not hard code this.
                        target_comp_id: message.sender_comp_id.clone(),

                    };
                    new_incoming_connections.borrow_mut().insert(token.0,new_incoming_connection);

                    tx.send(EngineEvent::ConnectionLoggingOn(listener,Connection(token.0),Box::new(message.clone()))).unwrap();

                    return future::ok(());
                }
                else {
                    future::err(io::Error::new(io::ErrorKind::Other,"First message not Logon"))
                }
            }
            else {
                future::err(io::Error::new(io::ErrorKind::Other,"Parsing error"))
            }
        })
    });

    approve_logon
}
