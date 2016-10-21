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
use std::marker::PhantomData;
use std::io::Write;

use constant::VALUE_END;
use message::Message;
use rule::Rule;

pub trait FieldType {
    type Type;

    fn rule() -> Option<Rule> {
        None
    }

    fn set_value(_field: &mut Self::Type,_bytes: &[u8]) -> bool {
        false
    }

    fn set_groups(_field: &mut Self::Type,_groups: &[Box<Message>]) -> bool {
        false
    }

    fn is_empty(field: &Self::Type) -> bool;
    fn len(field: &Self::Type) -> usize;
    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize;
}

pub struct NoneFieldType {
}

impl FieldType for NoneFieldType {
    type Type = PhantomData<()>;

    fn is_empty(_field: &Self::Type) -> bool {
        true
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(_field: &Self::Type,_buf: &mut Vec<u8>) -> usize {
        0
    }
}

pub struct StringFieldType {
}

impl FieldType for StringFieldType {
    type Type = String;

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> bool {
        *field = String::from_utf8_lossy(bytes).into_owned();
        true
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(field.as_bytes()).unwrap()
    }
}

pub struct DataFieldType {
}

impl FieldType for DataFieldType {
    type Type = Vec<u8>;

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> bool {
        field.resize(bytes.len(),0);
        field.copy_from_slice(bytes);
        true
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(field).unwrap()
    }
}

pub struct RepeatingGroupFieldType<T: Message + PartialEq> {
    message_type: PhantomData<T>,
}

impl<T: Message + Any + Clone + Default + PartialEq> FieldType for RepeatingGroupFieldType<T> {
    type Type = Vec<Box<T>>;

    fn rule() -> Option<Rule> {
        Some(Rule::BeginGroup{ message: Box::new(<T as Default>::default()) })
    }

    fn set_groups(field: &mut Self::Type,groups: &[Box<Message>]) -> bool {
        field.clear();

        for group in groups {
            match group.as_any().downcast_ref::<T>() {
                //TODO: Avoid the clone below.
                Some(casted_group) => field.push(Box::new(casted_group.clone())),
                None => return false,
            }
        }

        true
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let group_count_str = field.len().to_string();
        let mut result = 1;

        result += buf.write(group_count_str.as_bytes()).unwrap();
        buf.push(VALUE_END);

        for group in field {
            result += group.read_body(buf);
        }

        result
    }
}

