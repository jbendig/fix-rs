use std::collections::HashMap;
use std::fmt;

const BEGINSTR_TAG: &'static str = "8";
const BODYLENGTH_TAG: &'static str = "9";
const MSGTYPE_TAG: &'static str = "35";
const CHECKSUM_TAG: &'static str = "10";

#[derive(Debug)]
pub enum ParseError {
    MissingRequiredTag, //BeginStr, BodyLength, MsgType, or Checksum is missing.
    BeginStrNotFirstTag,
    BodyLengthNotSecondTag,
    BodyLengthNotNumber,
    MsgTypeNotThirdTag,
    ChecksumNotLastTag, //Checksum is not exactly where BodyLength says it should be.
    ChecksumDoesNotMatch(u8,u8), //Calculated checksum, Stated checksum
    ChecksumNotNumber,
    DuplicateTag,
    WrongFormatTag(String),
    MissingPrecedingLengthTag(String), //Tag was found that requires a preceding length tag which was omitted.
    MissingFollowingLengthTag(String), //Length tag that was specified does not match the following tag.
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::MissingRequiredTag => write!(f,"ParseError::MissingRequiredTag"),
            ParseError::BeginStrNotFirstTag => write!(f,"ParseError::BeginStrNotFirstTag"),
            ParseError::BodyLengthNotSecondTag => write!(f,"ParseError::BodyLengthNotSecondTag"),
            ParseError::BodyLengthNotNumber => write!(f,"ParseError::BodyLengthNotNumber"),
            ParseError::MsgTypeNotThirdTag => write!(f,"ParseError::MsgTypeNotThirdTag"),
            ParseError::ChecksumNotLastTag => write!(f,"ParseError::ChecksumNotLastTag"),
            ParseError::ChecksumDoesNotMatch(ref calculated_checksum,ref stated_checksum) => write!(f,"ParseError::ChecksumDoesNotMatch({},{})",calculated_checksum,stated_checksum),
            ParseError::ChecksumNotNumber => write!(f,"ParseError::ChecksumNotNumber"),
            ParseError::DuplicateTag => write!(f,"ParseError::DuplicateTag"),
            ParseError::WrongFormatTag(ref tag) => write!(f,"ParseError::WrongFormatTag({})",tag),
            ParseError::MissingPrecedingLengthTag(ref value_tag) => write!(f,"ParseError::MissingPrecedingLengthTag({})",value_tag),
            ParseError::MissingFollowingLengthTag(ref length_tag) => write!(f,"ParseError::MissingFollowingLengthTag({})",length_tag),
        }
    }
}

fn validate_checksum(calculated_checksum: u8,checksum_string: &str) -> Result<(),ParseError> {
    //Remove checksum tag that should not be part of the current checksum.
    let mut checksum = calculated_checksum.overflowing_sub('1' as u8 + '0' as u8 + '=' as u8 + '\u{1}' as u8).0;
    for c in checksum_string.bytes() {
        checksum = checksum.overflowing_sub(c).0;
    }

    match u8::from_str_radix(&checksum_string,10) {
        Ok(stated_checksum) => if checksum != stated_checksum {
            return Err(ParseError::ChecksumDoesNotMatch(checksum,stated_checksum));
        },
        Err(_) => return Err(ParseError::ChecksumNotNumber),
    }

    return Ok(());
}

pub fn parse_message(message: &str) -> Result<HashMap<String,String>,ParseError> {
    let message_bytes = Vec::from(message);

    let mut length_to_value_tags : HashMap<String,String> = HashMap::new();
    length_to_value_tags.insert(String::from("212"),String::from("213")); //XmlDataLen -> XmlData
    let mut value_to_length_tags : HashMap<String,String> = HashMap::new();
    for (length_tag,value_tag) in &length_to_value_tags {
        value_to_length_tags.insert(value_tag.clone(),length_tag.clone());
    }

    let mut tags : HashMap<String,String> = HashMap::new();

    //TODO: Handle child tag groups.
    //TODO: Handle streams of data.
    let mut current_tag = String::new();
    let mut current_string = String::new();
    let mut checksum: u8 = 0;
    let mut body_length: u64 = 0; //TODO: Do we really need this to be this long?
    let mut previous_tag = String::new();
    let mut next_tag_checksum = false;
    let mut next_expected_tag = String::new();
    let mut index = 0;
    while index < message_bytes.len() {
        let c = message_bytes[index];
        checksum = checksum.overflowing_add(c).0;

        body_length = body_length.overflowing_sub(1).0;
        if body_length == 0 {
            if c != 1 { //SOH
                return Err(ParseError::ChecksumNotLastTag);
            }
            next_tag_checksum = true;
        }

        match c {
            61 => { //=
                current_tag = current_string.clone(); //TODO: Can we move without copying here?
                current_string = String::new();

                if (current_tag == CHECKSUM_TAG && !next_tag_checksum) || (current_tag != CHECKSUM_TAG && next_tag_checksum) {
                    return Err(ParseError::ChecksumNotLastTag);
                }

                if !next_expected_tag.is_empty() {
                    if current_tag != next_expected_tag {
                        return Err(ParseError::MissingFollowingLengthTag(previous_tag)); //TODO: Probably should pass previous tag here instead.
                    }
                    next_expected_tag = String::new();

                    //Fast track to read in the specified number of bytes.
                    let length_str = tags.get(&previous_tag).unwrap();
                    if let Ok(length) = usize::from_str_radix(&length_str,10) {
                        let mut index2 = index + 1;
                        loop {
                            if index2 >= message_bytes.len() || index2 - index - 1 >= length {
                                index = index2 - 1;
                                break;
                            }

                            let c = message_bytes[index2];
                            checksum = checksum.overflowing_add(c).0;

                            body_length = body_length.overflowing_sub(1).0;
                            if body_length == 0 {
                                if c != 1 { //SOH
                                    return Err(ParseError::ChecksumNotLastTag);
                                }
                                next_tag_checksum = true;
                            }

                            current_string.push(c as char);

                            index2 += 1;
                        }
                    }
                    else {
                        return Err(ParseError::WrongFormatTag(previous_tag));
                    }
                }
                else {
                    if let Some(required_preceding_tag) = value_to_length_tags.get(&current_tag) {
                        if required_preceding_tag != &previous_tag {
                            return Err(ParseError::MissingPrecedingLengthTag(current_tag));
                        }
                    }
                }

                if let Some(following_tag) = length_to_value_tags.get(&current_tag) {
                    next_expected_tag = following_tag.clone();
                }
            },
            1 => { //SOH
                if tags.len() == 0 && current_tag != BEGINSTR_TAG {
                    return Err(ParseError::BeginStrNotFirstTag);
                }
                else if tags.len() == 1 {
                    if current_tag != BODYLENGTH_TAG {
                        return Err(ParseError::BodyLengthNotSecondTag);
                    }

                    match u64::from_str_radix(&current_string,10) {
                        Ok(length) => body_length = length,
                        Err(_) => return Err(ParseError::BodyLengthNotNumber),
                    }
                }
                else if tags.len() == 2 && current_tag != MSGTYPE_TAG {
                    return Err(ParseError::MsgTypeNotThirdTag);
                }

                let mut is_message_end = false;
                if current_tag == CHECKSUM_TAG {
                    try!(validate_checksum(checksum,&current_string));
                    is_message_end = true;
                }

                if tags.insert(current_tag.clone(),current_string) != None {
                    return Err(ParseError::DuplicateTag);
                }

                if is_message_end {
                    break;
                }

                previous_tag = current_tag.clone();
                current_string = String::new()
            }
            _ => {
                current_string.push(c as char);
            }
        }

        index += 1;
    }

    //Make sure all required tags are specified.
    //TODO: The required tags might vary based on msgtype.
    for tag in &[BEGINSTR_TAG,BODYLENGTH_TAG,MSGTYPE_TAG,CHECKSUM_TAG] {
        if !tags.contains_key(*tag) {
            return Err(ParseError::MissingRequiredTag);
        }
    }

    return Ok(tags);
}

