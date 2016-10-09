extern crate fix_rs;

use fix_rs::fix::{ParseState,print_group,TagMap};

//TODO: Copy and pasted from tests/lib.rs.
fn assert_tag_matches_string(tags: &TagMap,tag_name: &str,expected_value: &str) {
    if let fix_rs::fix::TagValue::String(ref str) = *tags.get(tag_name).unwrap() {
        assert_eq!(str,expected_value);
    }
    else {
        assert!(false); //Not a string.
    }
}

fn main() {
    //let message = "8=FIX.4.2\u{1}9=251\u{1}35=D\u{1}49=AFUNDMGR\u{1}56=ABROKER\u{1}34=2\u{1}52=20030615-01:14:49\u{1}11=12345\u{1}1=111111\u{1}63=0\u{1}64=20030621\u{1}21=3\u{1}110=1000\u{1}111=50000\u{1}55=IBM\u{1}48=459200101\u{1}22=1\u{1}54=1\u{1}60=2003061501:14:49\u{1}38=5000\u{1}40=1\u{1}44=15.75\u{1}15=USD\u{1}59=0\u{1}10=221\u{1}";
    let message = b"8=FIX.4.2\x019=65\x0135=A\x0149=SERVER\x0156=CLIENT\x0134=177\x0152=20090107-18:15:16\x0198=0\x01108=30\x0110=062\x01";

    //let message_bytes = Vec::from(message);
    let mut parse_state = ParseState::new();
    for byte in message.iter() {
        let mut message_bytes = Vec::new();
        message_bytes.push(*byte);
        let (bytes_read,_) = parse_state.parse(&message_bytes);
        assert_eq!(bytes_read,1);
    }

    let message = parse_state.messages.first().unwrap();
    print_group(message,0);
    assert_tag_matches_string(message,"8","FIX.4.2");
    assert_tag_matches_string(message,"9","65");
    assert_tag_matches_string(message,"35","A");
    assert_tag_matches_string(message,"49","SERVER");
    assert_tag_matches_string(message,"56","CLIENT");
    assert_tag_matches_string(message,"34","177");
    assert_tag_matches_string(message,"52","20090107-18:15:16");
    assert_tag_matches_string(message,"98","0");
    assert_tag_matches_string(message,"108","30");
    assert_tag_matches_string(message,"10","062");
}
