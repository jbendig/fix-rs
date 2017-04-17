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

use futures::{Async, AsyncSink, Poll, StartSend};
use futures::sink::Sink;
use tokio_core::net::TcpStream;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::ops::DerefMut;
use std::rc::Rc;

use byte_buffer::ByteBuffer;
use fix_version::FIXVersion;
use fixt::message::FIXTMessage;
use message_version::MessageVersion;

struct OutboundMessage {
    message: Box<FIXTMessage + Send>,
    message_version: Option<MessageVersion>,
    auto_msg_seq_num: bool,
}

pub struct WriteMessages {
    socket: Rc<RefCell<TcpStream>>,
    outbound_buffer: ByteBuffer,
    outbound_messages: VecDeque<OutboundMessage>,
    outbound_msg_seq_num: u64,
    fix_version: FIXVersion,
    default_message_version: MessageVersion,
    sender_comp_id: Vec<u8>,
    target_comp_id: Vec<u8>,
}

impl WriteMessages {
    pub fn new(socket: Rc<RefCell<TcpStream>>) -> Self {
        let fix_version = FIXVersion::FIXT_1_1;
        WriteMessages {
            socket: socket,
            outbound_buffer: ByteBuffer::with_capacity(1024), //TODO: Do not make this a magic number.
            outbound_messages: VecDeque::new(),
            outbound_msg_seq_num: 1,
            fix_version: fix_version,
            default_message_version: fix_version.max_message_version(),
            sender_comp_id: b"TX".to_vec(), //TODO: Need to get this info from Listener when it's setup.
            target_comp_id: b"TEST".to_vec(),
        }
    }

    fn increment_outbound_msg_seq_num(&mut self) -> Result<(),io::Error> {
        //Check for overflow before incrementing. Just force the connection to terminate if this
        //occurs. This number is so large that the only way it can be reached is if the other party
        //issues SequenceReset-Reset with a crazy high NewSeqNo. NewSeqNo values higher than
        //u64::max_value() are outright rejected as parsing errors.
        if self.outbound_msg_seq_num == u64::max_value() {
            return Err(io::Error::new(io::ErrorKind::Other,"Outbound MsgSeqNum overflowed"));
        }

        self.outbound_msg_seq_num += 1;
        Ok(())
    }
}

impl Sink for WriteMessages {
    type SinkItem = Box<FIXTMessage + Send>;
    type SinkError = io::Error; //TODO: Should not be just an io::Error because other things might happen like outbound_msg_seq_num overflowed.

    fn start_send(&mut self,item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        //TODO: May have this function take OutboundMessage directly or at least have a better interface.
        let outbound_message = OutboundMessage {
            message: item,
            message_version: None,
            auto_msg_seq_num: true,
        };
        self.outbound_messages.push_back(outbound_message);

        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        //Send data until no more messages are available or until the socket returns WouldBlock.
        loop { //TODO: This loop might make this function too greedy. Maybe not?
            //Fill an outbound buffer by serializing each message in a FIFO order. Once this buffer
            //is drained, the process repeats itself.
            if self.outbound_buffer.is_empty() {
                if self.outbound_messages.is_empty() {
                    //Nothing left to write.

                    /* TODO: Handle clean logout processing. The following code was copy & pasted
                     * from fixt/engine_thread.rs as a reference.
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
                    }*/
                    return Ok(Async::Ready(()));
                }

                //Setup message to go out and serialize it.
                let mut message = self.outbound_messages.pop_front().unwrap();
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
                self.outbound_buffer.clear_and_read_all(|ref mut bytes| { message.message.read(fix_version,message_version,bytes); });

                //TODO: Hold onto message and pass it off to the engine or some callback so the
                //library user knows exactly which messages have been sent -- although not
                //necessarily acknowledged.
            }

            //Send data. Simple.
            match self.outbound_buffer.write(self.socket.borrow_mut().deref_mut()) {
                Ok(_) => {},
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => {
                            return Ok(Async::NotReady);
                        },
                        _ => return Err(e),
                    };
                }
            }
        }
    }
}
