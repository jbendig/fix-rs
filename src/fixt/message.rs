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

use std::fmt;

use dictionary::fields::{MsgSeqNum,OrigSendingTime,SenderCompID,SendingTime,TargetCompID};
use field::Field;
use field_type::FieldType;
use message::Message;

pub trait FIXTMessage: Message {
    fn new_into_box(&self) -> Box<FIXTMessage + Send>;
    fn msg_type(&self) -> &'static [u8];
    fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type;
    fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type;
    fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type;
    fn is_poss_dup(&self) -> bool;
    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type;
    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type;
    fn setup_fixt_session_header(&mut self,
                                 msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                 sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                 target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type);
}

impl fmt::Debug for FIXTMessage + Send {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buffer = Vec::new();
        self.read(&mut buffer);
        let buffer: Vec<u8> = buffer.into_iter().map(|c| if c == b'\x01' { b'|' } else { c } ).collect();
        write!(f,"{:?}",String::from_utf8_lossy(&buffer[..]))
    }
}

#[macro_export]
macro_rules! define_fixt_message {
    ( $message_name:ident $( : $message_type:expr => )* { $( $field_required:expr, $field_name:ident : $field_type:ty $(=> EXCEPT_WHEN $message_ident:ident, $required_when_expr:expr)* ),* $(),* } ) => {
        define_message!($message_name $( : $message_type => )* {
            //Standard Header
            //Note: BeginStr, BodyLength, and MsgType are built into parser.
            $crate::message::REQUIRED, sender_comp_id: $crate::dictionary::fields::SenderCompID, //Must be first here to be 4th field when serialized.
            $crate::message::REQUIRED, target_comp_id: $crate::dictionary::fields::TargetCompID, //Must be second here to be 5th field when serialized.
            //TODO: This and the following three tags should not be ever used with Logon, Logout, Reject, ResendRequest, SequenceReset, TestRequest, and Heartbeat.
            $crate::message::NOT_REQUIRED, appl_ver_id: $crate::dictionary::fields::ApplVerID,  //Must be third here to be 6th field when serialized.
            $crate::message::NOT_REQUIRED, appl_ext_id: $crate::dictionary::fields::ApplExtID,
            $crate::message::NOT_REQUIRED, cstm_appl_ver_id: $crate::dictionary::fields::CstmApplVerID,
            $crate::message::NOT_REQUIRED, on_behalf_of_comp_id: $crate::dictionary::fields::OnBehalfOfCompID,
            $crate::message::NOT_REQUIRED, deliver_to_comp_id: $crate::dictionary::fields::DeliverToCompID,
            $crate::message::NOT_REQUIRED, secure_data_len: $crate::dictionary::fields::SecureDataLen,
            $crate::message::NOT_REQUIRED, secure_data: $crate::dictionary::fields::SecureData,
            $crate::message::REQUIRED, msg_seq_num: $crate::dictionary::fields::MsgSeqNum,
            $crate::message::NOT_REQUIRED, sender_sub_id: $crate::dictionary::fields::SenderSubID,
            $crate::message::NOT_REQUIRED, sender_location_id: $crate::dictionary::fields::SenderLocationID,
            $crate::message::NOT_REQUIRED, target_sub_id: $crate::dictionary::fields::TargetSubID,
            $crate::message::NOT_REQUIRED, target_location_id: $crate::dictionary::fields::TargetLocationID,
            $crate::message::NOT_REQUIRED, on_behalf_of_sub_id: $crate::dictionary::fields::OnBehalfOfSubID,
            $crate::message::NOT_REQUIRED, on_behalf_of_location_id: $crate::dictionary::fields::OnBehalfOfLocationID,
            $crate::message::NOT_REQUIRED, deliver_to_sub_id: $crate::dictionary::fields::DeliverToSubID,
            $crate::message::NOT_REQUIRED, deliver_to_location_id: $crate::dictionary::fields::DeliverToLocationID,
            $crate::message::NOT_REQUIRED, poss_dup_flag: $crate::dictionary::fields::PossDupFlag,
            $crate::message::NOT_REQUIRED, poss_resend: $crate::dictionary::fields::PossResend,
            $crate::message::REQUIRED, sending_time: $crate::dictionary::fields::SendingTime,
            $crate::message::NOT_REQUIRED, orig_sending_time: $crate::dictionary::fields::OrigSendingTime => EXCEPT_WHEN message, message.poss_dup_flag,
            $crate::message::NOT_REQUIRED, xml_data_len: $crate::dictionary::fields::XmlDataLen,
            $crate::message::NOT_REQUIRED, xml_data: $crate::dictionary::fields::XmlData,
            $crate::message::NOT_REQUIRED, message_encoding: $crate::dictionary::fields::MessageEncoding,
            $crate::message::NOT_REQUIRED, last_msg_seq_num_processed: $crate::dictionary::fields::LastMsgSeqNumProcessed,
            $crate::message::NOT_REQUIRED, hops: $crate::dictionary::fields::NoHops,

            //Other
            $( $field_required, $field_name : $field_type $(=> EXCEPT_WHEN $message_ident, $required_when_expr)*, )*

            //Standard Footer
            //Note: Checksum is built into parser.
            $crate::message::NOT_REQUIRED, signature_length: $crate::dictionary::fields::SignatureLength,
            $crate::message::NOT_REQUIRED, signature: $crate::dictionary::fields::Signature,
        });

        impl $crate::fixt::message::FIXTMessage for $message_name {
            fn new_into_box(&self) -> Box<$crate::fixt::message::FIXTMessage + Send> {
                Box::new($message_name::new())
            }

            #[allow(unreachable_code)]
            fn msg_type(&self) -> &'static [u8] {
                $( return $message_type )*; //Only one message type can be specified.

                b""
            }

            fn msg_seq_num(&self) -> <<$crate::dictionary::fields::MsgSeqNum as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                self.msg_seq_num
            }

            fn sender_comp_id(&self) -> &<<$crate::dictionary::fields::SenderCompID as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                &self.sender_comp_id
            }

            fn target_comp_id(&self) -> &<<$crate::dictionary::fields::TargetCompID as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                &self.target_comp_id
            }

            fn is_poss_dup(&self) -> bool {
                self.poss_dup_flag
            }

            fn sending_time(&self) -> <<$crate::dictionary::fields::SendingTime as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                self.sending_time
            }

            fn orig_sending_time(&self) -> <<$crate::dictionary::fields::OrigSendingTime as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                self.orig_sending_time
            }

            fn setup_fixt_session_header(&mut self,
                                         msg_seq_num: Option<<<$crate::dictionary::fields::MsgSeqNum as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type>,
                                         sender_comp_id: <<$crate::dictionary::fields::SenderCompID as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type,
                                         target_comp_id: <<$crate::dictionary::fields::TargetCompID as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type) {
                if let Some(msg_seq_num) = msg_seq_num {
                    self.msg_seq_num = msg_seq_num;
                }
                self.sender_comp_id = sender_comp_id;
                self.target_comp_id = target_comp_id;
                self.sending_time = <$crate::dictionary::fields::SendingTime as $crate::field::Field>::Type::new_now();
            }
        }
    };
}

