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

use chrono::Datelike;
use chrono::TimeZone;
use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;
use chrono::naive::datetime::NaiveDateTime;
use std::any::Any;
use std::marker::PhantomData;
use std::io::Write;
use std::str::FromStr;

use constant::VALUE_END;
use message::{Message,SetValueError};
use rule::Rule;

pub trait FieldType {
    type Type;

    fn rule() -> Option<Rule> {
        None
    }

    fn default_value() -> Self::Type;

    fn set_value(_field: &mut Self::Type,_bytes: &[u8]) -> Result<(),SetValueError> {
        Err(SetValueError::WrongFormat)
    }

    fn set_groups(_field: &mut Self::Type,_groups: &[Box<Message>]) -> bool {
        false
    }

    fn is_empty(field: &Self::Type) -> bool;
    fn len(field: &Self::Type) -> usize;
    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize;
}

pub struct NoneFieldType {
}

impl FieldType for NoneFieldType {
    type Type = PhantomData<()>;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn is_empty(_field: &Self::Type) -> bool {
        true
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(_field: &Self::Type,_buf: &mut Vec<u8>) -> usize {
        0
    }
}

pub struct CharFieldType {
}

impl FieldType for CharFieldType {
    type Type = u8;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if bytes.len() == 1 {
            *field = bytes[0];
            Ok(())
        }
        else {
            Err(SetValueError::WrongFormat)
        }
    }

    fn is_empty(field: &Self::Type) -> bool {
        *field == 0
    }

    fn len(_field: &Self::Type) -> usize {
        1
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(&[*field]).unwrap()
    }
}

pub struct StringFieldType {
}

impl FieldType for StringFieldType {
    type Type = String;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        *field = String::from_utf8_lossy(bytes).into_owned();
        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(field.as_bytes()).unwrap()
    }
}

pub struct DataFieldType {
}

impl FieldType for DataFieldType {
    type Type = Vec<u8>;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        field.resize(bytes.len(),0);
        field.copy_from_slice(bytes);
        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(field).unwrap()
    }
}

pub struct UTCTimestampFieldType {
}

impl UTCTimestampFieldType {
    pub fn new_now() -> <UTCTimestampFieldType as FieldType>::Type {
        let spec = ::time::get_time();

        //Strip nanoseconds so only whole milliseconds remain (with truncation based rounding).
        //This is because UTCTimestamp does not support sub-millisecond precision.
        let mut nsec = spec.nsec as u32;
        nsec -= nsec % 1_000_000;

        let naive = NaiveDateTime::from_timestamp(spec.sec,nsec);
        DateTime::from_utc(naive,UTC)
    }

    pub fn new_empty() -> <UTCTimestampFieldType as FieldType>::Type {
        //Create a new time stamp that can be considered empty. An Option<_> might be preferred
        //but that would make using the timestamp needlessly complicated.
        UTC.ymd(-1,1,1).and_hms(0,0,0)
    }
}

impl FieldType for UTCTimestampFieldType {
    type Type = DateTime<UTC>;

    fn default_value() -> Self::Type {
        UTCTimestampFieldType::new_empty()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        use chrono::TimeZone;

        //TODO: Support making the .sss, indicating milliseconds, optional.
        //TODO: Share the format string in a constant.
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_timestamp) = field.offset().datetime_from_str(&value_string,"%Y%m%d-%T%.3f") {
            *field = new_timestamp;

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.year() < 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        assert!(!Self::is_empty(&field)); //Was required field not set?

        let value_string = field.format("%Y%m%d-%T%.3f").to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct IntFieldType {
}

impl FieldType for IntFieldType {
    //The spec just says an integer but does not specify a minimum or maximum value.
    //TODO: Investigate if any field will ever need BigInt-style support instead.
    type Type = i64;

    fn default_value() -> Self::Type {
        0
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_value) = Self::Type::from_str(&value_string) {
            *field = new_value;
            
            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(_field: &Self::Type) -> bool {
        //Always required. Use OptionIntFieldType instead if field is optional.
        false
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let value_string = field.to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct SeqNumFieldType {
}

impl FieldType for SeqNumFieldType {
    //The spec just says a positive integer but does not specify a maximum value. This should allow
    //one number per millisecond for 5.85 * 10^8 years.
    type Type = u64;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_value) = Self::Type::from_str(&value_string) {
            *field = new_value;
            
            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        //First sequence number is 1. Fields where SeqNum can be 0 (ie. ResetRequest::EndSeqNo) are
        //marked as required so they will still be included.
        *field == 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let value_string = field.to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct BoolTrueOrBlankFieldType {
}

impl FieldType for BoolTrueOrBlankFieldType {
    type Type = bool;

    fn default_value() -> Self::Type {
        false
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if bytes.len() == 1 {
            *field = match bytes[0] {
                b'Y' => true,
                b'N' => false,
                _ => return Err(SetValueError::WrongFormat),
            };

            return Ok(())
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        !field
    }

    fn len(_field: &Self::Type) -> usize {
        1
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(if *field { b"Y" } else { b"N" }).unwrap()
    }
}

pub struct RepeatingGroupFieldType<T: Message + PartialEq> {
    message_type: PhantomData<T>,
}

impl<T: Message + Any + Clone + Default + PartialEq> FieldType for RepeatingGroupFieldType<T> {
    type Type = Vec<Box<T>>;

    fn rule() -> Option<Rule> {
        Some(Rule::BeginGroup{ message: Box::new(<T as Default>::default()) })
    }

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_groups(field: &mut Self::Type,groups: &[Box<Message>]) -> bool {
        field.clear();

        for group in groups {
            match group.as_any().downcast_ref::<T>() {
                //TODO: Avoid the clone below.
                Some(casted_group) => field.push(Box::new(casted_group.clone())),
                None => return false,
            }
        }

        true
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let group_count_str = field.len().to_string();
        let mut result = 1;

        result += buf.write(group_count_str.as_bytes()).unwrap();
        buf.push(VALUE_END);

        for group in field {
            result += group.read_body(buf);
        }

        result
    }
}

enum SessionRejectReason {
    InvalidTagNumber,
    RequiredTagMissing,
    TagNotDefinedForThisMessageType,
    UndefinedTag,
    TagSpecifiedWithoutAValue,
    ValueIsIncorrectForThisTag,
    IncorrectDataFormatForValue,
    DecryptionProblem,
    SignatureProblem,
    CompIDProblem,
    SendingTimeAccuracyProblem,
    InvalidMsgType,
    XMLValidationError,
    TagAppearsMoreThanOnce,
    TagSpecifiedOutOfRequiredOrder,
    RepeatingGroupFieldsOutOfOrder,
    IncorrectNumInGroupCountForRepeatingGroup,
    NonDataValueIncludesFieldDelimiter,
    InvalidOrUnsupportedApplicationVersion,
    Other,
    Reserved100Plus,
}

#[derive(Clone,PartialEq)]
pub enum Side {
    Buy,
    Sell,
    BuyMinus,
    SellPlus,
    SellShort,
    SellShortExempt,
    Undisclosed,
    Cross,
    CrossShort,
    CrossShortExempt,
    AsDefined,
    Opposite,
    Subscribe,
    Redeem,
    Lend,
    Borrow
}

impl Side {
    fn new(byte: u8) -> Option<Side> {
        match byte {
            b'1' => Some(Side::Buy),
            b'2' => Some(Side::Sell),
            b'3' => Some(Side::BuyMinus),
            b'4' => Some(Side::SellPlus),
            b'5' => Some(Side::SellShort),
            b'6' => Some(Side::SellShortExempt),
            b'7' => Some(Side::Undisclosed),
            b'8' => Some(Side::Cross),
            b'9' => Some(Side::CrossShort),
            b'A' => Some(Side::CrossShortExempt),
            b'B' => Some(Side::AsDefined),
            b'C' => Some(Side::Opposite),
            b'D' => Some(Side::Subscribe),
            b'E' => Some(Side::Redeem),
            b'F' => Some(Side::Lend),
            b'G' => Some(Side::Borrow),
            _ => None,
        }
    }

    fn to_byte(&self) -> u8 {
        match *self {
            Side::Buy => b'1',
            Side::Sell => b'2',
            Side::BuyMinus => b'3',
            Side::SellPlus => b'4',
            Side::SellShort => b'5',
            Side::SellShortExempt => b'6',
            Side::Undisclosed => b'7',
            Side::Cross => b'8',
            Side::CrossShort => b'9',
            Side::CrossShortExempt => b'A',
            Side::AsDefined => b'B',
            Side::Opposite => b'C',
            Side::Subscribe => b'D',
            Side::Redeem => b'E',
            Side::Lend => b'F',
            Side::Borrow => b'G',
        }
    }
}

pub struct SideFieldType {
}

impl FieldType for SideFieldType {
    type Type = Side;

    fn default_value() -> Self::Type {
        Side::Buy
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if bytes.len() == 1 {
            if let Some(side) = Side::new(bytes[0]) {
                *field = side;
                return Ok(());
            }
        }

        Err(SetValueError::OutOfRange)
    }

    fn is_empty(_field: &Self::Type) -> bool {
        false
    }

    fn len(_field: &Self::Type) -> usize {
        1
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let byte = field.to_byte();
        buf.write(&[byte]).unwrap()
    }
}

