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

use mio::{Events,Poll,PollOpt,Ready,Token};
use mio::channel::{channel,Receiver,Sender};
use std::collections::{HashMap,HashSet};
use std::fmt;
use std::io;
use std::mem;
use std::net::ToSocketAddrs;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::{Duration,Instant};

use fixt::client_thread::{CONNECTION_COUNT_MAX,BASE_CONNECTION_TOKEN,INTERNAL_CLIENT_EVENT_TOKEN,InternalClientToThreadEvent,internal_client_thread};
use fixt::message::FIXTMessage;
use dictionary::fields::{SenderCompID,TargetCompID};
use field::Field;
use field_type::FieldType;
use fix::ParseError;
use fix_version::FIXVersion;
use message_version::MessageVersion;

const CLIENT_EVENT_TOKEN: Token = Token(0);

pub enum ConnectionTerminatedReason {
    BeginStrWrongError{ received: FIXVersion, expected: FIXVersion },
    ClientRequested,
    InboundMsgSeqNumMaxExceededError,
    InboundMsgSeqNumLowerThanExpectedError,
    LogonHeartBtIntNegativeError,
    LogonParseError(ParseError),
    LogonNotFirstMessageError,
    LogoutNoHangUpError,
    LogoutNoResponseError,
    OutboundMsgSeqNumMaxExceededError,
    SenderCompIDWrongError,
    ServerRequested,
    SocketReadError(io::Error),
    SocketWriteError(io::Error),
    TargetCompIDWrongError,
    TestRequestNotRespondedError,
}

impl fmt::Debug for ConnectionTerminatedReason {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionTerminatedReason::BeginStrWrongError{ref received,ref expected} => {
                let received_str = String::from_utf8_lossy(received.begin_string()).into_owned();
                let expected_str = String::from_utf8_lossy(expected.begin_string()).into_owned();
                write!(f,"Received message with BeginStr containing '{}' but expected '{}'.",received_str,expected_str)
            },
            ConnectionTerminatedReason::ClientRequested => write!(f,"Client requested logout and it was performed cleanly."),
            ConnectionTerminatedReason::InboundMsgSeqNumMaxExceededError => write!(f,"Expected inbound MsgSeqNum exceeded maximum allowed."),
            ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError => write!(f,"Received message with lower MsgSeqNum than expected."),
            ConnectionTerminatedReason::LogonHeartBtIntNegativeError => write!(f,"Response to logon included negative HeartBtInt."),
            ConnectionTerminatedReason::LogonParseError(_) => write!(f,"Could not parse logon response."), //Did you connect to a server not running a FIX engine?
            ConnectionTerminatedReason::LogonNotFirstMessageError => write!(f,"Server responded to logon with a non-logon message."),
            ConnectionTerminatedReason::LogoutNoHangUpError => write!(f,"Server requested logout but did not close socket after response."),
            ConnectionTerminatedReason::LogoutNoResponseError => write!(f,"Client requested logout but server did not respond within a reasonable amount of time."),
            ConnectionTerminatedReason::OutboundMsgSeqNumMaxExceededError => write!(f,"Expected outbound MsgSeqNum exceeded maximum allowed."),
            ConnectionTerminatedReason::SenderCompIDWrongError => write!(f,"Received message with SenderCompID not matching the expected value."),
            ConnectionTerminatedReason::ServerRequested => write!(f,"Server requested logout and it was performed cleanly."),
            ConnectionTerminatedReason::SocketReadError(ref error) => write!(f,"Socket could not be read from: {}",error),
            ConnectionTerminatedReason::SocketWriteError(ref error) => write!(f,"Socket could not be written to: {}",error),
            ConnectionTerminatedReason::TargetCompIDWrongError => write!(f,"Received message with TargetCompID not matching the expected value."),
            ConnectionTerminatedReason::TestRequestNotRespondedError => write!(f,"TestRequest message not responded with Heartbeat message within a reasonable amount of time."),
        }
    }
}

pub enum ClientEvent {
    ConnectionFailed(usize,io::Error), //Could not setup connection.
    ConnectionTerminated(usize,ConnectionTerminatedReason), //Connection ended for ConnectionTerminatedReason reason.
    SessionEstablished(usize), //Connection completed logon process successfully.
    MessageReceived(usize,Box<FIXTMessage + Send>), //New valid message was received.
    MessageReceivedGarbled(usize,ParseError), //New message could not be parsed correctly. (If not garbled (FIXT 1.1, page 40), a Reject will be issued first)
    MessageReceivedDuplicate(usize,Box<FIXTMessage + Send>), //Message with MsgSeqNum already seen was received.
    MessageRejected(usize,Box<FIXTMessage + Send>), //New message breaks session rules and was rejected.
    SequenceResetResetHasNoEffect(usize),
    SequenceResetResetInThePast(usize),
    FatalError(&'static str,io::Error), //TODO: Probably should have an error type instead of static str here.
}

impl fmt::Debug for ClientEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClientEvent::ConnectionFailed(connection_id,ref error) => write!(f,"ClientEvent::ConnectionFailed({},{:?})",connection_id,error),
            ClientEvent::ConnectionTerminated(connection_id,ref reason) => write!(f,"ClientEvent::ConnectionTerminated({},{:?})",connection_id,reason),
            ClientEvent::SessionEstablished(connection_id) => write!(f,"ClientEvent::SessionEstablished({})",connection_id),
            ClientEvent::MessageReceived(connection_id,ref message) => write!(f,"ClientEvent::MessageReceived({},{:?})",connection_id,message),
            ClientEvent::MessageReceivedGarbled(connection_id,ref parse_error) => write!(f,"ClientEvent::MessageReceivedGarbled({},{:?})",connection_id,parse_error),
            ClientEvent::MessageReceivedDuplicate(connection_id,ref message) => write!(f,"ClientEvent::MessageReceivedDuplicate({},{:?})",connection_id,message),
            ClientEvent::MessageRejected(connection_id,ref message) => write!(f,"ClientEvent::MessageRejected({},{:?})",connection_id,message),
            ClientEvent::SequenceResetResetHasNoEffect(connection_id) => write!(f,"ClientEvent:SequenceResetResetHasNoEffect({})",connection_id),
            ClientEvent::SequenceResetResetInThePast(connection_id) => write!(f,"ClientEvent:SequenceResetResetInThePast({})",connection_id),
            ClientEvent::FatalError(description,ref error) => write!(f,"ClientEvent::FatalError({},{:?})",description,error),
        }
    }
}

pub struct Client {
    connection_id_seed: usize,
    active_connections: HashSet<usize>,
    tx: Sender<InternalClientToThreadEvent>,
    rx: Receiver<ClientEvent>,
    poll: Poll,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Client {
    pub fn new(message_dictionary: HashMap<&'static [u8],Box<FIXTMessage + Send>>,
               sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
               target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) -> Result<Client,io::Error> {
        let client_poll = try!(Poll::new());
        let (thread_to_client_tx,thread_to_client_rx) = channel::<ClientEvent>();
        try!(client_poll.register(&thread_to_client_rx,CLIENT_EVENT_TOKEN,Ready::readable(),PollOpt::level()));

        let poll = try!(Poll::new());
        let (client_to_thread_tx,client_to_thread_rx) = channel::<InternalClientToThreadEvent>();
        try!(poll.register(&client_to_thread_rx,INTERNAL_CLIENT_EVENT_TOKEN,Ready::readable(),PollOpt::level()));

        Ok(Client {
            connection_id_seed: BASE_CONNECTION_TOKEN.0,
            active_connections: HashSet::new(),
            tx: client_to_thread_tx,
            rx: thread_to_client_rx,
            poll: client_poll,
            thread_handle: Some(thread::spawn(move || {
                internal_client_thread(poll,thread_to_client_tx,client_to_thread_rx,message_dictionary,sender_comp_id,target_comp_id);
            })),
        })
    }

    pub fn add_connection<A: ToSocketAddrs>(&mut self,fix_version: FIXVersion,mut default_message_version: MessageVersion,address: A) -> Option<usize> {
        //Use first socket address. This more or less emulates TcpStream::connect.
        let address = match address.to_socket_addrs() {
            Ok(mut address_iter) => {
                match address_iter.next() {
                    Some(address) => address,
                    None => return None,
                }
            },
            Err(_) => return None,
        };

        //Force older FIX versions that don't support message versioning to use their respective
        //message versions.
        default_message_version = match fix_version {
            FIXVersion::FIX_4_0 => MessageVersion::FIX40,
            FIXVersion::FIX_4_1 => MessageVersion::FIX41,
            FIXVersion::FIX_4_2 => MessageVersion::FIX42,
            FIXVersion::FIX_4_3 => MessageVersion::FIX43,
            FIXVersion::FIX_4_4 => MessageVersion::FIX44,
            FIXVersion::FIXT_1_1 => default_message_version,
        };

        //Create unique id to refer to connection by.
        let connection_id = match self.generate_connection_id() {
            Some(connection_id) => connection_id,
            None => return None,
        };

        //Tell thread to setup this connection by connecting a socket and logging on.
        self.tx.send(InternalClientToThreadEvent::NewConnection(Token(connection_id),fix_version,default_message_version,address)).unwrap();

        Some(connection_id)
    }

    pub fn send_message<T: 'static + FIXTMessage + Send>(&mut self,connection_id: usize,message: T) {
        let message = Box::new(message);
        self.send_message_box(connection_id,message);
    }

    pub fn send_message_box(&mut self,connection_id: usize,message: Box<FIXTMessage + Send>) {
        self.send_message_box_with_message_version(connection_id,None,message);
    }

    pub fn send_message_box_with_message_version<MV: Into<Option<MessageVersion>>>(&mut self,connection_id: usize,message_version: MV,message: Box<FIXTMessage + Send>) {
        self.tx.send(InternalClientToThreadEvent::SendMessage(Token(connection_id),message_version.into(),message)).unwrap();
    }

    pub fn logout(&mut self,connection_id: usize) {
        self.tx.send(InternalClientToThreadEvent::Logout(Token(connection_id))).unwrap();
    }

    pub fn poll<D: Into<Option<Duration>>>(&mut self,duration: D) -> Option<ClientEvent> {
        //Perform any book keeping needed to manage client's state.
        fn update_client(client: &mut Client,event: &ClientEvent) {
            match *event {
                ClientEvent::ConnectionFailed(connection_id,_) |
                ClientEvent::ConnectionTerminated(connection_id,_) => {
                    client.active_connections.remove(&connection_id);
                },
                _ => {},
            }
        };

        if let Ok(event) = self.rx.try_recv() {
            update_client(self,&event);
            return Some(event);
        }

        if let Some(poll_duration) = duration.into() {
            let now = Instant::now(); //Watch time manually because Mio's poll::poll() can wake immediatelly and we'll have no idea how long has elapsed.

            while let Some(poll_duration) = poll_duration.checked_sub(now.elapsed()) {
                let mut events = Events::with_capacity(1);
                if self.poll.poll(&mut events,Some(poll_duration)).is_err() {
                    return None;
                }

                let result = self.rx.try_recv();
                match result {
                    Ok(event) => {
                        update_client(self,&event);
                        return Some(event);
                    },
                    Err(e) if e == TryRecvError::Disconnected => return None,
                    _ => {},
                }
            }
        }

        None
    }

    fn generate_connection_id(&mut self) -> Option<usize> {
        //Check that we don't already have all possible connections created. This should only be
        //possible if a whole bunch of connections are created and poll() is never called. A
        //majority of these connections very likely failed because of the limited number of socket
        //ports.
        if self.active_connections.len() == (CONNECTION_COUNT_MAX - BASE_CONNECTION_TOKEN.0) {
            return None;
        }

        loop {
            let connection_id = self.connection_id_seed;
            self.connection_id_seed = self.connection_id_seed.overflowing_add(1).0;

            if !self.active_connections.contains(&connection_id) && connection_id >= BASE_CONNECTION_TOKEN.0 {
                return Some(connection_id);
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        //Shutdown thread and wait until it completes. No attempt is made to make connections
        //logout cleanly.
        self.tx.send(InternalClientToThreadEvent::Shutdown).unwrap();
        let thread_handle = mem::replace(&mut self.thread_handle,None);
        let _ = thread_handle.unwrap().join();
    }
}

