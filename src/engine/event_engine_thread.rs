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

use futures::{future, Future, Stream};
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::unsync::mpsc::UnboundedSender as UnsyncUnboundedSender;
use mio::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};

use dictionary::CloneDictionary;
use dictionary::messages::{Heartbeat, TestRequest};
use engine::accept_logon::accept_logon;
use engine::connection::Connection;
use engine::event_engine::{InternalEngineToThreadEvent, EngineEvent};
use engine::new_incoming_connection::NewIncomingConnection;
use engine::read_messages::ConnectionReadMessage;
use fixt::engine::Listener;
use fixt::message::BuildFIXTMessage;
use token_generator::TokenGenerator;

//TODO: Be VERY careful with error handling so connections (and their tokens) are cleaned up.
//TODO: Support cleaning-up a connection (including stopping related futures).

fn on_new_listener(token: Token,
                   token_generator: Arc<Mutex<TokenGenerator>>,
                   tx: UnboundedSender<EngineEvent>,
                   new_incoming_connections: Rc<RefCell<HashMap<usize,NewIncomingConnection>>>,
                   message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
                   max_message_size: u64,
                   handle: Handle,
                   tcp_listener: TcpListener,
                   address: SocketAddr) -> impl Future<Item=(),Error=()> {
    tcp_listener.incoming().for_each(move |(socket,address)| {
        let accepted_logon = accept_logon(Listener(token.0),
                                          token_generator.clone(),
                                          tx.clone(),
                                          new_incoming_connections.clone(),
                                          message_dictionary.clone(),
                                          max_message_size,
                                          socket,
                                          address)
                             .map_err(|_| unimplemented!());
        handle.spawn(accepted_logon);

        future::ok(())
    })
    .map_err(|_| unimplemented!())
}


pub fn run_event_engine_thread(token_generator: Arc<Mutex<TokenGenerator>>,
                               tx: UnboundedSender<EngineEvent>,
                               rx: UnboundedReceiver<InternalEngineToThreadEvent>,
                               message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
                               max_message_size: u64) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let new_incoming_connections = Rc::new(RefCell::new(HashMap::new()));
    let mut connections = HashMap::<usize,Rc<RefCell<Connection>>>::new();

    let events = rx.for_each(move |event| {
        match event {
            InternalEngineToThreadEvent::NewConnection(_token,_fix_version,_default_message_version,_sender_comp_id,_target_comp_id,_address) => {
                unimplemented!();
            },
            InternalEngineToThreadEvent::NewListener(token,_sender_comp_id,listener,address) => {
                let listener = TcpListener::from_listener(listener,&address,&handle).unwrap(); //TODO: This can only fail if the socket cannot be set to non-blocking on Linux.
                let listener_future = on_new_listener(token,
                                                      token_generator.clone(),
                                                      tx.clone(),
                                                      new_incoming_connections.clone(),
                                                      message_dictionary.clone(),
                                                      max_message_size,
                                                      handle.clone(),
                                                      listener,
                                                      address);
                handle.spawn(listener_future);
            },
            InternalEngineToThreadEvent::SendMessage(token,_message_version,message) => {
                if let Some(connection) = connections.get(&token.0) {
                    let _ = UnsyncUnboundedSender::send(&connection.borrow().outbound_messages,message);
                }
            },
            InternalEngineToThreadEvent::ResendMessages(_token,_response) => {
                unimplemented!();
            },
            InternalEngineToThreadEvent::ApproveNewConnection(connection,message,_inbound_msg_seq_num) => {
                let mut new_incoming_connections = new_incoming_connections.borrow_mut();
                if let Some(new_incoming_connection) = new_incoming_connections.remove(&connection.0) {
                    let (connection,inbound_messages,outbound_messages) = new_incoming_connection.to_connection();
                    let token = connection.token.0;
                    let connection = Rc::new(RefCell::new(connection));
                    connections.insert(token,connection.clone());

                    //Send Logon response back to connection.
                    let _ = UnsyncUnboundedSender::send(&connection.borrow().outbound_messages,message);

                    //Process received messages.
                    let on_recv_message = inbound_messages.for_each(move |messages| -> Result<(),io::Error> {
                        for message in messages {
                            match message {
                                ConnectionReadMessage::Message(message) => {
                                    //TODO: Perform actual message handling.
                                    if let Some(test_request) = message.as_any().downcast_ref::<TestRequest>() {
                                        let mut heartbeat = Heartbeat::new();
                                        heartbeat.test_req_id = test_request.test_req_id.clone();
                                        let _ = UnsyncUnboundedSender::send(&connection.borrow().outbound_messages,Box::new(heartbeat));
                                    }
                                },
                                ConnectionReadMessage::Error(_err) => {
                                    unimplemented!();
                                },
                            }
                        }

                        Ok(())
                    }).map_err(|_| unimplemented!());
                    handle.spawn(on_recv_message);

                    //Send outgoing messages as they are queued.
                    let on_send_message = outbound_messages.map_err(|_| unimplemented!());
                    handle.spawn(on_send_message);

                    //TODO: Setup timers for things like heartbeats and testrequests.
                }
                //TODO: Take the connection awaiting approval and spin up its future for input/ouput here.
            },
            InternalEngineToThreadEvent::RejectNewConnection(_connection,_reason) => {
                unimplemented!();
            },
            InternalEngineToThreadEvent::Logout(_token) => {
                unimplemented!();
            },
            InternalEngineToThreadEvent::Shutdown => {
                return Err(());
            },
        };

        Ok(())
    });

    let _ = core.run(events);
}

