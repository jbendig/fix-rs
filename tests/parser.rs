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

#![feature(attr_literals)]
#![allow(unknown_lints)]

extern crate chrono;
#[macro_use]
extern crate fix_rs;
#[macro_use]
extern crate fix_rs_macros;

use chrono::offset::utc::UTC;
use chrono::TimeZone;
use std::any::Any;
use std::collections::HashMap;

use fix_rs::byte_buffer::ByteBuffer;
use fix_rs::dictionary::field_types::generic::RepeatingGroupFieldType;
use fix_rs::dictionary::field_types::other::{EncryptMethod,RateSource,RateSourceType};
use fix_rs::dictionary::fields::{EncryptMethod as EncryptMethodField,HeartBtInt,MsgSeqNum,SendingTime,SenderCompID,TargetCompID,NoMsgTypeGrp,RawData,RawDataLength,Symbol,TestReqID,Text,OrigSendingTime,ClOrdID,AllocAccount,RateSource as RateSourceField,RateSourceType as RateSourceTypeField,ReferencePage as ReferencePageField};
use fix_rs::dictionary::messages::Heartbeat;
use fix_rs::field::Field;
use fix_rs::field_tag::{self,FieldTag};
use fix_rs::field_type::FieldType;
use fix_rs::fix::{Parser,ParseError};
use fix_rs::fix_version::FIXVersion;
use fix_rs::fixt;
use fix_rs::fixt::message::{BuildFIXTMessage,FIXTMessage,FIXTMessageBuildable};
use fix_rs::message::{self,MessageDetails,REQUIRED,NOT_REQUIRED};
use fix_rs::message_version::{self,MessageVersion};

const PARSE_MESSAGE_BY_STREAM: bool = true;
const MAX_MESSAGE_SIZE: u64 = 4096;

define_message!(LogonTest: b"L" => {
    REQUIRED, encrypt_method: EncryptMethodField [FIX40..],
    REQUIRED, heart_bt_int: HeartBtInt [FIX40..],
    REQUIRED, msg_seq_num: MsgSeqNum [FIX40..],
    NOT_REQUIRED, sending_time: SendingTime [FIX40..],
    NOT_REQUIRED, sender_comp_id: SenderCompID [FIX40..],
    NOT_REQUIRED, target_comp_id: TargetCompID [FIX40..],
    NOT_REQUIRED, raw_data_length: RawDataLength [FIX40..],
    NOT_REQUIRED, raw_data: RawData [FIX40..],
    NOT_REQUIRED, msg_type_grp: NoMsgTypeGrp [FIX40..],
    NOT_REQUIRED, text: Text [FIX40..],
});

impl FIXTMessage for LogonTest {
    fn new_into_box(&self) -> Box<FIXTMessage + Send> {
        Box::new(Self::new())
    }

    fn msg_type(&self) -> &'static [u8] {
        <LogonTest as MessageDetails>::msg_type()
    }

    fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
        self.msg_seq_num
    }

    fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn is_poss_dup(&self) -> bool {
        unimplemented!();
    }

    fn set_is_poss_dup(&mut self,_is_poss_dup: bool) {
        unimplemented!();
    }

    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn set_orig_sending_time(&mut self,_orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type) {
        unimplemented!();
    }

    fn setup_fixt_session_header(&mut self,
                                 _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                 _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                 _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
        unimplemented!();
    }
}

fn parse_message_with_ver<T: FIXTMessage + FIXTMessageBuildable + MessageDetails + Default + Any + Clone + PartialEq + Send>(fix_version: FIXVersion,message_version: MessageVersion,message: &[u8]) -> Result<T,ParseError> {
    let mut message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>> = HashMap::new();
    let builder: Box<BuildFIXTMessage + Send> = <T as Default>::default().builder();
    message_dictionary.insert(<T as MessageDetails>::msg_type(),builder);

    let mut parser = Parser::new(message_dictionary,MAX_MESSAGE_SIZE);

    let message_bytes = Vec::from(message);
    if PARSE_MESSAGE_BY_STREAM {
        //Stream in the message one byte at a time. This is a worst case scenario test to make sure all
        //everything tested works while streaming.
        for byte in &message_bytes {
            let mut message_bytes = Vec::new();
            message_bytes.push(*byte);

            let (_,result) = parser.parse(&message_bytes);
            if let Err(err) = result {
                return Err(err);
            }
        }
    }
    else {
        //Process the entire message in one call. This is the best case scenario which is more
        //useful when debugging new tests.
        let (_,result) = parser.parse(&message_bytes);
        if let Err(err) = result {
            return Err(err);
        }
    }

    assert_eq!(parser.messages.len(),1);
    let casted_message = parser.messages.first().unwrap().as_any().downcast_ref::<T>().unwrap().clone();

    //Serialize and parse again to help check for potential bugs in the serialization system.
    {
        let mut new_message_bytes = ByteBuffer::new();
        casted_message.read(fix_version,message_version,&mut new_message_bytes);
        parser.messages.clear();
        let(_,result) = parser.parse(new_message_bytes.bytes());
        if result.is_err() {
            println!("{:?}",result.as_ref().err().unwrap());
        }
        assert!(result.is_ok());
        assert_eq!(parser.messages.len(),1);

        let new_casted_message = parser.messages.first().unwrap().as_any().downcast_ref::<T>().unwrap().clone();
        if casted_message != new_casted_message {
            println!("");
            println!("{}",casted_message.debug(fix_version,message_version));
            println!("{}",new_casted_message.debug(fix_version,message_version));
        }
        assert!(casted_message == new_casted_message);
    }

    Ok(casted_message)
}

fn parse_message<T: FIXTMessage + FIXTMessageBuildable + MessageDetails + Default + Any + Clone + PartialEq + Send>(message: &[u8]) -> Result<T,ParseError> {
    parse_message_with_ver::<T>(FIXVersion::FIX_4_2,MessageVersion::FIX42,message)
}

#[test]
fn simple_test() {
    let message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";

    let message = parse_message::<LogonTest>(message).unwrap();
    assert_eq!(message.encrypt_method,EncryptMethod::None);
    assert_eq!(message.heart_bt_int,30);
    assert_eq!(message.msg_seq_num,177);
    assert_eq!(message.sending_time,UTC.ymd(2009,1,7).and_hms(18,15,16));
    assert_eq!(message.sender_comp_id,b"SERVER".to_vec());
    assert_eq!(message.target_comp_id,b"CLIENT".to_vec());
}

#[test]
fn body_length_second_tag_test() {
    let body_length_second_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    let message = parse_message::<LogonTest>(body_length_second_tag_message).unwrap();
    assert_eq!(message.meta.unwrap().body_length,65);

    let body_length_third_tag_message = b"8=FIX.4.2\x0135=L\x019=65\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let result = parse_message::<LogonTest>(body_length_third_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let missing_body_length_tag_message = b"8=FIX.4.2\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let result = parse_message::<LogonTest>(missing_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let negative_number_body_length_tag_message = b"8=FIX.4.2\x019=-65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let result = parse_message::<LogonTest>(negative_number_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotNumber => {},
        _ => assert!(false),
    }

    let nonnumber_number_body_length_tag_message = b"8=FIX.4.2\x019=TEXT\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let result = parse_message::<LogonTest>(nonnumber_number_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotNumber => {},
        _ => assert!(false),
    }
}

#[test]
fn msg_type_third_tag_test() {
    let msg_type_third_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    parse_message::<LogonTest>(msg_type_third_tag_message).unwrap();

    let msg_type_fourth_tag_message = b"8=FIX.4.2\x019=65\x0149=SERVER\x0135=L\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let result = parse_message::<LogonTest>(msg_type_fourth_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MsgTypeNotThirdTag => {},
        _ => assert!(false),
    }

    let missing_msg_type_tag_message = b"8=FIX.4.2\x019=65\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let result = parse_message::<LogonTest>(missing_msg_type_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MsgTypeNotThirdTag => {},
        _ => assert!(false),
    }
}

#[test]
fn sender_comp_id_fourth_tag_test() {
    //FIXT.1.1
    {
        let sender_comp_id_fourth_tag_message = b"8=FIXT.1.1\x019=52\x0135=0\x0149=SERVER\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=012\x01";
        parse_message::<Heartbeat>(sender_comp_id_fourth_tag_message).unwrap();

        let sender_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=52\x0135=0\x0156=CLIENT\x0149=SERVER\x0134=10\x0152=20170105-01:01:01\x0110=012\x01";
        let result = parse_message::<Heartbeat>(sender_comp_id_fifth_tag_message);
        match result.err().unwrap() {
            fix_rs::fix::ParseError::SenderCompIDNotFourthTag => {},
            _ => assert!(false),
        }

        let missing_sender_comp_id_tag_message = b"8=FIXT.1.1\x019=49\x0135=0\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=086\x01";
        let result = parse_message::<Heartbeat>(missing_sender_comp_id_tag_message);
        match result.err().unwrap() {
            fix_rs::fix::ParseError::SenderCompIDNotFourthTag => {},
            _ => assert!(false),
        }
    }

    //FIX.4.0
    {
        let sender_comp_id_fourth_tag_message = b"8=FIX.4.0\x019=52\x0135=0\x0149=SERVER\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=186\x01";
        parse_message::<Heartbeat>(sender_comp_id_fourth_tag_message).unwrap();

        let sender_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=52\x0135=0\x0156=CLIENT\x0149=SERVER\x0134=10\x0152=20170105-01:01:01\x0110=186\x01";
        parse_message::<Heartbeat>(sender_comp_id_fifth_tag_message).unwrap(); //Okay! Order doesn't matter here.

        let missing_sender_comp_id_tag_message = b"8=FIX.4.0\x019=42\x0135=0\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=055\x01";
        let result = parse_message::<Heartbeat>(missing_sender_comp_id_tag_message);
        match result.err().unwrap() {
            fix_rs::fix::ParseError::MissingRequiredTag(tag,_) => { assert_eq!(tag,FieldTag(49)); },
            _ => assert!(false),
        }
    }
}

#[test]
fn target_comp_id_fifth_tag_test() {
    //FIXT.1.1
    {
        let target_comp_id_fifth_tag_message = b"8=FIXT.1.1\x019=52\x0135=0\x0149=SERVER\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=012\x01";
        parse_message::<Heartbeat>(target_comp_id_fifth_tag_message).unwrap();

        let target_comp_id_sixth_tag_message = b"8=FIXT.1.1\x019=52\x0135=0\x0149=SERVER\x0134=10\x0156=CLIENT\x0152=20170105-01:01:01\x0110=012\x01";
        let result = parse_message::<Heartbeat>(target_comp_id_sixth_tag_message);
        match result.err().unwrap() {
            fix_rs::fix::ParseError::TargetCompIDNotFifthTag => {},
            _ => assert!(false),
        }

        let missing_target_comp_id_tag_message = b"8=FIXT.1.1\x019=49\x0135=0\x0149=SERVER\x0134=10\x0152=20170105-01:01:01\x0110=086\x01";
        let result = parse_message::<Heartbeat>(missing_target_comp_id_tag_message);
        match result.err().unwrap() {
            fix_rs::fix::ParseError::TargetCompIDNotFifthTag => {},
            _ => assert!(false),
        }
    }

    //FIX.4.0
    {
        let target_comp_id_fifth_tag_message = b"8=FIX.4.0\x019=52\x0135=0\x0149=SERVER\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=186\x01";
        parse_message::<Heartbeat>(target_comp_id_fifth_tag_message).unwrap();

        let target_comp_id_sixth_tag_message = b"8=FIX.4.0\x019=52\x0135=0\x0149=SERVER\x0134=10\x0156=CLIENT\x0152=20170105-01:01:01\x0110=186\x01";
        parse_message::<Heartbeat>(target_comp_id_sixth_tag_message).unwrap(); //Okay! Order doesn't matter here.

        let missing_target_comp_id_tag_message = b"8=FIX.4.0\x019=42\x0135=0\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=055\x01";
        let result = parse_message::<Heartbeat>(missing_target_comp_id_tag_message);
        match result.err().unwrap() {
            fix_rs::fix::ParseError::MissingRequiredTag(tag,_) => { assert_eq!(tag,FieldTag(49)); },
            _ => assert!(false),
        }
    }
}

#[test]
fn appl_ver_id_sixth_tag_test() {
    define_fixt_message!(TestMessage: b"9999" => {
        NOT_REQUIRED, test_req_id: TestReqID [FIX50..],
    });

    let appl_ver_id_sixth_tag_message = b"8=FIXT.1.1\x019=62\x0135=9999\x0149=SERVER\x0156=CLIENT\x011128=9\x0134=10\x0152=20170105-01:01:01\x0110=004\x01";
    parse_message_with_ver::<TestMessage>(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,appl_ver_id_sixth_tag_message).unwrap();

    let appl_ver_id_fourth_tag_message = b"8=FIXT.1.1\x019=34\x0135=9999\x011128=9\x0156=CLIENT\x01112=Test\x0110=000\x01";
    let result = parse_message::<TestMessage>(appl_ver_id_fourth_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::SenderCompIDNotFourthTag => {},
        _ => assert!(false),
    }

    let appl_ver_id_seventh_tag_message = b"8=FIXT.1.1\x019=44\x0135=9999\x0149=SERVER\x0156=CLIENT\x01112=Test\x011128=9\x0110=000\x01";
    let result = parse_message::<TestMessage>(appl_ver_id_seventh_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ApplVerIDNotSixthTag => {},
        _ => assert!(false),
    }

    let missing_appl_ver_id_tag_message = b"8=FIXT.1.1\x019=55\x0135=9999\x0149=SERVER\x0156=CLIENT\x0134=10\x0152=20170105-01:01:01\x0110=195\x01";
    parse_message_with_ver::<TestMessage>(FIXVersion::FIXT_1_1,MessageVersion::FIX50SP2,missing_appl_ver_id_tag_message).unwrap();
}

#[test]
fn checksum_tag_test() {
    let valid_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    let message = parse_message::<LogonTest>(valid_checksum_tag_message).unwrap();
    assert_eq!(message.meta.unwrap().checksum,73);

    let incorrect_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=000\x01";
    let result = parse_message::<LogonTest>(incorrect_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumDoesNotMatch(calculated_checksum,stated_checksum) => {
            assert_eq!(calculated_checksum,73);
            assert_eq!(stated_checksum,0);
        },
        _ => assert!(false),
    }

    let negative_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=-62\x01";
    let result = parse_message::<LogonTest>(negative_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumWrongFormat => {},
        _ => assert!(false),
    }

    let two_char_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=99\x01";
    let result = parse_message::<LogonTest>(two_char_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumWrongFormat => {},
        _ => assert!(false),
    }

    let one_char_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=9\x01";
    let result = parse_message::<LogonTest>(one_char_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumWrongFormat => {},
        _ => assert!(false),
    }

    let empty_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=\x01";
    let result = parse_message::<LogonTest>(empty_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NoValueAfterTag(tag) => { assert_eq!(tag,FieldTag(10)); },
        _ => assert!(false),
    }

    let nonnumber_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=TST\x01";
    let result = parse_message::<LogonTest>(nonnumber_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumWrongFormat => {},
        _ => assert!(false),
    }

    let early_checksum_tag_message = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x0110=TST\x01108=30\x01";
    let result = parse_message::<LogonTest>(early_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }

    let late_checksum_tag_message = b"8=FIX.4.2\x019=58\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=TST\x01";
    let result = parse_message::<LogonTest>(late_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }
}

#[test]
fn duplicate_tag_test() {
    //TODO: Duplicate tags for string lists should be grouped up instead.

    let duplicate_tag_message = b"8=FIX.4.2\x019=70\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x0198=0\x01108=30\x0110=040\x01";
    let result = parse_message::<LogonTest>(duplicate_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::DuplicateTag(tag) => assert_eq!(tag,FieldTag(98)),
        _ => assert!(false),
    }
}

#[test]
fn length_tag_test() {
    define_message!(LengthTagTestMessage: b"L" => {
        REQUIRED, raw_data_length: RawDataLength [FIX40..],
        REQUIRED, raw_data: RawData [FIX40..],
    });

    impl FIXTMessage for LengthTagTestMessage {
        fn new_into_box(&self) -> Box<FIXTMessage + Send> {
            Box::new(Self::new())
        }

        fn msg_type(&self) -> &'static [u8] {
            b"L"
        }

        fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn is_poss_dup(&self) -> bool {
            unimplemented!();
        }

        fn set_is_poss_dup(&mut self,_is_poss_dup: bool) {
            unimplemented!();
        }

        fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn set_orig_sending_time(&mut self,_orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type) {
            unimplemented!();
        }

        fn setup_fixt_session_header(&mut self,
                                     _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                     _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                     _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
            unimplemented!();
        }
    }

    impl BuildFIXTMessage for BuildLengthTagTestMessage {
        fn new_into_box(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
            Box::new(BuildLengthTagTestMessage::new())
        }

        fn build(&self) -> Box<fixt::message::FIXTMessage + Send> {
            Box::new(LengthTagTestMessage::new())
        }
    }

    impl FIXTMessageBuildable for LengthTagTestMessage {
        fn builder(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
            Box::new(BuildLengthTagTestMessage::new())
        }
    }

    let valid_length_tag_message = b"8=FIX.4.2\x019=28\x0135=L\x0195=13\x0196=This\x01is=atest\x0110=130\x01";
    let message = parse_message::<LengthTagTestMessage>(valid_length_tag_message).unwrap();
    assert_eq!(message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
    assert_eq!(message.meta.clone().unwrap().body_length,28);
    assert_eq!(message.meta.clone().unwrap().checksum,130);
    assert_eq!(message.raw_data,b"This\x01is=atest");

    let missing_length_tag_message = b"8=FIX.4.2\x019=28\x0135=L\x0196=This\x01is=atest\x0110=190\x01";
    let result = parse_message::<LengthTagTestMessage>(missing_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingPrecedingLengthTag(value_tag) => assert_eq!(value_tag,FieldTag(96)),
        _ => assert!(false),
    }

    let late_length_tag_message = b"8=FIX.4.2\x019=28\x0135=L\x0196=This\x01is=atest\x0195=13\x0110=190\x01";
    let result = parse_message::<LengthTagTestMessage>(late_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingPrecedingLengthTag(value_tag) => assert_eq!(value_tag,FieldTag(96)),
        _ => assert!(false),
    }

    let early_length_tag_message = b"8=FIX.4.2\x019=28\x0135=L\x0195=13\x0156=CLIENT\x0196=This\x01is=atest\x0110=190\x01";
    let result = parse_message::<LengthTagTestMessage>(early_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingFollowingLengthTag(length_tag) => assert_eq!(length_tag,FieldTag(95)),
        _ => assert!(false),
    }
}

#[test]
fn repeating_groups_test() {
    define_fields!(
        NoRateSources: RepeatingGroupFieldType<RateSourceGrp> = 1445,
    );

    define_message!(RateSourceGrp {
        REQUIRED, rate_source: RateSourceField [FIX40..],
        REQUIRED, rate_source_type: RateSourceTypeField [FIX40..],
        NOT_REQUIRED, reference_page: ReferencePageField [FIX40..],
    });

    define_message!(RepeatingGroupsTestMessage: b"L" => {
        NOT_REQUIRED, rate_sources: NoRateSources [FIX40..],
        NOT_REQUIRED, symbol: Symbol [FIX40..],
    });

    impl FIXTMessage for RepeatingGroupsTestMessage {
        fn new_into_box(&self) -> Box<FIXTMessage + Send> {
            Box::new(Self::new())
        }

        fn msg_type(&self) -> &'static [u8] {
            b"L"
        }

        fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn is_poss_dup(&self) -> bool {
            unimplemented!();
        }

        fn set_is_poss_dup(&mut self,_is_poss_dup: bool) {
            unimplemented!();
        }

        fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn set_orig_sending_time(&mut self,_orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type) {
            unimplemented!();
        }

        fn setup_fixt_session_header(&mut self,
                                     _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                     _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                     _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
            unimplemented!();
        }
    }

    impl BuildFIXTMessage for BuildRepeatingGroupsTestMessage {
        fn new_into_box(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
            Box::new(BuildRepeatingGroupsTestMessage::new())
        }

        fn build(&self) -> Box<fixt::message::FIXTMessage + Send> {
            Box::new(RepeatingGroupsTestMessage::new())
        }
    }

    impl FIXTMessageBuildable for RepeatingGroupsTestMessage {
        fn builder(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
            Box::new(BuildRepeatingGroupsTestMessage::new())
        }
    }

    let no_repeating_groups_message = b"8=FIX.4.2\x019=12\x0135=L\x011445=0\x0110=039\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(no_repeating_groups_message).unwrap();
    assert_eq!(message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
    assert_eq!(message.meta.clone().unwrap().body_length,12);
    assert_eq!(message.meta.clone().unwrap().checksum,39);
    assert_eq!(message.rate_sources.len(),0);

    let one_repeating_group_message = b"8=FIX.4.2\x019=26\x0135=L\x011445=1\x011446=0\x011447=0\x0110=168\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(one_repeating_group_message).unwrap();
    assert_eq!(message.rate_sources.len(),1);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,RateSource::Bloomberg);
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,RateSourceType::Primary);

    let one_repeating_group_with_optional_message = b"8=FIX.4.2\x019=43\x0135=L\x011445=1\x011446=99\x011447=0\x011448=SomeSource\x0110=253\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(one_repeating_group_with_optional_message).unwrap();
    assert_eq!(message.rate_sources.len(),1);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,RateSource::Other);
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,RateSourceType::Primary);
    assert_eq!(message.rate_sources.first().unwrap().reference_page,b"SomeSource".to_vec());

    let two_repeating_groups_message = b"8=FIX.4.2\x019=40\x0135=L\x011445=2\x011446=0\x011447=0\x011446=1\x011447=1\x0110=034\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,RateSource::Bloomberg);
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,RateSourceType::Primary);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,RateSource::Reuters);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,RateSourceType::Secondary);

    let two_repeating_groups_with_optional_first_message = b"8=FIX.4.2\x019=57\x0135=L\x011445=2\x011446=99\x011447=0\x011448=SomeSource\x011446=1\x011447=1\x0110=128\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_with_optional_first_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,RateSource::Other);
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,RateSourceType::Primary);
    assert_eq!(message.rate_sources.first().unwrap().reference_page,b"SomeSource".to_vec());
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,RateSource::Reuters);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,RateSourceType::Secondary);

    let two_repeating_groups_with_optional_second_message = b"8=FIX.4.2\x019=57\x0135=L\x011445=2\x011446=0\x011447=0\x011446=99\x011447=1\x011448=SomeSource\x0110=127\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_with_optional_second_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,RateSource::Bloomberg);
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,RateSourceType::Primary);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,RateSource::Other);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,RateSourceType::Secondary);
    assert_eq!(message.rate_sources.get(1).unwrap().reference_page,b"SomeSource".to_vec());

    let two_repeating_groups_not_body_end_message = b"8=FIX.4.2\x019=66\x0135=L\x011445=2\x011446=0\x011447=0\x011446=99\x011447=1\x011448=SomeSource\x0155=[N/A]\x0110=157\x01";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_not_body_end_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,RateSource::Bloomberg);
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,RateSourceType::Primary);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,RateSource::Other);
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,RateSourceType::Secondary);
    assert_eq!(message.rate_sources.get(1).unwrap().reference_page,b"SomeSource".to_vec());
    assert_eq!(message.symbol,b"[N/A]".to_vec());

    let missing_one_repeating_group_message = b"8=FIX.4.2\x019=35\x0135=L\x011445=2\x011446=0\x011447=0\x0155=[N/A]\x0110=244\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(missing_one_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,FieldTag(55)),
        _ => assert!(false),
    }

    let extra_one_repeating_group_message = b"8=FIX.4.2\x019=67\x0135=L\x011445=1\x011446=0\x011447=0\x011446=99\x011447=1\x011448=SomeSource\x0155=[N/A]\x0110=244\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(extra_one_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::RepeatingGroupTagWithNoRepeatingGroup(tag) => assert_eq!(tag,FieldTag(1446)),
        _ => assert!(false),
    }

    let non_repeating_group_tag_in_repeating_group_message = b"8=FIX.4.2\x019=66\x0135=L\x011445=2\x011446=0\x011447=0\x0155=[N/A]\x011446=99\x011447=1\x011448=SomeSource\x0110=145\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(non_repeating_group_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,FieldTag(55)),
        _ => assert!(false),
    }

    let wrong_first_tag_in_repeating_group_message = b"8=FIX.4.2\x019=43\x0135=L\x011445=1\x011447=0\x011446=99\x011448=SomeSource\x0110=244\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(wrong_first_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(number_of_tag) => assert_eq!(number_of_tag,FieldTag(1445)),
        _ => assert!(false),
    }

    let wrong_first_tag_in_second_repeating_group_message = b"8=FIX.4.2\x019=40\x0135=L\x011445=2\x011446=0\x011447=0\x011447=1\x011446=1\x0110=244\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(wrong_first_tag_in_second_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::DuplicateTag(tag) => assert_eq!(tag,FieldTag(1447)),
        _ => assert!(false),
    }

    let missing_required_tag_in_repeating_group_message = b"8=FIX.4.2\x019=19\x0135=L\x011445=1\x011446=0\x0110=108\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(missing_required_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredTag(tag,_) => assert_eq!(tag,FieldTag(1447)),
        _ => assert!(false),
    }

    let missing_required_tag_in_first_repeating_group_message = b"8=FIX.4.2\x019=33\x0135=L\x011445=2\x011446=0\x011446=1\x011447=1\x0110=230\x01";
    let result = parse_message::<RepeatingGroupsTestMessage>(missing_required_tag_in_first_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredTag(tag,_) => assert_eq!(tag,FieldTag(1447)),
        _ => assert!(false),
    }
}

#[test]
fn nested_repeating_groups_test() {
    define_fields!(
        NoOrders: RepeatingGroupFieldType<Order> = 73,
        NoAllocs: RepeatingGroupFieldType<Alloc> = 78,
    );

    define_message!(Alloc {
        REQUIRED, alloc_account: AllocAccount [FIX40..],
    });

    define_message!(Order {
        REQUIRED, cl_ord_id: ClOrdID [FIX40..],
        NOT_REQUIRED, allocs: NoAllocs [FIX40..],
    });

    define_message!(NestedRepeatingGroupsTestMessage: b"L" => {
        REQUIRED, orders: NoOrders [FIX40..],
    });

    impl FIXTMessage for NestedRepeatingGroupsTestMessage {
        fn new_into_box(&self) -> Box<FIXTMessage + Send> {
            Box::new(Self::new())
        }

        fn msg_type(&self) -> &'static [u8] {
            b"L"
        }

        fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn is_poss_dup(&self) -> bool {
            unimplemented!();
        }

        fn set_is_poss_dup(&mut self,_is_poss_dup: bool) {
            unimplemented!();
        }

        fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
            unimplemented!();
        }

        fn set_orig_sending_time(&mut self,_orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type) {
            unimplemented!();
        }

        fn setup_fixt_session_header(&mut self,
                                     _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                     _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                     _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
            unimplemented!();
        }
    }

    impl BuildFIXTMessage for BuildNestedRepeatingGroupsTestMessage {
        fn new_into_box(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
            Box::new(BuildNestedRepeatingGroupsTestMessage::new())
        }

        fn build(&self) -> Box<fixt::message::FIXTMessage + Send> {
            Box::new(NestedRepeatingGroupsTestMessage::new())
        }
    }

    impl FIXTMessageBuildable for NestedRepeatingGroupsTestMessage {
        fn builder(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
            Box::new(BuildNestedRepeatingGroupsTestMessage::new())
        }
    }

    let one_nested_repeating_group_message = b"8=FIX.4.2\x019=35\x0135=L\x0173=1\x0111=uniqueid\x0178=1\x0179=acct\x0110=244\x01";
    let message = parse_message::<NestedRepeatingGroupsTestMessage>(one_nested_repeating_group_message).unwrap();
    assert_eq!(message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
    assert_eq!(message.meta.clone().unwrap().body_length,35);
    assert_eq!(message.meta.clone().unwrap().checksum,244);
    assert_eq!(message.orders.len(),1);
    assert_eq!(message.orders.first().unwrap().cl_ord_id,b"uniqueid".to_vec());
    assert_eq!(message.orders.first().unwrap().allocs.len(),1);
    assert_eq!(message.orders.first().unwrap().allocs.first().unwrap().alloc_account,b"acct".to_vec());
}

#[test]
fn stream_test() {
    define_dictionary!(
        LogonTest,
    );

    let two_messages = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x018=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    let mut parser = Parser::new(build_dictionary(),MAX_MESSAGE_SIZE);
    let (bytes_read,result) = parser.parse(&two_messages.to_vec());
    assert!(result.is_ok());
    assert_eq!(bytes_read,two_messages.len());
    assert_eq!(parser.messages.len(),2);
    for message in parser.messages {
        let casted_message = message.as_any().downcast_ref::<LogonTest>().unwrap();
        assert_eq!(casted_message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
        assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
        assert_eq!(casted_message.meta.clone().unwrap().checksum,73);
        assert_eq!(casted_message.sender_comp_id,b"SERVER".to_vec());
        assert_eq!(casted_message.target_comp_id,b"CLIENT".to_vec());
        assert_eq!(casted_message.msg_seq_num,177);
        assert_eq!(casted_message.sending_time,UTC.ymd(2009,1,7).and_hms(18,15,16));
        assert_eq!(casted_message.encrypt_method,EncryptMethod::None);
        assert_eq!(casted_message.heart_bt_int,30);
    }

    let garbage_before_message = b"garbage\x01before=message8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    let mut parser = Parser::new(build_dictionary(),MAX_MESSAGE_SIZE);
    let (bytes_read,result) = parser.parse(&garbage_before_message.to_vec());
    assert_eq!(bytes_read,garbage_before_message.len());
    assert!(result.is_ok());
    let casted_message = parser.messages.first().unwrap().as_any().downcast_ref::<LogonTest>().unwrap();
    assert_eq!(casted_message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
    assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
    assert_eq!(casted_message.meta.clone().unwrap().checksum,73);
    assert_eq!(casted_message.sender_comp_id,b"SERVER".to_vec());
    assert_eq!(casted_message.target_comp_id,b"CLIENT".to_vec());
    assert_eq!(casted_message.msg_seq_num,177);
    assert_eq!(casted_message.sending_time,UTC.ymd(2009,1,7).and_hms(18,15,16));
    assert_eq!(casted_message.encrypt_method,EncryptMethod::None);
    assert_eq!(casted_message.heart_bt_int,30);

    let garbage_between_messages = b"8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01garbage=before\x01m8ssage8=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    let mut parser = Parser::new(build_dictionary(),MAX_MESSAGE_SIZE);
    let (bytes_read,result) = parser.parse(&garbage_between_messages.to_vec());
    assert!(result.is_ok());
    assert_eq!(bytes_read,garbage_between_messages.len());
    assert_eq!(parser.messages.len(),2);
    for message in parser.messages {
        let casted_message = message.as_any().downcast_ref::<LogonTest>().unwrap();
        assert_eq!(casted_message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
        assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
        assert_eq!(casted_message.meta.clone().unwrap().checksum,73);
        assert_eq!(casted_message.sender_comp_id,b"SERVER".to_vec());
        assert_eq!(casted_message.target_comp_id,b"CLIENT".to_vec());
        assert_eq!(casted_message.msg_seq_num,177);
        assert_eq!(casted_message.sending_time,UTC.ymd(2009,1,7).and_hms(18,15,16));
        assert_eq!(casted_message.encrypt_method,EncryptMethod::None);
        assert_eq!(casted_message.heart_bt_int,30);
    }

    let invalid_message_before_valid_message = b"8=FIX.4.2\x0110=0\x018=FIX.4.2\x019=65\x0135=L\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=073\x01";
    let mut parser = Parser::new(build_dictionary(),MAX_MESSAGE_SIZE);
    let (bytes_read_failure,result) = parser.parse(&invalid_message_before_valid_message.to_vec());
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }
    let (bytes_read_success,result) = parser.parse(&invalid_message_before_valid_message[bytes_read_failure..].to_vec());
    assert!(result.is_ok());
    assert_eq!(bytes_read_failure + bytes_read_success,invalid_message_before_valid_message.len());
    assert_eq!(parser.messages.len(),1);
    let casted_message = parser.messages.first().unwrap().as_any().downcast_ref::<LogonTest>().unwrap();
    assert_eq!(casted_message.meta.clone().unwrap().begin_string,FIXVersion::FIX_4_2);
    assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
    assert_eq!(casted_message.meta.clone().unwrap().checksum,73);
    assert_eq!(casted_message.sender_comp_id,b"SERVER".to_vec());
    assert_eq!(casted_message.target_comp_id,b"CLIENT".to_vec());
    assert_eq!(casted_message.msg_seq_num,177);
    assert_eq!(casted_message.sending_time,UTC.ymd(2009,1,7).and_hms(18,15,16));
    assert_eq!(casted_message.encrypt_method,EncryptMethod::None);
    assert_eq!(casted_message.heart_bt_int,30);
}

#[test]
fn equal_character_in_text_test() {
    let message = b"8=FIX.4.2\x019=37\x0135=L\x0134=177\x0198=0\x01108=30\x0158=some=text\x0110=176\x01";

    let message = parse_message::<LogonTest>(message).unwrap();
    assert_eq!(message.encrypt_method,EncryptMethod::None);
    assert_eq!(message.heart_bt_int,30);
    assert_eq!(message.msg_seq_num,177);
    assert_eq!(message.text,b"some=text".to_vec());
}

#[test]
fn no_value_after_tag_test() {
    let message = b"8=FIX.4.2\x019=37\x0135=L\x0134=\x0198=0\x01108=30\x0158=some=text\x0110=165\x01";

    let result = parse_message::<LogonTest>(message);
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NoValueAfterTag(tag) => assert_eq!(tag,FieldTag(34)),
        _ => assert!(false),
    }
}

