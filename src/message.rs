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

use field_tag::FieldTag;
use fix_version::FIXVersion;
use hash::BuildFieldHasher;
use message_version::MessageVersion;
use rule::Rule;

pub type FieldHashMap = HashMap<FieldTag,Rule,BuildFieldHasher>;
pub type FieldHashSet = HashSet<FieldTag,BuildFieldHasher>;

pub trait BuildMessage {
    fn first_field(&self,version: MessageVersion) -> FieldTag;
    fn field_count(&self,version: MessageVersion) -> usize;
    fn fields(&mut self,version: MessageVersion) -> FieldHashMap;
    fn required_fields(&self,version: MessageVersion) -> FieldHashSet;

    fn new_into_box(&self) -> Box<BuildMessage + Send>;
    fn build(&self) -> Box<Message + Send>;
}

pub trait MessageBuildable {
    fn builder(&self) -> Box<BuildMessage + Send>;
    fn builder_func(&self) -> fn() -> Box<BuildMessage + Send>;
}

pub trait MessageDetails {
    fn msg_type() -> &'static [u8];
}

#[derive(Clone,PartialEq)]
pub struct Meta {
    pub begin_string: FIXVersion,
    pub body_length: u64,
    pub checksum: u8,
}

pub enum SetValueError {
    WrongFormat,
    OutOfRange,
}

pub trait Message {
    fn conditional_required_fields(&self,version: MessageVersion) -> Vec<FieldTag>;
    fn meta(&self) -> &Option<Meta>;
    fn set_meta(&mut self,meta: Meta);
    fn set_value(&mut self,key: FieldTag,value: &[u8]) -> Result<(),SetValueError>;
    fn set_groups(&mut self,key: FieldTag,groups: &[Box<Message>]) -> bool;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
    fn new_into_box(&self) -> Box<Message + Send>;
    fn msg_type_header(&self) -> &'static [u8];
    fn read_body(&self,fix_version: FIXVersion,message_version: MessageVersion,buf: &mut Vec<u8>) -> usize;
    fn read(&self,fix_version: FIXVersion,message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        //TODO: Try and avoid reallocations by providing a start offset and then inserting
        //the header in a reserved space at the beginning.

        //Read entire body first so we can get the body length.
        let mut body = Vec::new();
        self.read_body(fix_version,message_version,&mut body);

        //Prepare header.
        let message_type = self.msg_type_header();
        let message_type_len = message_type.len();
        let body_length_str = (body.len() + message_type_len).to_string();

        //Write header and body of message.
        let write_start_offset = buf.len();
        let mut byte_count = buf.write(b"8=").unwrap();
        byte_count += buf.write(fix_version.begin_string()).unwrap();
        byte_count += buf.write(b"\x01").unwrap();
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
        if checksum < 100 {
            //Checksum must always have a length of 3.
            //FIXT version 1.1, page 55.
            buf.write(b"0").unwrap();
            if checksum < 10 {
                buf.write(b"0").unwrap();
            }
        }
        byte_count += buf.write(checksum_str.as_bytes()).unwrap();
        byte_count += buf.write(b"\x01").unwrap();

        byte_count
    }
}

pub const REQUIRED: bool = true;
pub const NOT_REQUIRED: bool = false;

#[derive(Clone)]
pub struct BuildMessageInternalCache {
    pub fields_fix40: Option<FieldHashMap>,
    pub fields_fix41: Option<FieldHashMap>,
    pub fields_fix42: Option<FieldHashMap>,
    pub fields_fix43: Option<FieldHashMap>,
    pub fields_fix44: Option<FieldHashMap>,
    pub fields_fix50: Option<FieldHashMap>,
    pub fields_fix50sp1: Option<FieldHashMap>,
    pub fields_fix50sp2: Option<FieldHashMap>,
}

#[macro_export]
macro_rules! symbol_to_message_version {
    ( FIX40 ) => { $crate::message_version::MessageVersion::FIX40 };
    ( FIX41 ) => { $crate::message_version::MessageVersion::FIX41 };
    ( FIX42 ) => { $crate::message_version::MessageVersion::FIX42 };
    ( FIX43 ) => { $crate::message_version::MessageVersion::FIX43 };
    ( FIX44 ) => { $crate::message_version::MessageVersion::FIX44 };
    ( FIX50 ) => { $crate::message_version::MessageVersion::FIX50 };
    ( FIX50SP1 ) => { $crate::message_version::MessageVersion::FIX50SP1 };
    ( FIX50SP2 ) => { $crate::message_version::MessageVersion::FIX50SP2 };
}

#[macro_export]
macro_rules! match_message_version {
    ( $version:ident, $minimum_version:tt ) => {{
        match_message_version!($version,$minimum_version .. $minimum_version)
    }};

    ( $version:ident, $minimum_version:tt .. ) => {{
        match_message_version!($version,$minimum_version .. FIX50SP2)
    }};

    ( $version:ident, $minimum_version:tt .. $maximum_version:tt ) => {
        if $version.as_value() >= symbol_to_message_version!($minimum_version).as_value() && $version.as_value() <= symbol_to_message_version!($maximum_version).as_value() {
            true
        }
        else {
            false
        }
    };
}

#[macro_export]
macro_rules! define_message {
    //TODO: Remove this helper when version is added to all fields.
    ( $message_name:ident $( : $message_type:expr => )* { $( $field_required:expr, $field_name:ident : $field_type:ty $(=> REQUIRED_WHEN $required_when_expr:expr)* ),* $(),* } ) => {
        define_message!($message_name $( : $message_type => )* { $( $field_required, $field_name : $field_type [FIX40..] $(=> REQUIRED_WHEN $required_when_expr)*, )* });
    };

    ( $message_name:ident $( : $message_type:expr => )* { $( $field_required:expr, $field_name:ident : $field_type:ty [$( $version:tt )*] $(=> REQUIRED_WHEN $required_when_expr:expr)* ),* $(),* } ) => {
        #[derive(BuildMessage)]
        pub struct $message_name {
            pub meta: Option<$crate::message::Meta>,
            $( pub $field_name: <<$field_type as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type, )*
            $( #[message_type=$message_type] )*
            _message_type_gen: ::std::marker::PhantomData<()>,
        }

        impl Clone for $message_name {
            fn clone(&self) -> Self {
                $message_name {
                    meta: self.meta.clone(),
                    $( $field_name: self.$field_name.clone(), )*
                    _message_type_gen: ::std::marker::PhantomData,
                }
            }
        }

        impl $message_name {
            pub fn new() -> $message_name {
                $message_name {
                    meta: None,
                    $( $field_name: <<$field_type as $crate::field::Field>::Type as $crate::field_type::FieldType>::default_value(), )*
                    _message_type_gen: ::std::marker::PhantomData,
                }
            }

            #[allow(unreachable_code)]
            fn first_field(version: $crate::message_version::MessageVersion) -> $crate::field_tag::FieldTag {
                $( if match_message_version!(version,$( $version)*) {
                    return <$field_type as $crate::field::Field>::tag();
                } )*

                $crate::field_tag::FieldTag::empty()
            }

            fn field_count(version: $crate::message_version::MessageVersion) -> usize {
                let mut result = 0;
                $( if match_message_version!(version,$( $version )*) {
                    let _ = $field_required; result += 1;
                } )*

                result
            }

            fn required_field_count(version: $crate::message_version::MessageVersion) -> usize {
                let mut result = 0;
                $( if match_message_version!(version,$( $version )*) && $field_required {
                    result += 1;
                } )*

                result
            }

            fn fields(version: $crate::message_version::MessageVersion) -> $crate::message::FieldHashMap {
                let mut fields = ::std::collections::HashMap::with_capacity_and_hasher($message_name::field_count(version) * 1,$crate::hash::BuildFieldHasher);
                $( if match_message_version!(version,$( $version )*) {
                    fields.insert(<$field_type as $crate::field::Field>::tag(),<$field_type as $crate::field::Field>::rule());
                } )*

                fields
            }

            fn required_fields(version: $crate::message_version::MessageVersion) -> $crate::message::FieldHashSet {
                let mut result = ::std::collections::HashSet::with_capacity_and_hasher($message_name::required_field_count(version) * 1,$crate::hash::BuildFieldHasher);
                $( if match_message_version!(version,$( $version )*) && $field_required {
                    result.insert(<$field_type as $crate::field::Field>::tag());
                } )*

                result
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
            #[allow(unused_mut,unused_variables)]
            fn conditional_required_fields(&self,version: $crate::message_version::MessageVersion) -> Vec<$crate::field_tag::FieldTag> {
                let mut result = Vec::new();
                $( $(
                assert!(!$field_required); //Required fields are always required. Do not add conditional.
                if $required_when_expr(self,version) {
                    result.push(<$field_type as $crate::field::Field>::tag());
                }
                )* )*

                result
            }

            fn meta(&self) -> &Option<$crate::message::Meta> {
                &self.meta
            }

            fn set_meta(&mut self,meta: $crate::message::Meta) {
                self.meta = Some(meta);
            }

            fn set_value(&mut self,key: $crate::field_tag::FieldTag,value: &[u8]) -> Result<(),$crate::message::SetValueError> {
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

            fn set_groups(&mut self,key: $crate::field_tag::FieldTag,groups: &[Box<$crate::message::Message>]) -> bool {
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

            fn msg_type_header(&self) -> &'static [u8] {
                $message_name::msg_type_header()
            }

            fn read_body(&self,fix_version: $crate::fix_version::FIXVersion,message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
                let mut byte_count: usize = 0;
                $( if match_message_version!(message_version,$( $version )*) {
                    byte_count += <$field_type as $crate::field::Field>::read(&self.$field_name,fix_version,message_version,buf,$field_required);
                } )*

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

