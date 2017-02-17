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
use std::collections::HashMap;
use mio::tcp::TcpListener;
use std::fmt;
use std::io;
use std::mem;
use std::net::{SocketAddr,ToSocketAddrs};
use std::sync::{Arc,Mutex};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::{Duration,Instant};

use dictionary::messages::Logon;
use fixt::client_thread::{CONNECTION_COUNT_MAX,BASE_CONNECTION_TOKEN,INTERNAL_CLIENT_EVENT_TOKEN,InternalClientToThreadEvent,internal_client_thread};
use fixt::message::{BuildFIXTMessage,FIXTMessage,debug_format_fixt_message};
use fix::ParseError;
use fix_version::FIXVersion;
use message_version::MessageVersion;
use token_generator::TokenGenerator;

const CLIENT_EVENT_TOKEN: Token = Token(0);

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
pub struct Connection(pub usize);

impl fmt::Display for Connection {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
pub struct Listener(pub usize);

impl fmt::Display for Listener {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

pub enum ConnectionTerminatedReason {
    BeginStrWrongError{ received: FIXVersion, expected: FIXVersion },
    ClientRequested,
    InboundMsgSeqNumMaxExceededError,
    InboundMsgSeqNumLowerThanExpectedError,
    InboundResendRequestLoopError,
    LogonHeartBtIntNegativeError,
    LogonParseError(ParseError),
    LogonNeverReceivedError,
    LogonNotFirstMessageError,
    LogonRejectedError,
    LogoutNoHangUpError,
    LogoutNoResponseError,
    OutboundMsgSeqNumMaxExceededError,
    SenderCompIDWrongError,
    ServerRequested,
    SocketNotWritableTimeoutError,
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
            ConnectionTerminatedReason::InboundResendRequestLoopError => write!(f,"Received too many ResendRequests with the same BeginSeqNo."),
            ConnectionTerminatedReason::LogonHeartBtIntNegativeError => write!(f,"Response to logon included negative HeartBtInt."),
            ConnectionTerminatedReason::LogonParseError(_) => write!(f,"Could not parse logon response."), //Did you connect to a server not running a FIX engine?
            ConnectionTerminatedReason::LogonNeverReceivedError => write!(f,"Never received logon from new connection."),
            ConnectionTerminatedReason::LogonNotFirstMessageError => write!(f,"Server responded to logon with a non-logon message."),
            ConnectionTerminatedReason::LogonRejectedError => write!(f,"Server rejected logon for arbitrary reason."),
            ConnectionTerminatedReason::LogoutNoHangUpError => write!(f,"Server requested logout but did not close socket after response."),
            ConnectionTerminatedReason::LogoutNoResponseError => write!(f,"Client requested logout but server did not respond within a reasonable amount of time."),
            ConnectionTerminatedReason::OutboundMsgSeqNumMaxExceededError => write!(f,"Expected outbound MsgSeqNum exceeded maximum allowed."),
            ConnectionTerminatedReason::SenderCompIDWrongError => write!(f,"Received message with SenderCompID not matching the expected value."),
            ConnectionTerminatedReason::ServerRequested => write!(f,"Server requested logout and it was performed cleanly."),
            ConnectionTerminatedReason::SocketNotWritableTimeoutError => write!(f,"Socket returned WouldBlock on write for an unreasonable amount of time."),
            ConnectionTerminatedReason::SocketReadError(ref error) => write!(f,"Socket could not be read from: {}",error),
            ConnectionTerminatedReason::SocketWriteError(ref error) => write!(f,"Socket could not be written to: {}",error),
            ConnectionTerminatedReason::TargetCompIDWrongError => write!(f,"Received message with TargetCompID not matching the expected value."),
            ConnectionTerminatedReason::TestRequestNotRespondedError => write!(f,"TestRequest message not responded with Heartbeat message within a reasonable amount of time."),
        }
    }
}

pub enum ClientEvent {
    ConnectionFailed(Connection,io::Error), //Could not setup connection.
    ConnectionSucceeded(Connection), //Connection completed and ready to begin logon.
    ConnectionTerminated(Connection,ConnectionTerminatedReason), //Connection ended for ConnectionTerminatedReason reason.
    ConnectionDropped(Listener,SocketAddr), //Connection was dropped by listener because of a lock of resources.
    ConnectionAccepted(Listener,Connection,SocketAddr), //Listener accepted a new connection and is awaiting a Logon message.
    ConnectionLoggingOn(Listener,Connection,Box<Logon>),
    SessionEstablished(Connection), //Connection completed logon process successfully.
    ListenerFailed(Listener,io::Error), //Could not accept a connection with listener.
    MessageReceived(Connection,Box<FIXTMessage + Send>), //New valid message was received.
    MessageReceivedGarbled(Connection,ParseError), //New message could not be parsed correctly. (If not garbled (FIXT 1.1, page 40), a Reject will be issued first)
    MessageReceivedDuplicate(Connection,Box<FIXTMessage + Send>), //Message with MsgSeqNum already seen was received.
    MessageRejected(Connection,Box<FIXTMessage + Send>), //New message breaks session rules and was rejected.
    SequenceResetResetHasNoEffect(Connection),
    SequenceResetResetInThePast(Connection),
    FatalError(&'static str,io::Error), //TODO: Probably should have an error type instead of static str here.
}

impl fmt::Debug for ClientEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClientEvent::ConnectionFailed(connection,ref error) => write!(f,"ClientEvent::ConnectionFailed({:?},{:?})",connection,error),
            ClientEvent::ConnectionSucceeded(connection) => write!(f,"ClientEvent::ConnectionSucceeded({:?})",connection),
            ClientEvent::ConnectionTerminated(connection,ref reason) => write!(f,"ClientEvent::ConnectionTerminated({:?},{:?})",connection,reason),
            ClientEvent::ConnectionDropped(connection,addr) => write!(f,"ClientEvent::ConnectionDropped({:?},{:?})",connection,addr),
            ClientEvent::ConnectionAccepted(listener,connection,addr) => write!(f,"ClientEvent::ConnectionAccepted({:?},{:?},{:?})",listener,connection,addr),
            ClientEvent::ConnectionLoggingOn(listener,connection,ref message) => write!(f,"ClientEvent::ConnectionLoggingOn({:?},{:?},",listener,connection)
                                                                                 .and_then(|_| debug_format_fixt_message(&**message as &FIXTMessage,f))
                                                                                 .and_then(|_| write!(f,")")),
            ClientEvent::SessionEstablished(connection) => write!(f,"ClientEvent::SessionEstablished({:?})",connection),
            ClientEvent::ListenerFailed(listener,ref error) => write!(f,"ClientEvent::ListenerFailed({:?},{:?})",listener,error),
            ClientEvent::MessageReceived(connection,ref message) => write!(f,"ClientEvent::MessageReceived({:?},{:?})",connection,message),
            ClientEvent::MessageReceivedGarbled(connection,ref parse_error) => write!(f,"ClientEvent::MessageReceivedGarbled({:?},{:?})",connection,parse_error),
            ClientEvent::MessageReceivedDuplicate(connection,ref message) => write!(f,"ClientEvent::MessageReceivedDuplicate({:?},{:?})",connection,message),
            ClientEvent::MessageRejected(connection,ref message) => write!(f,"ClientEvent::MessageRejected({:?},{:?})",connection,message),
            ClientEvent::SequenceResetResetHasNoEffect(connection) => write!(f,"ClientEvent:SequenceResetResetHasNoEffect({:?})",connection),
            ClientEvent::SequenceResetResetInThePast(connection) => write!(f,"ClientEvent:SequenceResetResetInThePast({:?})",connection),
            ClientEvent::FatalError(description,ref error) => write!(f,"ClientEvent::FatalError({:?},{:?})",description,error),
        }
    }
}


fn to_socket_addr<A: ToSocketAddrs>(address: A) -> Option<SocketAddr> {
    //Use first socket address. This more or less emulates TcpStream::connect.
    match address.to_socket_addrs() {
        Ok(mut address_iter) => address_iter.next(),
        Err(_) => None,
    }
}

pub struct Client {
    token_generator: Arc<Mutex<TokenGenerator>>,
    tx: Sender<InternalClientToThreadEvent>,
    rx: Receiver<ClientEvent>,
    poll: Poll,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Client {
    pub fn new(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
               max_message_size: u64) -> Result<Client,io::Error> {
        let client_poll = try!(Poll::new());
        let (thread_to_client_tx,thread_to_client_rx) = channel::<ClientEvent>();
        try!(client_poll.register(&thread_to_client_rx,CLIENT_EVENT_TOKEN,Ready::readable(),PollOpt::level()));

        let poll = try!(Poll::new());
        let (client_to_thread_tx,client_to_thread_rx) = channel::<InternalClientToThreadEvent>();
        try!(poll.register(&client_to_thread_rx,INTERNAL_CLIENT_EVENT_TOKEN,Ready::readable(),PollOpt::level()));

        let token_generator = Arc::new(Mutex::new(TokenGenerator::new(BASE_CONNECTION_TOKEN.0,Some(CONNECTION_COUNT_MAX - BASE_CONNECTION_TOKEN.0))));

        Ok(Client {
            token_generator: token_generator.clone(),
            tx: client_to_thread_tx,
            rx: thread_to_client_rx,
            poll: client_poll,
            thread_handle: Some(thread::spawn(move || {
                internal_client_thread(poll,token_generator,thread_to_client_tx,client_to_thread_rx,message_dictionary,max_message_size);
            })),
        })
    }

    pub fn add_connection<A: ToSocketAddrs>(&mut self,
                                            fix_version: FIXVersion,
                                            mut default_message_version: MessageVersion,
                                            sender_comp_id: &[u8],
                                            target_comp_id: &[u8],
                                            address: A) -> Option<Connection> {
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
        self.tx.send(InternalClientToThreadEvent::NewConnection(token.clone(),fix_version,default_message_version,sender_comp_id.to_vec(),target_comp_id.to_vec(),address)).unwrap();

        let connection = Connection(token.0);
        Some(connection)
    }

    pub fn add_listener<A: ToSocketAddrs>(&mut self,sender_comp_id: &[u8],address: A) -> Result<Option<Listener>,io::Error> {
        let address = match to_socket_addr(address) {
            Some(address) => address,
            None => return Ok(None),
        };
        let listener = try!(TcpListener::bind(&address));

        let token = match self.token_generator.lock().unwrap().create() {
            Some(token) => token,
            None => return Ok(None),
        };

        self.tx.send(InternalClientToThreadEvent::NewListener(token.clone(),sender_comp_id.to_vec(),listener)).unwrap();

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
        self.tx.send(InternalClientToThreadEvent::SendMessage(Token(connection.0),message_version.into(),message)).unwrap();
    }

    pub fn approve_new_connection<IMSN: Into<Option<u64>>>(&mut self,connection: Connection,message: Box<Logon>,inbound_msg_seq_num: IMSN) {
        self.tx.send(InternalClientToThreadEvent::ApproveNewConnection(connection,message,inbound_msg_seq_num.into().unwrap_or(2))).unwrap();
    }

    pub fn reject_new_connection(&mut self,connection: Connection,reason: Option<Vec<u8>>) {
        self.tx.send(InternalClientToThreadEvent::RejectNewConnection(connection,reason)).unwrap();
    }

    pub fn logout(&mut self,connection: Connection) {
        self.tx.send(InternalClientToThreadEvent::Logout(Token(connection.0))).unwrap();
    }

    pub fn poll<D: Into<Option<Duration>>>(&mut self,duration: D) -> Option<ClientEvent> {
        //Perform any book keeping needed to manage client's state.
        fn update_client(client: &mut Client,event: &ClientEvent) {
            match *event {
                ClientEvent::ConnectionFailed(connection,_) |
                ClientEvent::ConnectionTerminated(connection,_) => {
                    client.token_generator.lock().unwrap().remove(Token(connection.0));
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

