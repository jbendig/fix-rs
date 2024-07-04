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

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum MessageVersion {
    //Based on the ApplVerId(1128) field.
    //Unsupported FIX27,
    //Unsupported FIX30,
    FIX40,
    FIX41,
    FIX42,
    FIX43,
    FIX44,
    FIX50,
    FIX50SP1,
    FIX50SP2,
}

impl MessageVersion {
    pub fn new(value: u32) -> Option<MessageVersion> {
        match value {
            //Unsupported 0 => MessageVersion::FIX27,
            //Unsupported 1 => MessageVersion::FIX30,
            2 => Some(MessageVersion::FIX40),
            3 => Some(MessageVersion::FIX41),
            4 => Some(MessageVersion::FIX42),
            5 => Some(MessageVersion::FIX43),
            6 => Some(MessageVersion::FIX44),
            7 => Some(MessageVersion::FIX50),
            8 => Some(MessageVersion::FIX50SP1),
            9 => Some(MessageVersion::FIX50SP2),
            _ => None,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<MessageVersion> {
        match bytes {
            //Unsupported b"0" => MessageVersion::FIX27,
            //Unsupported b"1" => MessageVersion::FIX30,
            b"2" => Some(MessageVersion::FIX40),
            b"3" => Some(MessageVersion::FIX41),
            b"4" => Some(MessageVersion::FIX42),
            b"5" => Some(MessageVersion::FIX43),
            b"6" => Some(MessageVersion::FIX44),
            b"7" => Some(MessageVersion::FIX50),
            b"8" => Some(MessageVersion::FIX50SP1),
            b"9" => Some(MessageVersion::FIX50SP2),
            _ => None,
        }
    }

    pub fn as_value(&self) -> u8 {
        match *self {
            //Unsupported MessageVersion::FIX27 => 0,
            //Unsupported MessageVersion::FIX30 => 1,
            MessageVersion::FIX40 => 2,
            MessageVersion::FIX41 => 3,
            MessageVersion::FIX42 => 4,
            MessageVersion::FIX43 => 5,
            MessageVersion::FIX44 => 6,
            MessageVersion::FIX50 => 7,
            MessageVersion::FIX50SP1 => 8,
            MessageVersion::FIX50SP2 => 9,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match *self {
            //Unsupported MessageVersion::FIX27 => b"0",
            //Unsupported MessageVersion::FIX30 => b"1",
            MessageVersion::FIX40 => b"2",
            MessageVersion::FIX41 => b"3",
            MessageVersion::FIX42 => b"4",
            MessageVersion::FIX43 => b"5",
            MessageVersion::FIX44 => b"6",
            MessageVersion::FIX50 => b"7",
            MessageVersion::FIX50SP1 => b"8",
            MessageVersion::FIX50SP2 => b"9",
        }
    }

    pub fn all() -> Vec<MessageVersion> {
        vec![
            MessageVersion::FIX40,
            MessageVersion::FIX41,
            MessageVersion::FIX42,
            MessageVersion::FIX43,
            MessageVersion::FIX44,
            MessageVersion::FIX50,
            MessageVersion::FIX50SP1,
            MessageVersion::FIX50SP2,
        ]
    }
}
