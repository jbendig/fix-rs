#![feature(test)]

#[macro_use]
extern crate fix_rs;
extern crate test;

use std::collections::HashMap;
use fix_rs::message::Message;
use fix_rs::dictionary::NewOrderSingle;
use fix_rs::fix::Parser;
use test::Bencher;

#[bench]
fn parse_simple_message_bench(b: &mut Bencher) {
    define_dictionary!(
        b"D" => NewOrderSingle : NewOrderSingle,
    );

    let message = "8=FIX.4.2\u{1}9=197\u{1}35=D\u{1}49=AFUNDMGR\u{1}56=ABROKER\u{1}34=2\u{1}52=20030615-01:14:49\u{1}11=12345\u{1}1=111111\u{1}63=0\u{1}64=20030621\u{1}21=3\u{1}110=1000\u{1}111=50000\u{1}55=IBM\u{1}48=459200101\u{1}22=1\u{1}54=1\u{1}60=2003061501:14:49\u{1}38=5000\u{1}40=1\u{1}44=15.75\u{1}15=USD\u{1}59=0\u{1}10=230\u{1}";
    let message_bytes = Vec::from(message);

    let mut parser = Parser::new(build_dictionary());
    b.iter(|| {
        let (bytes_read,result) = parser.parse(&message_bytes);
        assert!(result.is_ok());
        assert!(bytes_read == message_bytes.len());
    });
}
