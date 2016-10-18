#[macro_use]
extern crate fix_rs;

use std::collections::HashMap;
use fix_rs::fix::Parser;
use fix_rs::message::Message;
use fix_rs::dictionary::Logon;

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
        b"A" => Logon : Logon,
    );

    let message_bytes = b"8=FIX.4.2\x019=132\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0195=13\x0196=This\x01is=atest\x011137=4\x01384=2\x01372=Test\x01385=A\x01372=Test2\x01385=B\x0110=171\x01";

    let mut parser = Parser::new(build_dictionary());
    let (bytes_read,result) = parser.parse(message_bytes);
    assert!(result.is_ok());
    assert_eq!(bytes_read,message_bytes.len());

    let message1;
    let mut serialized_bytes = Vec::new();
    match message_to_enum(&**(parser.messages.first().unwrap())) {
        MessageEnum::Logon(message) => {
            assert_eq!(*message.encrypt_method,"0");
            assert_eq!(*message.heart_bt_int,"30");
            assert_eq!(*message.msg_seq_num,"177");
            assert_eq!(*message.sender_comp_id,"SERVER");
            assert_eq!(*message.target_comp_id,"CLIENT");
            assert_eq!(*message.sending_time,"20090107-18:15:16");
            assert_eq!(*message.raw_data,b"This\x01is=atest");
            assert_eq!(*message.default_appl_ver_id,"4");
            assert_eq!(message.msg_type_grp.len(),2);

            let message_type_0 = &message.msg_type_grp[0];
            assert_eq!(*message_type_0.ref_msg_type,"Test");
            assert_eq!(*message_type_0.msg_direction,"A");

            let message_type_1 = &message.msg_type_grp[1];
            assert_eq!(*message_type_1.ref_msg_type,"Test2");
            assert_eq!(*message_type_1.msg_direction,"B");

            message1 = Some(message.clone());
        }
    }

    let message1 = message1.unwrap();
    message1.read(&mut serialized_bytes);

    println!("{}",String::from_utf8_lossy(serialized_bytes.as_slice()).into_owned());
    println!("Compared to...");
    println!("{}",String::from_utf8_lossy(message_bytes).into_owned());

    let (bytes_read,result) = parser.parse(serialized_bytes.as_slice());
    assert!(result.is_ok());
    assert_eq!(bytes_read,message_bytes.len());

    match message_to_enum(&**(parser.messages.first().unwrap())) {
        MessageEnum::Logon(message) => {
            assert!(message1 == message);
        }
    }
}

