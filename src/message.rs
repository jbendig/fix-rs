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

use std::any::Any;
use std::collections::{HashMap,HashSet};
use std::io::Write;

use rule::Rule;

#[derive(Clone,Default,PartialEq)]
pub struct Meta {
    pub protocol: Vec<u8>,
    pub body_length: u64,
    pub checksum: u8,
}

pub trait MessageDetails {
    fn msg_type() -> &'static [u8];
}

pub enum SetValueError {
    WrongFormat,
    OutOfRange,
}

pub trait Message {
    fn first_field(&self) -> &'static [u8];
    fn field_count(&self) -> usize;
    fn fields(&self) -> HashMap<&'static [u8],Rule>;
    fn required_fields(&self) -> HashSet<&'static [u8]>;
    fn conditional_required_fields(&self) -> Vec<&'static [u8]>;
    fn set_meta(&mut self,meta: Meta);
    fn set_value(&mut self,key: &[u8],value: &[u8]) -> Result<(),SetValueError>;
    fn set_groups(&mut self,key: &[u8],groups: &[Box<Message>]) -> bool;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
    fn new_into_box(&self) -> Box<Message + Send>; //TODO: Investigate having a builder type afterall....
    fn msg_type_header(&self) -> Vec<u8>;
    fn read_body(&self,buf: &mut Vec<u8>) -> usize;
    fn read(&self,buf: &mut Vec<u8>) -> usize {
        //TODO: Try and avoid reallocations by providing a start offset and then inserting
        //the header in a reserved space at the beginning.

        //Read entire body first so we can get the body length.
        let mut body = Vec::new();
        self.read_body(&mut body);

        //Prepare header.
        let protocol_header = b"8=FIXT.1.1\x01"; //TODO: Make the protocol version adjustable.
        let message_type = self.msg_type_header();
        let message_type_len = message_type.len();
        let body_length_str = (body.len() + message_type_len).to_string();

        //Write header and body of message.
        let write_start_offset = buf.len();
        let mut byte_count = buf.write(protocol_header).unwrap();
        byte_count += buf.write(b"9=").unwrap();
        byte_count += buf.write(body_length_str.as_bytes()).unwrap();
        byte_count += buf.write(b"\x01").unwrap();
        byte_count += buf.write(&message_type[..]).unwrap();
        byte_count += buf.write(body.as_slice()).unwrap();

        //Calculate checksum.
        let mut checksum: u8 = 0;
        for byte in buf.iter().skip(write_start_offset) {
            checksum = checksum.overflowing_add(*byte).0;
        }
        let checksum_str = checksum.to_string();

        //Write checksum.
        byte_count += buf.write(b"10=").unwrap();
        if checksum < 100 {
            //Checksum must always have a length of 3.
            //FIXT version 1.1, page 55.
            buf.write(b"0").unwrap();
        }
        byte_count += buf.write(checksum_str.as_bytes()).unwrap();
        byte_count += buf.write(b"\x01").unwrap();

        byte_count
    }
}

pub const REQUIRED: bool = true;
pub const NOT_REQUIRED: bool = false;

#[macro_export]
macro_rules! define_message {
    ( $message_name:ident $( : $message_type:expr => )* { $( $field_required:expr, $field_name:ident : $field_type:ty $(=> EXCEPT_WHEN $message_ident:ident, $required_when_expr:expr)* ),* $(),* } ) => {
        #[derive(Clone)]
        pub struct $message_name {
            pub meta: Option<$crate::message::Meta>,
            $( pub $field_name: <<$field_type as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type, )*
        }

        impl $message_name {
            pub fn new() -> Self {
                $message_name {
                    meta: None,
                    $( $field_name: <<$field_type as $crate::field::Field>::Type as $crate::field_type::FieldType>::default_value(), )*
                }
            }
        }

        impl Default for $message_name {
            fn default() -> Self {
                $message_name::new()
            }
        }

        impl $crate::message::MessageDetails for $message_name {
            #[allow(unreachable_code)]
            fn msg_type() -> &'static [u8] {
                $( return $message_type )*; //Only one message type can be specified.

                b""
            }
        }

        impl $crate::message::Message for $message_name {
            #[allow(unreachable_code)]
            fn first_field(&self) -> &'static [u8] {
                $( return { <$field_type as $crate::field::Field>::tag() }; )*

                b""
            }

            fn field_count(&self) -> usize {
                let mut result = 0;
                $( let _ = $field_required; result += 1; )*

                result
            }

            fn fields(&self) -> ::std::collections::HashMap<&'static [u8],$crate::rule::Rule> {
                let mut result = ::std::collections::HashMap::with_capacity(self.field_count() * 2);
                $( result.insert(<$field_type as $crate::field::Field>::tag(),<$field_type as $crate::field::Field>::rule()); )*

                result
            }

            fn required_fields(&self) -> ::std::collections::HashSet<&'static [u8]> {
                let mut result = ::std::collections::HashSet::new();
                $( if $field_required { result.insert(<$field_type as $crate::field::Field>::tag()); } )*

                result
            }

            #[allow(unused_mut)]
            fn conditional_required_fields(&self) -> Vec<&'static [u8]> {
                let mut result = Vec::new();
                $( $(
                assert!(!$field_required); //Required fields are always required. Do not add conditional.
                let $message_ident = self;
                if $required_when_expr {
                    result.push(<$field_type as $crate::field::Field>::tag());
                }
                )* )*

                result
            }

            fn set_meta(&mut self,meta: $crate::message::Meta) {
                self.meta = Some(meta);
            }

            fn set_value(&mut self,key: &[u8],value: &[u8]) -> Result<(),$crate::message::SetValueError> {
                use $crate::field::Field;
                use $crate::field_type::FieldType;

                if false {
                    Err($crate::message::SetValueError::WrongFormat)
                }
                $( else if key == <$field_type as Field>::tag() { <$field_type as Field>::Type::set_value(&mut self.$field_name,value) } )*
                else {
                    Err($crate::message::SetValueError::WrongFormat)
                }
            }

            fn set_groups(&mut self,key: &[u8],groups: &[Box<$crate::message::Message>]) -> bool {
                use $crate::field::Field;
                use $crate::field_type::FieldType;

                if false {
                    false
                }
                $( else if key == <$field_type as Field>::tag() { <$field_type as Field>::Type::set_groups(&mut self.$field_name,groups) } )*
                else {
                    false
                }
            }

            fn as_any(&self) -> &::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut ::std::any::Any {
                self
            }

            fn new_into_box(&self) -> Box<$crate::message::Message + Send> {
                Box::new($message_name::new())
            }

            fn msg_type_header(&self) -> Vec<u8> {
                //TODO: It would be nice if this returned a &'static [u8] instead but this isn't
                //possible without Procedural Macros 1.1.
                //See: https://github.com/rust-lang/rfcs/pull/566.

                //TODO: See if we can get the buffer construction to compile into a single
                //constant.
                let mut buffer = b"35=".to_vec(); 
                buffer.extend_from_slice(
                    <$message_name as $crate::message::MessageDetails>::msg_type()
                );
                buffer.push(b'\x01');

                buffer
            }

            fn read_body(&self,buf: &mut Vec<u8>) -> usize {
                let mut byte_count: usize = 0;
                $( byte_count += <$field_type as $crate::field::Field>::read(&self.$field_name,buf,$field_required); )*

                byte_count
            }
        }

        impl PartialEq for $message_name {
            fn eq(&self,other: &$message_name) -> bool {
                //Note: Meta is not compared here because the resulting body length and checksum
                //can be different for messages that should be treated the same. For example, when
                //a repeating group count is specified with 0, the field could have been optionally
                //(and recommended to be) left out.
                $( self.$field_name == other.$field_name && )*
                true
            }
        }
    };
}

