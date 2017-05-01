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

extern crate chrono;
extern crate fix_rs;
extern crate mio;

use mio::{Events,Poll,PollOpt,Ready,Token};
use mio::tcp::{TcpListener,TcpStream};
use mio::unix::UnixReady;
use std::any::Any;
use std::collections::HashMap;
use std::net::{Ipv4Addr,SocketAddr,SocketAddrV4};
use std::io::Read;
use std::sync::atomic::{AtomicUsize,Ordering};
use std::thread;
use std::time::{Duration,Instant};

use fix_rs::byte_buffer::ByteBuffer;
use fix_rs::dictionary::CloneDictionary;
use fix_rs::dictionary::field_types::other::EncryptMethod;
use fix_rs::dictionary::messages::Logon;
use fix_rs::fix::Parser;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt::engine::{Engine,EngineEvent,Connection,Listener};
use fix_rs::fixt::message::{BuildFIXTMessage,FIXTMessage};
use fix_rs::message_version::MessageVersion;

const SOCKET_BASE_PORT: usize = 7000;
static SOCKET_PORT: AtomicUsize = AtomicUsize::new(SOCKET_BASE_PORT);

pub const CLIENT_TARGET_COMP_ID: &'static [u8] = b"TX"; //Test Exchange
pub const CLIENT_SENDER_COMP_ID: &'static [u8] = b"TEST";
pub const SERVER_TARGET_COMP_ID: &'static [u8] = CLIENT_SENDER_COMP_ID;
pub const SERVER_SENDER_COMP_ID: &'static [u8] = CLIENT_TARGET_COMP_ID;

const MAX_MESSAGE_SIZE: u64 = 4096;

//Helper function to make it easier to figure out what the body_length tag should be set to.
#[allow(unused)]
fn estimate_body_length(message_bytes: &[u8]) -> usize {
    let mut previous_byte = 0;
    let mut found_body_length_tag = false;
    let mut body_start = 0;
    for (index,byte) in message_bytes.iter().enumerate() {
        if body_start == 0 && found_body_length_tag && *byte == b'\x01' {
            body_start = index + 1;
        }
        if previous_byte == b'9' && *byte == b'=' {
            found_body_length_tag = true;
        }
        if previous_byte == b'0' && *byte == b'=' && message_bytes[index - 2] == b'1' && message_bytes[index - 3] == 1 {
            return index - 2 - body_start;
        }
        previous_byte = *byte;
    }

    panic!("Message is malformed.");
}

#[macro_export]
macro_rules! engine_poll_event {
    ( $engine:ident,$pat:pat => $body:expr ) => {{
        let result = $engine.poll(Some(Duration::from_secs(5))).expect("Engine does not have any events");
        if let $pat = result {
            $body
        }
        else {
            panic!("Engine has wrong event: {:?}",result)
        }
    }};
}

#[macro_export]
macro_rules! engine_poll_no_event {
    ( $engine:ident ) => {{
        let result = $engine.poll(Some(Duration::from_secs(5)));
        if let Some(result) = result {
            panic!("Engine has an event: {:?}",result)
        }
    }};
}

#[macro_export]
macro_rules! engine_poll_message {
    ( $engine:ident, $connection:ident, $message_type:ty ) => {
        engine_poll_event!($engine,EngineEvent::MessageReceived(msg_connection,response_message) => {
            assert_eq!(msg_connection,$connection);

            response_message.as_any().downcast_ref::<$message_type>().expect("Not expected message type").clone()
        });
    };
}

#[macro_export]
macro_rules! engine_gap_fill_resend_request {
    ( $engine:ident, $connection:ident, $expected_range:expr ) => {
        engine_poll_event!($engine,EngineEvent::ResendRequested(connection,range) => {
            let expected_start = $expected_range.start;
            let expected_end = $expected_range.end;

            assert_eq!(connection,$connection);
            assert_eq!(range.start,expected_start);
            assert_eq!(range.end,expected_end);

            let mut response = Vec::new();
            response.push(ResendResponse::Gap(range));
            $engine.send_resend_response(connection,response);
        });
    };
}

#[macro_export]
macro_rules! new_fixt_message {
    ( FROM_SERVER $message_type:ident ) => {{
        let mut message = $message_type::new();
        message.setup_fixt_session_header(
            Some(1),
            $crate::common::SERVER_SENDER_COMP_ID.to_vec(),
            $crate::common::SERVER_TARGET_COMP_ID.to_vec()
        );

        message
    }};

    ( FROM_CLIENT $message_type:ident ) => {{
        let mut message = $message_type::new();
        message.setup_fixt_session_header(
            Some(1),
            $crate::common::CLIENT_SENDER_COMP_ID.to_vec(),
            $crate::common::CLIENT_TARGET_COMP_ID.to_vec()
        );

        message
    }};

    ( $message_type:ident ) => {{
        new_fixt_message!(FROM_SERVER $message_type)
    }};
}

pub fn new_logon_message() -> Logon {
    let mut message = new_fixt_message!(FROM_SERVER Logon);
    message.encrypt_method = EncryptMethod::None;
    message.heart_bt_int = 5;
    message.default_appl_ver_id = MessageVersion::FIX50SP2;

    message
}

pub fn accept_with_timeout(listener: &TcpListener,timeout: Duration) -> Option<TcpStream> {
    let now = Instant::now();

    while now.elapsed() <= timeout {
        if let Ok((stream,_)) = listener.accept() {
            return Some(stream);
        }

        thread::yield_now();
    }

    None
}

pub fn recv_bytes_with_timeout(stream: &mut TcpStream,timeout: Duration) -> Option<Vec<u8>> {
    let now = Instant::now();

    let mut buffer = Vec::new();
    buffer.resize(1024,0);

    while now.elapsed() <= timeout {
        if let Ok(bytes_read) = stream.read(&mut buffer[..]) {
            if bytes_read > 0 {
                buffer.resize(bytes_read,0u8);
                return Some(buffer);
            }
        }

        thread::yield_now();
    }

    None
}

pub fn send_message_with_timeout(stream: &mut TcpStream,fix_version: FIXVersion,message_version: MessageVersion,message: Box<FIXTMessage + Send>,timeout: Option<Duration>) -> Result<(),usize> {
    let mut bytes = ByteBuffer::with_capacity(512);
    message.read(fix_version,message_version,&mut bytes);

    let now = Instant::now();
    while !bytes.is_empty() {
        if let Some(timeout) = timeout {
            if now.elapsed() > timeout {
                return Err(bytes.len());
            }
        }

        if let Err(e) = bytes.write(stream) {
            if e.kind() == ::std::io::ErrorKind::WouldBlock {
                continue;
            }
            panic!("Could not write bytes: {}",e);
        }
    }

    Ok(())
}

pub fn send_message(stream: &mut TcpStream,fix_version: FIXVersion,message_version: MessageVersion,message: Box<FIXTMessage + Send>) {
    let _ = send_message_with_timeout(stream,fix_version,message_version,message,None);
}

pub struct TestStream {
    fix_version: FIXVersion,
    message_version: MessageVersion,
    pub stream: TcpStream,
    poll: Poll,
    parser: Parser,
}

impl TestStream {
    fn new(fix_version: FIXVersion,message_version: MessageVersion,stream: TcpStream,message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> TestStream {
        //Setup a single Poll to watch the TCPStream. This way we can check for disconnects in
        //is_stream_closed(). Unfortunately, as of mio 0.6.1, Linux implementation emulates OS X
        //and Windows where a stream can only be registered with one Poll for the life of the
        //socket. See: https://github.com/carllerche/mio/issues/327
        let poll = Poll::new().unwrap();
        poll.register(&stream,Token(0),Ready::readable() | Ready::writable() | UnixReady::hup() | UnixReady::error(),PollOpt::edge()).unwrap();

        let mut parser = Parser::new(message_dictionary,MAX_MESSAGE_SIZE);
        parser.set_default_message_version(message_version);

        TestStream {
            fix_version: fix_version,
            message_version: message_version,
            stream: stream,
            poll: poll,
            parser: parser
        }
    }

    pub fn setup_test_server_with_ver(fix_version: FIXVersion,message_version: MessageVersion,message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Connection) {
        //Setup server listener socket.
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1),SOCKET_PORT.fetch_add(1,Ordering::SeqCst) as u16));
        let listener = TcpListener::bind(&addr).unwrap();

        //Setup client and connect to socket.
        let mut client = Engine::new(message_dictionary.clone(),MAX_MESSAGE_SIZE).unwrap();
        let connection = client.add_connection(fix_version,message_version,CLIENT_SENDER_COMP_ID,CLIENT_TARGET_COMP_ID,addr).unwrap();

        //Try to accept connection from client. Fails on timeout or socket error.
        let stream = accept_with_timeout(&listener,Duration::from_secs(5)).expect("Could not accept connection");

        //Confirm client was able to connect.
        let event = client.poll(Duration::from_secs(5)).expect("Could not connect");
        assert!(if let EngineEvent::ConnectionSucceeded(success_connection) = event { success_connection == connection } else { false });

        (TestStream::new(fix_version,message_version,stream,message_dictionary),
         client,
         connection)
    }

    pub fn setup_test_server(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Connection) {
        Self::setup_test_server_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message_dictionary)
    }

    pub fn setup_test_server_and_logon_with_ver(fix_version: FIXVersion,message_version: MessageVersion,message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Connection) {
        //Connect.
        let (mut test_server,mut client,connection) = Self::setup_test_server_with_ver(fix_version,message_version,message_dictionary);
        test_server.parser.set_default_message_version(MessageVersion::FIX50);

        //Logon.
        let mut logon_message = new_logon_message();
        logon_message.default_appl_ver_id = message_version;
        client.send_message_box_with_message_version(connection,fix_version.max_message_version(),Box::new(logon_message));
        let message = test_server.recv_message::<Logon>();
        assert_eq!(message.msg_seq_num,1);

        let mut response_message = new_fixt_message!(Logon);
        response_message.encrypt_method = message.encrypt_method;
        response_message.heart_bt_int = message.heart_bt_int;
        response_message.default_appl_ver_id = message.default_appl_ver_id;
        test_server.send_message_with_ver(fix_version,fix_version.max_message_version(),response_message);
        engine_poll_event!(client,EngineEvent::SessionEstablished(_) => {});
        let message = engine_poll_message!(client,connection,Logon);
        assert_eq!(message.msg_seq_num,1);

        //After logon, just like the Engine, setup the default message version that future messages
        //should adhere to.
        test_server.parser.set_default_message_version(message_version);

        (test_server,client,connection)
    }

    pub fn setup_test_server_and_logon(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Connection) {
        Self::setup_test_server_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message_dictionary)
    }

    pub fn setup_test_client_with_ver(fix_version: FIXVersion,message_version: MessageVersion,message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Listener,Connection) {
        //Setup client and listener.
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1),SOCKET_PORT.fetch_add(1,Ordering::SeqCst) as u16));
        let mut client = Engine::new(message_dictionary.clone(),MAX_MESSAGE_SIZE).unwrap();
        let listener = client.add_listener(SERVER_SENDER_COMP_ID,&addr).unwrap().unwrap();

        //Setup a client socket and connect to server.
        let stream = TcpStream::connect(&addr).unwrap();

        //Confirm client was able to connect.
        let event = client.poll(Duration::from_secs(5)).expect("Could not accept");
        let connection = match event {
            EngineEvent::ConnectionAccepted(success_listener,accepted_connection,_) => {
                assert_eq!(success_listener,listener);
                accepted_connection
            },
            _ => panic!("Unexpected event")
        };

        (TestStream::new(fix_version,message_version,stream,message_dictionary),
         client,
         listener,
         connection)
    }

    pub fn setup_test_client(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Listener,Connection) {
        Self::setup_test_client_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message_dictionary)
    }

    pub fn setup_test_client_and_logon_with_ver(fix_version: FIXVersion,message_version: MessageVersion,message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Listener,Connection) {
        //Connect.
        let (mut test_client,mut engine,listener,connection) = Self::setup_test_client_with_ver(fix_version,message_version,message_dictionary);
        test_client.parser.set_default_message_version(MessageVersion::FIX50);

        //Logon.
        let mut logon_message = new_logon_message();
        logon_message.sender_comp_id = CLIENT_SENDER_COMP_ID.to_vec();
        logon_message.target_comp_id = CLIENT_TARGET_COMP_ID.to_vec();
        logon_message.default_appl_ver_id = message_version;
        test_client.send_message_with_ver(fix_version,fix_version.max_message_version(),logon_message);

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

        let message = test_client.recv_message::<Logon>();
        assert_eq!(message.msg_seq_num,1);

        //After logon, just like the engine, setup the default message version that future messages
        //should adhere to.
        test_client.parser.set_default_message_version(message_version);

        (test_client,engine,listener,connection)
    }

    pub fn setup_test_client_and_logon(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> (TestStream,Engine,Listener,Connection) {
        Self::setup_test_client_and_logon_with_ver(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,message_dictionary)
    }

    pub fn is_stream_closed(&self,timeout: Duration) -> bool {
        let now = Instant::now();

        while now.elapsed() <= timeout {
            let mut events = Events::with_capacity(1);
            self.poll.poll(&mut events,Some(Duration::from_millis(0))).unwrap();

            for event in events.iter() {
                let readiness = UnixReady::from(event.readiness());
                if readiness.is_hup() || readiness.is_error() {
                    return true;
                }
            }

            thread::yield_now();
        }

        false
    }

    pub fn try_recv_fixt_message(&mut self,timeout: Duration) -> Option<Box<FIXTMessage + Send>> {
        if !self.parser.messages.is_empty() {
            return Some(self.parser.messages.remove(0));
        }

        let now = Instant::now();

        let mut buffer = Vec::new();
        buffer.resize(1024,0);

        while now.elapsed() <= timeout {
            let bytes_read = if let Ok(bytes_read) = self.stream.read(&mut buffer[..]) {
                bytes_read
            }
            else {
                thread::yield_now();
                continue;
            };

            let mut total_bytes_parsed = 0;
            while total_bytes_parsed < bytes_read {
                let (bytes_parsed,result) = self.parser.parse(&buffer[total_bytes_parsed..bytes_read]);
                if result.is_err() {
                    println!("try_recv_fixt_message: Parse error"); //TODO: Use Result instead of Option.
                    println!("\t{}",result.err().unwrap());
                    return None;
                }

                total_bytes_parsed += bytes_parsed;
            }

            if !self.parser.messages.is_empty() {
                return Some(self.parser.messages.remove(0));
            }
        }

        println!("try_recv_fixt_message: Timed out");
        None
    }

    pub fn recv_fixt_message(&mut self) -> Box<FIXTMessage + Send> {
        self.try_recv_fixt_message(Duration::from_secs(5)).expect("Did not receive FIXT message")
    }

    pub fn recv_message<T: FIXTMessage + Any + Clone>(&mut self) -> T {
        let fixt_message = self.recv_fixt_message();
        if !fixt_message.as_any().is::<T>() {
            println!("{:?}",fixt_message);
        }
        fixt_message.as_any().downcast_ref::<T>().expect("^^^ Not expected message type").clone()
    }

    pub fn send_message_with_ver<T: FIXTMessage + Any + Send>(&mut self,fix_version: FIXVersion,message_version: MessageVersion,message: T) {
        send_message(&mut self.stream,fix_version,message_version,Box::new(message));
    }

    pub fn send_message<T: FIXTMessage + Any + Send>(&mut self,message: T) {
        let fix_version = self.fix_version;
        let message_version = self.message_version;
        self.send_message_with_ver::<T>(fix_version,message_version,message);
    }

    pub fn send_message_with_timeout<T: FIXTMessage + Any + Send>(&mut self,message: T,timeout: Duration) -> Result<(),usize> {
        let fix_version = self.fix_version;
        let message_version = self.message_version;
        send_message_with_timeout(&mut self.stream,fix_version,message_version,Box::new(message),Some(timeout))
    }
}

