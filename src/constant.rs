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

#![doc(hidden)]

pub const TAG_END: u8 = b'=';
pub const VALUE_END: u8 = b'\x01'; //SOH

pub const FIXT_1_1_BEGIN_STRING: &'static [u8] = b"FIXT.1.1";
pub const FIX_4_0_BEGIN_STRING: &'static [u8] = b"FIX.4.0";
pub const FIX_4_1_BEGIN_STRING: &'static [u8] = b"FIX.4.1";
pub const FIX_4_2_BEGIN_STRING: &'static [u8] = b"FIX.4.2";
pub const FIX_4_3_BEGIN_STRING: &'static [u8] = b"FIX.4.3";
pub const FIX_4_4_BEGIN_STRING: &'static [u8] = b"FIX.4.4";
