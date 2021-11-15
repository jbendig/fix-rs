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

#![allow(deprecated)]

use mio::channel::{channel, Receiver, Sender};
use mio::tcp::TcpListener;
use mio::{Events, Poll, PollOpt, Ready, Token};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::mem;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Range;
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::dictionary::messages::Logon;
use crate::fix::ParseError;
use crate::fix_version::FIXVersion;
use crate::fixt::engine_thread::{
    internal_engine_thread, InternalEngineToThreadEvent, BASE_CONNECTION_TOKEN,
    CONNECTION_COUNT_MAX, INTERNAL_ENGINE_EVENT_TOKEN,
};
use crate::fixt::message::{BuildFIXTMessage, FIXTMessage};
use crate::message_version::MessageVersion;
use crate::token_generator::TokenGenerator;

const ENGINE_EVENT_TOKEN: Token = Token(0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Connection(pub usize);

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Listener(pub usize);

impl fmt::Display for Listener {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum ConnectionTerminatedReason {
    BeginStrWrongError {
        received: FIXVersion,
        expected: FIXVersion,
    },
    InboundMsgSeqNumMaxExceededError,
    InboundMsgSeqNumLowerThanExpectedError,
    InboundResendRequestLoopError,
    LocalRequested,
    LogonHeartBtIntNegativeError,
    LogonParseError(ParseError),
    LogonNeverReceivedError,
    LogonNotFirstMessageError,
    LogonRejectedError,
    LogoutNoHangUpError,
    LogoutNoResponseError,
    OutboundMsgSeqNumMaxExceededError,
    RemoteRequested,
    SenderCompIDWrongError,
    SocketNotWritableTimeoutError,
    SocketReadError(io::Error),
    SocketWriteError(io::Error),
    TargetCompIDWrongError,
    TestRequestNotRespondedError,
}

impl fmt::Debug for ConnectionTerminatedReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionTerminatedReason::BeginStrWrongError{ref received,ref expected} => {
                let received_str = String::from_utf8_lossy(received.begin_string()).into_owned();
                let expected_str = String::from_utf8_lossy(expected.begin_string()).into_owned();
                write!(f,"Received message with BeginStr containing '{}' but expected '{}'.",received_str,expected_str)
            },
            ConnectionTerminatedReason::InboundMsgSeqNumMaxExceededError => write!(f,"Expected inbound MsgSeqNum exceeded maximum allowed."),
            ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError => write!(f,"Received message with lower MsgSeqNum than expected."),
            ConnectionTerminatedReason::InboundResendRequestLoopError => write!(f,"Received too many ResendRequests with the same BeginSeqNo."),
            ConnectionTerminatedReason::LocalRequested => write!(f,"Local requested logout and it was performed cleanly."),
            ConnectionTerminatedReason::LogonHeartBtIntNegativeError => write!(f,"Response to logon included negative HeartBtInt."),
            ConnectionTerminatedReason::LogonParseError(_) => write!(f,"Could not parse logon response."), //Did you connect to a server not running a FIX engine?
            ConnectionTerminatedReason::LogonNeverReceivedError => write!(f,"Never received logon from new connection."),
            ConnectionTerminatedReason::LogonNotFirstMessageError => write!(f,"Remote responded to logon with a non-logon message."),
            ConnectionTerminatedReason::LogonRejectedError => write!(f,"Remote rejected logon for arbitrary reason."),
            ConnectionTerminatedReason::LogoutNoHangUpError => write!(f,"Remote requested logout but did not close socket after response."),
            ConnectionTerminatedReason::LogoutNoResponseError => write!(f,"Local requested logout but remote did not respond within a reasonable amount of time."),
            ConnectionTerminatedReason::OutboundMsgSeqNumMaxExceededError => write!(f,"Expected outbound MsgSeqNum exceeded maximum allowed."),
            ConnectionTerminatedReason::RemoteRequested => write!(f,"Remote requested logout and it was performed cleanly."),
            ConnectionTerminatedReason::SenderCompIDWrongError => write!(f,"Received message with SenderCompID not matching the expected value."),
            ConnectionTerminatedReason::SocketNotWritableTimeoutError => write!(f,"Socket returned WouldBlock on write for an unreasonable amount of time."),
            ConnectionTerminatedReason::SocketReadError(ref error) => write!(f,"Socket could not be read from: {}",error),
            ConnectionTerminatedReason::SocketWriteError(ref error) => write!(f,"Socket could not be written to: {}",error),
            ConnectionTerminatedReason::TargetCompIDWrongError => write!(f,"Received message with TargetCompID not matching the expected value."),
            ConnectionTerminatedReason::TestRequestNotRespondedError => write!(f,"TestRequest message not responded with Heartbeat message within a reasonable amount of time."),
        }
    }
}

pub enum EngineEvent {
    ConnectionFailed(Connection, io::Error), //Could not setup connection.
    ConnectionSucceeded(Connection),         //Connection completed and ready to begin logon.
    ConnectionTerminated(Connection, ConnectionTerminatedReason), //Connection ended for ConnectionTerminatedReason reason.
    ConnectionDropped(Listener, SocketAddr), //Connection was dropped by listener because of a lock of resources.
    ConnectionAccepted(Listener, Connection, SocketAddr), //Listener accepted a new connection and is awaiting a Logon message.
    ConnectionLoggingOn(Listener, Connection, Box<Logon>),
    SessionEstablished(Connection), //Connection completed logon process successfully.
    ListenerFailed(Listener, io::Error), //Could not setup listener.
    ListenerAcceptFailed(Listener, io::Error), //Could not accept a connection with listener.
    MessageReceived(Connection, Box<dyn FIXTMessage + Send>), //New valid message was received.
    MessageReceivedGarbled(Connection, ParseError), //New message could not be parsed correctly. (If not garbled (FIXT 1.1, page 40), a Reject will be issued first)
    MessageReceivedDuplicate(Connection, Box<dyn FIXTMessage + Send>), //Message with MsgSeqNum already seen was received.
    MessageRejected(Connection, Box<dyn FIXTMessage + Send>), //New message breaks session rules and was rejected.
    ResendRequested(Connection, Range<u64>), //Range of messages by MsgSeqNum that are requested to be resent. [Range::start,Range::end)
    SequenceResetResetHasNoEffect(Connection),
    SequenceResetResetInThePast(Connection),
    FatalError(&'static str, io::Error), //A critical error has occurred. No more events can be received and no more messages will be sent.
}

impl fmt::Debug for EngineEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EngineEvent::ConnectionFailed(connection, ref error) => write!(
                f,
                "EngineEvent::ConnectionFailed({:?},{:?})",
                connection, error
            ),
            EngineEvent::ConnectionSucceeded(connection) => {
                write!(f, "EngineEvent::ConnectionSucceeded({:?})", connection)
            }
            EngineEvent::ConnectionTerminated(connection, ref reason) => write!(
                f,
                "EngineEvent::ConnectionTerminated({:?},{:?})",
                connection, reason
            ),
            EngineEvent::ConnectionDropped(connection, addr) => write!(
                f,
                "EngineEvent::ConnectionDropped({:?},{:?})",
                connection, addr
            ),
            EngineEvent::ConnectionAccepted(listener, connection, addr) => write!(
                f,
                "EngineEvent::ConnectionAccepted({:?},{:?},{:?})",
                listener, connection, addr
            ),
            EngineEvent::ConnectionLoggingOn(listener, connection, ref message) => write!(
                f,
                "EngineEvent::ConnectionLoggingOn({:?},{:?},{:?})",
                listener, connection, &**message as &dyn FIXTMessage
            ),
            EngineEvent::SessionEstablished(connection) => {
                write!(f, "EngineEvent::SessionEstablished({:?})", connection)
            }
            EngineEvent::ListenerFailed(listener, ref error) => {
                write!(f, "EngineEvent::ListenerFailed({:?},{:?})", listener, error)
            }
            EngineEvent::ListenerAcceptFailed(listener, ref error) => write!(
                f,
                "EngineEvent::ListenerAcceptFailed({:?},{:?})",
                listener, error
            ),
            EngineEvent::MessageReceived(connection, ref message) => write!(
                f,
                "EngineEvent::MessageReceived({:?},{:?})",
                connection, message
            ),
            EngineEvent::MessageReceivedGarbled(connection, ref parse_error) => write!(
                f,
                "EngineEvent::MessageReceivedGarbled({:?},{:?})",
                connection, parse_error
            ),
            EngineEvent::MessageReceivedDuplicate(connection, ref message) => write!(
                f,
                "EngineEvent::MessageReceivedDuplicate({:?},{:?})",
                connection, message
            ),
            EngineEvent::MessageRejected(connection, ref message) => write!(
                f,
                "EngineEvent::MessageRejected({:?},{:?})",
                connection, message
            ),
            EngineEvent::ResendRequested(connection, ref range) => write!(
                f,
                "EngineEvent::ResendRequested({:?},{:?})",
                connection, range
            ),
            EngineEvent::SequenceResetResetHasNoEffect(connection) => write!(
                f,
                "EngineEvent:SequenceResetResetHasNoEffect({:?})",
                connection
            ),
            EngineEvent::SequenceResetResetInThePast(connection) => write!(
                f,
                "EngineEvent:SequenceResetResetInThePast({:?})",
                connection
            ),
            EngineEvent::FatalError(description, ref error) => {
                write!(f, "EngineEvent::FatalError({:?},{:?})", description, error)
            }
        }
    }
}

pub enum ResendResponse {
    Message(Option<MessageVersion>, Box<dyn FIXTMessage + Send>),
    Gap(Range<u64>),
}

fn to_socket_addr<A: ToSocketAddrs>(address: A) -> Option<SocketAddr> {
    //Use first socket address. This more or less emulates TcpStream::connect.
    match address.to_socket_addrs() {
        Ok(mut address_iter) => address_iter.next(),
        Err(_) => None,
    }
}

pub struct Engine {
    token_generator: Arc<Mutex<TokenGenerator>>,
    tx: Sender<InternalEngineToThreadEvent>,
    rx: Receiver<EngineEvent>,
    poll: Poll,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Engine {
    pub fn new(
        message_dictionary: HashMap<&'static [u8], Box<dyn BuildFIXTMessage + Send>>,
        max_message_size: u64,
    ) -> Result<Engine, io::Error> {
        let engine_poll = Poll::new()?;
        let (thread_to_engine_tx, thread_to_engine_rx) = channel::<EngineEvent>();
        engine_poll.register(
            &thread_to_engine_rx,
            ENGINE_EVENT_TOKEN,
            Ready::readable(),
            PollOpt::level(),
        )?;

        let poll = Poll::new()?;
        let (engine_to_thread_tx, engine_to_thread_rx) = channel::<InternalEngineToThreadEvent>();
        poll.register(
            &engine_to_thread_rx,
            INTERNAL_ENGINE_EVENT_TOKEN,
            Ready::readable(),
            PollOpt::level(),
        )?;

        let token_generator = Arc::new(Mutex::new(TokenGenerator::new(
            BASE_CONNECTION_TOKEN.0,
            Some(CONNECTION_COUNT_MAX - BASE_CONNECTION_TOKEN.0),
        )));

        Ok(Engine {
            token_generator: token_generator.clone(),
            tx: engine_to_thread_tx,
            rx: thread_to_engine_rx,
            poll: engine_poll,
            thread_handle: Some(thread::spawn(move || {
                internal_engine_thread(
                    poll,
                    token_generator,
                    thread_to_engine_tx,
                    engine_to_thread_rx,
                    message_dictionary,
                    max_message_size,
                );
            })),
        })
    }

    pub fn add_connection<A: ToSocketAddrs>(
        &mut self,
        fix_version: FIXVersion,
        mut default_message_version: MessageVersion,
        sender_comp_id: &[u8],
        target_comp_id: &[u8],
        address: A,
    ) -> Option<Connection> {
        let address = match to_socket_addr(address) {
            Some(address) => address,
            None => return None,
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
        let token = match self.token_generator.lock().unwrap().create() {
            Some(token) => token,
            None => return None,
        };

        //Tell thread to setup this connection by connecting a socket and logging on.
        self.tx
            .send(InternalEngineToThreadEvent::NewConnection(
                token.clone(),
                fix_version,
                default_message_version,
                sender_comp_id.to_vec(),
                target_comp_id.to_vec(),
                address,
            ))
            .unwrap();

        let connection = Connection(token.0);
        Some(connection)
    }

    pub fn add_listener<A: ToSocketAddrs>(
        &mut self,
        sender_comp_id: &[u8],
        address: A,
    ) -> Result<Option<Listener>, io::Error> {
        let address = match to_socket_addr(address) {
            Some(address) => address,
            None => return Ok(None),
        };
        let listener = TcpListener::bind(&address)?;

        let token = match self.token_generator.lock().unwrap().create() {
            Some(token) => token,
            None => return Ok(None),
        };

        self.tx
            .send(InternalEngineToThreadEvent::NewListener(
                token.clone(),
                sender_comp_id.to_vec(),
                listener,
            ))
            .unwrap();

        let listener = Listener(token.0);
        Ok(Some(listener))
    }

    pub fn send_message<T: 'static + FIXTMessage + Send>(
        &mut self,
        connection: Connection,
        message: T,
    ) {
        let message = Box::new(message);
        self.send_message_box(connection, message);
    }

    pub fn send_message_box(
        &mut self,
        connection: Connection,
        message: Box<dyn FIXTMessage + Send>,
    ) {
        self.send_message_box_with_message_version(connection, None, message);
    }

    pub fn send_message_box_with_message_version<MV: Into<Option<MessageVersion>>>(
        &mut self,
        connection: Connection,
        message_version: MV,
        message: Box<dyn FIXTMessage + Send>,
    ) {
        self.tx
            .send(InternalEngineToThreadEvent::SendMessage(
                Token(connection.0),
                message_version.into(),
                message,
            ))
            .unwrap();
    }

    pub fn send_resend_response(&mut self, connection: Connection, response: Vec<ResendResponse>) {
        if response.is_empty() {
            return;
        }

        //Perform a quick sanity check to make sure the response is strictly increasing.
        {
            fn resend_response_end(response: &ResendResponse) -> u64 {
                match *response {
                    ResendResponse::Message(_, ref message) => message.msg_seq_num(),
                    ResendResponse::Gap(ref range) => {
                        assert!(range.start <= range.end);
                        range.end
                    }
                }
            }

            let mut iter = response.iter();
            let mut previous = resend_response_end(iter.next().unwrap());
            for item in iter {
                let next = resend_response_end(item);
                assert!(previous < next);
                previous = next;
            }
        }

        //Pass response on to actually be sent.
        self.tx
            .send(InternalEngineToThreadEvent::ResendMessages(
                Token(connection.0),
                response,
            ))
            .unwrap();
    }

    pub fn approve_new_connection<IMSN: Into<Option<u64>>>(
        &mut self,
        connection: Connection,
        message: Box<Logon>,
        inbound_msg_seq_num: IMSN,
    ) {
        self.tx
            .send(InternalEngineToThreadEvent::ApproveNewConnection(
                connection,
                message,
                inbound_msg_seq_num.into().unwrap_or(2),
            ))
            .unwrap();
    }

    pub fn reject_new_connection(&mut self, connection: Connection, reason: Option<Vec<u8>>) {
        self.tx
            .send(InternalEngineToThreadEvent::RejectNewConnection(
                connection, reason,
            ))
            .unwrap();
    }

    pub fn logout(&mut self, connection: Connection) {
        self.tx
            .send(InternalEngineToThreadEvent::Logout(Token(connection.0)))
            .unwrap();
    }

    pub fn poll<D: Into<Option<Duration>>>(&mut self, duration: D) -> Option<EngineEvent> {
        //Perform any book keeping needed to manage engine's state.
        fn update_engine(engine: &mut Engine, event: &EngineEvent) {
            match *event {
                EngineEvent::ConnectionFailed(connection, _)
                | EngineEvent::ConnectionTerminated(connection, _) => {
                    engine
                        .token_generator
                        .lock()
                        .unwrap()
                        .remove(Token(connection.0));
                }
                _ => {}
            }
        };

        if let Ok(event) = self.rx.try_recv() {
            update_engine(self, &event);
            return Some(event);
        }

        if let Some(poll_duration) = duration.into() {
            let now = Instant::now(); //Watch time manually because Mio's poll::poll() can wake immediatelly and we'll have no idea how long has elapsed.

            while let Some(poll_duration) = poll_duration.checked_sub(now.elapsed()) {
                let mut events = Events::with_capacity(1);
                if self.poll.poll(&mut events, Some(poll_duration)).is_err() {
                    return None;
                }

                let result = self.rx.try_recv();
                match result {
                    Ok(event) => {
                        update_engine(self, &event);
                        return Some(event);
                    }
                    Err(e) if e == TryRecvError::Disconnected => return None,
                    _ => {}
                }
            }
        }

        None
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        //Shutdown thread and wait until it completes. No attempt is made to make connections
        //logout cleanly.
        self.tx.send(InternalEngineToThreadEvent::Shutdown).unwrap();
        let thread_handle = mem::replace(&mut self.thread_handle, None);
        let _ = thread_handle.unwrap().join();
    }
}
