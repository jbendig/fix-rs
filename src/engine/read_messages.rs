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

use futures::{Async, Poll, Stream};
use std::cell::RefCell;
use std::io;
use std::ops::DerefMut;
use std::rc::Rc;
use tokio_core::net::TcpStream;

use byte_buffer::ByteBuffer;
use engine::constants::INBOUND_MESSAGES_BUFFER_LEN_MAX;
use fix::{Parser, ParseError};
use fixt::message::FIXTMessage;

pub enum ConnectionReadMessage {
    Message(Box<FIXTMessage + Send>),
    Error(ParseError),
}

pub struct ReadMessages {
    pub socket: Rc<RefCell<TcpStream>>,
    pub inbound_buffer: ByteBuffer,
    pub parser: Parser,
}

impl Stream for ReadMessages {
    type Item = Vec<ConnectionReadMessage>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>,Self::Error> {
        fn parse_bytes(read_messages: &mut ReadMessages,messages: &mut Vec<ConnectionReadMessage>) -> bool {
            while !read_messages.inbound_buffer.is_empty() {
                let (bytes_parsed,result) = read_messages.parser.parse(read_messages.inbound_buffer.bytes());

                assert!(bytes_parsed > 0);
                read_messages.inbound_buffer.consume(bytes_parsed);

                //Retain order by extracting messages and then the error from parser.
                for message in read_messages.parser.messages.drain(..) {
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
            }

            true
        }

        let mut messages = Vec::new();
        let mut keep_reading = parse_bytes(self,&mut messages);

        while keep_reading {
            let result = self.inbound_buffer.clear_and_read(self.socket.borrow_mut().deref_mut());
            match result {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        //Socket closed. If no more messages have been parsed, return None to
                        //indicate the end of the stream.
                        if messages.is_empty() {
                            return Ok(Async::Ready(None));
                        }

                        //Otherwise, still need to return the remaining messages first.
                        break;
                    }

                    //Parse all of the read bytes.
                    keep_reading = parse_bytes(self,&mut messages);
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

        if messages.is_empty() {
            Ok(Async::NotReady)
        }
        else {
            Ok(Async::Ready(Some(messages)))
        }
    }
}
