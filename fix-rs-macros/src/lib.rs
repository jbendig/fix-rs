// Copyright 2017 James Bendig. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under:
//   the MIT license
//     <LICENSE-MIT or https://opensource.org/licenses/MIT>
//   or the Apache License, Version 2.0
//     <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0>,
// at your option. This file may not be copied, modified, or distributed
// except according to those terms.

#![crate_type = "proc-macro"]
#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use std::str::FromStr;

fn str_to_tokens(input: &str) -> TokenStream {
    TokenStream::from_str(input).expect("failed to convert str to tokens")
}

enum ExtractAttributeError {
    BodyNotStruct,
    FieldNotFound,
    AttributeNotFound,
    AttributeNotNameValue,
    AttributeValueWrongType,
}

fn extract_attribute_value(
    ast: &syn::DeriveInput,
    field_ident: &'static str,
    _attr_ident: &'static str,
) -> Result<syn::Lit, ExtractAttributeError> {
    if let syn::Data::Struct(ref data) = ast.data {
        for field in data.fields.iter() {
            if field.ident.as_ref().expect("Field must have an identifier") != field_ident {
                continue;
            }

            for attr in &field.attrs {
                if let syn::Meta::NameValue(name_value) = attr.meta.clone() {
                    if let syn::Expr::Lit(lit) = name_value.value {
                        return Ok(lit.lit.clone());
                    }
                } else {
                    return Err(ExtractAttributeError::AttributeNotNameValue);
                }
            }

            return Err(ExtractAttributeError::AttributeNotFound);
        }

        return Err(ExtractAttributeError::FieldNotFound);
    }

    Err(ExtractAttributeError::BodyNotStruct)
}

fn extract_attribute_byte_str(
    ast: &syn::DeriveInput,
    field_ident: &'static str,
    attr_ident: &'static str,
) -> Result<Vec<u8>, ExtractAttributeError> {
    let lit = extract_attribute_value(ast, field_ident, attr_ident)?;

    if let syn::Lit::ByteStr(ref bytes) = lit {
        return Ok(bytes.value());
    }

    Err(ExtractAttributeError::AttributeValueWrongType)
}

fn extract_attribute_int(
    ast: &syn::DeriveInput,
    field_ident: &'static str,
    attr_ident: &'static str,
) -> Result<u64, ExtractAttributeError> {
    let lit = extract_attribute_value(ast, field_ident, attr_ident)?;

    if let syn::Lit::Int(value) = lit {
        if let Ok(value) = value.base10_parse() {
            return Ok(value);
        }
    }

    Err(ExtractAttributeError::AttributeValueWrongType)
}

#[proc_macro_derive(BuildMessage, attributes(message_type))]
pub fn build_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let source = input.to_string();
    let ast = syn::parse(input).unwrap();

    let message_type = match extract_attribute_byte_str(&ast,"_message_type_gen","message_type") {
        Ok(bytes) => bytes,
        Err(ExtractAttributeError::BodyNotStruct) => panic!("#[derive(BuildMessage)] can only be used with structs"),
        Err(ExtractAttributeError::FieldNotFound) => panic!("#[derive(BuildMessage)] requires a _message_type_gen field to be specified"),
        Err(ExtractAttributeError::AttributeNotFound) => Vec::new(),
        Err(ExtractAttributeError::AttributeNotNameValue) |
        Err(ExtractAttributeError::AttributeValueWrongType) => panic!("#[derive(BuildMessage)] message_type attribute must be a byte string value like #[message_type=b\"1234\"]"),
    };
    let is_fixt_message = source.contains("sender_comp_id") && source.contains("target_comp_id");

    //Setup symbols.
    let message_name = ast.ident;
    let build_message_name = String::from("Build") + &message_name.to_string()[..];
    let mut message_type_header = "b\"35=".to_string();
    message_type_header += &String::from_utf8_lossy(&message_type[..]).into_owned()[..];
    message_type_header += "\\x01\"";

    //Convert symbols into tokens so quote's ToTokens trait doesn't quote them.
    let build_message_name = str_to_tokens(&build_message_name[..]);
    let message_type_header = str_to_tokens(&message_type_header[..]);

    let tokens = quote! {
        impl #message_name {
            fn msg_type_header() -> &'static [u8] {
                #message_type_header
            }
        }

        pub struct #build_message_name {
            cache: message::BuildMessageInternalCache,
        }

        impl #build_message_name {
            fn new() -> #build_message_name {
                #build_message_name {
                    cache: message::BuildMessageInternalCache {
                        fields_fix40: None,
                        fields_fix41: None,
                        fields_fix42: None,
                        fields_fix43: None,
                        fields_fix44: None,
                        fields_fix50: None,
                        fields_fix50sp1: None,
                        fields_fix50sp2: None,
                    },
                }
            }

            fn new_into_box() -> Box<message::BuildMessage + Send> {
                Box::new(#build_message_name::new())
            }
        }

        impl message::BuildMessage for #build_message_name {
            fn first_field(&self,version: message_version::MessageVersion) -> field_tag::FieldTag {
                #message_name::first_field(version)
            }

            fn field_count(&self,version: message_version::MessageVersion) -> usize {
                #message_name::field_count(version)
            }

            fn fields(&mut self,version: message_version::MessageVersion) -> message::FieldHashMap {
                fn get_or_set_fields(option_fields: &mut Option<message::FieldHashMap>,
                                     version: message_version::MessageVersion) -> message::FieldHashMap {
                    if option_fields.is_none() {
                        let fields = #message_name::fields(version);
                        *option_fields = Some(fields);
                    }

                    option_fields.as_ref().unwrap().clone()
                }

                match version {
                    message_version::MessageVersion::FIX40 => get_or_set_fields(&mut self.cache.fields_fix40,version),
                    message_version::MessageVersion::FIX41 => get_or_set_fields(&mut self.cache.fields_fix41,version),
                    message_version::MessageVersion::FIX42 => get_or_set_fields(&mut self.cache.fields_fix42,version),
                    message_version::MessageVersion::FIX43 => get_or_set_fields(&mut self.cache.fields_fix43,version),
                    message_version::MessageVersion::FIX44 => get_or_set_fields(&mut self.cache.fields_fix44,version),
                    message_version::MessageVersion::FIX50 => get_or_set_fields(&mut self.cache.fields_fix50,version),
                    message_version::MessageVersion::FIX50SP1 => get_or_set_fields(&mut self.cache.fields_fix50sp1,version),
                    message_version::MessageVersion::FIX50SP2 => get_or_set_fields(&mut self.cache.fields_fix50sp2,version),
                }
            }

            fn required_fields(&self,version: message_version::MessageVersion) -> message::FieldHashSet {
                #message_name::required_fields(version)
            }

            fn new_into_box(&self) -> Box<message::BuildMessage + Send> {
                #build_message_name::new_into_box()
            }

            fn build(&self) -> Box<message::Message + Send> {
                Box::new(#message_name::new())
            }
        }


        impl message::MessageBuildable for #message_name {
            fn builder(&self) -> Box<message::BuildMessage + Send> {
                #build_message_name::new_into_box()
            }

            fn builder_func(&self) -> fn() -> Box<message::BuildMessage + Send> {
                #build_message_name::new_into_box
            }
        }
    };
    let mut result = tokens;

    if is_fixt_message {
        let tokens = quote! {
            impl fixt::message::BuildFIXTMessage for #build_message_name {
                fn new_into_box(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
                    Box::new(#build_message_name::new())
                }

                fn build(&self) -> Box<fixt::message::FIXTMessage + Send> {
                    Box::new(#message_name::new())
                }
            }

            impl fixt::message::FIXTMessageBuildable for #message_name {
                fn builder(&self) -> Box<fixt::message::BuildFIXTMessage + Send> {
                    Box::new(#build_message_name::new())
                }
            }
        };
        result.extend(tokens);
    }

    result.into()
}

#[proc_macro_derive(BuildField, attributes(tag))]
pub fn build_field(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let source = input.to_string();
    let ast = syn::parse_str(&source[..]).unwrap();

    let tag = match extract_attribute_int(&ast, "_tag_gen", "tag") {
        Ok(bytes) => bytes,
        Err(ExtractAttributeError::BodyNotStruct) => {
            panic!("#[derive(BuildField)] can only be used with structs")
        }
        Err(ExtractAttributeError::FieldNotFound) => {
            panic!("#[derive(BuildField)] requires a _tag_gen field to be specified")
        }
        Err(ExtractAttributeError::AttributeNotFound) => {
            panic!("#[derive(BuildField)] requires the _tag_gen field to have the tag attribute")
        }
        Err(ExtractAttributeError::AttributeNotNameValue)
        | Err(ExtractAttributeError::AttributeValueWrongType) => panic!(
            "#[derive(BuildField)] tag attribute must be as an unsigned integer like #[tag=1234]"
        ),
    };
    let tag = tag.to_string();

    let mut tag_bytes = "b\"".to_string();
    tag_bytes += &tag[..];
    tag_bytes += "\"";

    let field_name = ast.ident;
    let tag = str_to_tokens(&tag[..]);
    let tag_bytes = str_to_tokens(&tag_bytes[..]);

    let tokens = quote! {
        impl #field_name {
            fn tag_bytes() -> &'static [u8] {
                #tag_bytes
            }

            fn tag() -> field_tag::FieldTag {
                field_tag::FieldTag(#tag)
            }
        }
    };
    tokens.into()
}
