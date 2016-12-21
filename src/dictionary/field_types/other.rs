use std::io::Write;
use std::str::FromStr;

//Helper macros

macro_rules! define_enum_field_type_impl {
    ( DEFAULT_VALUE_FUNC $field_type_type:ident { $( $base_type_field:path ),* $(),* } ) => {
        $( return $base_type_field; )*
    };

    ( DEFAULT_VALUE_FUNC $field_type_type:ty { $( $base_type_field:path ),* $(),* } ) => {
        $( return Some($base_type_field); )*
    };

    ( IS_EMPTY_FUNC $field_type_type:ident ) => {
        |_field: &$field_type_type| {
            false
        }
    };

    ( IS_EMPTY_FUNC $field_type_type:ty ) => {
        |field: &$field_type_type| {
            field.is_none()
        }
    };
}

macro_rules! define_enum_field_type {
    ( NEW_VALUE_FUNC $base_type:ident, $field_type_type:ident ) => {
        |bytes: &[u8]| {
            $base_type::new(bytes)
        }
    };

    ( NEW_VALUE_FUNC $base_type:ident, $field_type_type:ty ) => {
        |bytes: &[u8]| {
            let new_value = $base_type::new(bytes);
            if new_value.is_some() {
                Some(new_value)
            }
            else {
                None
            }
        }
    };

    ( READ_FUNC_DEF $field_type_type:ident ) => {
        fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
            let bytes = field.to_bytes();
            buf.write(bytes).unwrap()
        }
    };

    ( READ_FUNC_DEF $field_type_type:ty ) => {
        fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
            if let Some(ref field) = *field {
                let bytes = field.to_bytes();
                return buf.write(bytes).unwrap()
            }

            0
        }
    };

    ( => $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path => $base_type_value:expr ),* $(),* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK $custom_set_value_error_check_func:expr ) => {
        impl $base_type {
            fn new(bytes: &[u8]) -> Option<$base_type> {
                match bytes {
                    $( $base_type_value => Some($base_type_field), )*
                    _ => None,
                }
            }

            fn to_bytes(&self) -> &'static [u8] {
                match *self {
                    $( $base_type_field => $base_type_value, )*
                }
            }
        }

        pub struct $field_type;

        impl $crate::field_type::FieldType for $field_type {
            type Type = $( $field_type_type )*;

            #[allow(unreachable_code)]
            fn default_value() -> Self::Type {
                define_enum_field_type_impl!( DEFAULT_VALUE_FUNC $( $field_type_type )* { $( $base_type_field, )* } );
            }

            fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),$crate::message::SetValueError> {
                if let Some(value) = define_enum_field_type!( NEW_VALUE_FUNC $base_type, $( $field_type_type )* )(bytes) {
                    *field = value;
                    return Ok(());
                }

                $custom_set_value_error_check_func(bytes)
            }

            fn is_empty(field: &Self::Type) -> bool {
                define_enum_field_type_impl!( IS_EMPTY_FUNC $( $field_type_type )* )(field)
            }

            fn len(_field: &Self::Type) -> usize {
                0 //Unused for this type.
            }

            define_enum_field_type!( READ_FUNC_DEF $( $field_type_type )* );
        }
    };

    ( => $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path => $base_type_value:expr ),* $(),* } MUST_BE_CHAR ) => {
        define_enum_field_type!( => $base_type, $field_type [ $( $field_type_type )* ] { $( $base_type_field => $base_type_value, )* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK |bytes: &[u8]| {
            //Figure out what went wrong.
            if bytes.len() == 1 {
                //Not one of the supported bytes.
                Err($crate::message::SetValueError::OutOfRange)
            }
            else {
                //Too many bytes provided.
                Err($crate::message::SetValueError::WrongFormat)
            }
        });
    };

    ( => $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path => $base_type_value:expr ),* $(),* } MUST_BE_INT ) => {
        define_enum_field_type!( => $base_type, $field_type [ $( $field_type_type )* ] { $( $base_type_field => $base_type_value, )* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK |bytes: &[u8]| {
            //Figure out what went wrong.
            let value_string = String::from_utf8_lossy(bytes).into_owned();
            if <$crate::dictionary::field_types::generic::IntFieldType as $crate::field_type::FieldType>::Type::from_str(&value_string).is_ok() {
                //Not one of the supported integers.
                Err($crate::message::SetValueError::OutOfRange)
            }
            else {
                //Not an integer.
                Err($crate::message::SetValueError::WrongFormat)
            }
        });
    };

    ( REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $must_be_sym:tt ) => {
        define_enum_field_type!( => $base_type, $field_type [ $base_type ] { $( $base_type_field => $base_type_value, )* } $must_be_sym);
    };

    ( NOT_REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $must_be_sym:tt ) => {
        define_enum_field_type!( => $base_type, $field_type [ Option<$base_type> ] { $( $base_type_field => $base_type_value, )* } $must_be_sym);
    };
}

macro_rules! define_enum_field_type_with_reserved {
    ( NEW_VALUE_FUNC $base_type:ident, $field_type_type:ident ) => {
        |new_int_value: i64| {
            $base_type::new(new_int_value)
        }
    };

    ( NEW_VALUE_FUNC $base_type:ident, $field_type_type:ty ) => {
        |new_int_value: i64| {
            let new_value = $base_type::new(new_int_value);
            if new_value.is_some() {
                Some(new_value)
            }
            else {
                None
            }
        }
    };

    ( READ_FUNC_DEF $field_type_type:ident ) => {
        fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
            let value = field.to_value();
            let value_string = value.to_string();
            let value_bytes = value_string.as_bytes();
            buf.write(value_bytes).unwrap()
        }
    };

    ( READ_FUNC_DEF $field_type_type:ty ) => {
        fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
            if let Some(ref field) = *field {
                let value = field.to_value();
                let value_string = value.to_string();
                let value_bytes = value_string.as_bytes();
                return buf.write(value_bytes).unwrap()
            }

            0
        }
    };

    ( => $base_type:ident, $field_type:ident, [ $( $field_type_type:tt )* ] { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path => WITH_MINIMUM $base_type_reserved_field_minimum:expr ) => {
        impl $base_type {
            fn new(value: i64) -> Option<$base_type> {
                match value {
                    $( $base_type_value => Some($base_type_field), )*
                    _ if value >= $base_type_reserved_field_minimum => Some($base_type_reserved_field(value)),
                    _ => None,
                }
            }

            fn to_value(&self) -> i64 {
                match *self {
                    $( $base_type_field => $base_type_value, )*
                    $base_type_reserved_field(value) => value,
                }
            }
        }

        pub struct $field_type;

        impl $crate::field_type::FieldType for $field_type {
            type Type = $( $field_type_type )*;

            #[allow(unreachable_code)]
            #[allow(needless_return)]
            fn default_value() -> Self::Type {
                define_enum_field_type_impl!( DEFAULT_VALUE_FUNC $( $field_type_type )* { $( $base_type_field, )* } );
            }

            fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),$crate::message::SetValueError> {
                let value_string = String::from_utf8_lossy(bytes).into_owned();
                if let Ok(new_int_value) = i64::from_str(&value_string) {
                    if let Some(new_value) = define_enum_field_type_with_reserved!( NEW_VALUE_FUNC $base_type, $( $field_type_type )* )(new_int_value) {
                        *field = new_value;
                        Ok(())
                    }
                    else {
                        Err($crate::message::SetValueError::OutOfRange)
                    }
                }
                else {
                    Err($crate::message::SetValueError::WrongFormat)
                }
            }

            fn is_empty(field: &Self::Type) -> bool {
                define_enum_field_type_impl!( IS_EMPTY_FUNC $( $field_type_type )* )(field)
            }

            fn len(_field: &Self::Type) -> usize {
                0 //Unused for this type.
            }

            define_enum_field_type_with_reserved!( READ_FUNC_DEF $( $field_type_type )* );
        }
    };

    ( REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path => WITH_MINIMUM $base_type_reserved_field_minimum:expr ) => {
        define_enum_field_type_with_reserved!( => $base_type, $field_type, [ $base_type ] { $( $base_type_field => $base_type_value,)* } $base_type_reserved_field => WITH_MINIMUM $base_type_reserved_field_minimum);
    };

    ( NOT_REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path => WITH_MINIMUM $base_type_reserved_field_minimum:expr ) => {
        define_enum_field_type_with_reserved!( => $base_type, $field_type, [ Option<$base_type> ] { $( $base_type_field => $base_type_value,)* } $base_type_reserved_field => WITH_MINIMUM $base_type_reserved_field_minimum);
    };
}

//Enumerated Fields (Sorted Alphabetically)

#[derive(Clone,PartialEq)]
pub enum CPProgram {
    _3A3,
    _42,
    Other,
    Reserved100Plus(i64)
}

define_enum_field_type_with_reserved!(NOT_REQUIRED, CPProgram, CPProgramFieldType {
    CPProgram::_3A3 => 1,
    CPProgram::_42 => 2,
    CPProgram::Other => 99,
} CPProgram::Reserved100Plus => WITH_MINIMUM 100);

#[derive(Clone,PartialEq)]
pub enum HandlInst {
    AutomatedExecutionOrderPrivateNoBrokerIntervention,
    AutomatedExecutionOrderPublicBrokerInterventionOK,
    ManualOrderBestExecution,
}

define_enum_field_type!(NOT_REQUIRED, HandlInst, HandlInstFieldType {
    HandlInst::AutomatedExecutionOrderPrivateNoBrokerIntervention => b"1",
    HandlInst::AutomatedExecutionOrderPublicBrokerInterventionOK => b"2",
    HandlInst::ManualOrderBestExecution => b"3",
} MUST_BE_CHAR);

#[derive(Clone,Debug,PartialEq)]
pub enum RateSource {
    Bloomberg,
    Reuters,
    Telerate,
    Other
}

define_enum_field_type!(REQUIRED, RateSource, RateSourceFieldType {
    RateSource::Bloomberg => b"0",
    RateSource::Reuters => b"1",
    RateSource::Telerate => b"2",
    RateSource::Other => b"99",
} MUST_BE_INT);

#[derive(Clone,Debug,PartialEq)]
pub enum RateSourceType {
    Primary,
    Secondary
}

define_enum_field_type!(REQUIRED, RateSourceType, RateSourceTypeFieldType {
    RateSourceType::Primary => b"0",
    RateSourceType::Secondary => b"1",
} MUST_BE_INT);

pub enum SessionRejectReason {
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
    Reserved100Plus(i64),
}

define_enum_field_type_with_reserved!(NOT_REQUIRED, SessionRejectReason, SessionRejectReasonFieldType {
    SessionRejectReason::InvalidTagNumber => 0,
    SessionRejectReason::RequiredTagMissing => 1,
    SessionRejectReason::TagNotDefinedForThisMessageType => 2,
    SessionRejectReason::UndefinedTag => 3,
    SessionRejectReason::TagSpecifiedWithoutAValue => 4,
    SessionRejectReason::ValueIsIncorrectForThisTag => 5,
    SessionRejectReason::IncorrectDataFormatForValue => 6,
    SessionRejectReason::DecryptionProblem => 7,
    SessionRejectReason::SignatureProblem => 8,
    SessionRejectReason::CompIDProblem => 9,
    SessionRejectReason::SendingTimeAccuracyProblem => 10,
    SessionRejectReason::InvalidMsgType => 11,
    SessionRejectReason::XMLValidationError => 12,
    SessionRejectReason::TagAppearsMoreThanOnce => 13,
    SessionRejectReason::TagSpecifiedOutOfRequiredOrder => 14,
    SessionRejectReason::RepeatingGroupFieldsOutOfOrder => 15,
    SessionRejectReason::IncorrectNumInGroupCountForRepeatingGroup => 16,
    SessionRejectReason::NonDataValueIncludesFieldDelimiter => 17,
    SessionRejectReason::InvalidOrUnsupportedApplicationVersion => 18,
    SessionRejectReason::Other => 99,
} SessionRejectReason::Reserved100Plus => WITH_MINIMUM 100);

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

define_enum_field_type!(REQUIRED, Side, SideFieldType {
    Side::Buy => b"1",
    Side::Sell => b"2",
    Side::BuyMinus => b"3",
    Side::SellPlus => b"4",
    Side::SellShort => b"5",
    Side::SellShortExempt => b"6",
    Side::Undisclosed => b"7",
    Side::Cross => b"8",
    Side::CrossShort => b"9",
    Side::CrossShortExempt => b"A",
    Side::AsDefined => b"B",
    Side::Opposite => b"C",
    Side::Subscribe => b"D",
    Side::Redeem => b"E",
    Side::Lend => b"F",
    Side::Borrow => b"G",
} MUST_BE_CHAR);

