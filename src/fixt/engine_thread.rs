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

use mio::{Event,Events,Poll,PollOpt,Ready,Token};
use mio::channel::{Receiver,Sender};
use mio::tcp::{Shutdown,TcpListener,TcpStream};
use mio::unix::UnixReady;
use mio::timer::{Timeout,Timer};
use mio::timer::Builder as TimerBuilder;
use std::cmp;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::io::{self,Write};
use std::mem;
use std::net::SocketAddr;
use std::sync::{Arc,Mutex};
use std::time::Duration;

use byte_buffer::ByteBuffer;
use dictionary::{CloneDictionary,administrative_msg_types,standard_msg_types};
use dictionary::field_types::generic::UTCTimestampFieldType;
use dictionary::field_types::other::{BusinessRejectReason,MsgDirection,SessionRejectReason};
use dictionary::fields::{ApplVerID,MsgSeqNum,SenderCompID,TargetCompID,OrigSendingTime};
use dictionary::messages::{Logon,Logout,ResendRequest,TestRequest,Heartbeat,SequenceReset,Reject,BusinessMessageReject};
use field::Field;
use field_type::FieldType;
use fix::{Parser,ParseError};
use fix_version::FIXVersion;
use fixt::engine::{EngineEvent,Connection,ConnectionTerminatedReason,Listener,ResendResponse};
use fixt::message::{BuildFIXTMessage,FIXTMessage};
use message_version::MessageVersion;
use network_read_retry::NetworkReadRetry;
use token_generator::TokenGenerator;

//TODO: Make sure Logon message is sent automatically instead of waiting on caller. Althought, we
//might have to support this for testing purposes.
//TODO: Check for infinite resend loop when other side sends garbled messages, we later send
//ResendRequest, and the other side continues to send garbled messages.
//TODO: Implement ConnectionStatus handling using a state machine pattern to reduce chance of
//mistake.
//TODO: Need to make inbound and outbound MsgSeqNums adjustable at connection setup and available
//on connection termination to support persistent sessions.
//TODO: Stop allowing outgoing messages when performing an emergency logout.
//TODO: Need to sanitize output strings when serializing.

const NO_INBOUND_TIMEOUT_PADDING_MS: u64 = 250;
const AUTO_DISCONNECT_AFTER_LOGOUT_RESPONSE_SECS: u64 = 10;
const AUTO_DISCONNECT_AFTER_INITIATING_LOGOUT_SECS: u64 = 10;
const AUTO_CONTINUE_AFTER_LOGOUT_RESEND_REQUEST_SECS: u64 = 10;
const AUTO_DISCONNECT_AFTER_WRITE_BLOCKS_SECS: u64 = 10;
pub const AUTO_DISCONNECT_AFTER_INBOUND_RESEND_REQUEST_LOOP_COUNT: u64 = 5;
pub const AUTO_DISCONNECT_AFTER_NO_LOGON_RECEIVED_SECONDS: u64 = 10;
const EVENT_POLL_CAPACITY: usize = 1024;
pub const INBOUND_MESSAGES_BUFFER_LEN_MAX: usize = 10;
pub const INBOUND_BYTES_BUFFER_CAPACITY: usize = 2048;
const TIMER_TICK_MS: u64 = 100;
const TIMER_TIMEOUTS_PER_TICK_MAX: usize = 256;
pub const CONNECTION_COUNT_MAX: usize = 65536;
const TIMEOUTS_PER_CONNECTION_MAX: usize = 3;

pub const INTERNAL_ENGINE_EVENT_TOKEN: Token = Token(0);
const TIMEOUT_TOKEN: Token = Token(1);
const NETWORK_READ_RETRY_TOKEN: Token = Token(2);
pub const BASE_CONNECTION_TOKEN: Token = Token(3);

#[derive(Clone,Copy,PartialEq)]
enum LoggingOutInitiator {
    Local,
    Remote
}

enum LoggingOutType {
    Ok, //Engine requested logout.
    Error(ConnectionTerminatedReason), //An unrecoverable error occurred.
    ResendRequesting(LoggingOutInitiator), //LoggingOutInitiator requested logout but MsgSeqNum was higher than expected so we're trying to collect the missing messages before continuing.
    Responding, //Remote requested logout and we are about to send a response.
    Responded, //Remote requested logout and we sent a response.
}

enum ConnectionStatus {
    SendingLogon,
    ReceivingLogon(Listener,Timeout),
    ApprovingLogon,
    Established,
    LoggingOut(LoggingOutType),
}

impl ConnectionStatus {
    fn is_sending_logon(&self) -> bool {
        if let ConnectionStatus::SendingLogon = *self {
            true
        }
        else {
            false
        }
    }

    fn is_receiving_logon(&self) -> bool {
        if let ConnectionStatus::ReceivingLogon(_,_) = *self {
            true
        }
        else {
            false
        }
    }

    fn is_approving_logon(&self) -> bool {
        if let ConnectionStatus::ApprovingLogon = *self {
            true
        }
        else {
            false
        }
    }

    fn is_established(&self) -> bool {
        if let ConnectionStatus::Established = *self {
            true
        }
        else {
            false
        }
    }

    fn is_logging_out(&self) -> bool {
        if let ConnectionStatus::LoggingOut(_) = *self {
            true
        }
        else {
            false
        }
    }

    fn is_logging_out_with_error(&self) -> bool {
        if let ConnectionStatus::LoggingOut(ref logging_out_type) = *self {
            if let LoggingOutType::Error(_) = *logging_out_type {
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }

    fn is_logging_out_with_resending_request_initiated_by_local(&self) -> bool {
        if let ConnectionStatus::LoggingOut(ref logging_out_type) = *self {
            if let LoggingOutType::ResendRequesting(ref logging_out_initiator) = *logging_out_type {
                if let LoggingOutInitiator::Local = *logging_out_initiator {
                    return true;
                }
            }
        }

        false
    }

    fn is_logging_out_with_resending_request_initiated_by_remote(&self) -> bool {
        if let ConnectionStatus::LoggingOut(ref logging_out_type) = *self {
            if let LoggingOutType::ResendRequesting(ref logging_out_initiator) = *logging_out_type {
                if let LoggingOutInitiator::Remote = *logging_out_initiator {
                    return true;
                }
            }
        }

        false
    }

    fn is_logging_out_with_responding(&self) -> bool {
        if let ConnectionStatus::LoggingOut(ref logging_out_type) = *self {
            if let LoggingOutType::Responding = *logging_out_type {
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }

    fn is_logging_out_with_responded(&self) -> bool {
        if let ConnectionStatus::LoggingOut(ref logging_out_type) = *self {
            if let LoggingOutType::Responded = *logging_out_type {
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }
}

enum TimeoutType {
    Outbound,
    Inbound,
    InboundTestRequest,
    InboundBlocked,
    ContinueLogout,
    NoLogon,
    Logout,
    HangUp,
}

type MsgSeqNumType = <<MsgSeqNum as Field>::Type as FieldType>::Type;

struct OutboundMessage {
    message: Box<FIXTMessage + Send>,
    message_version: Option<MessageVersion>,
    auto_msg_seq_num: bool,
}

impl OutboundMessage {
    fn new<T: FIXTMessage + Send + Sized + 'static>(message: T,auto_msg_seq_num: bool) -> Self {
        OutboundMessage {
            message: Box::new(message),
            message_version: None,
            auto_msg_seq_num: auto_msg_seq_num,
        }
    }

    fn from<T: FIXTMessage + Send + Sized + 'static>(message: T) -> Self {
        OutboundMessage {
            message: Box::new(message),
            message_version: None,
            auto_msg_seq_num: true,
        }
    }

    fn from_box(message: Box<FIXTMessage + Send>) -> Self {
        OutboundMessage {
            message: message,
            message_version: None,
            auto_msg_seq_num: true,
        }
    }
}

fn reset_timeout(timer: &mut Timer<(TimeoutType,Token)>,timeout: &mut Option<Timeout>,timeout_duration: &Option<Duration>,timeout_type: TimeoutType,token: &Token) {
    if let Some(ref timeout) = *timeout {
        timer.cancel_timeout(timeout);
    }

    *timeout = if let Some(duration) = *timeout_duration {
        Some(
            timer.set_timeout(
                duration,
                (timeout_type,*token)
            ).unwrap()
        )
    }
    else {
        None
    };
}

fn reset_outbound_timeout(timer: &mut Timer<(TimeoutType,Token)>,outbound_timeout: &mut Option<Timeout>,outbound_timeout_duration: &Option<Duration>,token: &Token) {
    reset_timeout(
        timer,
        outbound_timeout,
        outbound_timeout_duration,
        TimeoutType::Outbound,
        token
    );
}

fn reset_inbound_timeout(timer: &mut Timer<(TimeoutType,Token)>,inbound_timeout: &mut Option<Timeout>,inbound_timeout_duration: &Option<Duration>,token: &Token) {
    reset_timeout(
        timer,
        inbound_timeout,
        inbound_timeout_duration,
        TimeoutType::Inbound,
        token
    );
}

pub enum InternalEngineToThreadEvent {
    NewConnection(Token,FIXVersion,MessageVersion,<<SenderCompID as Field>::Type as FieldType>::Type,<<TargetCompID as Field>::Type as FieldType>::Type,SocketAddr),
    NewListener(Token,<<SenderCompID as Field>::Type as FieldType>::Type,TcpListener),
    SendMessage(Token,Option<MessageVersion>,Box<FIXTMessage + Send>),
    ResendMessages(Token,Vec<ResendResponse>),
    ApproveNewConnection(Connection,Box<Logon>,u64),
    RejectNewConnection(Connection,Option<Vec<u8>>),
    Logout(Token),
    Shutdown,
}

impl fmt::Debug for InternalEngineToThreadEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //TODO: Actually implement this if its ever used. Write now this exists so some unwrap()
        //calls that can never fail will compile.
        write!(f,"")
    }
}

enum ConnectionEventError {
    TerminateConnection(InternalConnection,ConnectionTerminatedReason),
    Shutdown,
}

enum ConnectionReadMessage {
    Message(Box<FIXTMessage + Send>),
    Error(ParseError),
}

struct LastSeenResendRequest {
    begin_seq_no: MsgSeqNumType,
    count: u64,
}

struct InternalConnection {
    fix_version: FIXVersion,
    default_message_version: MessageVersion,
    socket: TcpStream,
    token: Token,
    outbound_messages: Vec<OutboundMessage>,
    outbound_buffer: ByteBuffer,
    outbound_msg_seq_num: MsgSeqNumType,
    outbound_heartbeat_timeout: Option<Timeout>,
    outbound_heartbeat_timeout_duration: Option<Duration>,
    inbound_buffer: ByteBuffer,
    inbound_msg_seq_num: MsgSeqNumType,
    inbound_testrequest_timeout: Option<Timeout>,
    inbound_testrequest_timeout_duration: Option<Duration>,
    inbound_resend_request_msg_seq_num: Option<MsgSeqNumType>,
    inbound_last_seen_resend_request: LastSeenResendRequest,
    inbound_blocked: bool,
    inbound_blocked_timeout: Option<Timeout>,
    logout_timeout: Option<Timeout>,
    parser: Parser,
    is_connected: bool, //TODO: Might belong better as part of ConnectionStatus if the state machine design works well.
    status: ConnectionStatus,
    sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
    target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type,
}

impl InternalConnection {
    fn new(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
           max_message_size: u64,
           fix_version: FIXVersion,
           default_message_version: MessageVersion,
           socket: TcpStream,
           token: Token,
           sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
           target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) -> InternalConnection {
        //Force all administrative messages to use the newest message version for the
        //specified FIX version. This way they can't be overridden during Logon and it
        //makes sure the Logon message supports all of the fields we support.
        let mut parser = Parser::new(message_dictionary,max_message_size);
        for msg_type in administrative_msg_types() {
            parser.set_default_message_type_version(msg_type,fix_version.max_message_version());
        }

        InternalConnection {
            fix_version: fix_version,
            default_message_version: default_message_version,
            socket: socket,
            token: token,
            outbound_messages: Vec::new(),
            outbound_buffer: ByteBuffer::new(),
            outbound_msg_seq_num: 1, //Starts at 1. FIXT v1.1, page 5.
            outbound_heartbeat_timeout: None,
            outbound_heartbeat_timeout_duration: None,
            inbound_buffer: ByteBuffer::with_capacity(INBOUND_BYTES_BUFFER_CAPACITY),
            inbound_msg_seq_num: 1, //Starts at 1 as well.
            inbound_testrequest_timeout: None,
            inbound_testrequest_timeout_duration: None,
            inbound_resend_request_msg_seq_num: None,
            inbound_last_seen_resend_request: LastSeenResendRequest {
                begin_seq_no: 0,
                count: 0,
            },
            inbound_blocked: false,
            inbound_blocked_timeout: None,
            logout_timeout: None,
            parser: parser,
            is_connected: false,
            status: ConnectionStatus::SendingLogon,
            sender_comp_id: sender_comp_id,
            target_comp_id: target_comp_id,
        }
    }

    fn write(&mut self,timer: &mut Timer<(TimeoutType,Token)>,network_read_retry: &mut NetworkReadRetry) -> Result<(),ConnectionTerminatedReason> {
        //Send data until no more messages are available or until the socket returns WouldBlock.
        let mut sent_data = false;
        loop { //TODO: This loop might make this function too greedy. Maybe not?
            //Fill an outbound buffer by serializing each message in a FIFO order. Once this buffer
            //is drained, the process repeats itself.
            if self.outbound_buffer.is_empty() {
                if self.outbound_messages.is_empty() {
                    //Nothing left to write.

                    //If a Logout message was sent after an unrecoverable error, close the socket
                    //immediately.
                    if self.status.is_logging_out_with_error() {
                        let status = mem::replace(&mut self.status,ConnectionStatus::LoggingOut(LoggingOutType::Ok)); //Need to get at the error. Status should not be used again...
                        if let ConnectionStatus::LoggingOut(logging_out_type) = status {
                            if let LoggingOutType::Error(reason) = logging_out_type {
                                let _ = self.socket.shutdown(Shutdown::Both);
                                return Err(reason);
                            }
                        }
                    }
                    //Similarly, if a Logout message was sent as a response to to the remote
                    //issuing a Logout, start a timer and wait so many seconds before closing the
                    //socket. This is the recommended way to respond to a Logout instead of
                    //disconnecting immediately.
                    else if self.status.is_logging_out_with_responding() {
                        self.status = ConnectionStatus::LoggingOut(LoggingOutType::Responded);

                        self.logout_timeout = Some(
                            timer.set_timeout(
                                Duration::from_secs(AUTO_DISCONNECT_AFTER_LOGOUT_RESPONSE_SECS),
                                (TimeoutType::HangUp,self.token)
                            ).unwrap()
                        );
                    }
                    break;
                }

                //Setup message to go out and serialize it.
                let mut message = self.outbound_messages.remove(0);
                message.message.setup_fixt_session_header(
                    if message.auto_msg_seq_num {
                        let result = Some(self.outbound_msg_seq_num);
                        try!(self.increment_outbound_msg_seq_num());
                        result
                    } else { None },
                    self.sender_comp_id.clone(),
                    self.target_comp_id.clone()
                );
                let fix_version = self.fix_version;
                let message_version = if let Some(message_version) = message.message_version { message_version } else { self.default_message_version };
                message.message.read(fix_version,message_version,&mut self.outbound_buffer);

                //TODO: Hold onto message and pass it off to the engine or some callback so the
                //library user knows exactly which messages have been sent -- although not
                //necessarily acknowledged.
            }

            //Send data. Simple.
            match self.outbound_buffer.write(&mut self.socket) {
                Ok(_) => {
                    sent_data = true;

                    //When data has been successfully sent, it's okay to start reading in new data
                    //again.
                    if self.inbound_blocked {
                        self.end_blocking_inbound(timer,network_read_retry);
                    }
                },
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => {
                            //Could not write anymore data at the moment. Just in case the other
                            //side of the connection is over whelmed or we're being attacked, stop
                            //processing any new messages.
                            self.begin_blocking_inbound(timer);
                            break;
                        },
                        io::ErrorKind::BrokenPipe => {
                            //TODO: This might not be an actual error if all logging out has been
                            //performed. Could be a Hup.
                            return Err(ConnectionTerminatedReason::SocketWriteError(e));
                        },
                        _ => return Err(ConnectionTerminatedReason::SocketWriteError(e)),
                    };
                }
            }
        }

        //If any data was sent, need to update timeout so we don't send an unnecessary Heartbeat
        //message.
        if sent_data {
            reset_outbound_timeout(timer,&mut self.outbound_heartbeat_timeout,&self.outbound_heartbeat_timeout_duration,&self.token);
        }

        Ok(())
    }

    fn read(&mut self,timer: &mut Timer<(TimeoutType,Token)>) -> Result<(Vec<ConnectionReadMessage>),::std::io::Error> {
        fn parse_bytes(connection: &mut InternalConnection,messages: &mut Vec<ConnectionReadMessage>) -> bool {
            while !connection.inbound_buffer.is_empty() {
                let (bytes_parsed,result) = connection.parser.parse(connection.inbound_buffer.bytes());

                assert!(bytes_parsed > 0);
                connection.inbound_buffer.consume(bytes_parsed);

                //Retain order by extracting messages and then the error from parser.
                for message in connection.parser.messages.drain(..) {
                    messages.push(ConnectionReadMessage::Message(message));
                }
                if let Err(e) = result {
                    messages.push(ConnectionReadMessage::Error(e));
                }

                //Stop reading once INBOUND_MESSAGES_BUFFER_LEN_MAX messages have been read.
                //This prevents a flood of messages from completely stalling the thread.
                if messages.len() >= INBOUND_MESSAGES_BUFFER_LEN_MAX {
                    return false;
                }
                //Stop reading temporarily after receiving the first message (that should be a
                //Logon or else we'll disconnect). This gives us a chance to use the Logon response
                //to setup message versioning defaults for the parser.
                else if connection.status.is_sending_logon() || connection.status.is_receiving_logon() {
                    assert!(messages.len() <= 1);
                    return false;
                }
            }

            true
        }

        let mut messages = Vec::new();
        let mut keep_reading = parse_bytes(self,&mut messages);

        //Don't read in any new messages for now. This happens when we can't write to the socket
        //right now. The block applies some back pressure and prevents us from being put into a
        //situation where we're over whelmed.
        if self.inbound_blocked {
            return Ok(messages);
        }

        //Keep reading all available bytes on the socket until it's exhausted or
        //INBOUND_MESSAGES_BUFFER_LEN_MAX messages have been read. The bytes are parsed
        //immediately into messages. Parse errors are stored in order of encounter relative to
        //messages because they often indicate an increase in expected inbound MsgSeqNum.
        while keep_reading {
            match self.inbound_buffer.clear_and_read(&mut self.socket) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        //Socket exhausted.
                        break;
                    }

                    //Parse all of the read bytes.
                    keep_reading = parse_bytes(self,&mut messages);
                },
                Err(e) => {
                    if let io::ErrorKind::WouldBlock = e.kind() {
                        //Socket exhausted.
                        break;
                    }

                    return Err(e);
                },
            };
        }

        //Update timeout so we don't send an unnecessary TestRequest message. read() should never
        //be called unless data is available (due to poll()) so we don't have to check if any data
        //bytes were actually read.
        reset_inbound_timeout(timer,&mut self.inbound_testrequest_timeout,&self.inbound_testrequest_timeout_duration,&self.token);

        Ok(messages)
    }

    fn shutdown(&mut self) {
        let _ = self.socket.shutdown(Shutdown::Both);
        self.outbound_messages.clear();
        self.outbound_buffer.clear();
    }

    fn initiate_logout(&mut self,timer: &mut Timer<(TimeoutType,Token)>,logging_out_type: LoggingOutType,text: &[u8]) {
        //Begin the logout process. Use respond_to_logout() to respond to a logout message.

        assert!(match logging_out_type {
            LoggingOutType::Ok => !self.status.is_logging_out() || self.status.is_logging_out_with_resending_request_initiated_by_local(),
            LoggingOutType::Error(_) => !self.status.is_logging_out_with_error(),
            _ => false,
        });

        let mut logout = Logout::new();
        logout.text = text.to_vec();

        //TODO: The clearing of outbound messages might be optional. Probably need a receipt or
        //something for those that are left unprocessed.
        self.outbound_messages.clear(); //TODO: May want to store unprocessed messages so engine knows what didn't go out.
        self.outbound_messages.push(OutboundMessage::from(logout));

        //If attempting to logout cleanly, setup timer to auto-logout if we don't get a Logout
        //response. LoggingOutType::Error just disconnects immediately.
        if let LoggingOutType::Ok = logging_out_type {
            self.logout_timeout = Some(
                timer.set_timeout(
                    Duration::from_secs(AUTO_DISCONNECT_AFTER_INITIATING_LOGOUT_SECS),
                    (TimeoutType::Logout,self.token)
                ).unwrap()
            );
        }

        self.status = ConnectionStatus::LoggingOut(logging_out_type);
    }

    fn respond_to_logout(&mut self) {
        assert!(self.status.is_established() || self.status.is_logging_out_with_resending_request_initiated_by_remote());

        let logout = Logout::new();
        self.outbound_messages.push(OutboundMessage::from(logout));

        self.status = ConnectionStatus::LoggingOut(LoggingOutType::Responding);
    }

    fn increment_outbound_msg_seq_num(&mut self) -> Result<(),ConnectionTerminatedReason> {
        //Check for overflow before incrementing. Just force the connection to terminate if this
        //occurs. This number is so large that the only way it can be reached is if the other party
        //issues SequenceReset-Reset with a crazy high NewSeqNo. NewSeqNo values higher than
        //u64::max_value() are outright rejected as parsing errors.
        if self.outbound_msg_seq_num == u64::max_value() {
            return Err(ConnectionTerminatedReason::OutboundMsgSeqNumMaxExceededError);
        }

        self.outbound_msg_seq_num += 1;
        Ok(())
    }

    fn increment_inbound_msg_seq_num(&mut self) -> Result<(),ConnectionTerminatedReason> {
        //See increment_outbound_msg_seq_num() for an explanation of this check.
        if self.inbound_msg_seq_num == u64::max_value() {
            return Err(ConnectionTerminatedReason::InboundMsgSeqNumMaxExceededError);
        }

        self.inbound_msg_seq_num += 1;
        Ok(())
    }

    fn clear_inbound_resend_request_msg_seq_num(&mut self,timer: &mut Timer<(TimeoutType,Token)>) {
        self.inbound_resend_request_msg_seq_num = None;

        //If remote started a logout, we noticed missing messaged, and have now
        //received all of those messages, finally respond to logout.
        if self.status.is_logging_out_with_resending_request_initiated_by_remote() {
            self.respond_to_logout();
        }
        //Same as above except engine initiated logout and suspended it long enough to
        //retrieve messages.
        else if self.status.is_logging_out_with_resending_request_initiated_by_local() {
            self.initiate_logout(timer,LoggingOutType::Ok,b"");
        }
    }

    fn begin_blocking_inbound(&mut self,timer: &mut Timer<(TimeoutType,Token)>) {
        if self.inbound_blocked {
            return;
        }

        self.inbound_blocked = true;

        //Setup a timer to disconnect if inbound blocking is not stopped shortly. Otherwise, the
        //connection is just sitting there wasting space and is unusable.
        self.inbound_blocked_timeout = Some(timer.set_timeout(
            Duration::from_secs(AUTO_DISCONNECT_AFTER_WRITE_BLOCKS_SECS),
            (TimeoutType::InboundBlocked,self.token)
        ).unwrap());
    }

    fn end_blocking_inbound(&mut self,timer: &mut Timer<(TimeoutType,Token)>,network_read_retry: &mut NetworkReadRetry) {
        if !self.inbound_blocked {
            return;
        }

        self.inbound_blocked = false;

        //Stop the auto-disconnect timer. We can read and write to the socket again.
        if let Some(ref timeout) = self.inbound_blocked_timeout {
            timer.cancel_timeout(timeout);
        }

        //Immediately try to read from the socket in case there is any data waiting.
        network_read_retry.queue(self.token);
    }

    fn as_connection(&self) -> Connection {
        Connection(self.token.0)
    }
}

macro_rules! try_write_connection_or_terminate {
    ( $connection_entry:ident, $internal_thread:ident ) => {
        if let Err(e) = $connection_entry.get_mut().write(&mut $internal_thread.timer,&mut $internal_thread.network_read_retry) {
            return Err(ConnectionEventError::TerminateConnection($connection_entry.remove(),e));
        }
    }
}

struct InternalListener {
    socket: TcpListener,
    token: Token,
    sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
}

impl InternalListener {
    fn as_listener(&self) -> Listener {
        Listener(self.token.0)
    }
}

struct InternalThread {
    poll: Poll,
    token_generator: Arc<Mutex<TokenGenerator>>,
    tx: Sender<EngineEvent>,
    rx: Receiver<InternalEngineToThreadEvent>,
    message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
    max_message_size: u64,
    connections: HashMap<Token,InternalConnection>,
    listeners: HashMap<Token,InternalListener>,
    timer: Timer<(TimeoutType,Token)>,
    network_read_retry: NetworkReadRetry,
}

impl InternalThread {
    fn on_internal_engine_event(&mut self) -> Result<(),ConnectionEventError> {
        let engine_event = match self.rx.try_recv() {
            Ok(e) => e,
            Err(_) => return Ok(()), //Shouldn't be possible but PROBABLY just means no engine events are available.
        };

        match engine_event {
            //Engine wants to setup a new connection.
            InternalEngineToThreadEvent::NewConnection(token,fix_version,default_message_version,sender_comp_id,target_comp_id,address) => {
                let socket = match TcpStream::connect(&address) {
                    Ok(socket) => socket,
                    Err(e) => {
                        self.tx.send(EngineEvent::ConnectionFailed(Connection(token.0),e)).unwrap();
                        return Ok(())
                    },
                };

                let connection = InternalConnection::new(self.message_dictionary.clone(),
                                                         self.max_message_size,
                                                         fix_version,
                                                         default_message_version,
                                                         socket,
                                                         token,
                                                         sender_comp_id,
                                                         target_comp_id);

                //Have poll let us know when we can can read or write.
                if let Err(e) = self.poll.register(&connection.socket,
                                                   connection.token,
                                                   Ready::readable() | Ready::writable() | UnixReady::hup() | UnixReady::error(),
                                                   PollOpt::edge()) {
                    self.tx.send(EngineEvent::ConnectionFailed(connection.as_connection(),e)).unwrap();
                    return Ok(())
                }

                self.connections.insert(token,connection);
            },
            //Engine wants to setup a listener to accept new connections.
            InternalEngineToThreadEvent::NewListener(token,sender_comp_id,socket) => {
                let listener = InternalListener {
                    socket: socket,
                    token: token,
                    sender_comp_id: sender_comp_id,
                };

                if let Err(e) = self.poll.register(&listener.socket,listener.token,Ready::readable(),PollOpt::edge()) {
                    self.tx.send(EngineEvent::ListenerFailed(listener.as_listener(),e)).unwrap();
                    return Ok(())
                }

                self.listeners.insert(token,listener);
            },
            //Engine wants to send a message over a connection.
            InternalEngineToThreadEvent::SendMessage(token,message_version,message) => {
                if let Entry::Occupied(mut connection_entry) = self.connections.entry(token) {
                    let mut outbound_message = OutboundMessage::from_box(message);
                    outbound_message.message_version = message_version;
                    connection_entry.get_mut().outbound_messages.push(outbound_message);
                    try_write_connection_or_terminate!(connection_entry,self);
                }
                else {
                    //Silently ignore message for invalid connection.
                    //TODO: Maybe submit this to a logging system or something?
                }
            },
            //Engine wants to send a set of messages as a response to a resend request.
            InternalEngineToThreadEvent::ResendMessages(token,response) => {
            if let Entry::Occupied(mut connection_entry) = self.connections.entry(token) {
                    //TODO: It might make sense to take these responses as a group and do a sorted
                    //insert into outbound_messages. This way we at least try to prevent excessive
                    //ResendRequests from being sent to us later.
                    for message in response {
                        match message {
                            ResendResponse::Message(message_version,mut message) => {
                                //Make sure message is marked as a potential duplicate or else
                                //we'll trigger an InboundMsgSeqNumLowerThanExpectedError or
                                //equivalent on the other side of the connection.
                                message.set_is_poss_dup(true);
                                let orig_sending_time = message.sending_time();
                                message.set_orig_sending_time(orig_sending_time);

                                let mut outbound_message = OutboundMessage::from_box(message);
                                outbound_message.message_version = message_version;
                                outbound_message.auto_msg_seq_num = false; //We must preserve MsgSeqNum for response.
                                connection_entry.get_mut().outbound_messages.push(outbound_message);
                            },
                            ResendResponse::Gap(range) => {
                                let mut sequence_reset = SequenceReset::new();
                                sequence_reset.gap_fill_flag = true;
                                sequence_reset.msg_seq_num = range.start;
                                sequence_reset.new_seq_no = range.end;
                                connection_entry.get_mut().outbound_messages.push(
                                    OutboundMessage::new(sequence_reset,false)
                                );
                            },
                        }
                    }

                    //If we are still waiting on a response to our own RespondRequest, send a new
                    //RespondRequest. Deferring like this is the correct behavior according to FIXT
                    //v1.1, page 13.
                    if connection_entry.get().inbound_resend_request_msg_seq_num.is_some() {
                        let mut resend_request = ResendRequest::new();
                        resend_request.begin_seq_no = connection_entry.get().inbound_msg_seq_num;
                        resend_request.end_seq_no = 0;
                        connection_entry.get_mut().outbound_messages.push(
                            OutboundMessage::from(resend_request)
                        );
                    }

                    try_write_connection_or_terminate!(connection_entry,self);
                }
                else {
                    //Silently ignore message for invalid connection.
                    //TODO: Maybe submit this to a logging system or something?
                }
            },
            //Engine wants to approve logon of a connection that was accepted by a listener.
            InternalEngineToThreadEvent::ApproveNewConnection(connection,message,inbound_msg_seq_num) => {
                if let Entry::Occupied(mut connection_entry) = self.connections.entry(Token(connection.0)) {
                    {
                        let connection = connection_entry.get_mut();
                        if !connection.status.is_approving_logon() {
                            //Silently ignore approval of connections that are not awaiting
                            //approval.
                            //TODO: Maybe submit this to a logging system or something?
                            return Ok(());
                        }

                        connection.status = ConnectionStatus::Established;

                        //Setup the version messages should be serialized against by default when
                        //being sent. Only FIXT 1.1 makes this adjustable and it MUST be set by the
                        //response Logon message in the DefaultApplVerID field.
                        connection.default_message_version = if let FIXVersion::FIXT_1_1 = connection.fix_version {
                            message.default_appl_ver_id
                        }
                        else {
                            connection.fix_version.max_message_version()
                        };

                        //Send the Logon response. It's always sent using the latest message version
                        //for the selected FIX version. This is probably what is always wanted unless a
                        //version is outright not supported. In which case, the connection should have
                        //been rejected, right?
                        let mut outbound_message = OutboundMessage::from_box(message);
                        outbound_message.message_version = Some(connection.fix_version.max_message_version());
                        assert!(connection.outbound_messages.is_empty());
                        connection.outbound_messages.push(outbound_message);

                        if inbound_msg_seq_num < connection.inbound_msg_seq_num {
                            connection.inbound_msg_seq_num = inbound_msg_seq_num;

                            //Fetch the messages the remote says were sent but we never
                            //received using a ResendRequest.
                            let mut resend_request = ResendRequest::new();
                            resend_request.begin_seq_no = inbound_msg_seq_num;
                            resend_request.end_seq_no = 0;
                            connection.outbound_messages.push(OutboundMessage::from(resend_request));
                        }
                        else if inbound_msg_seq_num > connection.inbound_msg_seq_num {
                            //TODO: Investigate exact handling of this. Maybe SequenceReset?
                        }

                        //Start the heartbeat timers to send messages periodically to make sure the
                        //connection is still active.
                        reset_outbound_timeout(&mut self.timer,&mut connection.outbound_heartbeat_timeout,&connection.outbound_heartbeat_timeout_duration,&connection.token);
                        reset_inbound_timeout(&mut self.timer,&mut connection.inbound_testrequest_timeout,&connection.inbound_testrequest_timeout_duration,&connection.token);
                    }

                    try_write_connection_or_terminate!(connection_entry,self);
                }
                else {
                    //Silently ignore message for an invalid connection.
                    //TODO: Maybe submit this to a logging system or something?
                }
            },
            //Engine wants to reject logon of a connection that was accepted by a listener.
            InternalEngineToThreadEvent::RejectNewConnection(connection,reason) => {
                //This should only be used for new connections but there is no check for now so
                //user of the engine has a way to arbitrarily disconnect over an error instead of
                //logging out cleanly.

                if let Entry::Occupied(mut connection_entry) = self.connections.entry(Token(connection.0)) {
                    //When a reason is supplied, send a Logout message with the reason as an
                    //explanation. Otherwise, disconnect immediately.
                    if let Some(reason) = reason {
                        connection_entry.get_mut().initiate_logout(&mut self.timer,LoggingOutType::Error(ConnectionTerminatedReason::LogonRejectedError),&reason[..]);
                        try_write_connection_or_terminate!(connection_entry,self);
                    }
                    else {
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::LogonRejectedError));
                    }
                }
                else {
                    //Silently ignore for an invalid connection.
                    //TODO: Maybe submit this to a logging system or something?
                }
            },
            //Engine wants to begin the clean logout process on a connection.
            InternalEngineToThreadEvent::Logout(token) => {
                if let Entry::Occupied(mut connection_entry) = self.connections.entry(token) {
                    match connection_entry.get_mut().status {
                        ConnectionStatus::SendingLogon |
                        ConnectionStatus::ReceivingLogon(_,_) |
                        ConnectionStatus::ApprovingLogon => {
                            //Just disconnect since connection hasn't had a chance to logon.
                            return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::LocalRequested));
                        },
                        ConnectionStatus::LoggingOut(_) => {}, //Already logging out.
                        ConnectionStatus::Established => {
                            //Begin logout.
                            connection_entry.get_mut().initiate_logout(&mut self.timer,LoggingOutType::Ok,b"");
                            try_write_connection_or_terminate!(connection_entry,self);
                        },
                    };
                }
                else {
                    //Silently ignore logout for invalid connection.
                    //TODO: Maybe submit this to a logging system or something?
                }
            },
            //Engine wants to shutdown all connections immediately. Incoming or outgoing messages
            //might be lost!
            InternalEngineToThreadEvent::Shutdown => return Err(ConnectionEventError::Shutdown),
        };

        Ok(())
    }

    fn on_timeout(&mut self) -> Result<(),ConnectionEventError> {
        if let Some((timeout_type,token)) = self.timer.poll() {
            if let Entry::Occupied(mut connection_entry) = self.connections.entry(token) {
                match timeout_type {
                    TimeoutType::Outbound if connection_entry.get().status.is_established() => {
                        //We haven't sent any data in a while. Send a Heartbeat to let other side
                        //know we're still around.
                        let mut heartbeat = Heartbeat::new();
                        heartbeat.test_req_id = Vec::new(); //Left blank when not responding to TestRequest.
                        connection_entry.get_mut().outbound_messages.push(OutboundMessage::from(heartbeat));
                    },
                    TimeoutType::Inbound if connection_entry.get().status.is_established() => {
                        //Other side hasn't sent any data in a while. Send a TestRequest to see if
                        //it's still around.
                        let mut test_request = TestRequest::new();

                        //Use current time as TestReqID as recommended. This might not exactly
                        //match the SendingTime field depending on when it gets sent though.
                        let now_time = UTCTimestampFieldType::new_now();
                        UTCTimestampFieldType::read(&now_time,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut test_request.test_req_id);

                        connection_entry.get_mut().outbound_messages.push(OutboundMessage::from(test_request));

                        //Start a TimeoutType::InboundTestRequest timer to auto-disconnect if we
                        //don't get a response in time. Note that any reploy what-so-ever will stop
                        //the auto-disconnect -- even if this TestRequest is ignored and later gap
                        //filled. The overhead in maintaining a list of sent TestReqIds does not
                        //seem worth the effort. It would only be useful for debugging reasons,
                        //right?
                        //TODO: This might belong in the InternalConnection::write() function so we
                        //don't disconnect before the TestRequest is actually sent. On the other
                        //hand, if this doesn't go out in a reasonable amount of time, we're
                        //backlogged and might be having negative consequences on the network.
                        connection_entry.get_mut().inbound_testrequest_timeout = Some(
                            self.timer.set_timeout(
                                connection_entry.get_mut().inbound_testrequest_timeout_duration.unwrap(),
                                (TimeoutType::InboundTestRequest,token),
                            ).unwrap()
                        );
                    },
                    TimeoutType::InboundTestRequest if connection_entry.get().status.is_established() => {
                        connection_entry.get_mut().shutdown();
                        println!("Shutting down connection after other side failed to respond to TestRequest before timeout");
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::TestRequestNotRespondedError));
                    },
                    TimeoutType::InboundBlocked => {
                        connection_entry.get_mut().shutdown();
                        println!("Shutting down connection after writing to socket resulted in WouldBlock for too long");
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::SocketNotWritableTimeoutError));
                    }
                    TimeoutType::ContinueLogout if connection_entry.get().status.is_logging_out_with_resending_request_initiated_by_remote() => {
                        connection_entry.get_mut().respond_to_logout();
                    },
                    TimeoutType::NoLogon => {
                        assert!(connection_entry.get().status.is_receiving_logon());
                        connection_entry.get_mut().shutdown();
                        println!("Shutting down connection after no initial Logon received before timeout");
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::LogonNeverReceivedError));
                    },
                    TimeoutType::Logout => {
                        connection_entry.get_mut().shutdown();
                        println!("Shutting down connection after no Logout response before timeout");
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::LogoutNoResponseError));
                    },
                    TimeoutType::HangUp => {
                        connection_entry.get_mut().shutdown();
                        println!("Shutting down connection after other side failed to disconnect before timeout");
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::LogoutNoHangUpError));
                    },
                    TimeoutType::Outbound |
                    TimeoutType::Inbound |
                    TimeoutType::InboundTestRequest |
                    TimeoutType::ContinueLogout => {}, //Special conditions only. Handled above.
                }

                //Write any new Heartbeat or TestRequest messages.
                try_write_connection_or_terminate!(connection_entry,self);
            }
        }

        Ok(())
    }

    fn on_network(&mut self,event: &Event) -> Result<(),ConnectionEventError> {
        //Note: Each event.kind() can indicate more than one state. For example: is_readable() and
        //is_hup() can both return true.

        if let Entry::Occupied(mut connection_entry) = self.connections.entry(event.token()) {
            //Read all of the bytes available on the socket, parse into messages, perform internal
            //book keeping on the messages, and then pass them off to the application.
            if event.kind().is_readable() {
                let result = connection_entry.get_mut().read(&mut self.timer);
                if let Err(e) = result {
                    return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::SocketReadError(e)));
                }

                if let Ok(messages) = result {
                    //Whenever at least one message is found, we need to assume there might still
                    //be bytes sitting on the socket to be read. The read() function call above
                    //might have left them there so we can process the messages that have already
                    //been parsed. This prevents a flood of messages from stalling the thread. By
                    //queueing with network_read_retry, we'll try again in a once these messages
                    //are processed and any other connections are handled.
                    if !messages.is_empty() {
                        self.network_read_retry.queue(event.token());
                    }

                    for message in messages {
                        let result = match message {
                            ConnectionReadMessage::Message(message) =>
                                InternalThread::on_network_message(connection_entry.get_mut(),message,&self.tx,&mut self.timer),
                            ConnectionReadMessage::Error(parse_error) =>
                                InternalThread::on_network_parse_error(connection_entry.get_mut(),parse_error,&self.tx),
                        };

                        if let Err(e) = result {
                            return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),e));
                        }
                    }
                }

                //Send any new messages that were generated automatically as a response.
                //Determining if a new message is available to go out can be kind of
                //complicated so just blindly try for now. We can optimize this if it's a
                //performance concern later.
                try_write_connection_or_terminate!(connection_entry,self);
            }

            //Write all pending messages out to the socket until they are exhausted or the socket
            //fills up and would block. Whichever happens first.
            if event.kind().is_writable() {
                try_write_connection_or_terminate!(connection_entry,self);

                if !connection_entry.get().is_connected {
                    //Let user know that the socket's connect() call succeeded.
                    connection_entry.get_mut().is_connected = true;
                    self.tx.send(EngineEvent::ConnectionSucceeded(connection_entry.get().as_connection())).unwrap();
                }
            }

            //Socket was closed on the other side. If already responded to a Logout initiated by
            //the other side, then this is expected and the logout operation was performed cleanly.
            //Otherwise, the connection dropped for some unknown reason.
            if event.kind().is_hup() {
                if connection_entry.get_mut().status.is_logging_out_with_responded() {
                    println!("Shutting down connection after remote logged out cleanly.");
                    return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::RemoteRequested));
                }
                else {
                    //Coax a socket write to fail in order to get an error code that we can pass
                    //along.
                    let result = connection_entry.get_mut().socket.write(b"\x00");
                    if let Err(e) = result {
                        return Err(ConnectionEventError::TerminateConnection(connection_entry.remove(),ConnectionTerminatedReason::SocketWriteError(e)));
                    }
                }
            }
        }

        if let Entry::Occupied(mut listener_entry) = self.listeners.entry(event.token()) {
            if event.kind().is_readable() {
                match listener_entry.get_mut().socket.accept() {
                    Ok((socket,addr)) => {
                        let token = match self.token_generator.lock().unwrap().create() {
                            Some(token) => token,
                            None => {
                                let _ = socket.shutdown(Shutdown::Both);
                                self.tx.send(EngineEvent::ConnectionDropped(listener_entry.get().as_listener(),addr)).unwrap();
                                return Ok(());
                            },
                        };

                        //Let engine know about the connection and have a chance to reject it
                        //before remote sends a Logon message.
                        self.tx.send(EngineEvent::ConnectionAccepted(listener_entry.get().as_listener(),Connection(token.0),addr.clone())).unwrap();

                        let fix_version = FIXVersion::max_version(); //Accept the latest message version at first. This works out because Logon is forwards version compatible.
                        let mut connection = InternalConnection::new(self.message_dictionary.clone(),
                                                                     self.max_message_size,
                                                                     fix_version, //Overwritten to whatever connection uses in first Logon message.
                                                                     MessageVersion::FIX50SP2, //Overwritten when connection is approved using the response message's default_appl_ver_id.
                                                                     socket,
                                                                     token,
                                                                     listener_entry.get().sender_comp_id.clone(),
                                                                     Vec::new());
                        connection.is_connected = true; //Accepted connections don't have to wait for connect().
                        let timeout = self.timer.set_timeout(
                            Duration::from_secs(AUTO_DISCONNECT_AFTER_NO_LOGON_RECEIVED_SECONDS),
                            (TimeoutType::NoLogon,token)).unwrap();
                        connection.status = ConnectionStatus::ReceivingLogon(listener_entry.get().as_listener(),timeout);

                        //Have poll let us know when we can can read or write.
                        if let Err(_) = self.poll.register(&connection.socket,
                                                           connection.token,
                                                           Ready::readable() | Ready::writable() | UnixReady::hup() | UnixReady::error(),
                                                           PollOpt::edge()) {
                            let _ = connection.socket.shutdown(Shutdown::Both);
                            self.tx.send(EngineEvent::ConnectionDropped(listener_entry.get().as_listener(),addr)).unwrap();
                            return Ok(())
                        }

                        self.connections.insert(token,connection);
                    },
                    Err(err) => {
                        self.tx.send(EngineEvent::ListenerAcceptFailed(listener_entry.get().as_listener(),err)).unwrap();
                    },
                }
            }
        }

        Ok(())
    }

    fn on_network_message(connection: &mut InternalConnection,mut message: Box<FIXTMessage + Send>,tx: &Sender<EngineEvent>,timer: &mut Timer<(TimeoutType,Token)>) -> Result<(),ConnectionTerminatedReason>  {
        //Perform book keeping needed to maintain the FIX connection and then pass off the message
        //to the engine.

        fn if_on_resend_request(connection: &mut InternalConnection,message: Box<FIXTMessage + Send>,msg_seq_num: MsgSeqNumType,tx: &Sender<EngineEvent>,timer: &mut Timer<(TimeoutType,Token)>) -> Option<Box<FIXTMessage + Send>> {
            let mut rejected = false;

            if let Some(resend_request) = message.as_any().downcast_ref::<ResendRequest>() {
                //Outright reject the message when BeginSeqNo > EndSeqNo because it doesn't make
                //sense. The exact response to this scenario does not appear to be described in the
                //spec.
                if resend_request.begin_seq_no > resend_request.end_seq_no && resend_request.end_seq_no != 0 {
                    let mut reject = Reject::new();
                    reject.ref_seq_num = msg_seq_num;
                    reject.session_reject_reason = Some(SessionRejectReason::ValueIsIncorrectForThisTag);
                    reject.text = b"EndSeqNo must be greater than BeginSeqNo or set to 0".to_vec();
                    connection.outbound_messages.push(OutboundMessage::from(reject));

                    rejected = true;
                }
                else {
                    //Cap the end range of the resend request to the highest sent MsgSeqNum. The spec
                    //doesn't describe what to do when EndSeqNo is greater than the highest sent
                    //MsgSeqNum. BUT, it apparently was a common pattern in older versions of the
                    //protocol to set EndSeqNo to a really high number (ie. 999999) to mean the same
                    //thing as setting it to 0 now.
                    let end_seq_no = if resend_request.end_seq_no > connection.outbound_msg_seq_num || resend_request.end_seq_no == 0 {
                        connection.outbound_msg_seq_num - 1
                    }
                    else {
                        resend_request.end_seq_no
                    };

                    //Detect if we're stuck in a ResendRequest loop by examining the BeginSeqNo
                    //field. If the same sequence is seen too many times, initiate logout and
                    //disconnect.
                    if resend_request.begin_seq_no == connection.inbound_last_seen_resend_request.begin_seq_no {
                        connection.inbound_last_seen_resend_request.count += 1;

                        if connection.inbound_last_seen_resend_request.count > AUTO_DISCONNECT_AFTER_INBOUND_RESEND_REQUEST_LOOP_COUNT {
                            let mut text = b"Detected ResendRequest loop for BeginSeqNo ".to_vec();
                            text.extend_from_slice(resend_request.begin_seq_no.to_string().as_bytes());
                            connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::InboundResendRequestLoopError),&text[..]);
                            return None;
                        }
                    }
                    else {
                        connection.inbound_last_seen_resend_request.begin_seq_no = resend_request.begin_seq_no;
                        connection.inbound_last_seen_resend_request.count = 1;
                    }

                    //Notify the engine of which messages are requested. Then it's up to the engine
                    //to give said messages to us so we can send them.
                    let end_seq_no = if resend_request.end_seq_no == 0 { connection.outbound_msg_seq_num } else { resend_request.end_seq_no + 1 }; //TODO: Handle potential overflow.
                    tx.send(EngineEvent::ResendRequested(connection.as_connection(),resend_request.begin_seq_no..end_seq_no)).unwrap();
                }

                //If:
                // 1. The remote initiates a logout
                // 2. We acknowledge the logout
                // 3. The remote sends a ResendRequest (instead of disconnecting AND before our
                //    disconnect timeout)
                //Then we need to assume the logout was cancelled.
                //See FIXT v1.1, page 42.
                if connection.status.is_logging_out_with_responded() {
                    connection.status = ConnectionStatus::Established;

                    //Stop timeout so we don't auto-disconnect.
                    if let Some(ref timeout) = connection.logout_timeout {
                        timer.cancel_timeout(timeout);
                    }
                }
            }

            //Appease the borrow checker by fully handling reject much later than where occurred.
            if rejected {
                tx.send(EngineEvent::MessageRejected(connection.as_connection(),message)).unwrap();
                None
            }
            else {
                Some(message)
            }
        }

        fn reject_for_sending_time_accuracy(connection: &mut InternalConnection,message: Box<FIXTMessage + Send>,msg_seq_num: MsgSeqNumType,tx: &Sender<EngineEvent>) {
            let mut reject = Reject::new();
            reject.ref_seq_num = msg_seq_num;
            reject.session_reject_reason = Some(SessionRejectReason::SendingTimeAccuracyProblem);
            reject.text = b"SendingTime accuracy problem".to_vec();
            connection.outbound_messages.push(OutboundMessage::from(reject));

            tx.send(EngineEvent::MessageRejected(connection.as_connection(),message)).unwrap();
        }

        fn on_greater_than_expected_msg_seq_num(connection: &mut InternalConnection,mut message: Box<FIXTMessage + Send>,msg_seq_num: MsgSeqNumType,tx: &Sender<EngineEvent>,timer: &mut Timer<(TimeoutType,Token)>) -> Option<Box<FIXTMessage + Send>> {
            //FIXT v1.1, page 13: We should reply to ResendRequest first when MsgSeqNum is higher
            //than expected. Afterwards, we should send our own ResendRequest.
            message = match if_on_resend_request(connection,message,msg_seq_num,tx,timer) {
                Some(message) => message,
                None => return None,
            };

            //Fetch the messages the remote says were sent but we never received using
            //ResendRequest. The one exception is if we are _receiving_ a ResendRequest message
            //because then we're suppose to defer until after we respond.
            if message.as_any().downcast_ref::<ResendRequest>().is_none() {
                let mut resend_request = ResendRequest::new();
                resend_request.begin_seq_no = connection.inbound_msg_seq_num;
                resend_request.end_seq_no = 0;
                connection.outbound_messages.push(OutboundMessage::from(resend_request));
            }

            //Keep track of the newest msg_seq_num that's been seen so we know when the message gap has
            //been filled.
            connection.inbound_resend_request_msg_seq_num = Some(
                cmp::max(connection.inbound_resend_request_msg_seq_num.unwrap_or(msg_seq_num),msg_seq_num)
            );

            //Handle Logout messages as a special case where we need to delicately retrieve the
            //missing messages while still going through with the logout process. See FIXT v1.1,
            //page 42 for details.
            //Start by figuring out if Logout message is a response to our Logout or the other side
            //is initiating a logout.
            if let Some(logout) = message.as_any().downcast_ref::<Logout>() {
                let logging_out_initiator = if let ConnectionStatus::LoggingOut(ref logging_out_type) = connection.status {
                    match logging_out_type {
                        &LoggingOutType::Ok => { //Remote acknowledged our logout but we're missing some messages.
                            Some(LoggingOutInitiator::Local)
                        },
                        &LoggingOutType::Responding | //Remote sent two diffrent Logouts in a row with messages inbetween missing.
                        &LoggingOutType::Responded => { //Remote cancelled original logout and we're some how missing some messages.
                            Some(LoggingOutInitiator::Remote)
                        },
                        &LoggingOutType::Error(_) => { None } //Does not matter. We are closing the connection immediately.
                        &LoggingOutType::ResendRequesting(logging_out_initiator) => { //Remote resent Logout before fully responding to our ResendRequest.
                            None //No change so timeout timer can't be kept alive perpetually.
                        },
                    }
                }
                else {
                    //Remote is initiating logout.
                    Some(LoggingOutInitiator::Remote)
                };

                //Begin watching for missing messages so we can finish logging out.
                if let Some(logging_out_initiator) = logging_out_initiator {
                    connection.status = ConnectionStatus::LoggingOut(LoggingOutType::ResendRequesting(logging_out_initiator));

                    //Start a timer to acknowledge Logout if messages are not fulfilled in a reasonable
                    //amount of time. If they are fulfilled sooner, we'll just acknowledge sooner.
                    match logging_out_initiator {
                        LoggingOutInitiator::Remote => {
                            let timeout_duration = Some(Duration::from_secs(AUTO_CONTINUE_AFTER_LOGOUT_RESEND_REQUEST_SECS));
                            reset_timeout(
                                timer,
                                &mut connection.logout_timeout,
                                &timeout_duration,
                                TimeoutType::ContinueLogout,
                                &connection.token
                            );
                        }
                        LoggingOutInitiator::Local => {
                            //Let the auto-disconnect timer continue even though some messages
                            //might be lost. This is because if the remote ignores our
                            //ResendRequest but responds to a new Logout attempt, we'll have three
                            //possibly catastrophic outcomes.
                            //1. Logout response has MsgSeqNum < expected: Critical error.
                            //2. Logout response has MsgSeqNum > expected: That's the current
                            //   situation so well be looping.
                            //3. Logout response has MsgSeqNum == expected: But last time the
                            //   MsgSeqNum was higher so there is a serious numbering issue.
                            //   Critical error.
                            //If the other side does respond to our ResendRequest appropriately,
                            //we'll restart a clean Logout process.
                        },
                    }
                }
            }

            Some(message)
        }

        fn on_less_than_expected_msg_seq_num(connection: &mut InternalConnection,message: Box<FIXTMessage + Send>,msg_seq_num: MsgSeqNumType,tx: &Sender<EngineEvent>,timer: &mut Timer<(TimeoutType,Token)>) {
            //Messages with MsgSeqNum lower than expected are never processed as normal. They are
            //either duplicates (as indicated) or an unrecoverable error where one side fell
            //out of sync.
            if message.is_poss_dup() {
                if message.orig_sending_time() <= message.sending_time() {
                    //Duplicate message that otherwise seems correct.
                    tx.send(EngineEvent::MessageReceivedDuplicate(connection.as_connection(),message)).unwrap();
                }
                else {
                    //Reject message even though it's a duplicate. Currently, we probably don't
                    //care about the OrigSendingTime vs SendingTime but this is correct processing
                    //according to the spec.
                    reject_for_sending_time_accuracy(connection,message,msg_seq_num,tx);
                }
            }
            else {
                let mut text = b"MsgSeqNum too low, expected ".to_vec();
                text.extend_from_slice(connection.inbound_msg_seq_num.to_string().as_bytes());
                text.extend_from_slice(b" but received ");
                text.extend_from_slice(msg_seq_num.to_string().as_bytes());
                connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::InboundMsgSeqNumLowerThanExpectedError),&text[..]);
            }
        }

        fn on_expected_msg_seq_num(connection: &mut InternalConnection,mut message: Box<FIXTMessage + Send>,msg_seq_num: MsgSeqNumType,tx: &Sender<EngineEvent>,timer: &mut Timer<(TimeoutType,Token)>) -> Result<Option<Box<FIXTMessage + Send>>,ConnectionTerminatedReason> {
            //Start by incrementing expected inbound MsgSeqNum since the message is at least
            //formatted correctly and matches the expected MsgSeqNum.
            try!(connection.increment_inbound_msg_seq_num());

            //Handle general FIXT message validation.
            if message.is_poss_dup() && message.orig_sending_time() > message.sending_time() {
                reject_for_sending_time_accuracy(connection,message,msg_seq_num,tx);
                return Ok(None);
            }

            //Handle SequenceReset-GapFill messages.
            if let Some(sequence_reset) = message.as_any_mut().downcast_mut::<SequenceReset>() {
                if sequence_reset.gap_fill_flag {
                    if sequence_reset.new_seq_no > connection.inbound_msg_seq_num {
                        //Fast forward to the new expected inbound MsgSeqNum.
                        connection.inbound_msg_seq_num = sequence_reset.new_seq_no;
                    }
                    else {
                        //Attempting to rewind MsgSeqNum is not allowed according to FIXT v1.1,
                        //page 29.

                        let mut reject = Reject::new();
                        reject.ref_seq_num = msg_seq_num;
                        reject.session_reject_reason = Some(SessionRejectReason::ValueIsIncorrectForThisTag);
                        reject.text = b"Attempt to lower sequence number, invalid value NewSeqNo=".to_vec();
                        reject.text.extend_from_slice(sequence_reset.new_seq_no.to_string().as_bytes());
                        connection.outbound_messages.push(OutboundMessage::from(reject));

                        tx.send(EngineEvent::MessageRejected(connection.as_connection(),Box::new(mem::replace(sequence_reset,SequenceReset::new())))).unwrap();
                    }
                }
                else {
                    //This should have been handled earlier as a special case that ignores
                    //MsgSeqNum.
                    unreachable!();
                }
            }

            //Handle ResendRequest messages.
            message = match if_on_resend_request(connection,message,msg_seq_num,tx,timer) {
                Some(message) => message,
                None => return Ok(None),
            };

            //Handle Logout messages.
            if let Some(logout) = message.as_any().downcast_ref::<Logout>() {
                //Remote responded to our Logout.
                if let ConnectionStatus::LoggingOut(_) = connection.status {
                    connection.shutdown();
                    return Err(ConnectionTerminatedReason::LocalRequested);
                }
                //Remote started logout process.
                else {
                    connection.respond_to_logout();
                }
            }

            Ok(Some(message))
        }

        //Start by making sure the message is using the expected FIX version. Otherwise, we should
        //logout and disconnect immediately. This test is skipped for newly accepted connections
        //because the expected FIX version has not been decided yet.
        if !connection.status.is_receiving_logon() {
            let ref received_fix_version = message.meta().as_ref().expect("Meta should be set by parser").begin_string;
            let expected_fix_version = connection.fix_version;
            if *received_fix_version != expected_fix_version {
                let mut error_text = b"BeginStr is wrong, expected '".to_vec();
                error_text.extend_from_slice(expected_fix_version.begin_string());
                error_text.extend_from_slice(b"' but received '");
                error_text.extend_from_slice(received_fix_version.begin_string());
                error_text.extend_from_slice(b"'");

                connection.initiate_logout(
                    timer,
                    LoggingOutType::Error(
                        ConnectionTerminatedReason::BeginStrWrongError {
                            received: *received_fix_version,
                            expected: expected_fix_version
                        }
                    ),
                    &error_text[..]
                );

                return Ok(());
            }
        }

        //Every message must have SenderCompID and TargetCompID set to the expected values or else
        //the message must be rejected and we should logout. See FIXT 1.1, page 52.
        //The first check is skipped when listener spawned connection is still receiving a Logon
        //message because it doesn't know who is connecting yet.
        if *message.sender_comp_id() != connection.target_comp_id && !connection.status.is_receiving_logon() {
            connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::SenderCompIDWrongError),b"SenderCompID is wrong");

            let mut reject = Reject::new();
            reject.ref_seq_num = connection.inbound_msg_seq_num;
            reject.session_reject_reason = Some(SessionRejectReason::CompIDProblem);
            reject.text = b"CompID problem".to_vec();
            connection.outbound_messages.insert(0,OutboundMessage::from(reject));

            tx.send(EngineEvent::MessageRejected(connection.as_connection(),message)).unwrap();

            return Ok(());
        }
        else if *message.target_comp_id() != connection.sender_comp_id {
            if connection.status.is_receiving_logon() {
                //Since connection hasn't even logged out, just disconnect immediately.
                connection.shutdown();
                return Err(ConnectionTerminatedReason::TargetCompIDWrongError);
            }
            else {
                //Reject message and then logout.
                connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::TargetCompIDWrongError),b"TargetCompID is wrong");

                let mut reject = Reject::new();
                reject.ref_seq_num = connection.inbound_msg_seq_num;
                reject.session_reject_reason = Some(SessionRejectReason::CompIDProblem);
                reject.text = b"CompID problem".to_vec();
                connection.outbound_messages.insert(0,OutboundMessage::from(reject));

                tx.send(EngineEvent::MessageRejected(connection.as_connection(),message)).unwrap();

                return Ok(());
            }
        }

        //When the connection first starts, it sends a Logon message to the remote. The remote then
        //must respond with a Logon acknowleding the Logon, a Logout rejecting the Logon, or just
        //disconnecting. In this case, if a  Logon is received, we setup timers to send periodic
        //messages in case there is no activity. We then notify the engine that the session is
        //established and other messages can now be sent or received.
        let just_logged_on = if connection.status.is_sending_logon() {
            if let Some(message) = message.as_any().downcast_ref::<Logon>() {
                connection.status = ConnectionStatus::Established;

                if message.heart_bt_int > 0 {
                    connection.outbound_heartbeat_timeout_duration = Some(
                        Duration::from_secs(message.heart_bt_int as u64)
                    );
                    reset_outbound_timeout(timer,&mut connection.outbound_heartbeat_timeout,&connection.outbound_heartbeat_timeout_duration,&connection.token);
                    connection.inbound_testrequest_timeout_duration = Some(
                        Duration::from_millis(message.heart_bt_int as u64 * 1000 + NO_INBOUND_TIMEOUT_PADDING_MS),
                    );
                    reset_inbound_timeout(timer,&mut connection.inbound_testrequest_timeout,&connection.inbound_testrequest_timeout_duration,&connection.token);
                }
                else if message.heart_bt_int < 0 {
                    connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::LogonHeartBtIntNegativeError),b"HeartBtInt cannot be negative");
                    return Ok(());
                }

                //Make parser use the specified message version by default. This is only used if
                //the FIXVersion >= FIXT_1_1. Earlier versions always use the same message version
                //as the FIX version specified in the BeginStr tag.
                connection.parser.set_default_message_version(message.default_appl_ver_id);

                //Make parser use the Message Type Default Application Version if specified.
                for msg_type in &message.no_msg_types {
                    if msg_type.default_ver_indicator && msg_type.msg_direction == MsgDirection::Send && msg_type.ref_appl_ver_id.is_some() {
                        connection.parser.set_default_message_type_version(&msg_type.ref_msg_type[..],msg_type.ref_appl_ver_id.unwrap());
                    }
                }

                //TODO: Need to take MaxMessageSize into account.
                //TODO: Optionally support filtering message types (NoMsgTypes).
                tx.send(EngineEvent::SessionEstablished(connection.as_connection())).unwrap();
            }
            else {
                connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::LogonNotFirstMessageError),b"First message not a logon");
                return Ok(());
            }

            true
        }
        else if connection.status.is_receiving_logon() {
            //Switch from ReceivingLogon to ApprovingLogon. Have to be careful to cancel the
            //timeout.
            let old_status = mem::replace(&mut connection.status,ConnectionStatus::ApprovingLogon);
            let (listener,no_logon_timeout) = if let ConnectionStatus::ReceivingLogon(listener,timeout) = old_status { (listener,timeout) } else { unreachable!() };
            timer.cancel_timeout(&no_logon_timeout);

            if let Some(message) = message.as_any().downcast_ref::<Logon>() {
                //Setup defaults for connection that could not be setup before receiving the Logon.
                //It'll be up to the user of the library to reject if they don't want to support
                //these to the full extent of the library (e.g. an older FIX version).
                connection.fix_version = message.meta.as_ref().expect("Meta should be set by parser").begin_string;
                connection.parser.set_default_message_version(message.default_appl_ver_id);
                connection.inbound_msg_seq_num = message.msg_seq_num + 1;
                connection.target_comp_id = message.sender_comp_id.clone();

                if message.heart_bt_int > 0 {
                    connection.outbound_heartbeat_timeout_duration = Some(
                        Duration::from_secs(message.heart_bt_int as u64)
                    );
                    connection.inbound_testrequest_timeout_duration = Some(
                        Duration::from_millis(message.heart_bt_int as u64 * 1000 + NO_INBOUND_TIMEOUT_PADDING_MS),
                    );
                }
                else if message.heart_bt_int < 0 {
                    connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::LogonHeartBtIntNegativeError),b"HeartBtInt cannot be negative");
                    return Ok(());
                }

                //Make parser use the max supported message version for the selected FIX protocol
                //by default. This must be done before the part below so defaults in the Logon
                //message can't maliciously overwrite them.
                connection.parser.clear_default_message_type_versions();
                for msg_type in administrative_msg_types() {
                    connection.parser.set_default_message_type_version(msg_type,connection.fix_version.max_message_version());
                }

                //Make parser use the Message Type Default Application Version if specified.
                for msg_type in &message.no_msg_types {
                    if msg_type.default_ver_indicator && msg_type.msg_direction == MsgDirection::Send && msg_type.ref_appl_ver_id.is_some() {
                        connection.parser.set_default_message_type_version(&msg_type.ref_msg_type[..],msg_type.ref_appl_ver_id.unwrap());
                    }
                }

                //Block reading of new messages until connection has been approved. This will be
                //automatically unblocked when the Logon response is sent.
                connection.begin_blocking_inbound(timer);

                tx.send(EngineEvent::ConnectionLoggingOn(listener,connection.as_connection(),Box::new(message.clone()))).unwrap();

                return Ok(());
            }
            else {
                connection.initiate_logout(timer,LoggingOutType::Error(ConnectionTerminatedReason::LogonNotFirstMessageError),b"First message not a logon");
                return Ok(());
            }
        }
        else {
            false
        };

        //Perform MsgSeqNum error handling if MsgSeqNum > or < expected. Otherwise, perform
        //administrative message handling and related book keeping.
        let msg_seq_num = message.msg_seq_num();
        if message.as_any_mut().downcast_mut::<SequenceReset>().map_or(false,|sequence_reset| {
            if !sequence_reset.gap_fill_flag {
                if sequence_reset.new_seq_no > connection.inbound_msg_seq_num {
                    connection.inbound_msg_seq_num = sequence_reset.new_seq_no;
                    connection.clear_inbound_resend_request_msg_seq_num(timer);
                }
                else if sequence_reset.new_seq_no == connection.inbound_msg_seq_num {
                    tx.send(EngineEvent::SequenceResetResetHasNoEffect(connection.as_connection())).unwrap();
                }
                else {//if sequence_reset.new_seq_no < connection.inbound_msg_seq_num
                    let mut reject = Reject::new();
                    reject.ref_seq_num = msg_seq_num;
                    reject.session_reject_reason = Some(SessionRejectReason::ValueIsIncorrectForThisTag); //TODO: Is there a better reason or maybe leave this blank?
                    reject.text = b"Attempt to lower sequence number, invalid value NewSeqNo=".to_vec();
                    reject.text.extend_from_slice(sequence_reset.new_seq_no.to_string().as_bytes());
                    connection.outbound_messages.push(OutboundMessage::from(reject));

                    tx.send(EngineEvent::SequenceResetResetInThePast(connection.as_connection())).unwrap();
                }

                true
            }
            else {
                false
            }
        }) {
            //Special case where MsgSeqNum does not matter. Handled above.
        }
        else if msg_seq_num > connection.inbound_msg_seq_num {
            message = match on_greater_than_expected_msg_seq_num(connection,message,msg_seq_num,tx,timer) {
                Some(message) => message,
                None => return Ok(()),
            };

            //The only message that can be processed out of order is the Logon message. Every other
            //one will be discarded and we'll wait for the in-order resend.
            if !just_logged_on {
                //Message is discarded.
                return Ok(());
            }
        }
        else if msg_seq_num < connection.inbound_msg_seq_num {
            on_less_than_expected_msg_seq_num(connection,message,msg_seq_num,tx,timer);
            return Ok(());
        }
        else {
            message = match try!(on_expected_msg_seq_num(connection,message,msg_seq_num,tx,timer)) {
                Some(message) => message,
                None => return Ok(()),
            };

            //If the current message has caught up with our outstanding ResendRequest, mark it as
            //such so we don't send another.
            if let Some(resend_request_msg_seq_num) = connection.inbound_resend_request_msg_seq_num {
                if resend_request_msg_seq_num <= connection.inbound_msg_seq_num {
                    connection.clear_inbound_resend_request_msg_seq_num(timer);
                }
            }
        }

        //Reply to TestRequest automatically with a Heartbeat. Typical keep alive stuff.
        if let Some(test_request) = message.as_any().downcast_ref::<TestRequest>() {
            let mut heartbeat = Heartbeat::new();
            heartbeat.test_req_id = test_request.test_req_id.clone();
            connection.outbound_messages.push(OutboundMessage::from(heartbeat));
        }

        tx.send(EngineEvent::MessageReceived(connection.as_connection(),message)).unwrap();

        Ok(())
    }

    fn on_network_parse_error(connection: &mut InternalConnection,parse_error: ParseError,tx: &Sender<EngineEvent>)-> Result<(),ConnectionTerminatedReason> {
        fn push_reject<T: Into<Vec<u8>>>(connection: &mut InternalConnection,ref_msg_type: &[u8],ref_tag_id: T,session_reject_reason: SessionRejectReason,text: &[u8]) -> Result<(),ConnectionTerminatedReason> {
            let mut reject = Reject::new();
            reject.ref_msg_type = ref_msg_type.to_vec();
            reject.ref_tag_id = ref_tag_id.into();
            reject.ref_seq_num = connection.inbound_msg_seq_num;
            reject.session_reject_reason = Some(session_reject_reason);
            reject.text = text.to_vec();
            connection.outbound_messages.push(OutboundMessage::from(reject));

            Ok(())
        }

        match connection.status {
            //There's no room for errors when attempting to logon. If the network data cannot be
            //parsed, just disconnect immediately.
            ConnectionStatus::SendingLogon => {
                connection.shutdown();
                return Err(ConnectionTerminatedReason::LogonParseError(parse_error));
            },
            //Handle parse error as normal. Usually just respond with a Reject
            _ => {
                match parse_error {
                    ParseError::MissingRequiredTag(ref tag,_) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::RequiredTagMissing,b"Required tag missing"));
                    },
                    ParseError::UnexpectedTag(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::TagNotDefinedForThisMessageType,b"Tag not defined for this message type"));
                    },
                    ParseError::UnknownTag(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::InvalidTagNumber,b"Invalid tag number"));
                    },
                    ParseError::NoValueAfterTag(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::TagSpecifiedWithoutAValue,b"Tag specified without a value"));
                    },
                    ParseError::OutOfRangeTag(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::ValueIsIncorrectForThisTag,b"Value is incorrect (out of range) for this tag"));
                    },
                    ParseError::WrongFormatTag(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::IncorrectDataFormatForValue,b"Incorrect data format for value"));
                    },
                    ParseError::SenderCompIDNotFourthTag => {
                        try!(push_reject(connection,b"",SenderCompID::tag_bytes(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder,b"SenderCompID must be the 4th tag"));
                    },
                    ParseError::TargetCompIDNotFifthTag => {
                        try!(push_reject(connection,b"",TargetCompID::tag_bytes(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder,b"TargetCompID must be the 5th tag"));
                    },
                    ParseError::ApplVerIDNotSixthTag => {
                        try!(push_reject(connection,b"",ApplVerID::tag_bytes(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder,b"ApplVerID must be the 6th tag if specified"));
                    },
                    ParseError::MessageSizeTooBig => {
                        let mut error_text = b"Message size exceeds MaxMessageSize=".to_vec();
                        error_text.extend_from_slice(connection.parser.max_message_size().to_string().as_bytes());
                        try!(push_reject(connection,b"",Vec::new(),SessionRejectReason::Other,&error_text[..]));
                    },
                    ParseError::BeginStrNotFirstTag |
                    ParseError::BodyLengthNotSecondTag |
                    ParseError::MsgTypeNotThirdTag |
                    ParseError::ChecksumNotLastTag |
                    ParseError::MissingPrecedingLengthTag(_) |
                    ParseError::MissingFollowingLengthTag(_) => {
                        try!(push_reject(connection,b"",Vec::new(),SessionRejectReason::TagSpecifiedOutOfRequiredOrder,b"Tag specified out of required order"));
                    },
                    ParseError::DuplicateTag(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::TagAppearsMoreThanOnce,b"Tag appears more than once"));
                    },
                    ParseError::MissingConditionallyRequiredTag(ref tag,ref message) => {
                        if *tag == OrigSendingTime::tag() { //Session level conditionally required tag.
                            try!(push_reject(connection,message.msg_type(),*tag,SessionRejectReason::RequiredTagMissing,b"Conditionally required tag missing"));
                        }
                        else {
                            let mut business_message_reject = BusinessMessageReject::new();
                            business_message_reject.ref_seq_num = connection.inbound_msg_seq_num;
                            business_message_reject.ref_msg_type = message.msg_type().to_vec();
                            business_message_reject.business_reject_reason = BusinessRejectReason::ConditionallyRequiredFieldMissing;
                            business_message_reject.business_reject_ref_id = tag.to_bytes();
                            business_message_reject.text = b"Conditionally required field missing".to_vec();
                            connection.outbound_messages.push(OutboundMessage::from(business_message_reject));
                        }
                    },
                    ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(ref tag) |
                    ParseError::NonRepeatingGroupTagInRepeatingGroup(ref tag) |
                    ParseError::RepeatingGroupTagWithNoRepeatingGroup(ref tag) => {
                        try!(push_reject(connection,b"",*tag,SessionRejectReason::IncorrectNumInGroupCountForRepeatingGroup,b"Incorrect NumInGroup count for repeating group"));
                    },
                    ParseError::MsgTypeUnknown(ref msg_type) => {
                        //If we're here, we know the MsgType is not user defined. So we just need
                        //to know if it's defined in the spec (Unsupported MsgType) or completely
                        //unknown (Invalid MsgType).
                        if standard_msg_types().contains(&msg_type[..]) {
                            //MsgType is unsupported.
                            let mut business_message_reject = BusinessMessageReject::new();
                            business_message_reject.ref_seq_num = connection.inbound_msg_seq_num;
                            business_message_reject.ref_msg_type = msg_type.to_vec();
                            business_message_reject.business_reject_reason = BusinessRejectReason::UnsupportedMessageType;
                            business_message_reject.business_reject_ref_id = business_message_reject.ref_msg_type.clone();
                            business_message_reject.text = b"Unsupported Message Type".to_vec();
                            connection.outbound_messages.push(OutboundMessage::from(business_message_reject));
                        }
                        else {
                            //MsgType is invalid.
                            try!(push_reject(connection,&msg_type[..],&msg_type[..],SessionRejectReason::InvalidMsgType,b"Invalid MsgType"));
                        }
                    },
                    _ => {}, //TODO: Support other errors as appropriate.
                };

                //Always increment expected inbound MsgSeqNum after encountering a message that is
                //garbled, cannot be parsed, or is otherwise invalid. See FIXT 1.1, page 26.
                try!(connection.increment_inbound_msg_seq_num());

                //Tell user about the garbled message just in case they care.
                tx.send(EngineEvent::MessageReceivedGarbled(connection.as_connection(),parse_error)).unwrap();
            },
        };

        Ok(())
    }
}

pub fn internal_engine_thread(poll: Poll,
                              token_generator: Arc<Mutex<TokenGenerator>>,
                              tx: Sender<EngineEvent>,
                              rx: Receiver<InternalEngineToThreadEvent>,
                              message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
                              max_message_size: u64) {
    //TODO: There should probably be a mechanism to log every possible message, even those we
    //handle automatically. One method might be to have a layer above this that handles the
    //automatic stuff and allows for logging...this is probably just too low level.

    let mut internal_thread = InternalThread {
        poll: poll,
        token_generator: token_generator,
        tx: tx,
        rx: rx,
        message_dictionary: message_dictionary,
        max_message_size: max_message_size,
        connections: HashMap::new(),
        listeners: HashMap::new(),
        timer: TimerBuilder::default()
            .tick_duration(Duration::from_millis(TIMER_TICK_MS))
            .num_slots(TIMER_TIMEOUTS_PER_TICK_MAX)
            .capacity(CONNECTION_COUNT_MAX * TIMEOUTS_PER_CONNECTION_MAX)
            .build(),
        network_read_retry: NetworkReadRetry::new(),
    };
    let mut terminated_connections: Vec<(InternalConnection,ConnectionTerminatedReason)> = Vec::new();

    //Have poll let us know when we need to send a heartbeat, testrequest, or respond to some other
    //timeout.
    if let Err(e) = internal_thread.poll.register(&internal_thread.timer,TIMEOUT_TOKEN,Ready::readable(),PollOpt::level()) {
        internal_thread.tx.send(EngineEvent::FatalError("Cannot register timer for polling",e)).unwrap();
        return;
    }

    //Have poll let us know when we need to retry parsing and/or reading incoming bytes. This
    //typically occurs when messages are being received faster than they can be parsed in order to
    //give the already parsed messages a chance to be processed.
    if let Err(e) = internal_thread.poll.register(&internal_thread.network_read_retry,NETWORK_READ_RETRY_TOKEN,Ready::readable(),PollOpt::level()) {
        internal_thread.tx.send(EngineEvent::FatalError("Cannot register network read retry for polling",e)).unwrap();
        return;
    }

    //Poll events sent by Engine, triggered by timer timeout, or network activity and act upon them
    //on a per-connection basis.
    let mut events = Events::with_capacity(EVENT_POLL_CAPACITY);
    loop {
        if let Err(e) = internal_thread.poll.poll(&mut events,None) {
            internal_thread.tx.send(EngineEvent::FatalError("Cannot poll events",e)).unwrap();
            return;
        }

        for event in events.iter() {
            let result = match event.token() {
                INTERNAL_ENGINE_EVENT_TOKEN => internal_thread.on_internal_engine_event(),
                TIMEOUT_TOKEN => internal_thread.on_timeout(),
                NETWORK_READ_RETRY_TOKEN => {
                    if let Some(token) = internal_thread.network_read_retry.poll() {
                        internal_thread.on_network(&Event::new(Ready::readable(),token))
                    }
                    else {
                        //Connection was probably removed before retry could be run.
                        Ok(())
                    }
                },
                _ => internal_thread.on_network(&event),
            };

            //Handle errors from event. Terminated connections are stored until all events have
            //been processed so we don't accidentally re-use an event token for a new connection
            //with one that was terminated and still has events pending.
            if let Err(e) = result {
                match e {
                    ConnectionEventError::TerminateConnection(connection,e) => {
                        terminated_connections.push((connection,e));
                    },
                    ConnectionEventError::Shutdown => return,
                };
            }
        }

        //Clean-up connections that have been shutdown (cleanly or on error).
        terminated_connections.drain(..).all(|terminated_connection| {
            let (connection,e) = terminated_connection;

            let _ = internal_thread.poll.deregister(&connection.socket);
            if let Some(ref timeout) = connection.outbound_heartbeat_timeout {
                internal_thread.timer.cancel_timeout(timeout);
            }
            if let Some(ref timeout) = connection.inbound_testrequest_timeout {
                internal_thread.timer.cancel_timeout(timeout);
            }
            if let Some(ref timeout) = connection.inbound_blocked_timeout {
                internal_thread.timer.cancel_timeout(timeout);
            }
            if let Some(ref timeout) = connection.logout_timeout {
                internal_thread.timer.cancel_timeout(timeout);
            }
            if let ConnectionStatus::ReceivingLogon(_,ref timeout) = connection.status {
                internal_thread.timer.cancel_timeout(timeout);
            }

            internal_thread.network_read_retry.remove_all(connection.token);

            //Notify user in the special case where connection was never even established. This
            //block is incredibly ugly but required to appease the borrow checker.
            let e = if let ConnectionTerminatedReason::SocketReadError(err) = e {
                if !connection.is_connected {
                    internal_thread.tx.send(EngineEvent::ConnectionFailed(connection.as_connection(),err)).unwrap();
                    return true;
                }

                ConnectionTerminatedReason::SocketReadError(err)
            } else { e };

            //Notify user that connection was terminated.
            internal_thread.tx.send(EngineEvent::ConnectionTerminated(connection.as_connection(),e)).unwrap();

            true
        });
    }
}
