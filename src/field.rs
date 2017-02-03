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

use field_tag::FieldTag;
use field_type::FieldType;
use fix_version::FIXVersion;
use message_version::MessageVersion;
use rule::Rule;

pub trait Field {
    type Type;
    fn rule() -> Rule;
    fn tag_bytes() -> &'static [u8];
    fn tag() -> FieldTag;
    fn read(field: &<<Self as Field>::Type as FieldType>::Type,fix_version: FIXVersion,message_version: MessageVersion,buf: &mut Vec<u8>,required: bool) -> usize
        where <Self as Field>::Type: FieldType;
}

#[macro_export]
macro_rules! define_fields {
    ( $( $field_name:ident : $field_type:ty = $tag:expr $( => $rule:expr )* ),* $(),* ) => { $(
        #[derive(BuildField)]
        pub struct $field_name {
            #[tag=$tag]
            _tag_gen: ::std::marker::PhantomData<()>,
        }

        impl $crate::field::Field for $field_name {
            type Type = $field_type;

            #[allow(unreachable_code)]
            fn rule() -> $crate::rule::Rule {
                //If a rule is provided, prefer it first.
                $(
                    return $rule //A maximum of one rule may be specified.
                )*;

                //Next, check if the field type provides a rule. This way the BeginGroup rule
                //can be specified automatically instead of using a nasty boilerplate in each field
                //definition.
                if let Some(rule) = <$field_type as $crate::field_type::FieldType>::rule() {
                    rule
                }
                //Otherwise, no rule was specified.
                else {
                    $crate::rule::Rule::Nothing
                }
            }

            fn tag_bytes() -> &'static [u8] {
                Self::tag_bytes()
            }

            fn tag() -> $crate::field_tag::FieldTag {
                Self::tag()
            }

            fn read(field: &<<Self as $crate::field::Field>::Type as $crate::field_type::FieldType>::Type,fix_version: $crate::fix_version::FIXVersion,message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>,required: bool) -> usize {
                use ::std::io::Write;

                if !required && <$field_type as $crate::field_type::FieldType>::is_empty(field) {
                    return 0;
                }

                let mut result = 1;

                match <$field_name as $crate::field::Field>::rule() {
                    //If this is the first part of a Rule::PrepareForBytes and Rule::ConfirmPreviousTag
                    //pair, skip the tag completely.
                    $crate::rule::Rule::PrepareForBytes{ .. } => {
                        return 0;
                    },
                    //If this is the second part of a Rule::PrepareForBytes and
                    //Rule::ConfirmPreviousTag pair, insert the length tag first.
                    $crate::rule::Rule::ConfirmPreviousTag{ previous_tag } => {
                        let previous_tag = previous_tag.to_bytes();
                        result += 2;
                        result += buf.write(&previous_tag[..]).unwrap();
                        buf.push($crate::constant::TAG_END);
                        result += buf.write(<$field_type as $crate::field_type::FieldType>::len(field).to_string().as_bytes()).unwrap();
                        buf.push($crate::constant::VALUE_END);
                    },
                    //If this tag should only be serialized with a different FIX version, skip the
                    //tag completely.
                    $crate::rule::Rule::RequiresFIXVersion{ fix_version: required_fix_version } => {
                        if fix_version != required_fix_version {
                            return 0;
                        }
                    },
                    _ => {},
                };

                //Write tag and value.
                result += buf.write(Self::tag_bytes()).unwrap();
                buf.push($crate::constant::TAG_END);
                result += <$field_type as $crate::field_type::FieldType>::read(field,fix_version,message_version,buf);

                //Avoid the VALUE_END symbol iff this is not a repeating group field. This is a
                //hack, under the assumption that the field itself adds this symbol, so the field
                //can append the remaining groups.
                if let $crate::rule::Rule::BeginGroup{ .. } = <$field_name as $crate::field::Field>::rule() {}
                else {
                    result += 1;
                    buf.push($crate::constant::VALUE_END);
                }

                result
            }
        }
    )*};
}

