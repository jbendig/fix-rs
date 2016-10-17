use std::any::Any;
use std::collections::{HashMap,HashSet};
use field::Action;

#[derive(Clone,Default)]
pub struct Meta {
    pub protocol: Vec<u8>,
    pub body_length: u64,
    pub checksum: u8,
}

pub trait Message {
    fn first_field(&self) -> &'static [u8];
    fn fields(&self) -> HashMap<&'static [u8],Action>;
    fn required_fields(&self) -> HashSet<&'static [u8]>;
    fn set_meta(&mut self,meta: Meta);
    fn set_value(&mut self,key: &[u8],value: &[u8]) -> bool;
    fn set_groups(&mut self,key: &[u8],groups: &[Box<Message>]) -> bool;
    fn as_any(&self) -> &Any;
    fn clone_into_box(&self) -> Box<Message>;
}

pub struct NullMessage {
}

impl Message for NullMessage {
    fn first_field(&self) -> &'static [u8] {
        unimplemented!();
    }

    fn fields(&self) -> HashMap<&'static [u8],Action> {
        unimplemented!();
    }

    fn required_fields(&self) -> HashSet<&'static [u8]> {
        unimplemented!();
    }

    fn set_meta(&mut self,_meta: Meta) {
        unimplemented!();
    }

    fn set_value(&mut self,_key: &[u8],_value: &[u8]) -> bool {
        unimplemented!();
    }

    fn set_groups(&mut self,_key: &[u8],_group: &[Box<Message>]) -> bool {
        unimplemented!();
    }

    fn as_any(&self) -> &Any {
        unimplemented!();
    }

    fn clone_into_box(&self) -> Box<Message> {
        unimplemented!();
    }
}

pub const REQUIRED: bool = true;
pub const NOT_REQUIRED: bool = false;

#[macro_export]
macro_rules! define_message {
    ( $message_name:ident { $( $field_required:expr, $field_name:ident : $field_type:ty),* $(),* } ) => {
        #[derive(Clone,Default)]
        pub struct $message_name {
            pub meta: Option<Meta>,
            $( pub $field_name: <$field_type as Field>::Type, )*
        }

        impl $message_name {
            pub fn new() -> Self {
                $message_name {
                    meta: None,
                    $( $field_name: Default::default(), )*
                }
            }
        }

        impl Message for $message_name {
            #[allow(needless_return)]
            fn first_field(&self) -> &'static [u8] {
                //TODO: Make sure this reduces to a single statement when compiled for release.
                return vec![$( <$field_type as Field>::tag(), )*][0];
            }

            fn fields(&self) -> HashMap<&'static [u8],Action> {
                let mut result = HashMap::new();
                $( result.insert(<$field_type as Field>::tag(),<$field_type as Field>::action()); )*

                result
            }

            fn required_fields(&self) -> HashSet<&'static [u8]> {
                let mut result = HashSet::new();
                $( if $field_required { result.insert(<$field_type as Field>::tag()); } )*

                result
            }

            fn set_meta(&mut self,meta: Meta) {
                self.meta = Some(meta);
            }

            fn set_value(&mut self,key: &[u8],value: &[u8]) -> bool {
                if false {
                    false
                }
                $( else if key == <$field_type as Field>::tag() { self.$field_name.set_value(value) } )*
                else {
                    false
                }
            }

            fn set_groups(&mut self,key: &[u8],groups: &[Box<Message>]) -> bool {
                if false {
                    false
                }
                $( else if key == <$field_type as Field>::tag() { self.$field_name.set_groups(groups) } )*
                else {
                    false
                }
            }

            fn as_any(&self) -> &Any {
                self
            }

            fn clone_into_box(&self) -> Box<Message> {
                Box::new($message_name::new())
            }
        }
    };
}

