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
use fix_version::FIXVersion;
use message::{BuildMessage,Message};
use message_version::MessageVersion;

pub trait BuildFIXTMessage: BuildMessage {
    fn new_into_box(&self) -> Box<BuildFIXTMessage + Send>;
    fn build(&self) -> Box<FIXTMessage + Send>;
}

pub trait FIXTMessageBuildable {
    fn builder(&self) -> Box<BuildFIXTMessage + Send>;
}

pub trait FIXTMessage: Message {
    fn new_into_box(&self) -> Box<FIXTMessage + Send>;
    fn msg_type(&self) -> &'static [u8];
    fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type;
    fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type;
    fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type;
    fn is_poss_dup(&self) -> bool;
    fn set_is_poss_dup(&mut self,is_poss_dup: bool);
    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type;
    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type;
    fn set_orig_sending_time(&mut self,orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type);
    fn setup_fixt_session_header(&mut self,
                                 msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                 sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                 target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type);
}

impl fmt::Debug for FIXTMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",Message::debug(self,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2))
    }
}

impl fmt::Debug for FIXTMessage + Send {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",Message::debug(self,FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2))
    }
}

#[macro_export]
macro_rules! define_fixt_message {
    ( $message_name:ident $( : ADMIN $message_type:expr => )* { $( $field_required:expr, $field_name:ident : $field_type:ty [$( $version:tt )*] $(=> REQUIRED_WHEN $required_when_expr:expr)* ),* $(),* } ) => {
        define_fixt_message!($message_name $( : $message_type => )* {
            //No extra header fields required for admin messages.
        } { $( $field_required, $field_name : $field_type [$( $version )*] $(=> REQUIRED_WHEN $required_when_expr)*, )* } );
    };
    ( $message_name:ident $( : $message_type:expr => )* { $( $field_required:expr, $field_name:ident : $field_type:ty [$( $version:tt )*] $(=> REQUIRED_WHEN $required_when_expr:expr)* ),* $(),* } ) => {
        define_fixt_message!($message_name $( : $message_type => )* {
            //These extra fields are used for all non-Session Level Messages starting with FIXT
            //1.1. See FIXT 1.1 page 33.
            $crate::message::NOT_REQUIRED, appl_ver_id: $crate::dictionary::fields::ApplVerID [FIX40..],  //Must be first here to be 6th field when serialized. Note: This field uses Rule::RequiresFIXVersion(FIXVersion::FIXT_1_1) to be excluded at the FIX level instead of the message level. This way it's processed correctly when using versioned messages. So leave message version as [FIX40..].
            $crate::message::NOT_REQUIRED, appl_ext_id: $crate::dictionary::fields::ApplExtID [FIX50SP1..],
            $crate::message::NOT_REQUIRED, cstm_appl_ver_id: $crate::dictionary::fields::CstmApplVerID [FIX50..]
        } { $( $field_required, $field_name : $field_type [$( $version )*] $(=> REQUIRED_WHEN $required_when_expr)*, )* } );
    };
    ( $message_name:ident $( : $message_type:expr => )* { $( $header_field_required:expr, $header_field_name:ident : $header_field_type:ty [$( $header_version:tt )*] $(=> REQUIRED_WHEN $header_required_when_expr:expr)* ),* } { $( $field_required:expr, $field_name:ident : $field_type:ty [$( $version:tt )*] $(=> REQUIRED_WHEN $required_when_expr:expr)* ),* $(),* } ) => {
        define_message!($message_name $( : $message_type => )* {
            //Standard Header
            //Note: BeginStr, BodyLength, and MsgType are built into parser.
            $crate::message::REQUIRED, sender_comp_id: $crate::dictionary::fields::SenderCompID [FIX40..], //Must be first here to be 4th field when serialized.
            $crate::message::REQUIRED, target_comp_id: $crate::dictionary::fields::TargetCompID [FIX40..], //Must be second here to be 5th field when serialized.
            $( $header_field_required, $header_field_name : $header_field_type [$( $header_version )*] $(=> REQUIRED_WHEN $header_required_when_expr)*, )*
            $crate::message::NOT_REQUIRED, on_behalf_of_comp_id: $crate::dictionary::fields::OnBehalfOfCompID [FIX40..],
            $crate::message::NOT_REQUIRED, deliver_to_comp_id: $crate::dictionary::fields::DeliverToCompID [FIX40..],
            $crate::message::NOT_REQUIRED, secure_data_len: $crate::dictionary::fields::SecureDataLen [FIX40..],
            $crate::message::NOT_REQUIRED, secure_data: $crate::dictionary::fields::SecureData [FIX40..],
            $crate::message::REQUIRED, msg_seq_num: $crate::dictionary::fields::MsgSeqNum [FIX40..],
            $crate::message::NOT_REQUIRED, sender_sub_id: $crate::dictionary::fields::SenderSubID [FIX40..],
            $crate::message::NOT_REQUIRED, sender_location_id: $crate::dictionary::fields::SenderLocationID [FIX41..],
            $crate::message::NOT_REQUIRED, target_sub_id: $crate::dictionary::fields::TargetSubID [FIX40..],
            $crate::message::NOT_REQUIRED, target_location_id: $crate::dictionary::fields::TargetLocationID [FIX41..],
            $crate::message::NOT_REQUIRED, on_behalf_of_sub_id: $crate::dictionary::fields::OnBehalfOfSubID [FIX40..],
            $crate::message::NOT_REQUIRED, on_behalf_of_location_id: $crate::dictionary::fields::OnBehalfOfLocationID [FIX41..],
            $crate::message::NOT_REQUIRED, deliver_to_sub_id: $crate::dictionary::fields::DeliverToSubID [FIX40..],
            $crate::message::NOT_REQUIRED, deliver_to_location_id: $crate::dictionary::fields::DeliverToLocationID [FIX41..],
            $crate::message::NOT_REQUIRED, poss_dup_flag: $crate::dictionary::fields::PossDupFlag [FIX40..],
            $crate::message::NOT_REQUIRED, poss_resend: $crate::dictionary::fields::PossResend [FIX40..],
            $crate::message::REQUIRED, sending_time: $crate::dictionary::fields::SendingTime [FIX40..],
            $crate::message::NOT_REQUIRED, orig_sending_time: $crate::dictionary::fields::OrigSendingTime [FIX40..] => REQUIRED_WHEN |message: &$message_name,_| { message.poss_dup_flag },
            $crate::message::NOT_REQUIRED, xml_data_len: $crate::dictionary::fields::XmlDataLen [FIX42..],
            $crate::message::NOT_REQUIRED, xml_data: $crate::dictionary::fields::XmlData [FIX42..],
            $crate::message::NOT_REQUIRED, message_encoding: $crate::dictionary::fields::MessageEncoding [FIX42..],
            $crate::message::NOT_REQUIRED, last_msg_seq_num_processed: $crate::dictionary::fields::LastMsgSeqNumProcessed [FIX42..],
            $crate::message::NOT_REQUIRED, on_behalf_of_sending_time: $crate::dictionary::fields::OnBehalfOfSendingTime [FIX42..FIX43],
            $crate::message::NOT_REQUIRED, hops: $crate::dictionary::fields::NoHops [FIX43..],

            //Other
            $( $field_required, $field_name : $field_type [$( $version )*] $(=> REQUIRED_WHEN $required_when_expr)*, )*

            //Standard Footer
            //Note: Checksum is built into parser.
            $crate::message::NOT_REQUIRED, signature_length: $crate::dictionary::fields::SignatureLength [FIX40..],
            $crate::message::NOT_REQUIRED, signature: $crate::dictionary::fields::Signature [FIX40..],
        });

        impl $message_name {
            #[allow(unreachable_code)]
            pub fn msg_type() -> &'static [u8] {
                $( return $message_type )*; //Only one message type can be specified.

                b""
            }
        }

        impl $crate::fixt::message::FIXTMessage for $message_name {
            fn new_into_box(&self) -> Box<$crate::fixt::message::FIXTMessage + Send> {
                Box::new($message_name::new())
            }

            fn msg_type(&self) -> &'static [u8] {
                $message_name::msg_type()
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

            fn set_is_poss_dup(&mut self,is_poss_dup: bool) {
                self.poss_dup_flag = is_poss_dup;
            }

            fn sending_time(&self) -> <<$crate::dictionary::fields::SendingTime as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                self.sending_time
            }

            fn orig_sending_time(&self) -> <<$crate::dictionary::fields::OrigSendingTime as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type {
                self.orig_sending_time
            }

            fn set_orig_sending_time(&mut self,orig_sending_time: <<$crate::dictionary::fields::OrigSendingTime as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type) {
                self.orig_sending_time = orig_sending_time;
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

