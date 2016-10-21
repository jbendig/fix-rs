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

use field_type::FieldType;
use rule::Rule;

pub trait Field {
    type Type;
    fn rule() -> Rule;
    fn tag() -> &'static [u8];
    fn read(field: &<<Self as Field>::Type as FieldType>::Type,buf: &mut Vec<u8>) -> usize
        where <Self as Field>::Type: FieldType;
}

#[macro_export]
macro_rules! define_field {
    ( $( $field_name:ident : $field_type:ty = $tag:expr $( => $rule:expr )* ),* $(),* ) => { $(
        pub struct $field_name;
        impl Field for $field_name {
            type Type = $field_type;

            #[allow(unreachable_code)]
            fn rule() -> Rule {
                //If a rule is provided, prefer it first.
                $(
                    return $rule;
                )*

                //Next, check if the field type provides a rule. This way the BeginGroup rule
                //can be specified automatically instead of using a nasty boilerplate in each field
                //definition.
                if let Some(rule) = <$field_type as FieldType>::rule() {
                    rule
                }
                //Otherwise, no rule was specified.
                else {
                    Rule::Nothing
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

                //If this is part of a Rule::PrepareForBytes and Rule::ConfirmPreviousTag pair,
                //insert the length tag first.
                if let Rule::ConfirmPreviousTag{ previous_tag } = <$field_name as Field>::rule() {
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
                if let Rule::BeginGroup{ .. } = <$field_name as Field>::rule() {}
                else {
                    result += 1;
                    buf.push(VALUE_END);
                }

                result
            }
        }
    )*};
}

