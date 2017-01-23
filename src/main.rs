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

extern crate chrono;
#[macro_use]
extern crate fix_rs;

use chrono::offset::utc::UTC;
use chrono::TimeZone;

use fix_rs::dictionary::field_types::other::{EncryptMethod,MsgDirection};
use fix_rs::dictionary::messages::Logon;
use fix_rs::fix::Parser;
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt::client::Client;
use fix_rs::message::Message;
use fix_rs::message_version::MessageVersion;

//Helper function to make it easier to figure out what the body_length tag should be set to.
fn estimate_body_length(message_bytes: &[u8]) -> usize {
    let mut previous_byte = 0;
    let mut found_body_length_tag = false;
    let mut body_start = 0;
    for (index,byte) in message_bytes.iter().enumerate() {
        if body_start == 0 && found_body_length_tag && *byte == b'\x01' {
            body_start = index + 1;
        }
        if previous_byte == b'9' && *byte == b'=' {
            found_body_length_tag = true;
        }
        if previous_byte == b'0' && *byte == b'=' && message_bytes[index - 2] == b'1' && message_bytes[index - 3] == 1 {
            return index - 2 - body_start;
        }
        previous_byte = *byte;
    }

    panic!("Message is malformed.");
}

fn main() {
    define_dictionary!(
        Logon,
    );

    let message_bytes = b"8=FIXT.1.1\x019=136\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16.000\x0198=0\x01108=30\x0195=13\x0196=This\x01is=atest\x011137=4\x01384=2\x01372=Test\x01385=S\x01372=Test2\x01385=R\x0110=223\x01";

    let mut parser = Parser::new(build_dictionary(),4096);
    let (bytes_read,result) = parser.parse(message_bytes);
    assert!(result.is_ok());
    assert_eq!(bytes_read,message_bytes.len());

    let mut message1;
    match message_to_enum(&**(parser.messages.first().unwrap())) {
        MessageEnum::Logon(message) => {
            assert_eq!(message.encrypt_method,EncryptMethod::None);
            assert_eq!(message.heart_bt_int,30);
            assert_eq!(message.msg_seq_num,177);
            assert_eq!(message.sender_comp_id,b"SERVER");
            assert_eq!(message.target_comp_id,b"CLIENT");
            assert_eq!(message.sending_time,UTC.ymd(2009,1,7).and_hms(18,15,16));
            assert_eq!(message.raw_data,b"This\x01is=atest");
            assert_eq!(message.default_appl_ver_id,MessageVersion::FIX42);
            assert_eq!(message.no_msg_types.len(),2);

            let message_type_0 = &message.no_msg_types[0];
            assert_eq!(message_type_0.ref_msg_type,b"Test");
            assert_eq!(message_type_0.msg_direction,MsgDirection::Send);

            let message_type_1 = &message.no_msg_types[1];
            assert_eq!(message_type_1.ref_msg_type,b"Test2");
            assert_eq!(message_type_1.msg_direction,MsgDirection::Receive);

            message1 = message.clone();
        }
    }

    let mut serialized_bytes = Vec::new();
    message1.appl_ver_id = None;
    message1.read(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,&mut serialized_bytes);

    println!("{}",String::from_utf8_lossy(serialized_bytes.as_slice()).into_owned());
    println!("Compared to...");
    println!("{}",String::from_utf8_lossy(message_bytes).into_owned());

    let (bytes_read,result) = parser.parse(serialized_bytes.as_slice());
    assert!(result.is_ok());
    assert_eq!(bytes_read,message_bytes.len());

    match message_to_enum(&**(parser.messages.first().unwrap())) {
        MessageEnum::Logon(mut message) => {
            message.appl_ver_id = None;
            assert!(message1 == message);
        }
    }

    let client = Client::new(build_dictionary(),b"TEST_C",b"TEST_S",4096);
}

