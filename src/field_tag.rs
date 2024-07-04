// Copyright 2017 James Bendig. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under:
//   the MIT license
//     <LICENSE-MIT or https://opensource.org/licenses/MIT>
//   or the Apache License, Version 2.0
//     <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0>,
// at your option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cmp::{Ord, Ordering, PartialOrd};
use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FieldTag(pub u64);

impl FieldTag {
    pub fn empty() -> Self {
        FieldTag(0)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        //TODO: Write a more optimized version that skips string here.
        //TODO: In fact, probably support writing to an existing vector.
        let string = self.0.to_string();
        string.as_bytes().to_vec()
    }

    pub fn is_empty(&self) -> bool {
        //There are no FIX field tags that start with 0.
        self.0 == 0
    }
}

impl fmt::Debug for FieldTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FieldTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<Vec<u8>> for FieldTag {
    fn into(self) -> Vec<u8> {
        self.to_bytes()
    }
}

impl Into<u64> for FieldTag {
    fn into(self) -> u64 {
        self.0
    }
}

impl<'a> From<&'a [u8]> for FieldTag {
    fn from(bytes: &[u8]) -> Self {
        //Unchecked way to change ASCII number into an unsigned integer.

        let mut tag = 0;
        for byte in bytes {
            tag *= 10;
            tag += byte.overflowing_sub(48).0 as u64;
        }

        FieldTag(tag)
    }
}

impl From<u64> for FieldTag {
    fn from(tag: u64) -> Self {
        FieldTag(tag)
    }
}

impl PartialOrd for FieldTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FieldTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}
