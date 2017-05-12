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

use std::borrow::Borrow;
use std::collections::{HashMap,HashSet};
use std::collections::hash_map::Entry;
use std::fmt;
use std::iter::FromIterator;
use std::mem;
use std::str::FromStr;

use constant::{FIX_4_0_BEGIN_STRING,FIX_4_1_BEGIN_STRING,FIX_4_2_BEGIN_STRING,FIX_4_3_BEGIN_STRING,FIX_4_4_BEGIN_STRING,FIXT_1_1_BEGIN_STRING,TAG_END,VALUE_END};
use dictionary::messages::{Logon,NullMessage};
use dictionary::fields::{ApplVerID,SenderCompID,TargetCompID};
use dictionary::field_types::other::DefaultApplVerIDFieldType;
use field::Field;
use field_tag::FieldTag;
use field_type::FieldType;
use fix_version::FIXVersion;
use fixt::message::{BuildFIXTMessage,FIXTMessage};
use hash::BuildFieldHasher;
use message::{BuildMessage,FieldHashMap,FieldHashSet,Meta,Message,SetValueError};
use message_version::MessageVersion;
use rule::Rule;

//TODO: Support configuration settings for things like MAX_VALUE_LENGTH, MAX_BODY_LENGTH,
//      MAX_TAG_LENGTH, the size of a "Length" and other types.

const BEGINSTR_TAG_BYTES: &'static [u8] = b"8";
const BEGINSTR_TAG: FieldTag = FieldTag(8);
const BODYLENGTH_TAG_BYTES: &'static [u8] = b"9";
const BODYLENGTH_TAG: FieldTag = FieldTag(9);
const MSGTYPE_TAG: FieldTag = FieldTag(35);
const CHECKSUM_TAG_BYTES: &'static [u8] = b"10";
const CHECKSUM_TAG: FieldTag = FieldTag(10);

pub enum ParseError {
    MissingRequiredTag(FieldTag,Box<FIXTMessage + Send>), //Required tag was not included in message.
    MissingConditionallyRequiredTag(FieldTag,Box<FIXTMessage + Send>), //Conditionally required tag was not included in message.
    BeginStrNotFirstTag,
    BodyLengthNotSecondTag,
    BodyLengthNotNumber,
    MsgTypeNotThirdTag,
    MsgTypeUnknown(Vec<u8>), //Message type not in dictionary passed to Parser::new().
    SenderCompIDNotFourthTag,
    TargetCompIDNotFifthTag,
    ApplVerIDNotSixthTag, //ApplVerID must be the sixth tag if specified at all.
    ChecksumNotLastTag, //Checksum is not exactly where BodyLength says it should be.
    ChecksumDoesNotMatch(u8,u8), //Calculated checksum, Stated checksum
    ChecksumWrongFormat,
    DuplicateTag(FieldTag),
    UnexpectedTag(FieldTag), //Tag found does not belong to the current message type.
    UnknownTag(FieldTag), //Tag found does not beling to any known message.
    WrongFormatTag(FieldTag),
    OutOfRangeTag(FieldTag),
    NoValueAfterTag(FieldTag),
    MissingPrecedingLengthTag(FieldTag), //Tag was found that requires a preceding length tag which was omitted.
    MissingFollowingLengthTag(FieldTag), //Length tag that was specified does not match the following tag.
    NonRepeatingGroupTagInRepeatingGroup(FieldTag), //Tag that doesn't belong in a repeating group was found.
    RepeatingGroupTagWithNoRepeatingGroup(FieldTag), //Repeating group tag was found outside of a repeating group.
    MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(FieldTag), //Tag indicating start of a repeating group was not found immediatelly after tag indicating the number of repeating groups.
    MessageSizeTooBig,
}

fn tag_to_string(tag: &[u8]) -> String {
    String::from_utf8_lossy(tag).into_owned()
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::MissingRequiredTag(ref tag,_) => write!(f,"ParseError::MissingRequiredTag({})",tag),
            ParseError::MissingConditionallyRequiredTag(ref tag,_) => write!(f,"ParseError::MissingConditionallyRequiredTag({})",tag),
            ParseError::BeginStrNotFirstTag => write!(f,"ParseError::BeginStrNotFirstTag"),
            ParseError::BodyLengthNotSecondTag => write!(f,"ParseError::BodyLengthNotSecondTag"),
            ParseError::BodyLengthNotNumber => write!(f,"ParseError::BodyLengthNotNumber"),
            ParseError::MsgTypeNotThirdTag => write!(f,"ParseError::MsgTypeNotThirdTag"),
            ParseError::MsgTypeUnknown(ref msg_type) => write!(f,"ParseError::MsgTypeUnknown({})",tag_to_string(msg_type)),
            ParseError::SenderCompIDNotFourthTag => write!(f,"ParseError::SenderCompIDNotFourthTag"),
            ParseError::TargetCompIDNotFifthTag => write!(f,"ParseError::TargetCompIDNotFifthTag"),
            ParseError::ApplVerIDNotSixthTag => write!(f,"ParseError::ApplVerIDNotSixthTag"),
            ParseError::ChecksumNotLastTag => write!(f,"ParseError::ChecksumNotLastTag"),
            ParseError::ChecksumDoesNotMatch(ref calculated_checksum,ref stated_checksum) => write!(f,"ParseError::ChecksumDoesNotMatch({},{})",calculated_checksum,stated_checksum),
            ParseError::ChecksumWrongFormat => write!(f,"ParseError::ChecksumWrongFormat"),
            ParseError::DuplicateTag(ref tag) => write!(f,"ParseError::DuplicateTag({})",tag),
            ParseError::UnexpectedTag(ref tag) => write!(f,"ParseError::UnexpectedTag({})",tag),
            ParseError::UnknownTag(ref tag) => write!(f,"ParseError::UnknownTag({})",tag),
            ParseError::WrongFormatTag(ref tag) => write!(f,"ParseError::WrongFormatTag({})",tag),
            ParseError::OutOfRangeTag(ref tag) => write!(f,"ParseError::OutOfRangeTag({})",tag),
            ParseError::NoValueAfterTag(ref tag) => write!(f,"ParseError::NoValueAfterTag({})",tag),
            ParseError::MissingPrecedingLengthTag(ref value_tag) => write!(f,"ParseError::MissingPrecedingLengthTag({})",value_tag),
            ParseError::MissingFollowingLengthTag(ref length_tag) => write!(f,"ParseError::MissingFollowingLengthTag({})",length_tag),
            ParseError::NonRepeatingGroupTagInRepeatingGroup(ref tag) => write!(f,"ParseError::NonRepeatingGroupTagInRepeatingGroup({})",tag),
            ParseError::RepeatingGroupTagWithNoRepeatingGroup(ref tag) => write!(f,"ParseError::RepeatingGroupTagWithNoRepeatingGroup({})",tag),
            ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(ref number_of_tag) => write!(f,"ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag({})",number_of_tag),
            ParseError::MessageSizeTooBig => write!(f,"ParseError::MessageSizeTooBig"),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <ParseError as fmt::Display>::fmt(self,f)
    }
}

struct ParseGroupState {
    remaining_fields: FieldHashMap,
    remaining_required_fields: FieldHashSet,
    message: Box<Message>,
}

struct ParseRepeatingGroupState {
    number_of_tag: FieldTag,
    group_count: usize,
    group_builder: Box<BuildMessage>,
    first_tag: FieldTag,
    groups: Vec<ParseGroupState>,
}

impl ParseRepeatingGroupState {
    fn check_last_group_complete(&self,message_version: MessageVersion,missing_tag: &mut FieldTag,missing_conditional_tag: &mut FieldTag) {
        //Mark the missing tag so we can emit an error when done parsing.
        //The error cannot be emitted immediately because the MsgSeqNum
        //might not have been parsed yet and it's required in order to
        //respond with a Reject message.
        //TODO: The above reasoning is likely not true. Although having the MsgType is useful in
        //the Reject, the MsgSeqNum is not actually needed at all. We can exit quickly if it turns
        //out to be more performant.

        if !missing_tag.is_empty() || !missing_conditional_tag.is_empty() {
            return;
        }

        //Check if the last group has had all of its required fields specified.
        if let Some(last_group) = self.groups.last() {
            if let Some(tag) = last_group.remaining_required_fields.iter().next() {
                *missing_tag = *tag;
                return;
            }

            //TODO: Add test to confirm conditional require works for this and outer message
            //fields.
            for tag in last_group.message.conditional_required_fields(message_version) {
                if last_group.remaining_fields.contains_key(&tag) {
                    *missing_conditional_tag = tag;
                    return;
                }
            }
        }
    }
}

enum TagRuleMode {
    LengthThenValue(FieldTag,usize),
    RepeatingGroups(Box<ParseRepeatingGroupState>),
    RepeatingGroupStart(FieldTag),
}

#[derive(PartialEq)]
enum FoundMessage {
    NotFound,
    FirstByte,
    SecondByte,
}

#[derive(PartialEq)]
enum MessageEnd {
    Yes,
    YesButStop,
    No
}

fn ascii_to_integer<T: FromStr>(ascii_bytes: &Vec<u8>) -> Result<T,<T as FromStr>::Err> {
    //Using String::from_utf8_lossy is faster than using str::from_utf8_unchecked() according to
    //the benchmark.
    T::from_str(String::from_utf8_lossy(ascii_bytes.as_slice()).borrow())
}

fn set_message_value<T: Message + ?Sized>(message: &mut T,tag: FieldTag,bytes: &[u8]) -> Result<(),ParseError> {
    if let Err(e) = message.set_value(tag,bytes) {
        match e {
            //This means either the key could not be found in the message (an
            //internal error) or the bytes are not formatted correctly. For
            //example, maybe it was suppose to be a number but non-digit characters
            //were used.
            SetValueError::WrongFormat => return Err(ParseError::WrongFormatTag(tag)),
            //Value was formatted correctly but outside of the defined range or not
            //part of the list of allowed choices.
            SetValueError::OutOfRange => return Err(ParseError::OutOfRangeTag(tag)),
        };
    }

    Ok(())
}

pub struct Parser {
    message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,
    max_message_length: u64,
    default_message_version: MessageVersion,
    default_message_type_version: HashMap<&'static [u8],MessageVersion>,
    value_to_length_tags: HashMap<FieldTag,FieldTag>,
    found_message: FoundMessage,
    current_tag: FieldTag, //Tag if completely parsed, otherwise empty.
    current_bytes: Vec<u8>, //Bytes being parsed for current tag or value.
    fix_version: FIXVersion,
    message_version: MessageVersion,
    body_length: u64,
    message_type: Vec<u8>,
    checksum: u8,
    sender_comp_id: Vec<u8>,
    target_comp_id: Vec<u8>,
    body_remaining_length: u64, //TODO: Do we really need this to be this long?
    previous_tag: FieldTag,
    next_tag_checksum: bool,
    tag_rule_mode_stack: Vec<Box<TagRuleMode>>,
    fast_track_bytes_remaining: usize,
    found_tag_count: usize,
    remaining_fields: FieldHashMap,
    remaining_required_fields: FieldHashSet,
    missing_tag: FieldTag,
    missing_conditional_tag: FieldTag,
    current_message: Box<FIXTMessage + Send>,
    pub messages: Vec<Box<FIXTMessage + Send>>,
}

impl Parser {
    pub fn new(message_dictionary: HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>,max_message_length: u64) -> Parser {
        //Perform a sanity check to make sure message dictionary was defined correctly. For now,
        //validate_message_dictionary() panics on failure because dictionaries should be composed
        //using a compile time macro. Thus, there's no practical reason to try and recover.
        Parser::validate_message_dictionary(&message_dictionary);

        //Walk every type of message provided and find any fields that require a
        //Rule::ConfirmPreviousTag and add it to this map. This way we can check while parsing if
        //the previous tag matches the required tag. This is an optional sanity check that's
        //provided for better error messages but probably isn't needed in practice.
        let mut value_to_length_tags = HashMap::new();
        let mut builder_stack = Vec::from_iter(message_dictionary.iter().map(|(_,builder)| { BuildMessage::new_into_box(&**builder) }));
        while let Some(mut builder) = builder_stack.pop() {
            for message_version in MessageVersion::all() {
                for (tag,rule) in builder.fields(message_version) {
                    match rule {
                        Rule::ConfirmPreviousTag{ previous_tag } => {
                            value_to_length_tags.insert(tag,previous_tag);
                        },
                        Rule::BeginGroup{ builder_func } => {
                            builder_stack.push(builder_func());
                        },
                        _ => {}
                    }
                }
            }
        }

        Parser {
            message_dictionary: message_dictionary,
            max_message_length: max_message_length,
            default_message_version: DefaultApplVerIDFieldType::default_value(),
            default_message_type_version: HashMap::new(),
            value_to_length_tags: value_to_length_tags,
            found_message: FoundMessage::NotFound,
            current_tag: FieldTag::empty(),
            current_bytes: Vec::with_capacity(64),
            fix_version: FIXVersion::FIX_4_0,
            message_version: MessageVersion::FIX40,
            body_length: 0,
            message_type: Vec::new(),
            checksum: 0,
            sender_comp_id: Vec::new(),
            target_comp_id: Vec::new(),
            body_remaining_length: 0,
            previous_tag: FieldTag::empty(),
            next_tag_checksum: false,
            tag_rule_mode_stack: Vec::new(),
            fast_track_bytes_remaining: 0,
            found_tag_count: 0,
            remaining_fields: HashMap::with_hasher(BuildFieldHasher),
            remaining_required_fields: HashSet::with_hasher(BuildFieldHasher),
            missing_tag: FieldTag::empty(),
            missing_conditional_tag: FieldTag::empty(),
            current_message: Box::new(NullMessage {}),
            messages: Vec::new(),
        }
    }

    pub fn reset_parser(&mut self) {
        self.found_message = FoundMessage::NotFound;
        self.current_tag = FieldTag::empty();
        self.current_bytes.clear();
        self.body_length = 0;
        self.message_type.clear();
        self.checksum = 0;
        self.sender_comp_id.clear();
        self.target_comp_id.clear();
        self.body_remaining_length = 0;
        self.previous_tag = FieldTag::empty();
        self.next_tag_checksum = false;
        self.tag_rule_mode_stack.clear();
        self.fast_track_bytes_remaining = 0;
        self.found_tag_count = 0;
        self.remaining_fields.clear();
        self.remaining_required_fields.clear();
        self.missing_tag = FieldTag::empty();
        self.missing_conditional_tag = FieldTag::empty();
        self.current_message = Box::new(NullMessage {});
    }

    pub fn set_default_message_version(&mut self,message_version: MessageVersion) {
        self.default_message_version = message_version;
    }

    pub fn clear_default_message_type_versions(&mut self) {
        self.default_message_type_version.clear();
    }

    pub fn set_default_message_type_version(&mut self,tag: &[u8],message_version: MessageVersion) {
        //Set the default version for a specific message type. If the version was already set,
        //there will be no change.

        //TODO: This could be a potential bottleneck. It's done this way because we want the exact
        //&'static [u8] key but only have the &[u8] that looks like the key.
        for message_dictionary_key in self.message_dictionary.keys() {
            if **message_dictionary_key == *tag {
                if let Entry::Vacant(entry) = self.default_message_type_version.entry(message_dictionary_key) {
                    entry.insert(message_version);
                }
                break;
            }
        }
    }

    pub fn max_message_size(&self) -> u64 {
        self.max_message_length
    }

    pub fn validate_message_dictionary(message_dictionary: &HashMap<&'static [u8],Box<BuildFIXTMessage + Send>>) {
        enum MessageType {
            Standard,
            RepeatingGroup,
        }

        //Start by walking the message_dictionary and collecting every possible message format --
        //including repeating and nested repeating groups.
        let mut all_messages = Vec::new();
        let mut builder_stack = Vec::from_iter(message_dictionary.iter().map(|(_,builder)| { (MessageType::Standard,BuildMessage::new_into_box(&**builder)) }));
        while let Some((message_type,mut builder)) = builder_stack.pop() {
            //Prevent lots of duplicates from different message versions.
            let mut found_repeating_groups = HashSet::new();

            for message_version in MessageVersion::all() {
                for (tag,rule) in builder.fields(message_version) {
                    if found_repeating_groups.contains(&tag) {
                        continue;
                    }

                    if let Rule::BeginGroup{ builder_func } = rule {
                        builder_stack.push((MessageType::RepeatingGroup,builder_func()));
                        found_repeating_groups.insert(tag);
                    }
                }
            }

            all_messages.push((message_type,builder));
        }

        //All messages must have at least one field. All repeating group messages must make the
        //first field required. This must all be true for at least one message version.
        for &mut (ref message_type,ref mut builder) in &mut all_messages {
            let mut no_fields = true;
            let mut first_field_not_in_fields = true;
            let mut repeating_group_first_field_not_in_required_fields = true;

            for message_version in MessageVersion::all() {
                let fields = builder.fields(message_version);
                no_fields = fields.is_empty();

                let first_field = builder.first_field(message_version);
                first_field_not_in_fields = !fields.contains_key(&first_field);

                repeating_group_first_field_not_in_required_fields = false;
                let repeating_group = if let MessageType::RepeatingGroup = *message_type {
                    let required_fields = builder.required_fields(message_version);
                    repeating_group_first_field_not_in_required_fields = !required_fields.contains(&first_field);
                    true
                }
                else {
                    false
                };

                if !no_fields && !first_field_not_in_fields && (!repeating_group_first_field_not_in_required_fields || !repeating_group) {
                    repeating_group_first_field_not_in_required_fields = false;
                    break;
                }
            }

            if no_fields {
                panic!("Found message with no fields.");
            }
            else if first_field_not_in_fields {
                panic!("Found message where first_field() is not in fields().");
            }
            else if repeating_group_first_field_not_in_required_fields {
                panic!("Found message where first_field() is not in required_fields().");
            }
        }

        //Run remaining validation against every supported message version.
        for message_version in MessageVersion::all() {
            //The required fields specified in a message must be a subset of the fields.
            for &mut (_,ref mut builder) in &mut all_messages {
                let fields = builder.fields(message_version);
                let required_fields = builder.required_fields(message_version);

                for required_field in required_fields {
                    if !fields.contains_key(&required_field) {
                        panic!("Found message where required_fields() is not a subset of fields().");
                    }
                }
            }

            //Fields that specify Rule::PrepareForBytes have exactly one matching field that
            //specifies Rule::ConfirmPreviousTag within the same message.
            for &mut (_,ref mut builder) in &mut all_messages {
                let fields = builder.fields(message_version);

                for (tag,rule) in &fields {
                    match *rule {
                        Rule::PrepareForBytes{ bytes_tag } => {
                            if let Some(bytes_tag_rule) = fields.get(&bytes_tag) {
                                if let Rule::ConfirmPreviousTag{ previous_tag } = *bytes_tag_rule {
                                    if previous_tag != *tag {
                                        panic!("Found field \"{}\" that defines Rule::PrepareForBytes but matching \"{}\" field's Rule::ConfirmPreviousTag is not circular.",tag,bytes_tag);
                                    }
                                }
                                else {
                                    panic!("Found field \"{}\" that defines Rule::PrepareForBytes but matching \"{}\" field does not define Rule::ConfirmPreviousTag.",tag,bytes_tag);
                                }
                            }
                            else {
                                panic!("Found field \"{}\" that defines Rule::PrepareForBytes but no matching \"{}\" field was found.",tag,bytes_tag);
                            }
                        },
                        Rule::ConfirmPreviousTag{ previous_tag } => {
                            if let Some(previous_tag_rule) = fields.get(&previous_tag) {
                                if let Rule::PrepareForBytes{ bytes_tag } = *previous_tag_rule {
                                    if bytes_tag != *tag {
                                        panic!("Found field \"{}\" that defines Rule::ConfirmPreviousTag but matching \"{}\" field's Rule::PrepareForBytes is not circular.",tag,previous_tag);
                                    }
                                }
                                else {
                                    panic!("Found field \"{}\" that defines Rule::ConfirmPreviousTag but matching \"{}\" field does not define Rule::PrepareForBytes.",tag,previous_tag)
                                }
                            }
                            else {
                                panic!("Found field \"{}\" that defines Rule::ConfirmPreviousTag but no matching \"{}\" field was found.",tag,previous_tag);
                            }
                        },
                        _ => {},
                    }
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
            if c != VALUE_END {
                return Err(ParseError::ChecksumNotLastTag);
            }
            self.next_tag_checksum = true;
        }

        Ok(())
    }

    fn prepare_for_message(&mut self) -> Result<(),ParseError> {
        if let Some(builder) = self.message_dictionary.get_mut(&self.message_type[..]) {
            self.current_message = BuildFIXTMessage::build(&**builder);
            self.remaining_fields = builder.fields(self.message_version);
            self.remaining_required_fields = builder.required_fields(self.message_version);

            return Ok(());
        }

        Err(ParseError::MsgTypeUnknown(self.message_type.clone()))
    }

    fn if_checksum_then_is_last_tag(&self) -> Result<(),ParseError> {
        if (self.current_tag == CHECKSUM_TAG && !self.next_tag_checksum) || (self.current_tag != CHECKSUM_TAG && self.next_tag_checksum) {
            return Err(ParseError::ChecksumNotLastTag);
        }

        Ok(())
    }

    fn validate_checksum(&mut self) -> Result<(),ParseError> {
        //Checksum must be EXACTLY three characters according to FIX 5.0SP2, Volume 6, page 7.
        if self.current_bytes.len() != 3 {
            return Err(ParseError::ChecksumWrongFormat);
        }

        //Remove checksum tag that should not be part of the current checksum.
        let mut checksum = self.checksum.overflowing_sub(CHECKSUM_TAG_BYTES[0] + CHECKSUM_TAG_BYTES[1] + TAG_END + VALUE_END).0;
        let checksum_bytes = &self.current_bytes;
        for c in checksum_bytes {
            checksum = checksum.overflowing_sub(*c).0;
        }

        match ascii_to_integer::<u8>(checksum_bytes) {
            Ok(stated_checksum) => if checksum != stated_checksum {
                return Err(ParseError::ChecksumDoesNotMatch(checksum,stated_checksum));
            },
            Err(_) => return Err(ParseError::ChecksumWrongFormat),
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
            FoundMessage::FirstByte => BEGINSTR_TAG_BYTES[0],
            _ => unreachable!(),
        };
        while *index < message_bytes.len() {
            let byte = message_bytes[*index];

            //Check if "8=" header has been found.
            if byte == TAG_END && previous_byte == BEGINSTR_TAG_BYTES[0] {
                self.found_message = FoundMessage::SecondByte;
                break;
            }

            previous_byte = byte;
            *index += 1;
        }

        if self.found_message == FoundMessage::SecondByte {
            //Act like the BeginStr tag was parsed so we don't duplicate work.
            self.current_tag = BEGINSTR_TAG;
            self.checksum = BEGINSTR_TAG_BYTES[0] + TAG_END;
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

            self.current_bytes.push(c);

            *index += 1;
            self.fast_track_bytes_remaining -= 1;
        }

        Ok(())
    }

    #[allow(match_same_arms)]
    fn handle_rule_after_value(&mut self,rule: &Rule) -> Result<bool,ParseError> {
        let mut skip_set_value = false;

        match rule {
            &Rule::Nothing => {}, //Nothing special to be done
            &Rule::BeginGroup{ builder_func: repeating_group_builder_func } => {
                let repeating_group_builder = repeating_group_builder_func();
                match ascii_to_integer::<usize>(&self.current_bytes) {
                    Ok(group_count) if group_count > 0 => {
                        let first_field = repeating_group_builder.first_field(self.message_version);
                        self.tag_rule_mode_stack.push(Box::new(TagRuleMode::RepeatingGroups(Box::new(ParseRepeatingGroupState {
                            number_of_tag: self.current_tag,
                            group_count: group_count,
                            first_tag: repeating_group_builder.first_field(self.message_version),
                            groups: Vec::new(),
                            group_builder: repeating_group_builder,
                        }))));
                        self.tag_rule_mode_stack.push(Box::new(TagRuleMode::RepeatingGroupStart(first_field)));
                    },
                    Ok(_) => {}, //group_count == 0. Just ignore.
                    Err(_) => return Err(ParseError::WrongFormatTag(self.current_tag.clone())),
                }
                skip_set_value = true;
            },
            &Rule::PrepareForBytes{ ref bytes_tag } => {
                //Next tag should be 'bytes_tag' and its value is made up of
                //the number of bytes specified in this tag.
                match ascii_to_integer::<usize>(&self.current_bytes) {
                    Ok(byte_count) => self.tag_rule_mode_stack.push(Box::new(TagRuleMode::LengthThenValue(*bytes_tag,byte_count))),
                    Err(_) => return Err(ParseError::WrongFormatTag(self.current_tag)),
                }
                skip_set_value = true;
            },
            &Rule::ConfirmPreviousTag{ .. } => {}, //Must be checked after parsing tag and before parsing value.
            &Rule::RequiresFIXVersion{ .. } => {}, //Unused by parser.
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
                                prgs.number_of_tag,
                                Vec::from_iter(groups.drain(0..).map(|group| { group.message }))
                            );
                            folded_down = true;
                        }
                    }

                    if !folded_down {
                        let mut groups = mem::replace(&mut prgs.groups,Vec::new());
                        self.current_message.set_groups(
                            prgs.number_of_tag,
                            Vec::from_iter(groups.drain(0..).map(|group| { group.message }))
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

    fn match_tag_end(&mut self,index: &mut usize,message_bytes: &[u8]) -> Result<(),ParseError> {
        self.current_tag = FieldTag::from(&self.current_bytes[..]);
        self.current_bytes.clear();

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
                    if self.current_tag != first_repeating_group_tag {
                        return Err(ParseError::MissingFirstRepeatingGroupTagAfterNumberOfRepeatingGroupTag(self.previous_tag));
                    }
                },
                _ => self.tag_rule_mode_stack.push(tag_rule_mode),
            }
        }
        //Otherwise, if the current tag requires some preceding tag that wasn't found,
        //return an error. This is a sanity check.
        else if let Some(required_preceding_tag) = self.value_to_length_tags.get(&self.current_tag) {
            if *required_preceding_tag != self.previous_tag {
                return Err(ParseError::MissingPrecedingLengthTag(self.current_tag.clone()));
            }
        }

        Ok(())
    }

    fn match_value_end(&mut self,index: &mut usize,message_bytes: &[u8]) -> Result<MessageEnd,ParseError> {
        //Validate that the first three tags of a message are, in order: BeginStr,
        //BodyLength, and MsgType.
        if self.found_tag_count == 0 {
            if self.current_tag != BEGINSTR_TAG {
                return Err(ParseError::BeginStrNotFirstTag);
            }

            //Figure out what message version should be supported while parsing.
            let (fix_version,message_version) = match &self.current_bytes[..] {
                FIX_4_0_BEGIN_STRING => (FIXVersion::FIX_4_0,MessageVersion::FIX40),
                FIX_4_1_BEGIN_STRING => (FIXVersion::FIX_4_1,MessageVersion::FIX41),
                FIX_4_2_BEGIN_STRING => (FIXVersion::FIX_4_2,MessageVersion::FIX42),
                FIX_4_3_BEGIN_STRING => (FIXVersion::FIX_4_3,MessageVersion::FIX43),
                FIX_4_4_BEGIN_STRING => (FIXVersion::FIX_4_4,MessageVersion::FIX44),
                //If no per-message version is specified, FIXT.1.1 and higher should fall back to
                //some specified default. For connection initiators, this must be specified during
                //Logon. For connection acceptors, this should start with the highest supported
                //version and then be lowered to the initiator's version.
                FIXT_1_1_BEGIN_STRING => (FIXVersion::FIXT_1_1,self.default_message_version),
                _ => return Err(ParseError::WrongFormatTag(BEGINSTR_TAG)),
            };
            self.fix_version = fix_version;
            self.message_version = message_version;
            self.current_bytes.clear();
        }
        else if self.found_tag_count == 1 {
            if self.current_tag != BODYLENGTH_TAG {
                return Err(ParseError::BodyLengthNotSecondTag);
            }

            //Body length must be a valid positive number or else the rest of the message
            //is garbage.
            match ascii_to_integer::<u64>(&self.current_bytes) {
                Ok(length) => {
                    self.body_length = length;
                    self.body_remaining_length = length;
                },
                Err(_) => return Err(ParseError::BodyLengthNotNumber),
            }

            //Messages that are too long are outright rejected. The remaining bytes will be skipped
            //on the next parse because they are considered garbled. If the actual body length is
            //different from the presented number, there will be an appropriate error.
            let total_message_length = BEGINSTR_TAG_BYTES.len() as u64 + b"=\x01".len() as u64 + self.fix_version.begin_string().len() as u64 +
                BODYLENGTH_TAG_BYTES.len() as u64 + b"=\x01".len() as u64 + self.current_bytes.len() as u64 +
                self.body_length +
                CHECKSUM_TAG_BYTES.len() as u64 + b"=000\x01".len() as u64;
            if total_message_length > self.max_message_length {
                return Err(ParseError::MessageSizeTooBig);
            }
        }
        else if self.found_tag_count == 2 {
            if self.current_tag != MSGTYPE_TAG {
                return Err(ParseError::MsgTypeNotThirdTag);
            }

            //Record message type. For older FIX versions, prepare a collection of which fields are
            //supported and which are required. Newer FIX versions require more complicated
            //handling that must be put off until after receiving the sixth field.
            self.message_type = self.current_bytes.clone();
            if self.fix_version != FIXVersion::FIXT_1_1 {
                try!(self.prepare_for_message());
            }
        }
        else if self.found_tag_count == 3 && self.fix_version == FIXVersion::FIXT_1_1 {
            //FIXT.1.1 requires the fourth field to be SenderCompID. Older FIX versions use generic
            //field handling because the order doesn't matter but the field is stil required.
            if self.current_tag != SenderCompID::tag() {
                return Err(ParseError::SenderCompIDNotFourthTag);
            }

            self.sender_comp_id = self.current_bytes.clone();
        }
        else if self.found_tag_count == 4 && self.fix_version == FIXVersion::FIXT_1_1 {
            //FIXT.1.1 requires the fifth field to be TargetCompID. Older FIX versions use generic
            //field handling because the order doesn't matter but the field is stil required.
            if self.current_tag != TargetCompID::tag() {
                return Err(ParseError::TargetCompIDNotFifthTag);
            }

            self.target_comp_id = self.current_bytes.clone();
        }
        else if self.current_bytes.is_empty() {
            //Tag was provided without a value.
            return Err(ParseError::NoValueAfterTag(self.current_tag.clone()));
        }
        else {
            //FIXT.1.1 requires that if the ApplVerID tag is specified, it must be the sixth field.
            let mut skip_set_value = false;
            if self.found_tag_count == 5 && self.fix_version == FIXVersion::FIXT_1_1 {
                //Handle if this is the optional ApplVerID field. This can override all other
                //methods for determining what FIX version this message is expected to adhere to.
                if self.current_tag == ApplVerID::tag() {
                    if let Some(appl_ver_id) = MessageVersion::from_bytes(&self.current_bytes[..]) {
                        self.message_version = appl_ver_id;
                        skip_set_value = true;
                    }
                    else {
                        return Err(ParseError::OutOfRangeTag(self.current_tag.clone()));
                    }
                }
                //Fall back to the message specific default (if specified) or the session default
                //(in that order).
                else {
                    self.message_version = *self.default_message_type_version.get(&self.message_type[..]).unwrap_or(&self.default_message_version);
                }

                //Now that the message version has been determined, prepare a collection of which
                //fields are supported and which are required.
                try!(self.prepare_for_message());

                //Start the message by filling out the SenderCompID and TargetCompID portions of
                //message. These fields are always required for FIXT.1.1 messages.
                try!(set_message_value(&mut *self.current_message,SenderCompID::tag(),&self.sender_comp_id[..]));
                self.remaining_fields.remove(&SenderCompID::tag());
                self.remaining_required_fields.remove(&SenderCompID::tag());
                try!(set_message_value(&mut *self.current_message,TargetCompID::tag(),&self.target_comp_id[..]));
                self.remaining_fields.remove(&TargetCompID::tag());
                self.remaining_required_fields.remove(&TargetCompID::tag());

                //Mark ApplVerID as found so we produce an error if it's encountered anywhere else
                //in the message.
                if self.current_tag == ApplVerID::tag() {
                    try!(set_message_value(&mut *self.current_message,ApplVerID::tag(),&self.current_bytes[..]));
                }
                self.remaining_fields.remove(&ApplVerID::tag());
            }

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
                let mut some_rule = None;
                if let Some(ref mut tag_rule_mode) = self.tag_rule_mode_stack.last_mut() {
                    if let TagRuleMode::RepeatingGroups(ref mut prgs) = ***tag_rule_mode {
                        if self.current_tag == prgs.first_tag {
                            //Make sure previous group has all required tags specified
                            //before we start a new one.
                            prgs.check_last_group_complete(self.message_version,&mut self.missing_tag,&mut self.missing_conditional_tag);

                            //Begin a new group.
                            let group = prgs.group_builder.build();
                            let remaining_fields = prgs.group_builder.fields(self.message_version);
                            let remaining_required_fields = prgs.group_builder.required_fields(self.message_version);
                            prgs.groups.push(ParseGroupState {
                                message: group,
                                remaining_fields: remaining_fields,
                                remaining_required_fields: remaining_required_fields,
                            });

                            //Make sure we haven't exceeded the number of repeating
                            //groups originally stated.
                            if prgs.groups.len() > prgs.group_count {
                                return Err(ParseError::RepeatingGroupTagWithNoRepeatingGroup(self.current_tag));
                            }
                        }

                        if let Some(group) = prgs.groups.last_mut() {
                            if let Some(rule) = group.remaining_fields.remove(&self.current_tag) {
                                //Try to mark the field as found in case it's required.
                                group.remaining_required_fields.remove(&self.current_tag);

                                //Apply parsed value to group.
                                if let Rule::BeginGroup{ .. } = rule {} //Ignore begin group tags, they will be handled below.
                                else {
                                    try!(set_message_value(&mut *group.message,self.current_tag,&self.current_bytes[..]));
                                }

                                //Save rule to handle later.
                                some_rule = Some(rule);

                                tag_in_group = true;
                            }
                        }

                        if !tag_in_group {
                            //Figure out if this is an error or the end of the group.
                            if prgs.group_builder.fields(self.message_version).contains_key(&self.current_tag) {
                                return Err(ParseError::DuplicateTag(self.current_tag.clone()));
                            }
                            else if prgs.groups.len() < prgs.group_count {
                                return Err(ParseError::NonRepeatingGroupTagInRepeatingGroup(self.current_tag));
                            }

                            //Make sure all required tags have been specified.
                            prgs.check_last_group_complete(self.message_version,&mut self.missing_tag,&mut self.missing_conditional_tag);

                            //Tag does not belong in this group and all stated groups are
                            //accounted for.
                            group_end = true;
                        }
                    }
                }

                //Out of the way result handling to appease the borrow checker.
                if let Some(rule) = some_rule {
                    try!(self.handle_rule_after_value(&rule));
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

            if !skip_set_value && !is_message_end && !tag_in_group {
                //Mark field as found if required so we can quickly check if all required
                //fields were found once we are done parsing the message.
                self.remaining_required_fields.remove(&self.current_tag);

                //Mark field as found so we can quickly check if a duplicate tag was
                //encountered. As a side effect, we also handle any tag specific
                //rules in consequence of being encountered.
                if let Some(rule) = self.remaining_fields.remove(&self.current_tag) {
                    skip_set_value = try!(self.handle_rule_after_value(&rule));
                }
                else {
                    if self.is_current_tag_known() {
                        let current_message_builder = self.message_dictionary.get_mut(&self.message_type[..]).unwrap();
                        if current_message_builder.fields(self.message_version).contains_key(&self.current_tag) {
                            //Special case where if ApplVerID tag is encountered after the sixth
                            //tag. This needs its own error so the correct SessionRejectReason can
                            //be specified in a Reject message.
                            if self.current_tag == ApplVerID::tag() {
                                return Err(ParseError::ApplVerIDNotSixthTag);
                            }

                            return Err(ParseError::DuplicateTag(self.current_tag.clone()));
                        }
                        else {
                            return Err(ParseError::UnexpectedTag(self.current_tag.clone()));
                        }
                    }
                    else {
                        return Err(ParseError::UnknownTag(self.current_tag.clone()));
                    }
                }
            }

            if !is_message_end && !tag_in_group && !skip_set_value {
                try!(set_message_value(&mut *self.current_message,self.current_tag,&self.current_bytes[..]));
            }

            if is_message_end {
                //Make sure all required tags are specified.
                if !self.missing_tag.is_empty() {
                    return Err(
                        ParseError::MissingRequiredTag(
                            self.missing_tag,
                            mem::replace(&mut self.current_message,Box::new(NullMessage {}))));
                }
                else if !self.missing_conditional_tag.is_empty() {
                    return Err(
                        ParseError::MissingConditionallyRequiredTag(
                            self.missing_conditional_tag,
                            mem::replace(&mut self.current_message,Box::new(NullMessage {}))));
                }

                if let Some(tag) = self.remaining_required_fields.iter().next() {
                    return Err(
                        ParseError::MissingRequiredTag(
                            *tag,
                            mem::replace(&mut self.current_message,Box::new(NullMessage {}))));
                }

                for tag in self.current_message.conditional_required_fields(self.message_version) {
                    if self.remaining_fields.contains_key(&tag) {
                        return Err(
                            ParseError::MissingConditionallyRequiredTag(
                                tag,
                                mem::replace(&mut self.current_message,Box::new(NullMessage {}))));
                    }
                }

                //Store meta info about the message. Mainly for debugging.
                self.current_message.set_meta(Meta {
                    begin_string: self.fix_version,
                    body_length: self.body_length,
                    message_version: self.message_version,
                    checksum: self.checksum,
                });

                //Save message.
                let is_logon_message = self.current_message.msg_type() == Logon::msg_type();
                self.messages.push(mem::replace(&mut self.current_message,Box::new(NullMessage {})));

                //Prepare for the next message.
                self.reset_parser();
                *index += 1;

                //Stop processing after Logon message so owner of parser can use the message to
                //determine versioning defaults.
                if is_logon_message {
                    return Ok(MessageEnd::YesButStop);
                }

                //Scan to the next message in case there is garbage between the end of this
                //one and the beginning of the next.
                self.scan_for_message(index,message_bytes);

                return Ok(MessageEnd::Yes);
            }
        }

        //Prepare for next tag.
        self.previous_tag = self.current_tag;
        self.current_tag = FieldTag::empty();
        self.current_bytes.clear();
        self.found_tag_count += 1;

        Ok(MessageEnd::No)
    }

    fn is_current_tag_known(&mut self) -> bool {
        for message in self.message_dictionary.values_mut() {
            if message.fields(self.message_version).contains_key(&self.current_tag) {
                return true;
            }
        }

        false
    }

    pub fn parse(&mut self,message_bytes: &[u8]) -> (usize,Result<(),ParseError>) {
        //Parse and bytes as possible. Either all bytes will be consumed or all bytes up until a
        //parse error is triggered -- whatever happens first.
        let mut index = 0;
        match self.parse_private(&mut index,message_bytes) {
            Ok(_) => (index,Ok(())),
            Err(err) => {
                //Reset automatically so the next parse won't fail immediatelly.
                self.reset_parser();

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
                b'=' if self.current_tag.is_empty() => {
                    try!(self.match_tag_end(index,message_bytes));
                },
                //Byte indicates a vale has finished being read. Now both the tag and value are known.
                b'\x01' => { //SOH
                    match self.match_value_end(index,message_bytes) {
                        Ok(ref result) if *result == MessageEnd::Yes => {
                            //Message finished and index was already forwaded to the end of
                            //message_bytes or the beginning of the next message.
                            continue;
                        },
                        Ok(ref result) if *result == MessageEnd::YesButStop => {
                            //Message finished but parsing has been suspended to handle a special
                            //case.
                            return Ok(());
                        },
                        Err(e) => {
                            //An error occurred. Manually move index forward so this byte isn't
                            //reprocessed in the next call to parse().
                            *index += 1;
                            return Err(e);
                        },
                        _ => {}, //Still reading a message and it's going okay!
                    };
                },
                //Byte is part of a tag or value.
                _ => {
                    self.current_bytes.push(c);
                }
            }

            *index += 1;
        }

        Ok(())
    }
}

