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

use crate::field_tag::FieldTag;
use crate::fix_version::FIXVersion;
use crate::message::BuildMessage;

//Special rules that describe what extra processing needs to be done to a field during parsing or
//serialization.
#[derive(Clone)]
pub enum Rule {
    Nothing,
    BeginGroup {
        builder_func: fn() -> Box<dyn BuildMessage + Send>,
    },
    PrepareForBytes {
        bytes_tag: FieldTag,
    },
    ConfirmPreviousTag {
        previous_tag: FieldTag,
    }, //TODO: Probably redundant to the PrepareForBytes definition. Should be automatically inferred.
    RequiresFIXVersion {
        fix_version: FIXVersion,
    }, //Used during serialization only.
}
