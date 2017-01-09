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

use fix_version::FIXVersion;
use message::Message;

//Special rules that describe what extra processing needs to be done to a field during parsing or
//serialization.
pub enum Rule {
    Nothing,
    BeginGroup{message: Box<Message>},
    PrepareForBytes{bytes_tag: &'static [u8]},
    ConfirmPreviousTag{previous_tag: &'static [u8]}, //TODO: Probably redundant to the PrepareForBytes definition. Should be automatically inferred.
    RequiresFIXVersion{fix_version: FIXVersion}, //Used during serialization only.
}
