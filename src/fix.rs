use std::collections::{HashMap,HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

//TODO: Support configuration settings for things like MAX_VALUE_LENGTH, MAX_BODY_LENGTH,
//      MAX_TAG_LENGTH, MAX_CHECKSUM_LENGTH(might just hard code it...), the size of a "Length" and
//      other types.

const BEGINSTR_TAG: &'static str = "8";
const BODYLENGTH_TAG: &'static str = "9";
const MSGTYPE_TAG: &'static str = "35";
const CHECKSUM_TAG: &'static str = "10";

#[derive(Debug)]
pub enum ParseError {
    MissingRequiredTag(String), //BeginStr, BodyLength, MsgType, or Checksum is missing.
    BeginStrNotFirstTag,
    BodyLengthNotSecondTag,
    BodyLengthNotNumber,
    MsgTypeNotThirdTag,
    ChecksumNotLastTag, //Checksum is not exactly where BodyLength says it should be.
    ChecksumDoesNotMatch(u8,u8), //Calculated checksum, Stated checksum
    ChecksumNotNumber,
    DuplicateTag(String),
    WrongFormatTag(String),
    MissingPrecedingLengthTag(String), //Tag was found that requires a preceding length tag which was omitted.
    MissingFollowingLengthTag(String), //Length tag that was specified does not match the following tag.
    NonRepeatingGroupTagInRepeatingGroup(String), //Tag that doesn't belong in a repeating group was found.
    RepeatingGroupTagWithNoRepeatingGroup(String), //Repeating group tag was found outside of a repeating group.
    MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(String), //Tag indicating start of a repeating group was not found immediatelly after tag indicating the number of repeating groups.
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::MissingRequiredTag(ref tag) => write!(f,"ParseError::MissingRequiredTag({})",tag),
            ParseError::BeginStrNotFirstTag => write!(f,"ParseError::BeginStrNotFirstTag"),
            ParseError::BodyLengthNotSecondTag => write!(f,"ParseError::BodyLengthNotSecondTag"),
            ParseError::BodyLengthNotNumber => write!(f,"ParseError::BodyLengthNotNumber"),
            ParseError::MsgTypeNotThirdTag => write!(f,"ParseError::MsgTypeNotThirdTag"),
            ParseError::ChecksumNotLastTag => write!(f,"ParseError::ChecksumNotLastTag"),
            ParseError::ChecksumDoesNotMatch(ref calculated_checksum,ref stated_checksum) => write!(f,"ParseError::ChecksumDoesNotMatch({},{})",calculated_checksum,stated_checksum),
            ParseError::ChecksumNotNumber => write!(f,"ParseError::ChecksumNotNumber"),
            ParseError::DuplicateTag(ref tag) => write!(f,"ParseError::DuplicateTag({})",tag),
            ParseError::WrongFormatTag(ref tag) => write!(f,"ParseError::WrongFormatTag({})",tag),
            ParseError::MissingPrecedingLengthTag(ref value_tag) => write!(f,"ParseError::MissingPrecedingLengthTag({})",value_tag),
            ParseError::MissingFollowingLengthTag(ref length_tag) => write!(f,"ParseError::MissingFollowingLengthTag({})",length_tag),
            ParseError::NonRepeatingGroupTagInRepeatingGroup(ref tag) => write!(f,"ParseError::NonRepeatingGroupTagInRepeatingGroup({})",tag),
            ParseError::RepeatingGroupTagWithNoRepeatingGroup(ref tag) => write!(f,"ParseError::RepeatingGroupTagWithNoRepeatingGroup({})",tag),
            ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(ref number_of_tag) => write!(f,"ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag({})",number_of_tag),
        }
    }
}

pub type TagMap = HashMap<String,TagValue>;

#[derive(Clone,Debug)]
pub enum TagValue {
    String(String),
    RepeatingGroup(Vec<TagMap>),
}

#[derive(Clone)]
struct RepeatingGroupTags {
    first_tag: String,
    other_tags: HashSet<String>,
}

impl RepeatingGroupTags {
    fn new(tags: Vec<String>) -> RepeatingGroupTags {
        let mut iter = tags.iter();

        RepeatingGroupTags {
            first_tag: {
                if let Some(tag) = iter.next() {
                    tag.clone()
                }
                else {
                    //At least one tag must be specified in each repeating group.
                    assert!(false);
                    String::from("")
                }
            },
            other_tags: HashSet::from_iter(iter.cloned()),
        }
    }
}

#[derive(Clone)]
struct ParseRepeatingGroupState {
    number_of_tag: String,
    group_count: usize,
    repeating_group_tags: RepeatingGroupTags,
    current_group_available_tags: HashSet<String>,
    groups: Vec<TagMap>,
}

#[derive(Clone)]
enum TagRuleMode {
    LengthThenValue(String),
    RepeatingGroups(ParseRepeatingGroupState),
    RepeatingGroupStart(String),
}

#[derive(PartialEq)]
enum FoundMessage {
    NotFound,
    FirstByte,
    SecondByte,
}

pub struct ParseState {
    found_message: FoundMessage,
    current_tag: String,
    current_string: String,
    checksum: u8,
    body_length: u64, //TODO: Do we really need this to be this long?
    previous_tag: String,
    next_tag_checksum: bool,
    tag_rule_mode_stack: Vec<TagRuleMode>,
    fast_track_bytes_remaining: usize,
    current_message: TagMap,
    pub messages: Vec<TagMap>,
}

impl ParseState {
    pub fn new() -> ParseState {
        ParseState {
            found_message: FoundMessage::NotFound,
            current_tag: String::new(),
            current_string: String::new(),
            checksum: 0,
            body_length: 0,
            previous_tag: String::new(),
            next_tag_checksum: false,
            tag_rule_mode_stack: Vec::new(),
            fast_track_bytes_remaining: 0,
            current_message: TagMap::new(),
            messages: Vec::new(),
        }
    }

    pub fn reset_parser(&mut self) {
        self.found_message = FoundMessage::NotFound;
        self.current_tag.clear();
        self.current_string.clear();
        self.checksum = 0;
        self.body_length = 0;
        self.previous_tag.clear();
        self.next_tag_checksum = false;
        self.tag_rule_mode_stack.clear();
        self.fast_track_bytes_remaining = 0;
        self.current_message.clear();
    }

    fn update_book_keeping(&mut self,c: u8) -> Result<(),ParseError> {
        //Update checksum.
        self.checksum = self.checksum.overflowing_add(c).0;

        //Update where we are when reading the body in case message is malformed and the checksum
        //is not at the offset where it's supposed to be.
        self.body_length = self.body_length.overflowing_sub(1).0;
        if self.body_length == 0 {
            if c != b'\x01' { //SOH
                return Err(ParseError::ChecksumNotLastTag);
            }
            self.next_tag_checksum = true;
        }

        Ok(())
    }

    fn if_checksum_then_is_last_tag(&self) -> Result<(),ParseError> {
        if (self.current_tag == CHECKSUM_TAG && !self.next_tag_checksum) || (self.current_tag != CHECKSUM_TAG && self.next_tag_checksum) {
            return Err(ParseError::ChecksumNotLastTag);
        }

        return Ok(())
    }

    fn validate_checksum(&self,checksum_string: &str) -> Result<(),ParseError> {
        //Remove checksum tag that should not be part of the current checksum.
        let mut checksum = self.checksum.overflowing_sub('1' as u8 + '0' as u8 + '=' as u8 + '\u{1}' as u8).0;
        for c in checksum_string.bytes() {
            checksum = checksum.overflowing_sub(c).0;
        }

        match u8::from_str_radix(&checksum_string,10) {
            Ok(stated_checksum) => if checksum != stated_checksum {
                return Err(ParseError::ChecksumDoesNotMatch(checksum,stated_checksum));
            },
            Err(_) => return Err(ParseError::ChecksumNotNumber),
        }

        Ok(())
    }

    fn scan_for_message(&mut self,index: &mut usize,message_bytes: &Vec<u8>) {
        //Scan for a message header. Bytes are read one by one and consumed until "8=" is found.
        //Where '8' is the BeginStr tag and '=' indicates the previous part is the tag. The state
        //machine here is designed to function even if given one byte at a time. In a properly
        //formed stream, the header should be found immediatelly. Hence, this probably isn't worth
        //optimizing.

        //Has a message already been found and is being parsed? Skip scan.
        if self.found_message == FoundMessage::SecondByte {
            return;
        }

        //If the scan previously found the BeginStr tag but ran out of bytes, resume from the same
        //state.
        let mut previous_byte = match self.found_message {
            FoundMessage::NotFound => 0,
            FoundMessage::FirstByte => b'8',
            _ => unreachable!(),
        };
        while *index < message_bytes.len() {
            let byte = message_bytes[*index];

            //Check if "8=" header has been found.
            if byte == b'=' && previous_byte == b'8' {
                self.found_message = FoundMessage::SecondByte;
                break;
            }

            previous_byte = byte;
            *index += 1;
        }

        if self.found_message == FoundMessage::SecondByte {
            //Act like the BeginStr tag was parsed so we don't duplicate work.
            self.current_tag = String::from("8");
            self.checksum = b'8' + b'=';
            *index += 1;
        }
        else if previous_byte == b'8' && *index == message_bytes.len() {
            //Ran out of bytes but the last byte could be the start of the header. Just make a note
            //so we can resume when more bytes are available.
            self.found_message = FoundMessage::FirstByte;
        }
    }

    fn fast_track_read_bytes(&mut self,index: &mut usize,message_bytes: &Vec<u8>) -> Result<(),ParseError> {
        loop {
            if *index >= message_bytes.len() || self.fast_track_bytes_remaining == 0 {
                break;
            }

            let c = message_bytes[*index];
            try!(self.update_book_keeping(c));

            self.current_string.push(c as char);

            *index += 1;
            self.fast_track_bytes_remaining -= 1;
        }

        return Ok(());
    }

    fn fold_top_repeating_group_down(&mut self) {
        let mut folded_down = false;
        {
            let mut tag_rule_mode_stack_iter = self.tag_rule_mode_stack.iter_mut().rev();
            if let Some(first_tag_rule_mode) = tag_rule_mode_stack_iter.next() {
                if let &mut TagRuleMode::RepeatingGroups(ref mut prgs) = first_tag_rule_mode {
                    for tag_rule_mode in tag_rule_mode_stack_iter {
                        if let &mut TagRuleMode::RepeatingGroups(ref mut parent_prgs) = tag_rule_mode {
                            parent_prgs.groups.last_mut().unwrap().insert(prgs.number_of_tag.clone(),TagValue::RepeatingGroup(prgs.groups.clone()));
                            folded_down = true;
                        }
                    }

                    if !folded_down {
                        self.current_message.insert(prgs.number_of_tag.clone(),TagValue::RepeatingGroup(prgs.groups.clone()));
                        folded_down = true;
                    }
                }
            }
        }

        if folded_down {
            self.tag_rule_mode_stack.pop();
        }
        else {
            unreachable!();
        }
    }

    fn get_number_from_str_by_top_tag<T: FromStr>(&self,tag: &str) -> Result<T,ParseError> {
        //Search tag modes for the top repeating group.
        for tag_rule_mode in self.tag_rule_mode_stack.iter().rev() {
            if let &TagRuleMode::RepeatingGroups(ref parse_repeating_group_state) = tag_rule_mode {
                if let Some(last_group) = parse_repeating_group_state.groups.last() {
                    //If the most recent group has the tag, parse and return as a number.
                    if let &TagValue::String(ref str) = last_group.get(tag).unwrap() {
                        if let Ok(number) = T::from_str(&str) {
                            return Ok(number);
                        }
                    }

                    //Otherwise, tag is not found. Do not search any other repeating groups.
                    return Err(ParseError::WrongFormatTag(String::from(tag)))
                }
            }
        }

        //There are no repeating groups, try the current message tags instead.
        if let &TagValue::String(ref str) = self.current_message.get(tag).unwrap() {
            if let Ok(number) = T::from_str(&str) {
                return Ok(number);
            }
        }
        
        //TODO: Maybe we need a different error...although this is the only one that SHOULD be
        //possible.
        Err(ParseError::WrongFormatTag(String::from(tag)))
    }

    pub fn parse(&mut self,message_bytes: &Vec<u8>) -> (usize,Result<(),ParseError>) {
        //Parse and bytes as possible. Either all bytes will be consumed or all bytes up until a
        //parse error is triggered -- whatever happens first.
        let mut index = 0;
        match self.parse_private(&mut index,message_bytes) {
            Ok(_) => (index,Ok(())),
            Err(err) => {
                self.reset_parser(); //Reset automatically so the next parse won't fail immediatelly.
                (index,Err(err))
            }
        }
    }

    fn parse_private(&mut self,index: &mut usize,message_bytes: &Vec<u8>) -> Result<(),ParseError> {
        //Build a mapping of repeating group No...(No = Number Of) tags and a list of child tags that
        //are part of the group. The first child tag specified is required to delimit the groups.
        let mut repeating_group_tags : HashMap<String,RepeatingGroupTags> = HashMap::new();
        repeating_group_tags.insert(String::from("887"),RepeatingGroupTags::new(vec![String::from("888"),String::from("889")]));
        repeating_group_tags.insert(String::from("1445"),RepeatingGroupTags::new(vec![String::from("1446"),String::from("1447"),String::from("1448")]));
        repeating_group_tags.insert(String::from("73"),RepeatingGroupTags::new(vec![String::from("11"),String::from("78")])); //TODO: Tag list incomplete here.
        repeating_group_tags.insert(String::from("78"),RepeatingGroupTags::new(vec![String::from("79"),String::from("467"),String::from("80")])); //TODO: Tag list incomplete here.

        //Build a mapping of length tags and the value tags they are followed by.
        let mut length_to_value_tags : HashMap<String,String> = HashMap::new();
        length_to_value_tags.insert(String::from("212"),String::from("213")); //XmlDataLen -> XmlData
        let mut value_to_length_tags : HashMap<String,String> = HashMap::new();
        for (length_tag,value_tag) in &length_to_value_tags {
            value_to_length_tags.insert(value_tag.clone(),length_tag.clone());
        }

        //Start by searching for the start of a message unless resuming.
        self.scan_for_message(index,&message_bytes);

        //Resume loading any bytes using the fast track if we ran out in the last call.
        try!(self.fast_track_read_bytes(index,&message_bytes));

        //Parse each byte in the message one by one.
        while *index < message_bytes.len() {
            let c = message_bytes[*index];

            //Perform basic checksum and body length updates.
            try!(self.update_book_keeping(c));

            //Check if this byte indicates a new tag=value, the end of a tag, part of a tag, or part of
            //a value.
            match c {
                //Byte indicates a tag has finished being read.
                b'=' => {
                    self.current_tag = self.current_string.clone(); //TODO: Can we move without copying here?
                    self.current_string = String::new();

                    //Make sure that iff the body of the message has already been read, this is the
                    //checksum tag.
                    try!(self.if_checksum_then_is_last_tag());

                    //If there is some tag ordering in effect, make sure this is the expected tag to
                    //follow the previous tag.
                    if let Some(tag_rule_mode) = self.tag_rule_mode_stack.pop() {
                        match tag_rule_mode {
                            TagRuleMode::LengthThenValue(ref value_tag) => {
                                if self.current_tag != *value_tag {
                                    return Err(ParseError::MissingFollowingLengthTag(self.previous_tag.clone()));
                                }

                                //Fast track to read in the specified number of bytes.
                                match self.get_number_from_str_by_top_tag::<usize>(&self.previous_tag) {
                                    Ok(length) => {
                                        self.fast_track_bytes_remaining = length;
                                        *index += 1;
                                        try!(self.fast_track_read_bytes(index,&message_bytes));
                                        *index -= 1;
                                    },
                                    Err(err) => return Err(err),
                                }
                            },
                            TagRuleMode::RepeatingGroupStart(ref first_repeating_group_tag) => {
                                //Sanity check that the first tag in a repeating group is what is
                                //expected.
                                if self.current_tag != *first_repeating_group_tag {
                                    return Err(ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(self.previous_tag.clone()));
                                }
                            },
                            _ => self.tag_rule_mode_stack.push(tag_rule_mode.clone()),
                        }
                    }
                   //Otherwise, if the current tag requires some preceding tag that wasn't found,
                    //return an error. This is a sanity check.
                    else {
                        if let Some(required_preceding_tag) = value_to_length_tags.get(&self.current_tag) {
                            if required_preceding_tag != &self.previous_tag {
                                return Err(ParseError::MissingPrecedingLengthTag(self.current_tag.clone()));
                            }
                        }
                    }
                },
                //Byte indicates a vale has finished being read. Now both the tag and value are known.
                b'\x01' => { //SOH
                    //Validate that the first three tags of a message are, in order: BeginStr,
                    //BodyLength, and MsgType.
                    if self.current_message.len() == 0 && self.current_tag != BEGINSTR_TAG {
                        return Err(ParseError::BeginStrNotFirstTag);
                    }
                    else if self.current_message.len() == 1 {
                        if self.current_tag != BODYLENGTH_TAG {
                            return Err(ParseError::BodyLengthNotSecondTag);
                        }

                        //Body length must be a valid positive number or else the rest of the message
                        //is garbage.
                        match u64::from_str_radix(&self.current_string,10) {
                            Ok(length) => self.body_length = length,
                            Err(_) => return Err(ParseError::BodyLengthNotNumber),
                        }
                    }
                    else if self.current_message.len() == 2 && self.current_tag != MSGTYPE_TAG {
                        return Err(ParseError::MsgTypeNotThirdTag);
                    }

                    //Make sure checksum checks out when done reading a message.
                    let mut is_message_end = false;
                    if self.current_tag == CHECKSUM_TAG {
                        try!(self.validate_checksum(&self.current_string));
                        is_message_end = true;
                    }

                    //Store tag with value.
                    let mut tag_in_group = false;
                    let mut group_end = false;
                    loop {
                        if let Some(ref mut tag_rule_mode) = self.tag_rule_mode_stack.last_mut() {
                            if let TagRuleMode::RepeatingGroups(ref mut parse_repeating_group_self) = **tag_rule_mode {
                                let mut prgs = parse_repeating_group_self;
                                if self.current_tag == prgs.repeating_group_tags.first_tag {
                                    let mut group = TagMap::new();
                                    group.insert(self.current_tag.clone(),TagValue::String(self.current_string.clone()));
                                    prgs.groups.push(group);
                                    prgs.current_group_available_tags = prgs.repeating_group_tags.other_tags.clone();

                                    if prgs.groups.len() > prgs.group_count {
                                        return Err(ParseError::RepeatingGroupTagWithNoRepeatingGroup(self.current_tag.clone()));
                                    }
                                    tag_in_group = true;
                                }
                                else if prgs.current_group_available_tags.remove(&self.current_tag) {
                                    prgs.groups.last_mut().unwrap().insert(self.current_tag.clone(),TagValue::String(self.current_string.clone()));
                                    tag_in_group = true;
                                }
                                else {
                                    if prgs.groups.last().unwrap().contains_key(&self.current_tag) {
                                        return Err(ParseError::DuplicateTag(self.current_tag.clone()));
                                    }
                                    else if prgs.groups.len() < prgs.group_count {
                                        return Err(ParseError::NonRepeatingGroupTagInRepeatingGroup(self.current_tag.clone()));
                                    }

                                    //Tag does not belong in this group and all stated groups are
                                    //accounted for.
                                    group_end = true;
                                }
                            }
                        }
                        if group_end {
                            //Put repeated group into next highest repeating group. If there are no
                            //repeating groups, put into the top-level set of tags.
                            self.fold_top_repeating_group_down();
                            group_end = false;
                        }
                        else {
                            break;
                        }
                    }
                    if !tag_in_group && self.current_message.insert(self.current_tag.clone(),TagValue::String(self.current_string.clone())).is_some() {
                        return Err(ParseError::DuplicateTag(self.current_tag.clone()));
                    }

                    if is_message_end {
                        //Make sure all required tags are specified.
                        //TODO: The required tags might vary based on msgtype.
                        for tag in &[BEGINSTR_TAG,BODYLENGTH_TAG,MSGTYPE_TAG,CHECKSUM_TAG] {
                            if !self.current_message.contains_key(*tag) {
                                return Err(ParseError::MissingRequiredTag(String::from(*tag)));
                            }
                        }

                        //Save message.
                        self.messages.push(self.current_message.clone());

                        //Prepare for the next message.
                        self.reset_parser();
                        *index += 1;

                        //Scan to the next message in case there is garbage between the end of this
                        //one and the beginning of the next.
                        self.scan_for_message(index,&message_bytes);
                        continue;
                    }

                    //Check if this tag indicates that a specific tag must follow this one.
                    if let Some(following_tag) = length_to_value_tags.get(&self.current_tag) {
                        self.tag_rule_mode_stack.push(TagRuleMode::LengthThenValue(following_tag.clone()));
                    }
                    if let Some(repeating_group) = repeating_group_tags.get(&self.current_tag) {
                        match self.get_number_from_str_by_top_tag::<usize>(&self.current_tag) {
                            Ok(group_count) => {
                                if group_count > 0 {
                                    self.tag_rule_mode_stack.push(TagRuleMode::RepeatingGroups(ParseRepeatingGroupState {
                                        number_of_tag: self.current_tag.clone(),
                                        group_count: group_count,
                                        repeating_group_tags: repeating_group.clone(),
                                        current_group_available_tags: HashSet::new(),
                                        groups: Vec::new(),
                                    }));
                                    self.tag_rule_mode_stack.push(TagRuleMode::RepeatingGroupStart(repeating_group.first_tag.clone()));
                                }
                            },
                            Err(parse_error) => return Err(parse_error),
                        }
                    }

                    self.previous_tag = self.current_tag.clone();
                    self.current_string = String::new()
                }
                //Byte is part of a tag or value.
                _ => {
                    self.current_string.push(c as char);
                }
            }

            *index += 1;
        }

        return Ok(());
    }
}

pub fn print_group(group: &TagMap,depth: u8) {
    //Print a new line at the beginning so it's easier to scan the output. This is also important
    //when running tests because the stdout output of a test is prepended with a bunch of spaces.
    if depth == 0 {
        println!("");
    }

    for (tag,value) in group {
        for _ in 0..depth {
            print!("\t");
        }

        if let TagValue::RepeatingGroup(_) = *value {
            println!("{} = {{",tag);
        } else {
            print!("{} = ",tag);
        }
        print_tag_value(value,depth);

        if let TagValue::RepeatingGroup(_) = *value {
            for _ in 0..depth {
                print!("\t");
            }
            println!("}}");
        }
    }
}

fn print_tag_value(tag_value: &TagValue,depth: u8) {
    match tag_value {
        &TagValue::String(ref str) => println!("{}",str),
        &TagValue::RepeatingGroup(ref repeating_group) => {
            for group in repeating_group {
                print_group(group,depth + 1);
            }
        }
    }
}

