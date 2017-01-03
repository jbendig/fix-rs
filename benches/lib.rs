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

use fix_rs::dictionary::messages::NewOrderSingle;
use fix_rs::fix::Parser;
use fix_rs::fix_version::FIXVersion;
use fix_rs::message::Message;

const MESSAGE_BYTES: &'static [u8] = b"8=FIX.4.2\x019=197\x0135=D\x0149=AFUNDMGR\x0156=ABROKER\x0134=2\x0152=20030615-01:14:49\x0111=12345\x011=111111\x0163=0\x0164=20030621\x0121=3\x01110=1000\x01111=50000\x0155=IBM\x0148=459200101\x0122=1\x0154=1\x0160=2003061501:14:49\x0138=5000\x0140=1\x0144=15.75\x0115=USD\x0159=0\x0110=230\x01";

#[bench]
fn parse_simple_message_bench(b: &mut Bencher) {
    define_dictionary!(
        NewOrderSingle : NewOrderSingle,
    );

    let mut parser = Parser::new(build_dictionary());
    b.iter(|| {
        let (bytes_read,result) = parser.parse(MESSAGE_BYTES);
        assert!(result.is_ok());
        assert!(bytes_read == MESSAGE_BYTES.len());
    });
}

#[bench]
fn serialize_simple_message_bench(b: &mut Bencher) {
    define_dictionary!(
        NewOrderSingle : NewOrderSingle,
    );

    let mut parser = Parser::new(build_dictionary());
    let (bytes_read,result) = parser.parse(MESSAGE_BYTES);
    assert!(result.is_ok());
    assert!(bytes_read == MESSAGE_BYTES.len());
    match message_to_enum(&**(parser.messages.first().unwrap())) {
        MessageEnum::NewOrderSingle(message) => {
            b.iter(|| {
                let mut data = Vec::new();
                message.read(&FIXVersion::FIXT_1_1,&mut data);
            });
        },
    }
}
