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

use futures::unsync::mpsc::UnboundedSender;
use mio::Token;
use std::time::Duration;

use dictionary::fields::{MsgSeqNum, SenderCompID, TargetCompID};
use field::Field;
use field_type::FieldType;
use fix_version::FIXVersion;
use fixt::message::FIXTMessage;
use message_version::MessageVersion;

pub type MsgSeqNumType = <<MsgSeqNum as Field>::Type as FieldType>::Type;

pub struct Connection {
    pub token: Token,
    pub fix_version: FIXVersion,
    pub default_message_version: MessageVersion,
    pub outbound_messages: UnboundedSender<Box<FIXTMessage + Send>>,
    pub outbound_heartbeat_timeout_duration: Option<Duration>,
    pub inbound_msg_seq_num: MsgSeqNumType,
    pub inbound_testrequest_timeout_duration: Option<Duration>,
    pub sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
    pub target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type,
}

