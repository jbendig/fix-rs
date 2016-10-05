extern crate fix_rs;

use fix_rs::fix::parse_message;

#[test]
fn simple_test() {
    let message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";

    let tags = fix_rs::fix::parse_message(message).unwrap();
    assert_eq!(tags.get("8").unwrap(),"FIX.4.2");
    assert_eq!(tags.get("9").unwrap(),"65");
    assert_eq!(tags.get("35").unwrap(),"A");
    assert_eq!(tags.get("49").unwrap(),"SERVER");
    assert_eq!(tags.get("56").unwrap(),"CLIENT");
    assert_eq!(tags.get("34").unwrap(),"177");
    assert_eq!(tags.get("52").unwrap(),"20090107-18:15:16");
    assert_eq!(tags.get("98").unwrap(),"0");
    assert_eq!(tags.get("108").unwrap(),"30");
    assert_eq!(tags.get("10").unwrap(),"062");
}

#[test]
fn body_length_second_tag_test() {
    let body_length_second_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let tags = fix_rs::fix::parse_message(body_length_second_tag_message).unwrap();
    assert_eq!(tags.get("9").unwrap(),"65");

    let body_length_third_tag_message = "8=FIX.4.2\u{1}35=A\u{1}9=65\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(body_length_third_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let missing_body_length_tag_message = "8=FIX.4.2\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(body_length_third_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let negative_number_body_length_tag_message = "8=FIX.4.2\u{1}9=-65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(negative_number_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotNumber => {},
        _ => assert!(false),
    }

    let nonnumber_number_body_length_tag_message = "8=FIX.4.2\u{1}9=TEXT\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(nonnumber_number_body_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotNumber => {},
        _ => assert!(false),
    }
}

#[test]
fn msg_type_third_tag_test() {
    let msg_type_third_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let tags = fix_rs::fix::parse_message(msg_type_third_tag_message).unwrap();
    assert_eq!(tags.get("35").unwrap(),"A");

    let msg_type_fourth_tag_message = "8=FIX.4.2\u{1}9=65\u{1}49=SERVER\u{1}35=A\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(msg_type_fourth_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MsgTypeNotThirdTag => {},
        _ => assert!(false),
    }

    let missing_msg_type_tag_message = "8=FIX.4.2\u{1}9=65\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(missing_msg_type_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MsgTypeNotThirdTag => {},
        _ => assert!(false),
    }
}

#[test]
fn checksum_tag_test() {
    let valid_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let tags = fix_rs::fix::parse_message(valid_checksum_tag_message).unwrap();
    assert_eq!(tags.get("10").unwrap(),"062");

    let incorrect_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=0\u{1}";
    let result = fix_rs::fix::parse_message(incorrect_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumDoesNotMatch(calculated_checksum,stated_checksum) => {
            assert_eq!(calculated_checksum,62);
            assert_eq!(stated_checksum,0);
        },
        _ => assert!(false),
    }

    let negative_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=-62\u{1}";
    let result = fix_rs::fix::parse_message(negative_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotNumber => {},
        _ => assert!(false),
    }

    let nonnumber_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=TST\u{1}";
    let result = fix_rs::fix::parse_message(nonnumber_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotNumber => {},
        _ => assert!(false),
    }

    let early_checksum_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}10=TST\u{1}108=30\u{1}";
    let result = fix_rs::fix::parse_message(early_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }

    let late_checksum_tag_message = "8=FIX.4.2\u{1}9=58\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=TST\u{1}";
    let result = fix_rs::fix::parse_message(late_checksum_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::ChecksumNotLastTag => {},
        _ => assert!(false),
    }
}

/*TODO: Duplicate tag test. Probably can be part of supporting grouped messages. */

#[test]
fn length_tag_test() {
    let valid_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}212=13\u{1}213=This\u{1}is=atest\u{1}10=190\u{1}";
    let tags = fix_rs::fix::parse_message(valid_length_tag_message).unwrap();
    assert_eq!(tags.get("8").unwrap(),"FIX.4.2");
    assert_eq!(tags.get("9").unwrap(),"30");
    assert_eq!(tags.get("35").unwrap(),"A");
    assert_eq!(tags.get("212").unwrap(),"13");
    assert_eq!(tags.get("213").unwrap(),"This\u{1}is=atest");
    assert_eq!(tags.get("10").unwrap(),"192");

    let missing_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}213=This\u{1}is=atest\u{1}10=190\u{1}";
    let result = fix_rs::fix::parse_message(missing_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredLengthTag => {},
        _ => assert!(false),
    }
    //TODO: Check that the tag where this is triggered is 213.

    let late_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}213=This\u{1}is=atest\u{1}212=13\u{1}10=190\u{1}";
    let result = fix_rs::fix::parse_message(late_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredLengthTag => {},
        _ => assert!(false),
    }
    //TODO: Check that the tag where this is triggered is 213.
    let early_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}212=13\u{1}56=CLIENT\u{1}213=This\u{1}is=atest\u{1}10=190\u{1}";
    let result = fix_rs::fix::parse_message(early_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingRequiredLengthTag => {},
        _ => assert!(false),
    }
    //TODO: Check that the tag where this is triggered is 213.
}
