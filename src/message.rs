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
use std::mem;
use std::io::Write;
use std::ptr;

use byte_buffer::ByteBuffer;
use constant::VALUE_END;
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
    pub message_version: MessageVersion,
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
    fn set_groups(&mut self,key: FieldTag,groups: Vec<Box<Message>>) -> bool;
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
    fn new_into_box(&self) -> Box<Message + Send>;
    fn msg_type_header(&self) -> &'static [u8];
    fn read_body(&self,fix_version: FIXVersion,message_version: MessageVersion,buf: &mut Vec<u8>) -> usize;

    fn read(&self,fix_version: FIXVersion,message_version: MessageVersion,buf: &mut ByteBuffer) -> usize {
        const HEADER_PADDING_LEN: usize = 32;

        //Leave rooom at beginning of buffer for header.
        buf.clear();
        buf.bytes.resize(HEADER_PADDING_LEN,0);

        //Read entire body first so we can get the body length.
        self.read_body(fix_version,message_version,&mut buf.bytes);

        //Prepare header.
        let message_type = self.msg_type_header();
        let message_type_len = message_type.len();
        let body_len_str = (buf.bytes.len() - HEADER_PADDING_LEN + message_type_len).to_string();
        let header_len = 2 + fix_version.begin_string().len() + 1 + //8=<FIXVersion>\x01
                         2 + body_len_str.as_bytes().len() + 1 +    //9=<BodyLength>\x01
                         message_type_len;                          //35=<MessageType>\x01

        //If the header won't fit in the room reserved at the beginning, make room for it. This
        //is much slower but shouldn't happen in practice because BodyLength would have to be in
        //the Petabyte range AND/OR the MessageType would have to be custom and several times
        //larger than normal.
        if header_len > HEADER_PADDING_LEN {
            let body_len = buf.bytes.len() - HEADER_PADDING_LEN;
            const CHECKSUM_LEN: usize = 7; //10=000\x01
            let message_len = header_len + body_len + CHECKSUM_LEN;

            //Create a new buffer with room for the header.
            let body_bytes = mem::replace(&mut buf.bytes,Vec::with_capacity(message_len));

            //Copy body from old buffer to new buffer.
            unsafe {
                buf.bytes.set_len(message_len);
                ptr::copy(body_bytes.as_ptr().offset(HEADER_PADDING_LEN as isize),
                          buf.bytes.as_mut_ptr().offset(header_len as isize),
                          body_len);
            }
        }

        //Write header to start of buffer.
        buf.valid_bytes_begin = HEADER_PADDING_LEN - header_len;
        unsafe {
            unsafe fn copy_and_advance<T>(src: *const T,dst: &mut *mut T,count: usize) {
                ptr::copy(src,*dst,count);
                *dst = dst.offset(count as isize);
            }
            unsafe fn copy_slice_and_advance(src: &[u8],dst: &mut *mut u8) {
                copy_and_advance(src.as_ptr(),dst,src.len());
            }

            let mut bytes_ptr = buf.bytes.as_mut_ptr().offset(buf.valid_bytes_begin as isize);
            copy_slice_and_advance(b"8=",&mut bytes_ptr);
            copy_slice_and_advance(fix_version.begin_string(),&mut bytes_ptr);
            copy_slice_and_advance(b"\x019=",&mut bytes_ptr);
            copy_slice_and_advance(body_len_str.as_bytes(),&mut bytes_ptr);
            copy_slice_and_advance(b"\x01",&mut bytes_ptr);
            copy_slice_and_advance(message_type,&mut bytes_ptr);
        }

        //Calculate checksum.
        let mut checksum: u8 = 0;
        for byte in buf.bytes.iter().skip(buf.valid_bytes_begin) {
            checksum = checksum.overflowing_add(*byte).0;
        }
        let checksum_str = checksum.to_string();

        //Write checksum at the end.
        buf.bytes.write(b"10=").unwrap();
        if checksum < 100 {
            //Checksum must always have a length of 3.
            //FIXT version 1.1, page 55.
            buf.bytes.write(b"0").unwrap();
            if checksum < 10 {
                buf.bytes.write(b"0").unwrap();
            }
        }
        buf.bytes.write(checksum_str.as_bytes()).unwrap();
        buf.bytes.write(b"\x01").unwrap();

        //Mark end of buffer.
        buf.valid_bytes_end = buf.bytes.len();

        buf.len()
    }

    fn debug(&self,fix_version: FIXVersion,message_version: MessageVersion) -> String {
        let mut buffer = ByteBuffer::with_capacity(512);
        self.read(fix_version,message_version,&mut buffer);

        //Replace SOH characters with | to be human readable.
        let buffer: Vec<u8> = buffer.bytes().into_iter().map(|c| if *c == VALUE_END { b'|' } else { *c } ).collect();

        String::from_utf8_lossy(&buffer[..]).into_owned()
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

#[doc(hidden)]
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

#[doc(hidden)]
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

            fn set_groups(&mut self,key: $crate::field_tag::FieldTag,groups: Vec<Box<$crate::message::Message>>) -> bool {
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

