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

#![feature(test)]

#[macro_use]
extern crate fix_rs;
extern crate test;

use test::Bencher;

use fix_rs::byte_buffer::ByteBuffer;
use fix_rs::dictionary::messages::NewOrderSingle;
use fix_rs::fix::Parser;
use fix_rs::fix_version::FIXVersion;
use fix_rs::message::Message;
use fix_rs::message_version::MessageVersion;

const MESSAGE_BYTES: &'static [u8] = b"8=FIX.4.2\x019=206\x0135=D\x0149=AFUNDMGR\x0156=ABROKER\x0134=2\x0152=20170101-01:01:01.001\x0111=12345\x011=111111\x0163=0\x0164=20170101\x0121=3\x01110=1000\x01111=50000\x0155=IBM\x0148=459200101\x0122=1\x0154=1\x0160=20170101-01:01:01.001\x0138=5000\x0140=1\x0144=15.75\x0115=USD\x0159=0\x0110=092\x01";

#[bench]
fn parse_simple_message_bench(b: &mut Bencher) {
    define_dictionary!(NewOrderSingle,);

    let mut parser = Parser::new(build_dictionary(), 4096);
    b.bytes = MESSAGE_BYTES.len() as u64;
    b.iter(|| {
        let (bytes_read, result) = parser.parse(MESSAGE_BYTES);
        assert!(result.is_ok());
        assert!(bytes_read == MESSAGE_BYTES.len());
    });
}

#[bench]
fn serialize_simple_message_bench(b: &mut Bencher) {
    define_dictionary!(NewOrderSingle,);

    let mut parser = Parser::new(build_dictionary(), 4096);
    let (bytes_read, result) = parser.parse(MESSAGE_BYTES);
    assert!(result.is_ok());
    assert!(bytes_read == MESSAGE_BYTES.len());
    match message_to_enum(parser.messages.remove(0)) {
        MessageEnum::NewOrderSingle(message) => {
            let mut data = ByteBuffer::with_capacity(512);
            let mut serialize_func =
                || message.read(FIXVersion::FIXT_1_1, MessageVersion::FIX50SP2, &mut data) as u64;

            b.bytes = serialize_func();
            b.iter(serialize_func);
        }
    }
}
