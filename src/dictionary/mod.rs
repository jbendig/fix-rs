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

pub mod fields;
pub mod messages;

use std::collections::HashMap;

use fixt::message::FIXTMessage;

#[macro_export]
macro_rules! define_dictionary {
    ( $( $msg:ty : $msg_enum:ident ),* $(),* ) => {
        fn build_dictionary() -> std::collections::HashMap<&'static [u8],Box<$crate::fixt::message::FIXTMessage + Send>> {
            let mut message_dictionary: std::collections::HashMap<&'static [u8],Box<$crate::fixt::message::FIXTMessage + Send>> = std::collections::HashMap::new();
            $( message_dictionary.insert(<$msg as $crate::message::MessageDetails>::msg_type(),Box::new(<$msg as Default>::default())); )*

            message_dictionary
        }

        #[allow(dead_code)]
        enum MessageEnum
        {
            $( $msg_enum($msg), )*
        };

        #[allow(dead_code)]
        fn message_to_enum(message: &$crate::fixt::message::FIXTMessage) -> MessageEnum {
            if false {
            }
            $( else if message.as_any().is::<$msg>() {
                //TODO: Avoid the clone.
                return MessageEnum::$msg_enum(message.as_any().downcast_ref::<$msg>().unwrap().clone());
            } )*

            panic!("Unsupported message");
        }
    };
}

pub trait CloneDictionary {
    fn clone(&self) -> HashMap<&'static [u8],Box<FIXTMessage + Send>>;
}

impl CloneDictionary for HashMap<&'static [u8],Box<FIXTMessage + Send>> {
    fn clone(&self) -> HashMap<&'static [u8],Box<FIXTMessage + Send>> {
        //TODO: This function wastes a lot of time and memory. Probably better to change Parser
        //so it isn't needed.

        let mut result = HashMap::<&'static [u8],Box<FIXTMessage + Send>>::new();
        for (key,value) in self {
            result.insert(key,FIXTMessage::new_into_box(&**value));
        }

        result
    }
}

