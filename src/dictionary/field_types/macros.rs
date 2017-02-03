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

//Helper macros

macro_rules! define_enum_field_type_impl {
    ( DEFAULT_VALUE_FUNC $field_type_type:ident { $( $base_type_field:path ),* $(),* } ) => {
        $( return $base_type_field; )*
    };

    ( DEFAULT_VALUE_FUNC $field_type_type:ty { $( $base_type_field:path ),* $(),* } ) => {
        return None;
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
        fn read(field: &Self::Type,_fix_version: $crate::fix_version::FIXVersion,_message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
            let bytes = field.to_bytes();
            buf.write(bytes).unwrap()
        }
    };

    ( READ_FUNC_DEF $field_type_type:ty ) => {
        fn read(field: &Self::Type,_fix_version: $crate::fix_version::FIXVersion,_message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
            if let Some(ref field) = *field {
                let bytes = field.to_bytes();
                return buf.write(bytes).unwrap()
            }

            0
        }
    };

    ( 1=> $base_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } ) => {
       impl $base_type {
            fn new(bytes: &[u8]) -> Option<$base_type> {
                static MAPPING: ::phf::Map<&'static [u8],$base_type> = phf_map! {
                    $( $base_type_value => $base_type_field, )*
                };

                MAPPING.get(bytes).cloned()
            }

            fn to_bytes(&self) -> &'static [u8] {
                match *self {
                    $( $base_type_field => $base_type_value, )*
                }
            }
        }
    };

    ( 2=> $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path ),* $(),* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK $custom_set_value_error_check_func:expr ) => {
        pub struct $field_type;

        impl $crate::field_type::FieldType for $field_type {
            type Type = $( $field_type_type )*;

            #[allow(unreachable_code)]
            fn default_value() -> Self::Type {
                define_enum_field_type_impl!( DEFAULT_VALUE_FUNC $( $field_type_type )* { $( $base_type_field, )* } );
            }

            fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),$crate::message::SetValueError> {
                if let Some(value) = define_enum_field_type_impl!( NEW_VALUE_FUNC $base_type, $( $field_type_type )* )(bytes) {
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

            define_enum_field_type_impl!( READ_FUNC_DEF $( $field_type_type )* );
        }
    };

    ( 2=> $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path ),* $(),* } MUST_BE_STRING ) => {
        define_enum_field_type_impl!( 2=> $base_type, $field_type [ $( $field_type_type )* ] { $( $base_type_field, )* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK |_bytes: &[u8]| {
            //Not one of the supported bytes.
            Err($crate::message::SetValueError::OutOfRange)
        });
    };

    ( 2=> $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path ),* $(),* } MUST_BE_CHAR ) => {
        define_enum_field_type_impl!( 2=> $base_type, $field_type [ $( $field_type_type )* ] { $( $base_type_field, )* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK |bytes: &[u8]| {
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

    ( 2=> $base_type:ident, $field_type:ident [ $( $field_type_type:tt )* ] { $( $base_type_field:path ),* $(),* } MUST_BE_INT ) => {
        define_enum_field_type_impl!( 2=> $base_type, $field_type [ $( $field_type_type )* ] { $( $base_type_field, )* } WITH_CUSTOM_SET_VALUE_ERROR_CHECK |bytes: &[u8]| {
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
        define_enum_field_type_impl!( 1=> $base_type { $( $base_type_field => $base_type_value, )* } );
        define_enum_field_type_impl!( 2=> $base_type, $field_type [ $base_type ] { $( $base_type_field, )* } $must_be_sym);
    };

    ( NOT_REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $must_be_sym:tt ) => {
        define_enum_field_type_impl!( 1=> $base_type { $( $base_type_field => $base_type_value, )* } );
        define_enum_field_type_impl!( 2=> $base_type, $field_type [ Option<$base_type> ] { $( $base_type_field, )* } $must_be_sym);
    };

    ( REQUIRED_AND_NOT_REQUIRED, $base_type:ident, $required_field_type:ident, $not_required_field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $must_be_sym:tt ) => {
        define_enum_field_type_impl!( 1=> $base_type { $( $base_type_field => $base_type_value, )* } );
        define_enum_field_type_impl!( 2=> $base_type, $required_field_type [ $base_type ] { $( $base_type_field, )* } $must_be_sym);
        define_enum_field_type_impl!( 2=> $base_type, $not_required_field_type [ Option<$base_type> ] { $( $base_type_field, )* } $must_be_sym);
    };
}

#[macro_export]
macro_rules! define_enum_field_type {
    (FIELD $enum_name:ident {
        $( $variant:ident => $value:expr, )+
    } $other_variant:tt,
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,BYTES] $required_field_type:ident $not_required_field_type:ident ) => {
        #[derive(Clone,Debug,PartialEq)]
        pub enum $enum_name {
            $( $variant ),+,
            $other_variant(Vec<u8>),
        }

        define_enum_field_type_with_reserved!(BYTES, $enum_name, $required_field_type, $not_required_field_type { $( $enum_name::$variant => $value ),+ , } $enum_name::$other_variant);
    };

    (FIELD $enum_name:ident {
        $( $variant:ident => $value:expr, )+
    } $other_variant:tt => WITH_MINIMUM $minimum_value:expr,
    FIELD_TYPE [$required_sym:tt] $field_type:ident ) => {
        #[derive(Clone,Debug,PartialEq)]
        pub enum $enum_name {
            $( $variant ),+,
            $other_variant(i64),
        }

        define_enum_field_type_with_reserved!($required_sym, $enum_name, $field_type { $( $enum_name::$variant => $value ),+ , } $enum_name::$other_variant => WITH_MINIMUM $minimum_value);
    };

    (FIELD $enum_name:ident {
        $( $variant:ident => $value:expr, )+
    },
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,$must_be_sym:tt] $required_field_type:ident $not_required_field_type:ident ) => {
        #[derive(Clone,Debug,PartialEq)]
        pub enum $enum_name {
            $( $variant ),+
        }

        define_enum_field_type_impl!(REQUIRED_AND_NOT_REQUIRED, $enum_name, $required_field_type, $not_required_field_type { $( $enum_name::$variant => $value ),+ , } $must_be_sym);
    };

    (FIELD $enum_name:ident {
        $( $variant:ident => $value:expr, )+
    },
    FIELD_TYPE [$required_sym:tt,$must_be_sym:tt] $field_type:ident ) => {
        #[derive(Clone,Debug,PartialEq)]
        pub enum $enum_name {
            $( $variant ),+
        }

        define_enum_field_type_impl!($required_sym, $enum_name, $field_type { $( $enum_name::$variant => $value ),+ , } $must_be_sym);
    };
}

#[macro_export]
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
        fn read(field: &Self::Type,_fix_version: $crate::fix_version::FIXVersion,_message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
            let value = field.to_value();
            let value_string = value.to_string();
            let value_bytes = value_string.as_bytes();
            buf.write(value_bytes).unwrap()
        }
    };

    ( READ_FUNC_DEF $field_type_type:ty ) => {
        fn read(field: &Self::Type,_fix_version: $crate::fix_version::FIXVersion,_message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
            if let Some(ref field) = *field {
                let value = field.to_value();
                let value_string = value.to_string();
                let value_bytes = value_string.as_bytes();
                return buf.write(value_bytes).unwrap()
            }

            0
        }
    };

    ( AS_INT $base_type:ident, $field_type:ident, [ $( $field_type_type:tt )* ] { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path => WITH_MINIMUM $base_type_reserved_field_minimum:expr ) => {
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

    ( AS_BYTES_REQUIRED $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } ) => {
        pub struct $field_type;

        impl $crate::field_type::FieldType for $field_type {
            type Type = $base_type;

            #[allow(unreachable_code)]
            #[allow(needless_return)]
            fn default_value() -> Self::Type {
                define_enum_field_type_impl!( DEFAULT_VALUE_FUNC $base_type { $( $base_type_field, )* } );
            }

            fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),$crate::message::SetValueError> {
                if let Some(new_value) = $base_type::new(bytes) {
                    *field = new_value;
                    return Ok(());
                }

                Err($crate::message::SetValueError::OutOfRange)
            }

            fn is_empty(_field: &Self::Type) -> bool {
                false
            }

            fn len(_field: &Self::Type) -> usize {
                0 //Unused for this type
            }

            fn read(field: &Self::Type,_fix_version: $crate::fix_version::FIXVersion,_message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
                let value_bytes = field.as_bytes();
                return buf.write(value_bytes).unwrap()
            }
        }
    };

    ( AS_BYTES_NOT_REQUIRED $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } ) => {
        pub struct $field_type;

        impl $crate::field_type::FieldType for $field_type {
            type Type = Option<$base_type>;

            #[allow(unreachable_code)]
            #[allow(needless_return)]
            fn default_value() -> Self::Type {
                define_enum_field_type_impl!( DEFAULT_VALUE_FUNC Self::Type { $( $base_type_field, )* } );
            }

            fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),$crate::message::SetValueError> {
                *field = $base_type::new(bytes);
                Ok(())
            }

            fn is_empty(field: &Self::Type) -> bool {
                field.is_none()
            }

            fn len(_field: &Self::Type) -> usize {
                0 //Unused for this type.
            }

            fn read(field: &Self::Type,_fix_version: $crate::fix_version::FIXVersion,_message_version: $crate::message_version::MessageVersion,buf: &mut Vec<u8>) -> usize {
                if let Some(ref field) = *field {
                    let value_bytes = field.as_bytes();
                    return buf.write(value_bytes).unwrap()
                }

                0
            }
        }
    };

    ( REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path => WITH_MINIMUM $base_type_reserved_field_minimum:expr ) => {
        define_enum_field_type_with_reserved!( AS_INT $base_type, $field_type, [ $base_type ] { $( $base_type_field => $base_type_value,)* } $base_type_reserved_field => WITH_MINIMUM $base_type_reserved_field_minimum);
    };

    ( NOT_REQUIRED, $base_type:ident, $field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path => WITH_MINIMUM $base_type_reserved_field_minimum:expr ) => {
        define_enum_field_type_with_reserved!( AS_INT $base_type, $field_type, [ Option<$base_type> ] { $( $base_type_field => $base_type_value,)* } $base_type_reserved_field => WITH_MINIMUM $base_type_reserved_field_minimum);
    };

    ( BYTES, $base_type:ident, $required_field_type:ident, $not_required_field_type:ident { $( $base_type_field:path => $base_type_value:expr ),* $(),* } $base_type_reserved_field:path ) => {
        impl $base_type {
            fn new(value: &[u8]) -> Option<$base_type> {
                static MAPPING: ::phf::Map<&'static [u8],$base_type> = phf_map! {
                    $( $base_type_value => $base_type_field, )*
                };

                MAPPING.get(value).cloned()
            }

            fn as_bytes<'a>(&'a self) -> &'a [u8] {
                match *self {
                    $( $base_type_field => $base_type_value, )*
                    $base_type_reserved_field(ref value) => &value[..],
                }
            }
        }

        define_enum_field_type_with_reserved!( AS_BYTES_REQUIRED $base_type, $required_field_type { $( $base_type_field => $base_type_value,)* } );
        define_enum_field_type_with_reserved!( AS_BYTES_NOT_REQUIRED $base_type, $not_required_field_type { $( $base_type_field => $base_type_value,)* } );
    };
}

