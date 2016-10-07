extern crate fix_rs;

use std::collections::HashMap;
use fix_rs::fix::parse_message;

fn assert_tag_matches_string(tags: &HashMap<String,fix_rs::fix::TagValue>,tag_name: &str,expected_value: &str) {
    if let &fix_rs::fix::TagValue::String(ref str) = tags.get(tag_name).unwrap() {
        assert_eq!(str,expected_value);
    }
    else {
        assert!(false); //Not a string.
    }
}

fn assert_repeating_group_tag_matches_string(tags: &HashMap<String,fix_rs::fix::TagValue>,group_tag_name: &str,index: usize,value_tag_name: &str,expected_value: &str) {
    if let &fix_rs::fix::TagValue::RepeatingGroup(ref repeating_group) = tags.get(group_tag_name).unwrap() {
        assert!(index < repeating_group.len());
        assert_tag_matches_string(&repeating_group[index],value_tag_name,expected_value);
    }
    else {
        assert!(false); //Not a repeating group.
    }
}

#[test]
fn simple_test() {
    let message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";

    let tags = fix_rs::fix::parse_message(message).unwrap();
    assert_tag_matches_string(&tags,"8","FIX.4.2");
    assert_tag_matches_string(&tags,"9","65");
    assert_tag_matches_string(&tags,"35","A");
    assert_tag_matches_string(&tags,"49","SERVER");
    assert_tag_matches_string(&tags,"56","CLIENT");
    assert_tag_matches_string(&tags,"34","177");
    assert_tag_matches_string(&tags,"52","20090107-18:15:16");
    assert_tag_matches_string(&tags,"98","0");
    assert_tag_matches_string(&tags,"108","30");
    assert_tag_matches_string(&tags,"10","062");
}

#[test]
fn body_length_second_tag_test() {
    let body_length_second_tag_message = "8=FIX.4.2\u{1}9=65\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let tags = fix_rs::fix::parse_message(body_length_second_tag_message).unwrap();
    assert_tag_matches_string(&tags,"9","65");

    let body_length_third_tag_message = "8=FIX.4.2\u{1}35=A\u{1}9=65\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(body_length_third_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::BodyLengthNotSecondTag => {},
        _ => assert!(false),
    }

    let missing_body_length_tag_message = "8=FIX.4.2\u{1}35=A\u{1}49=SERVER\u{1}56=CLIENT\u{1}34=177\u{1}52=20090107-18:15:16\u{1}98=0\u{1}108=30\u{1}10=062\u{1}";
    let result = fix_rs::fix::parse_message(missing_body_length_tag_message);
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
    assert_tag_matches_string(&tags,"35","A");

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
    assert_tag_matches_string(&tags,"10","062");

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
    assert_tag_matches_string(&tags,"8","FIX.4.2");
    assert_tag_matches_string(&tags,"9","30");
    assert_tag_matches_string(&tags,"35","A");
    assert_tag_matches_string(&tags,"212","13");
    assert_tag_matches_string(&tags,"213","This\u{1}is=atest");
    assert_tag_matches_string(&tags,"10","190");

    let missing_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}213=This\u{1}is=atest\u{1}10=190\u{1}";
    let result = fix_rs::fix::parse_message(missing_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingPrecedingLengthTag(value_tag) => assert_eq!(value_tag,"213"),
        _ => assert!(false),
    }

    let late_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}213=This\u{1}is=atest\u{1}212=13\u{1}10=190\u{1}";
    let result = fix_rs::fix::parse_message(late_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingPrecedingLengthTag(value_tag) => assert_eq!(value_tag,"213"),
        _ => assert!(false),
    }

    let early_length_tag_message = "8=FIX.4.2\u{1}9=30\u{1}35=A\u{1}212=13\u{1}56=CLIENT\u{1}213=This\u{1}is=atest\u{1}10=190\u{1}";
    let result = fix_rs::fix::parse_message(early_length_tag_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingFollowingLengthTag(length_tag) => assert_eq!(length_tag,"212"),
        _ => assert!(false),
    }
}

#[test]
fn repeating_groups_test() {
    let no_repeating_groups_message = "8=FIX.4.2\u{1}9=11\u{1}35=A\u{1}887=0\u{1}10=244\u{1}";
    let tags = fix_rs::fix::parse_message(no_repeating_groups_message).unwrap();
    assert_tag_matches_string(&tags,"8","FIX.4.2");
    assert_tag_matches_string(&tags,"9","11");
    assert_tag_matches_string(&tags,"35","A");
    assert_tag_matches_string(&tags,"887","0");
    assert_tag_matches_string(&tags,"10","244");

    let one_repeating_group_message = "8=FIX.4.2\u{1}9=26\u{1}35=A\u{1}1445=1\u{1}1446=0\u{1}1447=0\u{1}10=157\u{1}";
    let tags = fix_rs::fix::parse_message(one_repeating_group_message).unwrap();
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1446","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1447","0");

    let one_repeating_group_with_optional_message = "8=FIX.4.2\u{1}9=43\u{1}35=A\u{1}1445=1\u{1}1446=99\u{1}1447=0\u{1}1448=SomeSource\u{1}10=242\u{1}";
    let tags = fix_rs::fix::parse_message(one_repeating_group_with_optional_message).unwrap();
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1446","99");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1447","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1448","SomeSource");

    let two_repeating_groups_message = "8=FIX.4.2\u{1}9=40\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1446=1\u{1}1447=1\u{1}10=23\u{1}";
    let tags = fix_rs::fix::parse_message(two_repeating_groups_message).unwrap();
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1446","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1447","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1446","1");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1447","1");

    let two_repeating_groups_with_optional_first_message = "8=FIX.4.2\u{1}9=57\u{1}35=A\u{1}1445=2\u{1}1446=99\u{1}1447=0\u{1}1448=SomeSource\u{1}1446=1\u{1}1447=1\u{1}10=117\u{1}";
    let tags = fix_rs::fix::parse_message(two_repeating_groups_with_optional_first_message).unwrap();
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1446","99");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1447","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1448","SomeSource");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1446","1");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1447","1");

    let two_repeating_groups_with_optional_second_message = "8=FIX.4.2\u{1}9=57\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}10=116\u{1}";
    let tags = fix_rs::fix::parse_message(two_repeating_groups_with_optional_second_message).unwrap();
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1446","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1447","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1446","99");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1447","1");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1448","SomeSource");

    let two_repeating_groups_not_body_end_message = "8=FIX.4.2\u{1}9=66\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}55=[N/A]\u{1}10=146\u{1}";
    let tags = fix_rs::fix::parse_message(two_repeating_groups_not_body_end_message).unwrap();
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1446","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",0,"1447","0");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1446","99");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1447","1");
    assert_repeating_group_tag_matches_string(&tags,"1445",1,"1448","SomeSource");
    assert_tag_matches_string(&tags,"55","[N/A]");

    let missing_one_repeating_group_message = "8=FIX.4.2\u{1}9=35\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}55=[N/A]\u{1}10=244\u{1}";
    let result = fix_rs::fix::parse_message(missing_one_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,"55"),
        _ => assert!(false),
    }

    let extra_one_repeating_group_message = "8=FIX.4.2\u{1}9=67\u{1}35=A\u{1}1445=1\u{1}1446=0\u{1}1447=0\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}55=[N/A]\u{1}10=244\u{1}";
    let result = fix_rs::fix::parse_message(extra_one_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::RepeatingGroupTagWithNoRepeatingGroup(tag) => assert_eq!(tag,"1446"),
        _ => assert!(false),
    }

    let non_repeating_group_tag_in_repeating_group_message = "8=FIX.4.2\u{1}9=66\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}55=[N/A]\u{1}1446=99\u{1}1447=1\u{1}1448=SomeSource\u{1}10=145\u{1}";
    let result = fix_rs::fix::parse_message(non_repeating_group_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::NonRepeatingGroupTagInRepeatingGroup(tag) => assert_eq!(tag,"55"),
        _ => assert!(false),
    }

    let wrong_first_tag_in_repeating_group_message = "8=FIX.4.2\u{1}9=43\u{1}35=A\u{1}1445=1\u{1}1447=0\u{1}1446=99\u{1}1448=SomeSource\u{1}10=244\u{1}";
    let result = fix_rs::fix::parse_message(wrong_first_tag_in_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(number_of_tag) => assert_eq!(number_of_tag,"1445"),
        _ => assert!(false),
    }

    let wrong_first_tag_in_second_repeating_group_message = "8=FIX.4.2\u{1}9=40\u{1}35=A\u{1}1445=2\u{1}1446=0\u{1}1447=0\u{1}1447=1\u{1}1446=1\u{1}10=244\u{1}";
    let result = fix_rs::fix::parse_message(wrong_first_tag_in_second_repeating_group_message);
    assert!(result.is_err());
    match result.err().unwrap() {
        fix_rs::fix::ParseError::DuplicateTag(tag) => assert_eq!(tag,"1447"),
        _ => assert!(false),
    }
}
