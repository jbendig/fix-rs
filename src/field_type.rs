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

use crate::fix_version::FIXVersion;
use crate::message::{Message, SetValueError};
use crate::message_version::MessageVersion;
use crate::rule::Rule;

pub trait FieldType {
    type Type;

    fn rule() -> Option<Rule> {
        None
    }

    fn default_value() -> Self::Type;

    fn set_value(_field: &mut Self::Type, _bytes: &[u8]) -> Result<(), SetValueError> {
        Err(SetValueError::WrongFormat)
    }

    fn set_groups(_field: &mut Self::Type, _groups: Vec<Box<dyn Message>>) -> bool {
        false
    }

    fn is_empty(field: &Self::Type) -> bool;
    fn len(field: &Self::Type) -> usize;
    fn read(
        field: &Self::Type,
        fix_version: FIXVersion,
        message_version: MessageVersion,
        buf: &mut Vec<u8>,
    ) -> usize;
}
