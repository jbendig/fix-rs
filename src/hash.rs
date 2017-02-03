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

use std::cmp::max;
use std::hash::{BuildHasher,Hasher};
use std::mem::transmute;
use std::ptr::copy_nonoverlapping;

pub struct FieldHasher {
    index: usize,
    value: u64,
}

impl Hasher for FieldHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self,bytes: &[u8]) {
        //Copy the first 8 bytes directly into a u64. This seems laughable but:
        //1. The keys themselves are u64s.
        //2. We don't need HASHDOS protection because each key can only be encountered once and the
        //   set is defined at compile time.
        //3. Benchmarking shows collisions are a minimal problem for these sets.
        //4. The built-in hasher is by far the slowest component of parsing just from initializing
        //   with .insert(), nevermind the overhead of .get() or .remove() in a HashMap.
        unsafe {
            let src = bytes.get_unchecked(0);
            let dst = transmute::<&mut u64,*mut u8>(&mut self.value);
            let bytes_to_copy = max(bytes.len(),8 - self.index);
            copy_nonoverlapping(src,dst.offset(self.index as isize),bytes_to_copy);
            self.index += bytes_to_copy;
        }
    }
}

#[derive(Clone)]
pub struct BuildFieldHasher;

impl BuildHasher for BuildFieldHasher {
    type Hasher = FieldHasher;
    fn build_hasher(&self) -> Self::Hasher {
        FieldHasher {
            index: 0,
            value: 0,
        }
    }
}

