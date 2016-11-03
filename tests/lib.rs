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

#![allow(unknown_lints)]

#[macro_use]
extern crate fix_rs;

use std::any::Any;
use std::collections::HashMap;

use fix_rs::dictionary::{EncryptMethod,HeartBtInt,MsgSeqNum,SendingTime,SenderCompID,TargetCompID,NoMsgTypeGrp,RawData,RawDataLength,NoRateSources,Symbol,NoOrders};
use fix_rs::fix::{Parser,ParseError};
use fix_rs::message::{Message,REQUIRED,NOT_REQUIRED};

const PARSE_MESSAGE_BY_STREAM : bool = true;

define_message!(LogonTest {
    REQUIRED, encrypt_method: EncryptMethod,
    REQUIRED, heart_bt_int: HeartBtInt,
    REQUIRED, msg_seq_num: MsgSeqNum,
    NOT_REQUIRED, sending_time: SendingTime,
    NOT_REQUIRED, sender_comp_id: SenderCompID,
    NOT_REQUIRED, target_comp_id: TargetCompID,
    NOT_REQUIRED, raw_data_length: RawDataLength,
    NOT_REQUIRED, raw_data: RawData,
    NOT_REQUIRED, msg_type_grp: NoMsgTypeGrp,
});

fn parse_message<T: Message + Default + Any + Clone + PartialEq>(message: &str) -> Result<T,ParseError> {
    let mut message_dictionary: HashMap<&'static [u8],Box<Message>> = HashMap::new();
    message_dictionary.insert(b"A",Box::new(<T as Default>::default()));

    let mut parser = Parser::new(message_dictionary);

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
        let mut new_message_bytes = Vec::new();
        casted_message.read(&mut new_message_bytes);
        parser.messages.clear();
        let(_,result) = parser.parse(&new_message_bytes);
        assert!(result.is_ok());
        assert_eq!(parser.messages.len(),1);

        let new_casted_message = parser.messages.first().unwrap().as_any().downcast_ref::<T>().unwrap();
        assert!(casted_message == *new_casted_message);
    }

    Ok(casted_message)
}

#[test]
fn simple_test() {
    let message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";

    let message = parse_message::<LogonTest>(message).unwrap();
    assert_eq!(message.encrypt_method,"0");
    assert_eq!(message.heart_bt_int,"30");
    assert_eq!(message.msg_seq_num,"177");
    assert_eq!(message.sending_time,"20090107-18:15:16");
    assert_eq!(message.sender_comp_id,"SERVER");
    assert_eq!(message.target_comp_id,"CLIENT");
}

#[test]
fn body_length_second_tag_test() {
    let body_length_second_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let message = parse_message::<LogonTest>(body_length_second_tag_message).unwrap();
    assert_eq!(message.meta.unwrap().body_length,65);

    let body_length_third_tag_message = "8=FIX.4.2\u{1}35=A\u{1}9=65\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = parse_message::<LogonTest>(body_length_third_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let missing_body_length_tag_message = "8=FIX.4.2\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = parse_message::<LogonTest>(missing_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let negative_number_body_length_tag_message = "8=FIX.4.2\u{1}9=-65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = parse_message::<LogonTest>(negative_number_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotNumber => {},
        _ => assert!(false),
    }

    let nonnumber_number_body_length_tag_message = "8=FIX.4.2\u{1}9=TEXT\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = parse_message::<LogonTest>(nonnumber_number_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotNumber => {},
        _ => assert!(false),
    }
}

#[test]
fn msg_type_third_tag_test() {
    let msg_type_third_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    parse_message::<LogonTest>(msg_type_third_tag_message).unwrap();

    let msg_type_fourth_tag_message = "8=FIX.4.2\u{1}9=65\u{1}49=SERVER\u{1}35=A\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = parse_message::<LogonTest>(msg_type_fourth_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MsgTypeNotThirdTag => {},
        _ => assert!(false),
    }

    let missing_msg_type_tag_message = "8=FIX.4.2\u{1}9=65\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = parse_message::<LogonTest>(missing_msg_type_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MsgTypeNotThirdTag => {},
        _ => assert!(false),
    }
}

#[test]
fn checksum_tag_test() {
    let valid_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let message = parse_message::<LogonTest>(valid_checksum_tag_message).unwrap();
    assert_eq!(message.meta.unwrap().checksum,62);

    let incorrect_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=0\u{1}";
    let result = parse_message::<LogonTest>(incorrect_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumDoesNotMatch(calculated_checksum,stated_checksum) => {
            assert_eq!(calculated_checksum,62);
            assert_eq!(stated_checksum,0);
        },
        _ => assert!(false),
    }

    let negative_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=-62\u{1}";
    let result = parse_message::<LogonTest>(negative_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotNumber => {},
        _ => assert!(false),
    }

    let nonnumber_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=TST\u{1}";
    let result = parse_message::<LogonTest>(nonnumber_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotNumber => {},
        _ => assert!(false),
    }

    let early_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}10=TST\u{1}108=30\u{1}";
    let result = parse_message::<LogonTest>(early_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }

    let late_checksum_tag_message = "8=FIX.4.2\u{1}9=58\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=TST\u{1}";
    let result = parse_message::<LogonTest>(late_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }
}

/*TODO: Duplicate tag test. */

#[test]
fn length_tag_test() {
    define_message!(LengthTagTestMessage {
        REQUIRED, raw_data_length: RawDataLength,
        REQUIRED, raw_data: RawData,
    });

    let valid_length_tag_message = "8=FIX.4.2\u{1}9=28\u{1}35=A\u{1}95=13\u{1}96=This\u{1}is=atest\u{1}10=119\u{1}";
    let message = parse_message::<LengthTagTestMessage>(valid_length_tag_message).unwrap();
    assert_eq!(message.meta.clone().unwrap().protocol,b"FIX.4.2");
    assert_eq!(message.meta.clone().unwrap().body_length,28);
    assert_eq!(message.meta.clone().unwrap().checksum,119);
    assert_eq!(message.raw_data,b"This\x01is=atest");

    let missing_length_tag_message = "8=FIX.4.2\u{1}9=28\u{1}35=A\u{1}96=This\u{1}is=atest\u{1}10=190\u{1}";
    let result = parse_message::<LengthTagTestMessage>(missing_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingPrecedingLengthTag(value_tag) => assert_eq!(value_tag,b"96"),
        _ => assert!(false),
    }

    let late_length_tag_message = "8=FIX.4.2\u{1}9=28\u{1}35=A\u{1}96=This\u{1}is=atest\u{1}95=13\u{1}10=190\u{1}";
    let result = parse_message::<LengthTagTestMessage>(late_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingPrecedingLengthTag(value_tag) => assert_eq!(value_tag,b"96"),
        _ => assert!(false),
    }

    let early_length_tag_message = "8=FIX.4.2\u{1}9=28\u{1}35=A\u{1}95=13\u{1}56=CLIENT\u{1}96=This\u{1}is=atest\u{1}10=190\u{1}";
    let result = parse_message::<LengthTagTestMessage>(early_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingFollowingLengthTag(length_tag) => assert_eq!(length_tag,b"95"),
        _ => assert!(false),
    }
}

#[test]
fn repeating_groups_test() {
    define_message!(RepeatingGroupsTestMessage {
        NOT_REQUIRED, rate_sources: NoRateSources,
        NOT_REQUIRED, symbol: Symbol,
    });

    let no_repeating_groups_message = "8=FIX.4.2\u{1}9=12\u{1}35=A\u{1}1445=0\u{1}10=28\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(no_repeating_groups_message).unwrap();
    assert_eq!(message.meta.clone().unwrap().protocol,b"FIX.4.2");
    assert_eq!(message.meta.clone().unwrap().body_length,12);
    assert_eq!(message.meta.clone().unwrap().checksum,28);
    assert_eq!(message.rate_sources.len(),0);

    let one_repeating_group_message = "8=FIX.4.2\u{1}9=26\u{1}35=A\u{1}1445=1\u{1}1446=0\u{1}1447=0\u{1}10=157\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(one_repeating_group_message).unwrap();
    assert_eq!(message.rate_sources.len(),1);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,"0");
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,"0");

    let one_repeating_group_with_optional_message = "8=FIX.4.2\u{1}9=43\u{1}35=A\u{1}1445=1\u{1}1446=99\u{1}1447=0\u{1}1448=SomeSource\u{1}10=242\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(one_repeating_group_with_optional_message).unwrap();
    assert_eq!(message.rate_sources.len(),1);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,"99");
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,"0");
    assert_eq!(message.rate_sources.first().unwrap().reference_page,"SomeSource");

    let two_repeating_groups_message = "8=FIX.4.2\u{1}9=40\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1446=1\u{1}1447=1\u{1}10=23\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,"0");
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,"0");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,"1");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,"1");

    let two_repeating_groups_with_optional_first_message = "8=FIX.4.2\u{1}9=57\u{1}35=A\u{1}1445=2\u{1}1446=99\u{1}1447=0\u{1}1448=SomeSource\u{1}1446=1\u{1}1447=1\u{1}10=117\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_with_optional_first_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,"99");
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,"0");
    assert_eq!(message.rate_sources.first().unwrap().reference_page,"SomeSource");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,"1");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,"1");

    let two_repeating_groups_with_optional_second_message = "8=FIX.4.2\u{1}9=57\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}10=116\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_with_optional_second_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,"0");
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,"0");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,"99");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,"1");
    assert_eq!(message.rate_sources.get(1).unwrap().reference_page,"SomeSource");

    let two_repeating_groups_not_body_end_message = "8=FIX.4.2\u{1}9=66\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}55=[N/A]\u{1}10=146\u{1}";
    let message = parse_message::<RepeatingGroupsTestMessage>(two_repeating_groups_not_body_end_message).unwrap();
    assert_eq!(message.rate_sources.len(),2);
    assert_eq!(message.rate_sources.first().unwrap().rate_source,"0");
    assert_eq!(message.rate_sources.first().unwrap().rate_source_type,"0");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source,"99");
    assert_eq!(message.rate_sources.get(1).unwrap().rate_source_type,"1");
    assert_eq!(message.rate_sources.get(1).unwrap().reference_page,"SomeSource");
    assert_eq!(message.symbol,"[N/A]");

    let missing_one_repeating_group_message = "8=FIX.4.2\u{1}9=35\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}55=[N/A]\u{1}10=244\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(missing_one_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,b"55"),
        _ => assert!(false),
    }

    let extra_one_repeating_group_message = "8=FIX.4.2\u{1}9=67\u{1}35=A\u{1}1445=1\u{1}1446=0\u{1}1447=0\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}55=[N/A]\u{1}10=244\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(extra_one_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::RepeatingGroupTagWithNoRepeatingGroup(tag) => assert_eq!(tag,b"1446"),
        _ => assert!(false),
    }

    let non_repeating_group_tag_in_repeating_group_message = "8=FIX.4.2\u{1}9=66\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}55=[N/A]\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}10=145\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(non_repeating_group_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,b"55"),
        _ => assert!(false),
    }

    let wrong_first_tag_in_repeating_group_message = "8=FIX.4.2\u{1}9=43\u{1}35=A\u{1}1445=1\u{1}1447=0\u{1}1446=99\u{1}1448=SomeSource\u{1}10=244\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(wrong_first_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(number_of_tag) => assert_eq!(number_of_tag,b"1445"),
        _ => assert!(false),
    }

    let wrong_first_tag_in_second_repeating_group_message = "8=FIX.4.2\u{1}9=40\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1447=1\u{1}1446=1\u{1}10=244\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(wrong_first_tag_in_second_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::DuplicateTag(tag) => assert_eq!(tag,b"1447"),
        _ => assert!(false),
    }

    let missing_required_tag_in_repeating_group_message = "8=FIX.4.2\u{1}9=19\u{1}35=A\u{1}1445=1\u{1}1446=0\u{1}10=97\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(missing_required_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredTag(tag) => assert_eq!(tag,b"1447"),
        _ => assert!(false),
    }

    let missing_required_tag_in_first_repeating_group_message = "8=FIX.4.2\u{1}9=33\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1446=1\u{1}1447=1\u{1}10=23\u{1}";
    let result = parse_message::<RepeatingGroupsTestMessage>(missing_required_tag_in_first_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredTag(tag) => assert_eq!(tag,b"1447"),
        _ => assert!(false),
    }
}

#[test]
fn nested_repeating_groups_test() {
    define_message!(NestedRepeatingGroupsTestMessage {
        REQUIRED, orders: NoOrders,
    });

    let one_nested_repeating_group_message = "8=FIX.4.2\u{1}9=35\u{1}35=A\u{1}73=1\u{1}11=uniqueid\u{1}78=1\u{1}79=acct\u{1}10=233\u{1}";
    let message = parse_message::<NestedRepeatingGroupsTestMessage>(one_nested_repeating_group_message).unwrap();
    assert_eq!(message.meta.clone().unwrap().protocol,b"FIX.4.2");
    assert_eq!(message.meta.clone().unwrap().body_length,35);
    assert_eq!(message.meta.clone().unwrap().checksum,233);
    assert_eq!(message.orders.len(),1);
    assert_eq!(message.orders.first().unwrap().cl_ord_id,"uniqueid");
    assert_eq!(message.orders.first().unwrap().allocs.len(),1);
    assert_eq!(message.orders.first().unwrap().allocs.first().unwrap().alloc_account,"acct");
}

#[test]
fn stream_test() {
    define_dictionary!(
        b"A" => LogonTest : LogonTest,
    );

    let two_messages = b"8=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x018=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let mut parser = Parser::new(build_dictionary());
    let (bytes_read,result) = parser.parse(&two_messages.to_vec());
    assert!(result.is_ok());
    assert_eq!(bytes_read,two_messages.len());
    assert_eq!(parser.messages.len(),2);
    for message in parser.messages {
        let casted_message = message.as_any().downcast_ref::<LogonTest>().unwrap();
        assert_eq!(casted_message.meta.clone().unwrap().protocol,b"FIX.4.2");
        assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
        assert_eq!(casted_message.meta.clone().unwrap().checksum,62);
        assert_eq!(casted_message.sender_comp_id,"SERVER");
        assert_eq!(casted_message.target_comp_id,"CLIENT");
        assert_eq!(casted_message.msg_seq_num,"177");
        assert_eq!(casted_message.sending_time,"20090107-18:15:16");
        assert_eq!(casted_message.encrypt_method,"0");
        assert_eq!(casted_message.heart_bt_int,"30");
    }

    let garbage_before_message = b"garbage\x01before=message8=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let mut parser = Parser::new(build_dictionary());
    let (bytes_read,result) = parser.parse(&garbage_before_message.to_vec());
    assert_eq!(bytes_read,garbage_before_message.len());
    assert!(result.is_ok());
    let casted_message = parser.messages.first().unwrap().as_any().downcast_ref::<LogonTest>().unwrap();
    assert_eq!(casted_message.meta.clone().unwrap().protocol,b"FIX.4.2");
    assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
    assert_eq!(casted_message.meta.clone().unwrap().checksum,62);
    assert_eq!(casted_message.sender_comp_id,"SERVER");
    assert_eq!(casted_message.target_comp_id,"CLIENT");
    assert_eq!(casted_message.msg_seq_num,"177");
    assert_eq!(casted_message.sending_time,"20090107-18:15:16");
    assert_eq!(casted_message.encrypt_method,"0");
    assert_eq!(casted_message.heart_bt_int,"30");

    let garbage_between_messages = b"8=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01garbage=before\x01m8ssage8=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let mut parser = Parser::new(build_dictionary());
    let (bytes_read,result) = parser.parse(&garbage_between_messages.to_vec());
    assert!(result.is_ok());
    assert_eq!(bytes_read,garbage_between_messages.len());
    assert_eq!(parser.messages.len(),2);
    for message in parser.messages {
        let casted_message = message.as_any().downcast_ref::<LogonTest>().unwrap();
        assert_eq!(casted_message.meta.clone().unwrap().protocol,b"FIX.4.2");
        assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
        assert_eq!(casted_message.meta.clone().unwrap().checksum,62);
        assert_eq!(casted_message.sender_comp_id,"SERVER");
        assert_eq!(casted_message.target_comp_id,"CLIENT");
        assert_eq!(casted_message.msg_seq_num,"177");
        assert_eq!(casted_message.sending_time,"20090107-18:15:16");
        assert_eq!(casted_message.encrypt_method,"0");
        assert_eq!(casted_message.heart_bt_int,"30");
    }

    let invalid_message_before_valid_message = b"8=FIX.4.2\x0110=0\x018=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";
    let mut parser = Parser::new(build_dictionary());
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
    assert_eq!(casted_message.meta.clone().unwrap().protocol,b"FIX.4.2");
    assert_eq!(casted_message.meta.clone().unwrap().body_length,65);
    assert_eq!(casted_message.meta.clone().unwrap().checksum,62);
    assert_eq!(casted_message.sender_comp_id,"SERVER");
    assert_eq!(casted_message.target_comp_id,"CLIENT");
    assert_eq!(casted_message.msg_seq_num,"177");
    assert_eq!(casted_message.sending_time,"20090107-18:15:16");
    assert_eq!(casted_message.encrypt_method,"0");
    assert_eq!(casted_message.heart_bt_int,"30");
}
