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

#![doc(hidden)]

use std::hash::{BuildHasher, Hasher};

pub struct FieldHasher {
    value: u64,
}

impl Hasher for FieldHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }

    fn write_u64(&mut self, i: u64) {
        //Just use the input directly as the hash. This seems laughable but:
        //1. The keys themselves are u64s.
        //2. We don't need HASHDOS protection because each key can only be encountered once and the
        //   set is defined at compile time.
        //3. Benchmarking shows collisions are a minimal problem for these sets.
        //4. The built-in hasher is by far the slowest component of parsing just from initializing
        //   with .insert(), nevermind the overhead of .get() or .remove() in a HashMap.
        self.value = i;
    }
}

#[derive(Clone)]
pub struct BuildFieldHasher;

impl BuildHasher for BuildFieldHasher {
    type Hasher = FieldHasher;
    fn build_hasher(&self) -> Self::Hasher {
        FieldHasher { value: 0 }
    }
}
