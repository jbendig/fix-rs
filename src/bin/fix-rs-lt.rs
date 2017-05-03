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

#![feature(attr_literals)]

extern crate clap;
#[macro_use]
extern crate fix_rs;
#[macro_use]
extern crate fix_rs_macros;
extern crate mio;

use clap::{App,Arg};
use mio::{Events,Poll,PollOpt,Ready,Token};
use mio::tcp::TcpStream;
use mio::unix::UnixReady;
use std::any::Any;
use std::collections::HashMap;
use std::net::{Ipv4Addr,SocketAddr,SocketAddrV4};
use std::io::{self,Read};
use std::marker::PhantomData;
use std::mem;
use std::thread;
use std::time::{Duration,Instant};

use fix_rs::byte_buffer::ByteBuffer;
use fix_rs::dictionary::fields::{ApplVerID,MsgSeqNum,OrigSendingTime,SenderCompID,SendingTime,TargetCompID,TestReqID};
use fix_rs::dictionary::field_types::generic::StringFieldType;
use fix_rs::dictionary::field_types::other::EncryptMethod;
use fix_rs::dictionary::messages::Logon;
use fix_rs::field::Field;
use fix_rs::field_tag;
use fix_rs::field_type::FieldType;
use fix_rs::fix::Parser;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt;
use fix_rs::fixt::message::{BuildFIXTMessage,FIXTMessage};
use fix_rs::message::{self,REQUIRED,NOT_REQUIRED,Message,SetValueError};
use fix_rs::message_version::{self,MessageVersion};

const SEND_MESSAGE_TIMEOUT_SECS: u64 = 10;
const MAX_MESSAGE_SIZE: u64 = 4096;
const MESSAGE_COUNT: u64 = 1_000_000;

define_fields!(
    EmptySenderCompID: EmptyFieldType = 49,
    EmptyTargetCompID: EmptyFieldType = 56,
    EmptySendingTime: EmptyFieldType = 52,
    StringSendingTime: StringFieldType = 52,
);

pub struct EmptyFieldType;

impl FieldType for EmptyFieldType {
    type Type = PhantomData<()>;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(_: &mut Self::Type,_: &[u8]) -> Result<(),SetValueError> {
        Ok(())
    }

    fn is_empty(_: &Self::Type) -> bool {
        true
    }

    fn len(_: &Self::Type) -> usize {
        0
    }

    fn read(_: &Self::Type,_: FIXVersion,_: MessageVersion,_: &mut Vec<u8>) -> usize {
        0
    }
}

define_message!(Heartbeat: b"0" => {
    //Used parts of Standard Header
    REQUIRED, sender_comp_id: EmptySenderCompID [FIX40..],
    REQUIRED, target_comp_id: EmptyTargetCompID [FIX40..],
    REQUIRED, msg_seq_num: MsgSeqNum [FIX40..],
    REQUIRED, sending_time: EmptySendingTime [FIX40..],

    //Other
    NOT_REQUIRED, test_req_id: TestReqID [FIX40..],
});

impl FIXTMessage for Heartbeat {
    fn new_into_box(&self) -> Box<FIXTMessage + Send> {
        Box::new(Heartbeat::new())
    }

    fn msg_type(&self) -> &'static [u8] {
        b"0"
    }

    fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
        self.msg_seq_num
    }

    fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn is_poss_dup(&self) -> bool {
        unimplemented!();
    }

    fn set_is_poss_dup(&mut self,_is_poss_dup: bool) {
        unimplemented!();
    }

    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn set_orig_sending_time(&mut self,_orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type) {
        unimplemented!();
    }

    fn setup_fixt_session_header(&mut self,
                                 _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                 _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                 _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
        unimplemented!();
    }
}

define_message!(TestRequest: b"1" => {
    //Used parts of Standard Header
    REQUIRED, sender_comp_id: SenderCompID [FIX40..],
    REQUIRED, target_comp_id: TargetCompID [FIX40..],
    NOT_REQUIRED, appl_ver_id: ApplVerID [FIX40..],
    REQUIRED, msg_seq_num: MsgSeqNum [FIX40..],
    REQUIRED, sending_time: StringSendingTime [FIX40..],

    //Other
    NOT_REQUIRED, test_req_id: TestReqID [FIX40..],
});

impl FIXTMessage for TestRequest {
    fn new_into_box(&self) -> Box<FIXTMessage + Send> {
        Box::new(TestRequest::new())
    }

    fn msg_type(&self) -> &'static [u8] {
        b"1"
    }

    fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
        self.msg_seq_num
    }

    fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
        &self.sender_comp_id
    }

    fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
        &self.target_comp_id
    }

    fn is_poss_dup(&self) -> bool {
        unimplemented!();
    }

    fn set_is_poss_dup(&mut self,_is_poss_dup: bool) {
        unimplemented!();
    }

    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn set_orig_sending_time(&mut self,_orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type) {
        unimplemented!();
    }

    fn setup_fixt_session_header(&mut self,
                                 msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                 sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                 target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
        if let Some(msg_seq_num) = msg_seq_num {
            self.msg_seq_num = msg_seq_num;
        }
        self.sender_comp_id = sender_comp_id;
        self.target_comp_id = target_comp_id;
        self.sending_time = b"20170101-00:00:00".to_vec();
    }
}

#[derive(Clone)]
struct LatencyResult {
    begin_send_time: Instant,
    end_parse_time: Instant,
}

fn print_statistics(total_duration: Duration,latency_results: Vec<LatencyResult>) {
    let mut sum_latency = Duration::new(0,0);
    let mut min_latency = Duration::new(u64::max_value(),0);
    let mut max_latency = Duration::new(u64::min_value(),u32::min_value());
    for latency_result in &latency_results {
        let latency = latency_result.end_parse_time - latency_result.begin_send_time;

        sum_latency += latency;
        min_latency = std::cmp::min(min_latency,latency);
        max_latency = std::cmp::max(max_latency,latency);
    }
    let average_latency = sum_latency / latency_results.len() as u32;

    println!("Sent {} TestRequest messages",MESSAGE_COUNT);
    println!("  Total duration: {:?}",total_duration);
    println!("  Minimum Latency: {:?}",min_latency);
    println!("  Average latency: {:?}",average_latency);
    println!("  Maximum Latency: {:?}",max_latency);
}

struct Connection {
    poll: Poll,
    stream: TcpStream,
    parser: Parser,
    outbound_buffer: ByteBuffer,
    inbound_buffer: ByteBuffer,
}

impl Connection {
    pub fn connect_and_logon(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) -> Result<Connection,io::Error> {
        //Connect to server.
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1),7001));
        let mut connection = Connection {
            poll: Poll::new().unwrap(),
            stream: TcpStream::connect(&addr).unwrap(),
            parser: Parser::new(message_dictionary,MAX_MESSAGE_SIZE),
            outbound_buffer: ByteBuffer::with_capacity(1024),
            inbound_buffer: ByteBuffer::with_capacity(16384),
        };

        connection.poll.register(&connection.stream,
                                 Token(0),
                                 Ready::readable() | Ready::writable() | UnixReady::hup() | UnixReady::error(),
                                 PollOpt::edge()).unwrap();

        //Logon.
        let mut logon_message = Logon::new();
        logon_message.setup_fixt_session_header(
            Some(1),
            b"fix-rs-lt".to_vec(),
            b"Server".to_vec()
        );
        logon_message.encrypt_method = EncryptMethod::None;
        logon_message.heart_bt_int = 60;
        logon_message.default_appl_ver_id = MessageVersion::FIX50SP2;
        logon_message.username = b"some_user".to_vec();
        logon_message.password = b"some_password".to_vec();
        try!(connection.send_message(logon_message));
        try!(connection.recv_message::<Logon>().map(|_|()));

        Ok(connection)
    }

    /// Try to send an entire message within SEND_MESSAGE_TIMEOUT_SECS seconds. Failing to send all
    /// bytes before the timeout triggers a panic.
    fn send_message<T: FIXTMessage + Any + Send>(&mut self,message: T) -> Result<(),io::Error> {
        let mut bytes = ByteBuffer::new();
        message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);

        let now = Instant::now();
        let timeout = Some(Duration::from_secs(SEND_MESSAGE_TIMEOUT_SECS));
        while !bytes.is_empty() {
            if let Some(timeout) = timeout {
                if now.elapsed() > timeout {
                    panic!("Did not write all bytes");
                }
            }

            if let Err(e) = bytes.write(&mut self.stream) {
                if e.kind() == ::std::io::ErrorKind::WouldBlock {
                    continue;
                }
                panic!("Could not write bytes: {}",e);
            }
        }

        Ok(())
    }

    /// Send all messages until iter is empty or writing to socket would block.
    pub fn send_all_messages<'a, T: Iterator<Item=&'a TestRequest>,F>(&mut self,iter: &mut T,mut sending_message_func: F) -> Result<(),io::Error>
        where F: FnMut(&TestRequest) {
        loop {
            self.prepare_send_message(iter,&mut sending_message_func);
            if self.outbound_buffer.is_empty() {
                break;
            }

            match self.outbound_buffer.write(&mut self.stream) {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == ::std::io::ErrorKind::WouldBlock {
                        return Ok(());
                    }
                    return Err(e);
                },
            }
        }

        Ok(())
    }

    /// Send remaining output bytes or as much as possible of next message.
    pub fn send_next_message<'a, T: Iterator<Item=&'a TestRequest>,F>(&mut self,iter: &mut T,mut sending_message_func: F) -> Result<(),io::Error>
        where F: FnMut(&TestRequest) {
        self.prepare_send_message(iter,&mut sending_message_func);
        if self.outbound_buffer.is_empty() {
            return Ok(());
        }

        match self.outbound_buffer.write(&mut self.stream) {
            Ok(_) => {},
            Err(e) => {
                if e.kind() == ::std::io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                return Err(e);
            },
        }

        Ok(())
    }

    fn prepare_send_message<'a, T: Iterator<Item=&'a TestRequest>,F>(&mut self,iter: &mut T,mut sending_message_func: F)
        where F: FnMut(&TestRequest) {
        if self.outbound_buffer.is_empty() {
            if let Some(next_message) = iter.next() {
                next_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut self.outbound_buffer);
                sending_message_func(&next_message);
            }
        }
    }

    pub fn recv_fixt_message(&mut self) -> Result<Box<FIXTMessage + Send>,io::Error> {
        if !self.parser.messages.is_empty() {
            return Ok(self.parser.messages.remove(0));
        }

        let now = Instant::now();
        let timeout = Duration::from_secs(5);

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
                    println!("recv_fixt_message: Parse error");
                    println!("\t{}",result.err().unwrap());
                    panic!(""); //TODO: Use a separate error instead of panicing.
                }

                total_bytes_parsed += bytes_parsed;
            }

            if !self.parser.messages.is_empty() {
                return Ok(self.parser.messages.remove(0));
            }
        }

        panic!("recv_fixt_message: Timed out")
    }

    fn recv_message<T: FIXTMessage + Any + Clone>(&mut self) -> Result<T,io::Error> {
        let fixt_message = try!(self.recv_fixt_message());
        Ok(fixt_message.as_any().downcast_ref::<T>().expect("Not expected message type").clone())
    }

    pub fn recv_all_messages<F>(&mut self,mut received_message_func: F) -> Result<(),io::Error>
        where F: FnMut(&Box<FIXTMessage + Send>) {
        loop {
            match self.inbound_buffer.clear_and_read(&mut self.stream) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        return Ok(());
                    }

                    let (bytes_parsed,result) = self.parser.parse(self.inbound_buffer.bytes());
                    self.inbound_buffer.consume(bytes_parsed);
                    if let Err(e) = result {
                        //TODO: Bubble up error.
                        panic!("Could not parse message: {}",e);
                    }

                    for message in self.parser.messages.drain(..) {
                        received_message_func(&message);
                    }
                },
                Err(e) => {
                    if e.kind() == ::std::io::ErrorKind::WouldBlock {
                        return Ok(());
                    }
                    return Err(e);
                }
            }
        }
    }
}

//Use an iterator to generate messages that will be sent.
struct TestRequestIter<'a> {
    index: u64,
    count: u64,
    test_request_message: TestRequest,
    _marker: PhantomData<&'a TestRequest>,
}

impl<'a> TestRequestIter<'a> {
    fn with_start_msg_seq_num(start_msg_seq_num: u64) -> Self {
        let mut test_request_message = TestRequest::new();
        test_request_message.setup_fixt_session_header(
            Some(start_msg_seq_num - 1),
            b"fix-rs-lt".to_vec(),
            b"Server".to_vec());

        TestRequestIter {
            index: 0,
            count: 1000000,
            test_request_message: test_request_message,
            _marker: PhantomData,
        }
    }

    fn with_message_count(mut self,count: u64) -> Self {
        self.count = count;
        self
    }
}

impl<'a> Iterator for TestRequestIter<'a> {
    type Item = &'a TestRequest;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        self.test_request_message.msg_seq_num += 1;
        self.test_request_message.test_req_id = self.index.to_string().as_bytes().to_vec();
        self.index += 1;
        unsafe {
            //Yup, not safe. But this makes sure the message is never cloned or copied which is
            //super expensive.
            return Some(mem::transmute(&self.test_request_message as *const TestRequest));
        }
    }
}

fn test_request_load() -> Result<(),io::Error> {
    define_dictionary!(
        Logon,
        TestRequest,
        Heartbeat,
    );

    let mut latency_results = vec![LatencyResult { begin_send_time: Instant::now(), end_parse_time: Instant::now() };MESSAGE_COUNT as usize];
    let mut connection = try!(Connection::connect_and_logon(build_dictionary()));

    //Send TestRequest messages with a priority on sending over receiving. Then measure how long it
    //takes to get a response for each.
    let mut iter = TestRequestIter::with_start_msg_seq_num(2).with_message_count(MESSAGE_COUNT);
    let mut events = Events::with_capacity(8);
    let start_instant = Instant::now();
    let mut running = true;
    while running {
        if let Err(_) = connection.poll.poll(&mut events,None) {
            panic!("Poll failed");
        }

        for event in events.iter() {
            let readiness = event.readiness();
            if readiness.is_writable() {
                try!(connection.send_all_messages(&mut iter,|ref message| {
                    //TODO: Maybe check the test_req_id instead to be more general?
                    latency_results[message.msg_seq_num() as usize - 2].begin_send_time = Instant::now();
                }));
           }

            if readiness.is_readable() {
                try!(connection.recv_all_messages(|ref message| {
                    latency_results[message.msg_seq_num() as usize - 2].end_parse_time = Instant::now();

                    //Have all messages been received?
                    if message.msg_seq_num() >= MESSAGE_COUNT + 1 {
                        running = false;
                    }
                }));
            }

            let readiness = UnixReady::from(readiness);
            if readiness.is_hup() {
                panic!("Other side closed connection");
            }
        }
    }

    //Calculate and print statistics.
    let total_duration = start_instant.elapsed();
    print_statistics(total_duration,latency_results);

    return Ok(());
}

fn test_request_latency() -> Result<(),io::Error> {
    define_dictionary!(
        Logon,
        TestRequest,
        Heartbeat,
    );

    let mut latency_results = vec![LatencyResult { begin_send_time: Instant::now(), end_parse_time: Instant::now() };MESSAGE_COUNT as usize];
    let mut connection = try!(Connection::connect_and_logon(build_dictionary()));

    //Send TestRequest messages with a priority on receiving over sending. Then measure how long it
    //takes to get a response for each.
    let mut iter = TestRequestIter::with_start_msg_seq_num(2).with_message_count(MESSAGE_COUNT);
    let mut events = Events::with_capacity(8);
    let start_instant = Instant::now();
    let mut running = true;
    while running {
        if let Err(_) = connection.poll.poll(&mut events,None) {
            panic!("Poll failed");
        }

        for event in events.iter() {
            let readiness = event.readiness();
            if readiness.is_readable() {
                try!(connection.recv_all_messages(|ref message| {
                    latency_results[message.msg_seq_num() as usize - 2].end_parse_time = Instant::now();

                    //Have all messages been received?
                    if message.msg_seq_num() >= MESSAGE_COUNT + 1 {
                        running = false;
                    }
                }));
            }

            let readiness = UnixReady::from(readiness);
            if readiness.is_hup() {
                panic!("Other side closed connection");
            }

            try!(connection.send_next_message(&mut iter,|ref message| {
                //TODO: Maybe check the test_req_id instead to be more general?
                latency_results[message.msg_seq_num() as usize - 2].begin_send_time = Instant::now();
            }));
        }
    }

    //Calculate and print statistics.
    let total_duration = start_instant.elapsed();
    print_statistics(total_duration,latency_results);

    return Ok(());
}

fn main() {
    let matches = App::new("fix-rs-lt")
                       .version(env!("CARGO_PKG_VERSION"))
                       .author(env!("CARGO_PKG_AUTHORS"))
                       .about("Load/Latency testing tool for fix-rs")
                       .arg(Arg::with_name("type")
                                 .required(true)
                                 .index(1)
                                 .takes_value(true)
                                 .possible_values(&["test_request_load","test_request_latency"]))
                       .get_matches();

    //TODO: Make message count adjustable.
    //TODO: Make connection count adjustable.
    //TODO: Make thread count adjustable.
    //TODO: Make server address adjustable.
    //TODO: Make sender_comp_id and target_comp_id adjustable.

    let result = match matches.value_of("type").unwrap() {
        "test_request_load" => test_request_load(),
        "test_request_latency" => test_request_latency(),
        _ => panic!("Not a supported type"),
    };

    //TODO: Make use of result here.
}

