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

use futures::{Async, Future, Poll, Stream};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use mio::Token;
use std::collections::HashMap;
use std::io;
use std::mem;
use std::net::{SocketAddr, TcpListener as StdTcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio_core::reactor::{Core, Timeout};

use dictionary::fields::{SenderCompID,TargetCompID};
use dictionary::messages::Logon;
pub use engine::constants::{CONNECTION_COUNT_MAX, BASE_CONNECTION_TOKEN};
use engine::event_engine_thread::run_event_engine_thread;
use field::Field;
use field_type::FieldType;
pub use fixt::engine::{Connection, Listener, ConnectionTerminatedReason, EngineEvent, ResendResponse, to_socket_addr};
use fixt::message::{BuildFIXTMessage, FIXTMessage};
use fix_version::FIXVersion;
use message_version::MessageVersion;
use token_generator::TokenGenerator;

pub enum InternalEngineToThreadEvent {
    NewConnection(Token,FIXVersion,MessageVersion,<<SenderCompID as Field>::Type as FieldType>::Type,<<TargetCompID as Field>::Type as FieldType>::Type,SocketAddr),
    NewListener(Token,<<SenderCompID as Field>::Type as FieldType>::Type,StdTcpListener,SocketAddr),
    SendMessage(Token,Option<MessageVersion>,Box<FIXTMessage + Send>),
    ResendMessages(Token,Vec<ResendResponse>),
    ApproveNewConnection(Connection,Box<Logon>,u64),
    RejectNewConnection(Connection,Option<Vec<u8>>),
    Logout(Token),
    Shutdown,
}

pub struct EventEngine {
    token_generator: Arc<Mutex<TokenGenerator>>,
    tx: UnboundedSender<InternalEngineToThreadEvent>,
    rx: UnboundedReceiver<EngineEvent>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl EventEngine {
    pub fn new(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
               max_message_size: u64) -> Result<EventEngine,io::Error> {
        let (thread_to_engine_tx,thread_to_engine_rx) = unbounded::<EngineEvent>();
        let (engine_to_thread_tx,engine_to_thread_rx) = unbounded::<InternalEngineToThreadEvent>();

        let token_generator = Arc::new(Mutex::new(TokenGenerator::new(BASE_CONNECTION_TOKEN.0,Some(CONNECTION_COUNT_MAX - BASE_CONNECTION_TOKEN.0))));

        Ok(EventEngine {
            token_generator: token_generator.clone(),
            tx: engine_to_thread_tx,
            rx: thread_to_engine_rx,
            thread_handle: Some(thread::spawn(move ||{
                run_event_engine_thread(token_generator,thread_to_engine_tx,engine_to_thread_rx,message_dictionary,max_message_size);
            })),
        })
    }

    pub fn add_connection<A: ToSocketAddrs>(&mut self,
                                            _fix_version: FIXVersion,
                                            mut _default_message_version: MessageVersion,
                                            _sender_comp_id: &[u8],
                                            _target_comp_id: &[u8],
                                            _address: A) -> Option<Connection> {
        unimplemented!()
    }

    pub fn add_listener<A: ToSocketAddrs>(&mut self,sender_comp_id: &[u8],address: A) -> Result<Option<Listener>,io::Error> {
        let address = match to_socket_addr(address) {
            Some(address) => address,
            None => return Ok(None),
        };
        let listener = try!(StdTcpListener::bind(&address));

        let token = match self.token_generator.lock().unwrap().create() {
            Some(token) => token,
            None => return Ok(None),
        };

        self.tx.send(InternalEngineToThreadEvent::NewListener(token.clone(),sender_comp_id.to_vec(),listener,address)).unwrap();

        let listener = Listener(token.0);
        Ok(Some(listener))
    }

    pub fn send_message<T: 'static + FIXTMessage + Send>(&mut self,connection: Connection,message: T) {
        let message = Box::new(message);
        self.send_message_box(connection,message);
    }

    pub fn send_message_box(&mut self,connection: Connection,message: Box<FIXTMessage + Send>) {
        self.send_message_box_with_message_version(connection,None,message);
    }

    pub fn send_message_box_with_message_version<MV: Into<Option<MessageVersion>>>(&mut self,connection: Connection,message_version: MV,message: Box<FIXTMessage + Send>) {
        self.tx.send(InternalEngineToThreadEvent::SendMessage(Token(connection.0),message_version.into(),message)).unwrap();
    }

    pub fn send_resend_response(&mut self,_connection: Connection,_response: Vec<ResendResponse>) {
        unimplemented!()
    }

    pub fn approve_new_connection<IMSN: Into<Option<u64>>>(&mut self,connection: Connection,message: Box<Logon>,inbound_msg_seq_num: IMSN) {
        self.tx.send(InternalEngineToThreadEvent::ApproveNewConnection(connection,message,inbound_msg_seq_num.into().unwrap_or(2))).unwrap();
    }

    pub fn reject_new_connection(&mut self,connection: Connection,reason: Option<Vec<u8>>) {
        self.tx.send(InternalEngineToThreadEvent::RejectNewConnection(connection,reason)).unwrap();
    }

    pub fn logout(&mut self,connection: Connection) {
        self.tx.send(InternalEngineToThreadEvent::Logout(Token(connection.0))).unwrap();
    }

    pub fn poll<D: Into<Option<Duration>>>(&mut self,duration: D) -> Option<EngineEvent> {
        let duration = duration.into().unwrap_or(Duration::from_millis(0));
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let timeout = Timeout::new(duration,&handle).unwrap();
        let poll_with_timeout = self.select(timeout.map(|_| None).map_err(|_| ())).then(|result| {
            match result {
                Ok((Some(event), _timeout)) => Ok(event),
                _ => Err(()),
            }
        });
        core.run(poll_with_timeout).map(|event| Some(event)).unwrap_or(None)
    }
}

impl Future for EventEngine {
    type Item = Option<EngineEvent>;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item,Self::Error> {
        fn update_engine(engine: &mut EventEngine,event: &EngineEvent) {
            match *event {
                EngineEvent::ConnectionFailed(connection,_) |
                EngineEvent::ConnectionTerminated(connection,_) => {
                    engine.token_generator.lock().unwrap().remove(Token(connection.0));
                },
                _ => {},
            }
        };

        let result = self.rx.poll().map_err(|_| ());

        //Perform any book keeping needed to manage engine's state.
        if let Ok(ref async) = result {
            if let Async::Ready(ref event) = *async {
                if let Some(ref event) = *event {
                    update_engine(self,&event);
                }
            }
        }

        result
    }
}

impl Drop for EventEngine {
    fn drop(&mut self) {
        //Shutdown thread and wait until it completes. No attempt is made to make connections
        //logout cleanly.
        self.tx.send(InternalEngineToThreadEvent::Shutdown).unwrap();
        let thread_handle = mem::replace(&mut self.thread_handle,None);
        let _ = thread_handle.unwrap().join();
    }
}
