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

use constant::{FIX_4_0_BEGIN_STRING,FIX_4_1_BEGIN_STRING,FIX_4_2_BEGIN_STRING,FIX_4_3_BEGIN_STRING,FIX_4_4_BEGIN_STRING,FIXT_1_1_BEGIN_STRING};

#[derive(Clone,Copy,Debug,PartialEq)]
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
            FIXVersion::FIXT_1_1 => FIXT_1_1_BEGIN_STRING,
            FIXVersion::FIX_4_0 => FIX_4_0_BEGIN_STRING,
            FIXVersion::FIX_4_1 => FIX_4_1_BEGIN_STRING,
            FIXVersion::FIX_4_2 => FIX_4_2_BEGIN_STRING,
            FIXVersion::FIX_4_3 => FIX_4_3_BEGIN_STRING,
            FIXVersion::FIX_4_4 => FIX_4_4_BEGIN_STRING,
        }
    }
}

