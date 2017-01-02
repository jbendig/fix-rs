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

#[derive(Clone,Debug,PartialEq)]
#[allow(non_camel_case_types)]
pub enum FIXVersion {
    FIXT_1_1,
    FIX_4_0,
    FIX_4_1,
    FIX_4_2,
    FIX_4_3,
    FIX_4_4,
}

impl FIXVersion {
    pub fn begin_string(&self) -> &'static [u8] {
        match *self {
            FIXVersion::FIXT_1_1 => b"FIXT.1.1",
            FIXVersion::FIX_4_0 => b"FIX.4.0",
            FIXVersion::FIX_4_1 => b"FIX.4.1",
            FIXVersion::FIX_4_2 => b"FIX.4.2",
            FIXVersion::FIX_4_3 => b"FIX.4.3",
            FIXVersion::FIX_4_4 => b"FIX.4.4",
        }
    }
}

