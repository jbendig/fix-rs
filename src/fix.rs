use std::collections::{HashMap,HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::mem;
use std::str::FromStr;
use field::Action;
use message::{Meta,Message,NullMessage};

//TODO: Support configuration settings for things like MAX_VALUE_LENGTH, MAX_BODY_LENGTH,
//      MAX_TAG_LENGTH, MAX_CHECKSUM_LENGTH(might just hard code it...), the size of a "Length" and
//      other types.

const BEGINSTR_TAG: &'static str = "8";
const BODYLENGTH_TAG: &'static str = "9";
const MSGTYPE_TAG: &'static str = "35";
const CHECKSUM_TAG: &'static str = "10";

#[derive(Debug)]
pub enum ParseError {
    MissingRequiredTag(String), //Required tag was not included in message.
    BeginStrNotFirstTag,
    BodyLengthNotSecondTag,
    BodyLengthNotNumber,
    MsgTypeNotThirdTag,
    MsgTypeUnknown(String), //Message type not in dictionary passed to Parser::new().
    ChecksumNotLastTag, //Checksum is not exactly where BodyLength says it should be.
    ChecksumDoesNotMatch(u8,u8), //Calculated checksum, Stated checksum
    ChecksumNotNumber,
    DuplicateTag(String),
    UnexpectedTag(String), //Tag found does not belong to the current message type.
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
            ParseError::MsgTypeUnknown(ref msg_type) => write!(f,"ParseError::MsgTypeUnknown({})",msg_type),
            ParseError::ChecksumNotLastTag => write!(f,"ParseError::ChecksumNotLastTag"),
            ParseError::ChecksumDoesNotMatch(ref calculated_checksum,ref stated_checksum) => write!(f,"ParseError::ChecksumDoesNotMatch({},{})",calculated_checksum,stated_checksum),
            ParseError::ChecksumNotNumber => write!(f,"ParseError::ChecksumNotNumber"),
            ParseError::DuplicateTag(ref tag) => write!(f,"ParseError::DuplicateTag({})",tag),
            ParseError::UnexpectedTag(ref tag) => write!(f,"ParseError::UnexpectedTag({})",tag),
            ParseError::WrongFormatTag(ref tag) => write!(f,"ParseError::WrongFormatTag({})",tag),
            ParseError::MissingPrecedingLengthTag(ref value_tag) => write!(f,"ParseError::MissingPrecedingLengthTag({})",value_tag),
            ParseError::MissingFollowingLengthTag(ref length_tag) => write!(f,"ParseError::MissingFollowingLengthTag({})",length_tag),
            ParseError::NonRepeatingGroupTagInRepeatingGroup(ref tag) => write!(f,"ParseError::NonRepeatingGroupTagInRepeatingGroup({})",tag),
            ParseError::RepeatingGroupTagWithNoRepeatingGroup(ref tag) => write!(f,"ParseError::RepeatingGroupTagWithNoRepeatingGroup({})",tag),
            ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(ref number_of_tag) => write!(f,"ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag({})",number_of_tag),
        }
    }
}

struct ParseGroupState {
    remaining_fields: HashMap<&'static str,Action>,
    remaining_required_fields: HashSet<&'static str>,
    message: Box<Message>,
}

struct ParseRepeatingGroupState {
    number_of_tag: String,
    group_count: usize,
    group_template: Box<Message>,
    first_tag: &'static str,
    groups: Vec<ParseGroupState>,
}

impl ParseRepeatingGroupState {
    fn is_last_group_complete(&self) -> Result<(),ParseError> {
        //Check if the last group has had all of its required fields specified.
        if let Some(last_group) = self.groups.last() {
            if let Some(tag) = last_group.remaining_required_fields.iter().next() {
                return Err(ParseError::MissingRequiredTag(String::from(*tag)));
            }
        }

        Ok(())
    }
}

enum TagRuleMode {
    LengthThenValue(String,usize),
    RepeatingGroups(Box<ParseRepeatingGroupState>),
    RepeatingGroupStart(&'static str),
}

#[derive(PartialEq)]
enum FoundMessage {
    NotFound,
    FirstByte,
    SecondByte,
}

pub struct Parser {
    message_dictionary: HashMap<&'static str,Box<Message>>,
    value_to_length_tags: HashMap<&'static str,&'static str>,
    found_message: FoundMessage,
    current_tag: String,
    current_string: String,
    protocol: String,
    body_length: u64,
    checksum: u8,
    body_remaining_length: u64, //TODO: Do we really need this to be this long?
    previous_tag: String,
    next_tag_checksum: bool,
    tag_rule_mode_stack: Vec<Box<TagRuleMode>>,
    fast_track_bytes_remaining: usize,
    found_tag_count: usize,
    remaining_fields: HashMap<&'static str,Action>,
    remaining_required_fields: HashSet<&'static str>,
    current_message: Box<Message>,
    pub messages: Vec<Box<Message>>,
}

impl Parser {
    pub fn new(message_dictionary: HashMap<&'static str,Box<Message>>) -> Parser {
        //Perform a sanity check to make sure message dictionary was defined correctly. For now,
        //validate_message_dictionary() panics on failure because dictionaries should be composed
        //using a compile time macro. Thus, there's no practical reason to try and recover.
        Parser::validate_message_dictionary(&message_dictionary);

        //Walk every type of message provided and find any fields that define a
        //Action::ConfirmPreviousTag and add it to this map. This way we can check while parsing if
        //the previous tag matches the required tag. This is an optional sanity check that's
        //provided for better error messages but probably isn't needed in practice.
        let mut value_to_length_tags = HashMap::new();
        let mut message_stack = Vec::from_iter(message_dictionary.iter().map(|(_,message)| { message.clone_into_box() }));
        while let Some(message) = message_stack.pop() {
            for (tag,action) in message.fields() {
                match action {
                    Action::ConfirmPreviousTag{ previous_tag } => {
                        value_to_length_tags.insert(tag,previous_tag);
                    },
                    Action::BeginGroup{ message } => {
                        message_stack.push(message.clone_into_box());
                    },
                    _ => {}
                }
            }
        }

        Parser {
            message_dictionary: message_dictionary,
            value_to_length_tags: value_to_length_tags,
            found_message: FoundMessage::NotFound,
            current_tag: String::new(),
            current_string: String::new(),
            protocol: String::new(),
            body_length: 0,
            checksum: 0,
            body_remaining_length: 0,
            previous_tag: String::new(),
            next_tag_checksum: false,
            tag_rule_mode_stack: Vec::new(),
            fast_track_bytes_remaining: 0,
            found_tag_count: 0,
            remaining_fields: HashMap::new(),
            remaining_required_fields: HashSet::new(),
            current_message: Box::new(NullMessage {}),
            messages: Vec::new(),
        }
    }

    pub fn reset_parser(&mut self) {
        self.found_message = FoundMessage::NotFound;
        self.current_tag.clear();
        self.current_string.clear();
        self.protocol.clear();
        self.body_length = 0;
        self.checksum = 0;
        self.body_remaining_length = 0;
        self.previous_tag.clear();
        self.next_tag_checksum = false;
        self.tag_rule_mode_stack.clear();
        self.fast_track_bytes_remaining = 0;
        self.found_tag_count = 0;
        self.remaining_fields.clear();
        self.remaining_required_fields.clear();
        self.current_message = Box::new(NullMessage {});
    }

    pub fn validate_message_dictionary(message_dictionary: &HashMap<&'static str,Box<Message>>) {
        enum MessageType {
            Standard,
            RepeatingGroup,
        }

        //Start by walking the message_dictionary and collecting every possible message format --
        //including repeating and nested repeating groups.
        let mut all_messages = Vec::new();
        let mut message_stack = Vec::from_iter(message_dictionary.iter().map(|(_,message)| { (MessageType::Standard,message.clone_into_box()) }));
        while let Some((message_type,message)) = message_stack.pop() {
            for action in message.fields().values() {
                if let Action::BeginGroup{ ref message } = *action {
                    message_stack.push((MessageType::RepeatingGroup,message.clone_into_box()));
                }
            }
            all_messages.push((message_type,message));
        }

        //All messages must have at least one field. All repeating group messages must make the
        //first field required.
        for &(ref message_type,ref message) in &all_messages {
            let first_field = message.first_field();
            let fields = message.fields();
            let required_fields = message.required_fields();

            if fields.is_empty() {
                panic!("Found message with no fields.");
            }

            if !fields.contains_key(first_field) {
                panic!("Found message where first_field() is not in fields().");
            }

            if let MessageType::RepeatingGroup = *message_type {
                if !required_fields.contains(first_field) {
                    panic!("Found message where first_field() is not in required_fields().");
                }
            }
        }

        //The required fields specified in a message must be a subset of the fields.
        for &(_,ref message) in &all_messages {
            let fields = message.fields();
            let required_fields = message.required_fields();

            for required_field in required_fields {
                if !fields.contains_key(required_field) {
                    panic!("Found message where required_fields() is not a subset of fields().");
                }
            }
        }

        //Fields that specify Action::PrepareForBytes have exactly one matching field that
        //specifies Action::ConfirmPreviousTag within the same message.
        for &(_,ref message) in &all_messages {
            let fields = message.fields();

            for (tag,action) in &fields {
                match *action {
                    Action::PrepareForBytes{ bytes_tag } => {
                        if let Some(bytes_action) = fields.get(bytes_tag) {
                            if let Action::ConfirmPreviousTag{ previous_tag } = *bytes_action {
                                if previous_tag != *tag {
                                    panic!("Found field \"{}\" that defines Action::PrepareForBytes but matching \"{}\" field's Action::ConfirmPreviousTag is not circular.",tag,bytes_tag);
                                }
                            }
                            else {
                                panic!("Found field \"{}\" that defines Action::PrepareForBytes but matching \"{}\" field does not define Action::ConfirmPreviousTag.",tag,bytes_tag);
                            }
                        }
                        else {
                            panic!("Found field \"{}\" that defines Action::PrepareForBytes but no matching \"{}\" field was found.",tag,bytes_tag);
                        }
                    },
                    Action::ConfirmPreviousTag{ previous_tag } => {
                        if let Some(previous_action) = fields.get(previous_tag) {
                            if let Action::PrepareForBytes{ bytes_tag } = *previous_action {
                                if bytes_tag != *tag {
                                    panic!("Found field \"{}\" that defines Action::ConfirmPreviousTag but matching \"{}\" field's Action::PrepareForBytes is not circular.",tag,previous_tag);
                                }
                            }
                            else {
                                panic!("Found field \"{}\" that defines Action::ConfirmPreviousTag but matching \"{}\" field does not define Action::PrepareForBytes.",tag,previous_tag)
                            }
                        }
                        else {
                            panic!("Found field \"{}\" that defines Action::ConfirmPreviousTag but no matching \"{}\" field was found.",tag,previous_tag);
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    fn update_book_keeping(&mut self,c: u8) -> Result<(),ParseError> {
        //Update checksum.
        self.checksum = self.checksum.overflowing_add(c).0;

        //Update where we are when reading the body in case message is malformed and the checksum
        //is not at the offset where it's supposed to be.
        self.body_remaining_length = self.body_remaining_length.overflowing_sub(1).0;
        if self.body_remaining_length == 0 {
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

        Ok(())
    }

    fn validate_checksum(&mut self) -> Result<(),ParseError> {
        //Remove checksum tag that should not be part of the current checksum.
        let mut checksum = self.checksum.overflowing_sub(b'1' + b'0' + b'=' + b'\x01').0;
        let checksum_string = self.current_string.as_str();
        for c in checksum_string.bytes() {
            checksum = checksum.overflowing_sub(c).0;
        }

        match u8::from_str(checksum_string) {
            Ok(stated_checksum) => if checksum != stated_checksum {
                return Err(ParseError::ChecksumDoesNotMatch(checksum,stated_checksum));
            },
            Err(_) => return Err(ParseError::ChecksumNotNumber),
        }

        self.checksum = checksum;
        Ok(())
    }

    fn scan_for_message(&mut self,index: &mut usize,message_bytes: &[u8]) {
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

    fn fast_track_read_bytes(&mut self,index: &mut usize,message_bytes: &[u8]) -> Result<(),ParseError> {
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

        Ok(())
    }

    #[allow(match_same_arms)]
    fn handle_action_after_value(&mut self,action: Action) -> Result<bool,ParseError> {
        let mut skip_set_value = false;

        match action {
            Action::Nothing => {}, //Nothing special to be done
            Action::AddRequiredTags(_) => { //Make the stated tags required.
                //TODO: Need to make sure these new tags have not already been
                //found before adding them to the required tag set.
                unimplemented!();
            },
            Action::BeginGroup{ message: repeating_group_template } => {
                match usize::from_str(self.current_string.as_str()) {
                    Ok(group_count) if group_count > 0 => {
                        let first_field = repeating_group_template.first_field();
                        self.tag_rule_mode_stack.push(Box::new(TagRuleMode::RepeatingGroups(Box::new(ParseRepeatingGroupState {
                            number_of_tag: self.current_tag.clone(),
                            group_count: group_count,
                            first_tag: repeating_group_template.first_field(),
                            groups: Vec::new(),
                            group_template: repeating_group_template,
                        }))));
                        self.tag_rule_mode_stack.push(Box::new(TagRuleMode::RepeatingGroupStart(first_field)));
                    },
                    Ok(_) => {}, //group_count == 0. Just ignore.
                    Err(_) => return Err(ParseError::WrongFormatTag(self.current_tag.clone())),
                }
                skip_set_value = true;
            },
            Action::PrepareForBytes{ bytes_tag } => {
                //Next tag should be 'bytes_tag' and its value is made up of
                //the number of bytes specified in this tag.
                match usize::from_str(self.current_string.as_str()) {
                    Ok(byte_count) => self.tag_rule_mode_stack.push(Box::new(TagRuleMode::LengthThenValue(String::from(bytes_tag),byte_count))),
                    Err(_) => return Err(ParseError::WrongFormatTag(self.current_tag.clone())),
                }
                skip_set_value = true;
            },
            Action::ConfirmPreviousTag{ .. } => {}, //Must be checked after parsing tag and before parsing value.
        }

       Ok(skip_set_value)
    }

    fn fold_top_repeating_group_down(&mut self) {
        let mut folded_down = false;
        {
            let mut tag_rule_mode_stack_iter = self.tag_rule_mode_stack.iter_mut().rev();
            if let Some(first_tag_rule_mode) = tag_rule_mode_stack_iter.next() {
                if let TagRuleMode::RepeatingGroups(ref mut prgs) = **first_tag_rule_mode {
                    for tag_rule_mode in tag_rule_mode_stack_iter {
                        if let TagRuleMode::RepeatingGroups(ref mut parent_prgs) = **tag_rule_mode {
                            let mut groups = mem::replace(&mut prgs.groups,Vec::new());
                            parent_prgs.groups.last_mut().unwrap().message.set_groups(
                                prgs.number_of_tag.as_str(),
                                &Vec::from_iter(groups.drain(0..).map(|group| { group.message }))
                            );
                            folded_down = true;
                        }
                    }

                    if !folded_down {
                        let mut groups = mem::replace(&mut prgs.groups,Vec::new());
                        self.current_message.set_groups(
                            prgs.number_of_tag.as_str(),
                            &Vec::from_iter(groups.drain(0..).map(|group| { group.message }))
                        );
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

    pub fn parse(&mut self,message_bytes: &[u8]) -> (usize,Result<(),ParseError>) {
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

    fn parse_private(&mut self,index: &mut usize,message_bytes: &[u8]) -> Result<(),ParseError> {
        //Start by searching for the start of a message unless resuming.
        self.scan_for_message(index,message_bytes);

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
                        match *tag_rule_mode {
                            TagRuleMode::LengthThenValue(ref value_tag,byte_count) => {
                                if self.current_tag != *value_tag {
                                    return Err(ParseError::MissingFollowingLengthTag(self.previous_tag.clone()));
                                }

                                //Fast track to read in the specified number of bytes.
                                self.fast_track_bytes_remaining = byte_count;
                                *index += 1;
                                try!(self.fast_track_read_bytes(index,&message_bytes));
                                *index -= 1;
                            },
                            TagRuleMode::RepeatingGroupStart(first_repeating_group_tag) => {
                                //Sanity check that the first tag in a repeating group is what is
                                //expected.
                                if self.current_tag != *first_repeating_group_tag {
                                    return Err(ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(self.previous_tag.clone()));
                                }
                            },
                            _ => self.tag_rule_mode_stack.push(tag_rule_mode),
                        }
                    }
                    //Otherwise, if the current tag requires some preceding tag that wasn't found,
                    //return an error. This is a sanity check.
                    else if let Some(required_preceding_tag) = self.value_to_length_tags.get(self.current_tag.as_str()) {
                        if required_preceding_tag != &self.previous_tag {
                            return Err(ParseError::MissingPrecedingLengthTag(self.current_tag.clone()));
                        }
                    }
                },
                //Byte indicates a vale has finished being read. Now both the tag and value are known.
                b'\x01' => { //SOH
                    //Validate that the first three tags of a message are, in order: BeginStr,
                    //BodyLength, and MsgType.
                    if self.found_tag_count == 0 {
                        if self.current_tag != BEGINSTR_TAG {
                            return Err(ParseError::BeginStrNotFirstTag);
                        }
                        self.protocol = mem::replace(&mut self.current_string,String::new());
                    }
                    else if self.found_tag_count == 1 {
                        if self.current_tag != BODYLENGTH_TAG {
                            return Err(ParseError::BodyLengthNotSecondTag);
                        }

                        //Body length must be a valid positive number or else the rest of the message
                        //is garbage.
                        match u64::from_str(&self.current_string) {
                            Ok(length) => {
                                self.body_length = length;
                                self.body_remaining_length = length;
                            },
                            Err(_) => return Err(ParseError::BodyLengthNotNumber),
                        }
                    }
                    else if self.found_tag_count == 2 {
                        if self.current_tag != MSGTYPE_TAG {
                            return Err(ParseError::MsgTypeNotThirdTag);
                        }
                        else if let Some(message) = self.message_dictionary.get(self.current_string.as_str()) {
                            self.current_message = (*message).clone_into_box();
                            self.remaining_fields = message.fields();
                            self.remaining_required_fields = message.required_fields();
                        }
                        else {
                            return Err(ParseError::MsgTypeUnknown(self.current_tag.clone()));
                        }
                    }
                    else {
                        //Make sure checksum checks out when done reading a message.
                        let is_message_end = if self.current_tag == CHECKSUM_TAG {
                            try!(self.validate_checksum());
                            true
                        }
                        else {
                            false
                        };

                        //Store tag with value.
                        let mut tag_in_group = false;
                        let mut group_end = false;
                        loop {
                            let mut some_action = None;
                            if let Some(ref mut tag_rule_mode) = self.tag_rule_mode_stack.last_mut() {
                                if let TagRuleMode::RepeatingGroups(ref mut prgs) = ***tag_rule_mode {
                                    if self.current_tag == prgs.first_tag {
                                        //Make sure previous group has all required tags specified
                                        //before we start a new one.
                                        try!(prgs.is_last_group_complete());

                                        //Begin a new group.
                                        let group = prgs.group_template.clone_into_box();
                                        let remaining_fields = prgs.group_template.fields();
                                        let remaining_required_fields = prgs.group_template.required_fields();
                                        prgs.groups.push(ParseGroupState {
                                            message: group,
                                            remaining_fields: remaining_fields,
                                            remaining_required_fields: remaining_required_fields,
                                        });

                                        //Make sure we haven't exceeded the number of repeating
                                        //groups originally stated.
                                        if prgs.groups.len() > prgs.group_count {
                                            return Err(ParseError::RepeatingGroupTagWithNoRepeatingGroup(self.current_tag.clone()));
                                        }
                                    }

                                    if let Some(group) = prgs.groups.last_mut() {
                                        if let Some(action) = group.remaining_fields.remove(self.current_tag.as_str()) {
                                            //Try to mark the field as found in case it's required.
                                            group.remaining_required_fields.remove(self.current_tag.as_str());

                                            //Apply parsed value to group.
                                            if let Action::BeginGroup{ .. } = action {} //Ignore begin group tags, they will be handled below.
                                            else {
                                                if !group.message.set_value(self.current_tag.as_str(),self.current_string.as_bytes()) {
                                                    return Err(ParseError::WrongFormatTag(self.current_tag.clone()));
                                                }
                                            }

                                            //Save action to handle later.
                                            some_action = Some(action);

                                            tag_in_group = true;
                                        }
                                    }

                                    if !tag_in_group {
                                        //Figure out if this is an error or the end of the group.
                                        if prgs.group_template.fields().contains_key(self.current_tag.as_str()) {
                                            return Err(ParseError::DuplicateTag(self.current_tag.clone()));
                                        }
                                        else if prgs.groups.len() < prgs.group_count {
                                            return Err(ParseError::NonRepeatingGroupTagInRepeatingGroup(self.current_tag.clone()));
                                        }

                                        //Make sure all required tags have been specified.
                                        try!(prgs.is_last_group_complete());

                                        //Tag does not belong in this group and all stated groups are
                                        //accounted for.
                                        group_end = true;
                                    }
                                }
                            }

                            //Out of the way result handling to appease the borrow checker.
                            if let Some(action) = some_action {
                                try!(self.handle_action_after_value(action));
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

                        let mut skip_set_value = false;
                        if !is_message_end && !tag_in_group {
                            //Mark field as found if required so we can quickly check if all required
                            //fields were found once we are done parsing the message.
                            self.remaining_required_fields.remove(self.current_tag.as_str());

                            //Mark field as found so we can quickly check if a duplicate tag was
                            //encountered. As a side effect, we also handle any tag specific
                            //actions in consequence of being encountered.
                            if let Some(action) = self.remaining_fields.remove(self.current_tag.as_str()) {
                                skip_set_value = try!(self.handle_action_after_value(action));
                            }
                            else {
                                return Err(ParseError::UnexpectedTag(self.current_tag.clone()));
                            }
                        }

                        if !is_message_end && !tag_in_group && !skip_set_value && !self.current_message.set_value(self.current_tag.as_str(),self.current_string.as_bytes()) {
                            //This means either the key could not be found in the message (an
                            //internal error) or the bytes are not formatted correctly. For
                            //example, maybe it was suppose to be a number but non-digit characters
                            //were used.
                            return Err(ParseError::WrongFormatTag(self.current_tag.clone()));
                        }

                        if is_message_end {
                            //Make sure all required tags are specified.
                            if let Some(tag) = self.remaining_required_fields.iter().next() {
                                return Err(ParseError::MissingRequiredTag(String::from(*tag)));
                            }

                            //Store meta info about the message. Mainly for debugging.
                            self.current_message.set_meta(Meta {
                                protocol: mem::replace(&mut self.protocol,String::new()),
                                body_length: self.body_length,
                                checksum: self.checksum,
                            });

                            //Save message.
                            self.messages.push(mem::replace(&mut self.current_message,Box::new(NullMessage {})));

                            //Prepare for the next message.
                            self.reset_parser();
                            *index += 1;

                            //Scan to the next message in case there is garbage between the end of this
                            //one and the beginning of the next.
                            self.scan_for_message(index,message_bytes);
                            continue;
                        }
                    }

                    self.previous_tag = self.current_tag.clone();
                    self.current_string = String::new();
                    self.found_tag_count += 1;
                }
                //Byte is part of a tag or value.
                _ => {
                    self.current_string.push(c as char);
                }
            }

            *index += 1;
        }

        Ok(())
    }
}

