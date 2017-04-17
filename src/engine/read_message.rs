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

use futures::{Async, Future, Poll};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::mem;
use std::rc::Rc;
use tokio_core::net::TcpStream;

use byte_buffer::ByteBuffer;
use dictionary::administrative_msg_types;
use fix::Parser;
use engine::constants::INBOUND_BYTES_BUFFER_CAPACITY;
use engine::read_messages::ReadMessages;
pub use engine::read_messages::ConnectionReadMessage;
use engine::write_messages::WriteMessages;
use fixt::message::BuildFIXTMessage;
use fix_version::FIXVersion;

struct Internal {
    socket: TcpStream,
    inbound_buffer: ByteBuffer,
    parser: Parser,
}

pub struct ReadMessage {
    internal: Option<Internal>,
}

impl ReadMessage {
    pub fn new(socket: TcpStream,
               message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
               max_message_size: u64) -> ReadMessage {
        let fix_version = FIXVersion::FIXT_1_1; //TODO: Don't hard code this.

        //Force all administrative messages to use the newest message version for the
        //specified FIX version. This way they can't be overridden during Logon and it
        //makes sure the Logon message supports all of the fields we support.
        let mut parser = Parser::new(message_dictionary,max_message_size);
        for msg_type in administrative_msg_types() {
            parser.set_default_message_type_version(msg_type,fix_version.max_message_version());
        }

        ReadMessage {
            internal: Some(Internal {
                socket: socket,
                inbound_buffer: ByteBuffer::with_capacity(INBOUND_BYTES_BUFFER_CAPACITY),
                parser: parser,
            }),
        }
    }
}

impl Future for ReadMessage {
    type Item = (ReadMessages,WriteMessages,ConnectionReadMessage);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item,Self::Error> {
        fn parse_bytes(internal: &mut Internal,messages: &mut Vec<ConnectionReadMessage>) -> bool {
            while !internal.inbound_buffer.is_empty() {
                let (bytes_parsed,result) = internal.parser.parse(internal.inbound_buffer.bytes());

                assert!(bytes_parsed > 0);
                internal.inbound_buffer.consume(bytes_parsed);

                //Retain order by extracting messages and then the error from parser.
                for message in internal.parser.messages.drain(..) {
                    messages.push(ConnectionReadMessage::Message(message));
                }
                if let Err(e) = result {
                    messages.push(ConnectionReadMessage::Error(e));
                }

                if !messages.is_empty() {
                    return false;
                }
            }

            true
        }

        let mut messages = Vec::new();
        {
            if self.internal.is_none() {
                panic!("Future called twice!");
            }
            let mut internal = self.internal.as_mut().unwrap();

            if !internal.socket.poll_read().is_ready() {
                return Ok(Async::NotReady);
            }

            let mut keep_reading = parse_bytes(&mut internal,&mut messages);

            while keep_reading {
                match internal.inbound_buffer.clear_and_read(&mut internal.socket) {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            //Socket closed.
                            break;
                        }

                        //Parse all of the read bytes.
                        keep_reading = parse_bytes(&mut internal,&mut messages);
                    }
                    Err(e) => {
                        if let io::ErrorKind::WouldBlock = e.kind() {
                            //Socket exhausted.
                            break;
                        }

                        return Err(e);
                    }
                };
            }
        }

        if messages.is_empty() {
            Ok(Async::NotReady)
        }
        else {
            let internal = mem::replace(&mut self.internal,None).unwrap();
            let socket = Rc::new(RefCell::new(internal.socket));
            let read_messages = ReadMessages {
                socket: socket.clone(),
                inbound_buffer: internal.inbound_buffer,
                parser: internal.parser,
            };
            Ok(Async::Ready((read_messages,
                             WriteMessages::new(socket),
                             messages.into_iter().next().unwrap()
            )))
        }
    }
}

