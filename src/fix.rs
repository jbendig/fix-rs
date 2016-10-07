use std::collections::{HashMap,HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

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

#[derive(Clone,Debug)]
pub enum TagValue {
    String(String),
    RepeatingGroup(Vec<HashMap<String,TagValue>>),
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
    groups: Vec<HashMap<String,TagValue>>,
}

#[derive(Clone)]
enum TagRuleMode {
    LengthThenValue(String),
    RepeatingGroups(ParseRepeatingGroupState),
    RepeatingGroupStart(String),
}

struct ParseState {
    current_tag: String,
    current_string: String,
    checksum: u8,
    body_length: u64, //TODO: Do we really need this to be this long?
    previous_tag: String,
    next_tag_checksum: bool,
    tag_rule_mode_stack: Vec<TagRuleMode>,
    index: usize,
}

impl ParseState {
    fn update_book_keeping(&mut self,c: u8) -> Result<(),ParseError> {
        //Update checksum.
        self.checksum = self.checksum.overflowing_add(c).0;

        //Update where we are when reading the body in case message is malformed and the checksum
        //is not at the offset where it's supposed to be.
        self.body_length = self.body_length.overflowing_sub(1).0;
        if self.body_length == 0 {
            if c != 1 { //SOH
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

    fn fast_track_read_bytes(&mut self,tags: &HashMap<String,TagValue>,message_bytes: &Vec<u8>) -> Result<(),ParseError> {
        if let &TagValue::String(ref length_str) = tags.get(&self.previous_tag).unwrap() {
            if let Ok(length) = usize::from_str_radix(&length_str,10) {
                let mut index2 = self.index + 1;
                loop {
                    if index2 >= message_bytes.len() || index2 - self.index - 1 >= length {
                        self.index = index2 - 1;
                        break;
                    }

                    let c = message_bytes[index2];
                    try!(self.update_book_keeping(c));

                    self.current_string.push(c as char);

                    index2 += 1;
                }

                return Ok(());
            }
        }

        Err(ParseError::WrongFormatTag(self.previous_tag.clone()))
    }

    fn fold_top_repeating_group_down(&mut self,default_tags: &mut HashMap<String,TagValue>) {
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
                        default_tags.insert(prgs.number_of_tag.clone(),TagValue::RepeatingGroup(prgs.groups.clone()));
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

    fn get_number_from_str_by_top_tag<T: FromStr>(&self,default_tags: &HashMap<String,TagValue>,tag: &str) -> Result<T,ParseError> {
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

        //There are no repeating groups, try the default tags instead.
        if let &TagValue::String(ref str) = default_tags.get(tag).unwrap() {
            if let Ok(number) = T::from_str(&str) {
                return Ok(number);
            }
        }
        
        //TODO: Maybe we need a different error...although this is the only one that SHOULD be
        //possible.
        Err(ParseError::WrongFormatTag(String::from(tag)))
    }
}

pub fn parse_message(message: &str) -> Result<HashMap<String,TagValue>,ParseError> {
    let message_bytes = Vec::from(message);

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

    let mut tags : HashMap<String,TagValue> = HashMap::new();

    //TODO: Handle streams of data.
    let mut state = ParseState {
        current_tag: String::new(),
        current_string: String::new(),
        checksum: 0,
        body_length: 0,
        previous_tag: String::new(),
        next_tag_checksum: false,
        tag_rule_mode_stack: Vec::new(),
        index: 0,
    };
    while state.index < message_bytes.len() {
        let c = message_bytes[state.index];

        //Perform basic checksum and body length updates.
        try!(state.update_book_keeping(c));

        //Check if this byte indicates a new tag=value, the end of a tag, part of a tag, or part of
        //a value.
        match c {
            //Byte indicates a tag has finished being read.
            61 => { //=
                state.current_tag = state.current_string.clone(); //TODO: Can we move without copying here?
                state.current_string = String::new();

                //Make sure that iff the body of the message has already been read, this is the
                //checksum tag.
                try!(state.if_checksum_then_is_last_tag());

                //If there is some tag ordering in effect, make sure this is the expected tag to
                //follow the previous tag.
                if let Some(tag_rule_mode) = state.tag_rule_mode_stack.pop() {
                    match tag_rule_mode {
                        TagRuleMode::LengthThenValue(ref value_tag) => {
                            if state.current_tag != *value_tag {
                                return Err(ParseError::MissingFollowingLengthTag(state.previous_tag));
                            }

                            //Fast track to read in the specified number of bytes.
                            try!(state.fast_track_read_bytes(&tags,&message_bytes));
                        },
                        TagRuleMode::RepeatingGroupStart(ref first_repeating_group_tag) => {
                            //Sanity check that the first tag in a repeating group is what is
                            //expected.
                            if state.current_tag != *first_repeating_group_tag {
                                return Err(ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(state.previous_tag));
                            }
                        },
                        _ => state.tag_rule_mode_stack.push(tag_rule_mode.clone()),
                    }
                }
               //Otherwise, if the current tag requires some preceding tag that wasn't found,
                //return an error. This is a sanity check.
                else {
                    if let Some(required_preceding_tag) = value_to_length_tags.get(&state.current_tag) {
                        if required_preceding_tag != &state.previous_tag {
                            return Err(ParseError::MissingPrecedingLengthTag(state.current_tag));
                        }
                    }
                }
            },
            //Byte indicates a vale has finished being read. Now both the tag and value are known.
            1 => { //SOH
                //Validate that the first three tags of a message are, in order: BeginStr,
                //BodyLength, and MsgType.
                if tags.len() == 0 && state.current_tag != BEGINSTR_TAG {
                    return Err(ParseError::BeginStrNotFirstTag);
                }
                else if tags.len() == 1 {
                    if state.current_tag != BODYLENGTH_TAG {
                        return Err(ParseError::BodyLengthNotSecondTag);
                    }

                    //Body length must be a valid positive number or else the rest of the message
                    //is garbage.
                    match u64::from_str_radix(&state.current_string,10) {
                        Ok(length) => state.body_length = length,
                        Err(_) => return Err(ParseError::BodyLengthNotNumber),
                    }
                }
                else if tags.len() == 2 && state.current_tag != MSGTYPE_TAG {
                    return Err(ParseError::MsgTypeNotThirdTag);
                }

                //Make sure checksum checks out when done reading a message.
                let mut is_message_end = false;
                if state.current_tag == CHECKSUM_TAG {
                    try!(state.validate_checksum(&state.current_string));
                    is_message_end = true;
                }

                //Store tag with value.
                let mut tag_in_group = false;
                let mut group_end = false;
                loop {
                    if let Some(ref mut tag_rule_mode) = state.tag_rule_mode_stack.last_mut() {
                        if let TagRuleMode::RepeatingGroups(ref mut parse_repeating_group_state) = **tag_rule_mode {
                            let mut prgs = parse_repeating_group_state;
                            if state.current_tag == prgs.repeating_group_tags.first_tag {
                                let mut group = HashMap::new();
                                group.insert(state.current_tag.clone(),TagValue::String(state.current_string.clone()));
                                prgs.groups.push(group);
                                prgs.current_group_available_tags = prgs.repeating_group_tags.other_tags.clone();

                                if prgs.groups.len() > prgs.group_count {
                                    return Err(ParseError::RepeatingGroupTagWithNoRepeatingGroup(state.current_tag));
                                }
                                tag_in_group = true;
                            }
                            else if prgs.current_group_available_tags.remove(&state.current_tag) {
                                prgs.groups.last_mut().unwrap().insert(state.current_tag.clone(),TagValue::String(state.current_string.clone()));
                                tag_in_group = true;
                            }
                            else {
                                if prgs.groups.last().unwrap().contains_key(&state.current_tag) {
                                    return Err(ParseError::DuplicateTag(state.current_tag));
                                }
                                else if prgs.groups.len() < prgs.group_count {
                                    return Err(ParseError::NonRepeatingGroupTagInRepeatingGroup(state.current_tag));
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
                        state.fold_top_repeating_group_down(&mut tags);
                        group_end = false;
                    }
                    else {
                        break;
                    }
                }
                if !tag_in_group && tags.insert(state.current_tag.clone(),TagValue::String(state.current_string.clone())).is_some() {
                    return Err(ParseError::DuplicateTag(state.current_tag));
                }

                if is_message_end {
                    break;
                }

                //Check if this tag indicates that a specific tag must follow this one.
                if let Some(following_tag) = length_to_value_tags.get(&state.current_tag) {
                    state.tag_rule_mode_stack.push(TagRuleMode::LengthThenValue(following_tag.clone()));
                }
                if let Some(repeating_group) = repeating_group_tags.get(&state.current_tag) {
                    match state.get_number_from_str_by_top_tag::<usize>(&tags,&state.current_tag) {
                        Ok(group_count) => {
                            if group_count > 0 {
                                state.tag_rule_mode_stack.push(TagRuleMode::RepeatingGroups(ParseRepeatingGroupState {
                                    number_of_tag: state.current_tag.clone(),
                                    group_count: group_count,
                                    repeating_group_tags: repeating_group.clone(),
                                    current_group_available_tags: HashSet::new(),
                                    groups: Vec::new(),
                                }));
                                state.tag_rule_mode_stack.push(TagRuleMode::RepeatingGroupStart(repeating_group.first_tag.clone()));
                            }
                        },
                        Err(parse_error) => return Err(parse_error),
                    }
                }

                state.previous_tag = state.current_tag.clone();
                state.current_string = String::new()
            }
            //Byte is part of a tag or value.
            _ => {
                state.current_string.push(c as char);
            }
        }

        state.index += 1;
    }

    //Make sure all required tags are specified.
    //TODO: The required tags might vary based on msgtype.
    for tag in &[BEGINSTR_TAG,BODYLENGTH_TAG,MSGTYPE_TAG,CHECKSUM_TAG] {
        if !tags.contains_key(*tag) {
            return Err(ParseError::MissingRequiredTag(String::from(*tag)));
        }
    }

    return Ok(tags);
}

pub fn print_group(group: &HashMap<String,TagValue>,depth: u8) {
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

