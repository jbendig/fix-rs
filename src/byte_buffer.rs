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

use std::io::{Read, Result, Write};

pub struct ByteBuffer {
    #[doc(hidden)]
    pub bytes: Vec<u8>,
    #[doc(hidden)]
    pub valid_bytes_begin: usize,
    #[doc(hidden)]
    pub valid_bytes_end: usize,
}

impl ByteBuffer {
    pub fn new() -> ByteBuffer {
        ByteBuffer {
            bytes: Vec::new(),
            valid_bytes_begin: 0,
            valid_bytes_end: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> ByteBuffer {
        ByteBuffer {
            bytes: vec![0; capacity],
            valid_bytes_begin: 0,
            valid_bytes_end: 0,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.valid_bytes_begin..self.valid_bytes_end]
    }

    pub fn clear(&mut self) {
        self.valid_bytes_begin = 0;
        self.valid_bytes_end = 0;
    }

    pub fn clear_and_read<T: Read>(&mut self, reader: &mut T) -> Result<usize> {
        self.clear();
        let result = reader.read(&mut self.bytes[..]);
        if let Ok(bytes_read) = result {
            assert!(bytes_read <= self.bytes.len());
            self.valid_bytes_end = bytes_read;
        }

        result
    }

    pub fn clear_and_read_all<F>(&mut self, read_all_func: F)
    where
        F: Fn(&mut Vec<u8>),
    {
        self.bytes.clear();
        read_all_func(&mut self.bytes);

        self.valid_bytes_begin = 0;
        self.valid_bytes_end = self.bytes.len();
    }

    pub fn consume(&mut self, count: usize) {
        assert!(self.valid_bytes_begin + count <= self.valid_bytes_end);
        self.valid_bytes_begin += count;
    }

    pub fn is_empty(&self) -> bool {
        self.valid_bytes_begin == self.valid_bytes_end
    }

    pub fn len(&self) -> usize {
        self.valid_bytes_end - self.valid_bytes_begin
    }

    pub fn write<T: Write>(&mut self, writer: &mut T) -> Result<usize> {
        let result = writer.write(self.bytes());
        if let Ok(bytes_written) = result {
            self.consume(bytes_written);
        }

        result
    }
}
