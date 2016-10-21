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
use field::Rule;

#[derive(Clone,Default,PartialEq)]
pub struct Meta {
    pub protocol: Vec<u8>,
    pub body_length: u64,
    pub checksum: u8,
}

pub trait Message {
    fn first_field(&self) -> &'static [u8];
    fn fields(&self) -> HashMap<&'static [u8],Rule>;
    fn required_fields(&self) -> HashSet<&'static [u8]>;
    fn set_meta(&mut self,meta: Meta);
    fn set_value(&mut self,key: &[u8],value: &[u8]) -> bool;
    fn set_groups(&mut self,key: &[u8],groups: &[Box<Message>]) -> bool;
    fn as_any(&self) -> &Any;
    fn new_into_box(&self) -> Box<Message>;
    fn read_body(&self,buf: &mut Vec<u8>) -> usize;
    fn read(&self,buf: &mut Vec<u8>) -> usize {
        //TODO: Try and avoid reallocations by providing a start offset and then inserting
        //the header in a reserved space at the beginning.

        //Read entire body first so we can get the body length.
        let mut body = Vec::new();
        self.read_body(&mut body);

        //Prepare header.
        let protocol_header = b"8=FIX.4.2\x01"; //TODO: Make the protocol version adjustable.
        let message_type = b"35=A\x01"; //TODO: Make the message type adjustable.
        let message_type_len = message_type.len();
        let body_length_str = (body.len() + message_type_len).to_string();

        //Write header and body of message.
        let write_start_offset = buf.len();
        let mut byte_count = buf.write(protocol_header).unwrap();
        byte_count += buf.write(b"9=").unwrap();
        byte_count += buf.write(body_length_str.as_bytes()).unwrap();
        byte_count += buf.write(b"\x01").unwrap();
        byte_count += buf.write(message_type).unwrap();
        byte_count += buf.write(body.as_slice()).unwrap();

        //Calculate checksum.
        let mut checksum: u8 = 0;
        for byte in buf.iter().skip(write_start_offset) {
            checksum = checksum.overflowing_add(*byte).0;
        }
        let checksum_str = checksum.to_string();

        //Write checksum.
        byte_count += buf.write(b"10=").unwrap();
        byte_count += buf.write(checksum_str.as_bytes()).unwrap();
        byte_count += buf.write(b"\x01").unwrap();

        byte_count
    }
}

pub struct NullMessage {
}

impl Message for NullMessage {
    fn first_field(&self) -> &'static [u8] {
        unimplemented!();
    }

    fn fields(&self) -> HashMap<&'static [u8],Rule> {
        unimplemented!();
    }

    fn required_fields(&self) -> HashSet<&'static [u8]> {
        unimplemented!();
    }

    fn set_meta(&mut self,_meta: Meta) {
        unimplemented!();
    }

    fn set_value(&mut self,_key: &[u8],_value: &[u8]) -> bool {
        unimplemented!();
    }

    fn set_groups(&mut self,_key: &[u8],_group: &[Box<Message>]) -> bool {
        unimplemented!();
    }

    fn as_any(&self) -> &Any {
        unimplemented!();
    }

    fn new_into_box(&self) -> Box<Message> {
        unimplemented!();
    }

    fn read_body(&self,_buf: &mut Vec<u8>) -> usize {
        unimplemented!();
    }
}

pub const REQUIRED: bool = true;
pub const NOT_REQUIRED: bool = false;

#[macro_export]
macro_rules! define_message {
    ( $message_name:ident { $( $field_required:expr, $field_name:ident : $field_type:ty),* $(),* } ) => {
        #[derive(Clone,Default)]
        pub struct $message_name {
            pub meta: Option<Meta>,
            $( pub $field_name: <<$field_type as Field>::Type as FieldType>::Type, )*
        }

        impl $message_name {
            pub fn new() -> Self {
                $message_name {
                    meta: None,
                    $( $field_name: Default::default(), )*
                }
            }
        }

        impl Message for $message_name {
            #[allow(unreachable_code)]
            fn first_field(&self) -> &'static [u8] {
                $( return { <$field_type as Field>::tag() }; )*

                b"";
            }

            fn fields(&self) -> HashMap<&'static [u8],Rule> {
                let mut result = HashMap::new();
                $( result.insert(<$field_type as Field>::tag(),<$field_type as Field>::rule()); )*

                result
            }

            fn required_fields(&self) -> HashSet<&'static [u8]> {
                let mut result = HashSet::new();
                $( if $field_required { result.insert(<$field_type as Field>::tag()); } )*

                result
            }

            fn set_meta(&mut self,meta: Meta) {
                self.meta = Some(meta);
            }

            fn set_value(&mut self,key: &[u8],value: &[u8]) -> bool {
                if false {
                    false
                }
                $( else if key == <$field_type as Field>::tag() { <$field_type as Field>::Type::set_value(&mut self.$field_name,value) } )*
                else {
                    false
                }
            }

            fn set_groups(&mut self,key: &[u8],groups: &[Box<Message>]) -> bool {
                if false {
                    false
                }
                $( else if key == <$field_type as Field>::tag() { <$field_type as Field>::Type::set_groups(&mut self.$field_name,groups) } )*
                else {
                    false
                }
            }

            fn as_any(&self) -> &Any {
                self
            }

            fn new_into_box(&self) -> Box<Message> {
                Box::new($message_name::new())
            }

            fn read_body(&self,buf: &mut Vec<u8>) -> usize {
                let mut byte_count: usize = 0;
                $( byte_count += <$field_type as Field>::read(&self.$field_name,buf); )*

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

