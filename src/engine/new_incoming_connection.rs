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

use futures::{future, Future, Sink, Stream};
use futures::unsync::mpsc::unbounded;
use mio::Token;
use std::io;
use std::time::Duration;

use dictionary::fields::{SenderCompID,TargetCompID};
use engine::connection::{Connection, MsgSeqNumType};
use engine::read_messages::ReadMessages;
use engine::write_messages::WriteMessages;
use field::Field;
use field_type::FieldType;
use fix_version::FIXVersion;
use message_version::MessageVersion;

pub struct NewIncomingConnection {
    pub token: Token,
    pub fix_version: FIXVersion,
    pub default_message_version: MessageVersion,
    pub outbound_messages: WriteMessages,
    pub outbound_heartbeat_timeout_duration: Option<Duration>,
    pub inbound_messages: ReadMessages,
    pub inbound_msg_seq_num: MsgSeqNumType,
    pub inbound_testrequest_timeout_duration: Option<Duration>,
    pub sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
    pub target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type,
}

impl NewIncomingConnection {
    pub fn to_connection(self) -> (Connection,ReadMessages,impl Future<Item=(),Error=io::Error>) {
        let (outbound_messages_sender,outbound_messages_receiver) = unbounded();

        let connection = Connection {
            token: self.token,
            fix_version: self.fix_version,
            default_message_version: self.default_message_version,
            outbound_messages: outbound_messages_sender,
            outbound_heartbeat_timeout_duration: self.outbound_heartbeat_timeout_duration,
            inbound_msg_seq_num: self.inbound_msg_seq_num,
            inbound_testrequest_timeout_duration: self.inbound_testrequest_timeout_duration,
            sender_comp_id: self.sender_comp_id,
            target_comp_id: self.target_comp_id,
        };

        let outbound_messages_future = self.outbound_messages.send_all(outbound_messages_receiver.map_err(|_| io::Error::new(io::ErrorKind::Other,"")))
                                                             .and_then(|_| future::ok(()));
        (connection,self.inbound_messages,outbound_messages_future)
    }
}
