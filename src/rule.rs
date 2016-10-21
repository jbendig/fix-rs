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

use std::collections::HashSet;

use message::Message;

//Special rules that describe to the parser what extra processing needs to be done to handle a
//specific field.
pub enum Rule {
    Nothing,
    AddRequiredTags(HashSet<&'static [u8]>),
    BeginGroup{message: Box<Message>},
    PrepareForBytes{bytes_tag: &'static [u8]},
    ConfirmPreviousTag{previous_tag: &'static [u8]}, //TODO: Probably redundant to the PrepareForBytes definition. Should be automatically inferred.
}
