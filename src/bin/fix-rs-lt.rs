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

extern crate clap;
#[macro_use]
extern crate fix_rs;
extern crate mio;

use clap::{App,Arg};
use mio::{Events,Poll,PollOpt,Ready,Token};
use mio::tcp::TcpStream;
use std::any::Any;
use std::net::{Ipv4Addr,SocketAddr,SocketAddrV4};
use std::io::{self,Read,Write};
use std::marker::PhantomData;
use std::mem;
use std::thread;
use std::time::{Duration,Instant};

use fix_rs::byte_buffer::ByteBuffer;
use fix_rs::dictionary::field_types::other::EncryptMethod;
use fix_rs::dictionary::messages::{Logon,Heartbeat,TestRequest};
use fix_rs::fix::Parser;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt::message::FIXTMessage;
use fix_rs::message::Message;
use fix_rs::message_version::MessageVersion;

const SEND_MESSAGE_TIMEOUT_SECS: u64 = 10;
const MAX_MESSAGE_SIZE: u64 = 4096;
const MESSAGE_COUNT: u64 = 1000;

#[derive(Clone)]
struct LatencyResult {
    begin_send_time: Instant,
    end_parse_time: Instant,
}

struct Connection {
    poll: Poll,
    stream: TcpStream,
    parser: Parser,
    outbound_buffer: ByteBuffer,
    inbound_buffer: ByteBuffer,
}

impl Connection {
    fn send_message<T: FIXTMessage + Any + Send>(&mut self,message: T) -> Result<(),io::Error> {
        let mut bytes = Vec::new();
        message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut bytes);

        let now = Instant::now();
        let timeout = Some(Duration::from_secs(SEND_MESSAGE_TIMEOUT_SECS));
        let mut bytes_written_total = 0;
        while bytes_written_total < bytes.len() {
            if let Some(timeout) = timeout {
                if now.elapsed() > timeout {
                    panic!("Did not write all bytes");
                }
            }

            match self.stream.write(&bytes[bytes_written_total..bytes.len()]) {
                Ok(bytes_written) => bytes_written_total += bytes_written,
                Err(e) => {
                    if e.kind() == ::std::io::ErrorKind::WouldBlock {
                        continue;
                    }
                    panic!("Could not write bytes: {}",e);
                },
            }
        }

        Ok(())
    }

    fn send_all_messages<'a, T: Iterator<Item=&'a TestRequest>,F>(&mut self,iter: &mut T,mut sending_message_func: F) -> Result<(),io::Error> 
        where F: FnMut(&TestRequest) {
        loop {
            if self.outbound_buffer.is_empty() {
                if let Some(next_message) = iter.next() {
                    self.outbound_buffer.clear_and_read_all(|ref mut bytes| {
                        next_message.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,bytes);
                    });
                    sending_message_func(&next_message);
                }
                else {
                    return Ok(());
                }
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
    }

    fn recv_fixt_message(&mut self) -> Result<Box<FIXTMessage + Send>,io::Error> {
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

    fn recv_all_messages<F>(&mut self,mut received_message_func: F) -> Result<(),io::Error>
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

fn test_request() -> Result<(),io::Error> {
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

    //TODO: Use custom messages here with unused fields  as noops when calling set_value.
    define_dictionary!(
        Logon,
        TestRequest,
        Heartbeat,
    );

    //Connect to server.
    let mut latency_results = vec![LatencyResult { begin_send_time: Instant::now(), end_parse_time: Instant::now() };MESSAGE_COUNT as usize];
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1),7001));
    let mut connection = Connection {
        poll: Poll::new().unwrap(),
        stream: TcpStream::connect(&addr).unwrap(),
        parser: Parser::new(build_dictionary(),MAX_MESSAGE_SIZE),
        outbound_buffer: ByteBuffer::with_capacity(1024),
        inbound_buffer: ByteBuffer::with_capacity(16384),
    };

    connection.poll.register(&connection.stream,Token(0),Ready::all(),PollOpt::edge()).unwrap();

    //Logon.
    let mut logon_message = Logon::new();
    logon_message.setup_fixt_session_header(
        Some(1),
        b"fix-rs-lt".to_vec(),
        b"Server".to_vec());
    logon_message.encrypt_method = EncryptMethod::None;
    logon_message.heart_bt_int = 60;
    logon_message.default_appl_ver_id = MessageVersion::FIX50SP2;
    logon_message.username = b"some_user".to_vec();
    logon_message.password = b"some_password".to_vec();
    try!(connection.send_message(logon_message));
    try!(connection.recv_message::<Logon>().map(|_|()));

    //Send TestRequest messages and measure how long it takes to get a response for each.
    let mut iter = TestRequestIter::with_start_msg_seq_num(2).with_message_count(MESSAGE_COUNT);
    let mut events = Events::with_capacity(8);
    let start_instant = Instant::now();
    let mut running = true;
    while running {
        if let Err(_) = connection.poll.poll(&mut events,None) {
            panic!("Poll failed");
        }

        for event in events.iter() {
            if event.kind().is_writable() {
                try!(connection.send_all_messages(&mut iter,|ref message| {
                    //TODO: Maybe check the test_req_id instead to be more general?
                    latency_results[message.msg_seq_num() as usize - 2].begin_send_time = Instant::now();
                }));
            }

            if event.kind().is_readable() {
                try!(connection.recv_all_messages(|ref message| {
                    latency_results[message.msg_seq_num() as usize - 2].end_parse_time = Instant::now();

                    //Have all messages been received?
                    if message.msg_seq_num() >= MESSAGE_COUNT + 1 {
                        running = false;
                    }
                }));
            }

            if event.kind().is_hup() {
                panic!("Other side closed connection");
            }
        }
    }
    let total_duration = start_instant.elapsed();

    //Calculate and print statistics.
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

    return Ok(());
}

fn main() {
    let matches = App::new("fix-rs-lt")
                       .version("0.1.0")
                       .author("James Bendig")
                       .about("Load/Latency testing tool for fix-rs")
                       .arg(Arg::with_name("type")
                                 .required(true)
                                 .index(1)
                                 .takes_value(true)
                                 .possible_values(&["test_request",]))
                       .get_matches();

    //TODO: Make message count adjustable.
    //TODO: Make connection count adjustable.
    //TODO: Make thread count adjustable.
    //TODO: Make server address adjustable.
    //TODO: Make sender_comp_id and target_comp_id adjustable.

    let result = match matches.value_of("type").unwrap() {
        "test_request" => test_request(),
        _ => panic!("Not a supported type"),
    };

    //TODO: Make use of result here.
}

