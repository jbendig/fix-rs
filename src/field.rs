use message::Message;
use std::any::Any;
use std::collections::HashSet;
use std::ops::{Deref,DerefMut};

pub enum Action {
    Nothing,
    AddRequiredTags(HashSet<&'static [u8]>),
    BeginGroup{message: Box<Message>},
    PrepareForBytes{bytes_tag: &'static [u8]},
    ConfirmPreviousTag{previous_tag: &'static [u8]}, //TODO: Probably redundant to the PrepareForBytes definition. Should be automatically inferred.
}

pub trait FieldType {
    fn new() -> Self;

    fn action() -> Option<Action> {
        None
    }

    fn set_value(&mut self,_bytes: &[u8]) -> bool {
        false
    }

    fn set_groups(&mut self,_groups: &[Box<Message>]) -> bool {
        false
    }
}

#[derive(Clone,Default)]
pub struct NoneFieldType {
}

impl FieldType for NoneFieldType {
    fn new() -> Self {
        NoneFieldType {}
    }
}

#[derive(Clone,Default)]
pub struct StringFieldType {
    value: String,
}

impl FieldType for StringFieldType {
    fn new() -> Self {
        StringFieldType {
            value: String::new(),
        }
    }

    fn set_value(&mut self,bytes: &[u8]) -> bool {
        self.value = String::from_utf8_lossy(bytes).into_owned();
        true
    }
}

impl Deref for StringFieldType {
    type Target = String;

    fn deref(&self) -> &String {
        &self.value
    }
}

impl DerefMut for StringFieldType {
    fn deref_mut(&mut self) -> &mut String {
        &mut self.value
    }
}

#[derive(Clone,Default)]
pub struct DataFieldType {
    value: Vec<u8>,
}

impl FieldType for DataFieldType {
    fn new() -> Self {
        DataFieldType {
            value: Vec::new(),
        }
    }

    fn set_value(&mut self,bytes: &[u8]) -> bool {
        self.value.resize(bytes.len(),0);
        self.value.copy_from_slice(bytes);
        true
    }
}

impl Deref for DataFieldType {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.value
    }
}

#[derive(Clone,Default)]
pub struct RepeatingGroupFieldType<T: Message> {
    groups: Vec<Box<T>>,
}

impl<T: Message + Any + Clone + Default> FieldType for RepeatingGroupFieldType<T> {
    fn new() -> Self {
        RepeatingGroupFieldType {
            groups: Vec::new(),
        }
    }

    fn action() -> Option<Action> {
        Some(Action::BeginGroup{ message: Box::new(<T as Default>::default()) })
    }

    fn set_groups(&mut self,groups: &[Box<Message>]) -> bool {
        self.groups.clear();

        for group in groups {
            match group.as_any().downcast_ref::<T>() {
                //TODO: Avoid the clone below.
                Some(casted_group) => self.groups.push(Box::new(casted_group.clone())),
                None => return false,
            }
        }

        true
    }
}

impl<T: Message> Deref for RepeatingGroupFieldType<T> {
    type Target = Vec<Box<T>>;

    fn deref(&self) -> &Vec<Box<T>> {
        &self.groups
    }
}

pub trait Field {
    type Type;
    fn action() -> Action;
    fn tag() -> &'static [u8];
}

#[macro_export]
macro_rules! define_field {
    ( $( $field_name:ident : $field_type:ty = $tag:expr $( => $action:expr )* ),* $(),* ) => { $(
        pub struct $field_name;
        impl Field for $field_name {
            type Type = $field_type;

            #[allow(unreachable_code)]
            fn action() -> Action {
                //If an action is provided, prefer it first.
                $(
                    return $action;
                )*

                //Next, check if the field type provides an action. This way the BeginGroup action
                //can be specified automatically instead of using a nasty boilerplate in each field
                //definition.
                if let Some(action) = <$field_type as FieldType>::action() {
                    action
                }
                //Otherwise, no action was specified.
                else {
                    Action::Nothing
                }
            }

            fn tag() -> &'static [u8] {
                $tag
            }
        }
    )*};
}

