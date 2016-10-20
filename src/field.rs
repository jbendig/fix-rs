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

use message::Message;
use constant::VALUE_END;
use std::any::Any;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::io::Write;

pub enum Action {
    Nothing,
    AddRequiredTags(HashSet<&'static [u8]>),
    BeginGroup{message: Box<Message>},
    PrepareForBytes{bytes_tag: &'static [u8]},
    ConfirmPreviousTag{previous_tag: &'static [u8]}, //TODO: Probably redundant to the PrepareForBytes definition. Should be automatically inferred.
}

pub trait FieldType {
    type Type;

    fn action() -> Option<Action> {
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

    fn action() -> Option<Action> {
        Some(Action::BeginGroup{ message: Box::new(<T as Default>::default()) })
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

pub trait Field {
    type Type;
    fn action() -> Action;
    fn tag() -> &'static [u8];
    fn read(field: &<<Self as Field>::Type as FieldType>::Type,buf: &mut Vec<u8>) -> usize
        where <Self as Field>::Type: FieldType;
}

#[macro_export]
macro_rules! define_field {
    ( $( $field_name:ident : $field_type:ty = $tag:expr $( => $action:expr )* ),* $(),* ) => { $(
        pub struct $field_name;
        impl Field for $field_name {
            type Type = $field_type;

            #[allow(unreachable_code)]
            fn action() -> Action {
                //If an action is provided, prefer it first.
                $(
                    return $action;
                )*

                //Next, check if the field type provides an action. This way the BeginGroup action
                //can be specified automatically instead of using a nasty boilerplate in each field
                //definition.
                if let Some(action) = <$field_type as FieldType>::action() {
                    action
                }
                //Otherwise, no action was specified.
                else {
                    Action::Nothing
                }
            }

            fn tag() -> &'static [u8] {
                $tag
            }

            fn read(field: &<<Self as Field>::Type as FieldType>::Type,buf: &mut Vec<u8>) -> usize {
                if <$field_type as FieldType>::is_empty(field) {
                    return 0;
                }

                let mut result = 1;

                //If this is part of a Action::PrepareForBytes and Action::ConfirmPreviousTag pair,
                //insert the length tag first.
                if let Action::ConfirmPreviousTag{ previous_tag } = <$field_name as Field>::action() {
                    result += 2;
                    result += buf.write(previous_tag).unwrap();
                    buf.push(TAG_END);
                    result += buf.write(<$field_type as FieldType>::len(field).to_string().as_bytes()).unwrap();
                    buf.push(VALUE_END);
                }

                //Write tag and value.
                result += buf.write($tag).unwrap();
                buf.push(TAG_END);
                result += <$field_type as FieldType>::read(field,buf);

                //Avoid the VALUE_END symbol iff this is not a repeating group field. This is a
                //hack, under the assumption that the field itself adds this symbol, so the field
                //can append the remaining groups.
                if let Action::BeginGroup{ .. } = <$field_name as Field>::action() {}
                else {
                    result += 1;
                    buf.push(VALUE_END);
                }

                result
            }
        }
    )*};
}

