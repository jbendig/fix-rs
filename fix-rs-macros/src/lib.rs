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
#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

fn str_to_ident(input: &str) -> syn::Ident {
    syn::Ident::new(input, Span::call_site())
}

fn str_to_tokens(input: &str) -> TokenStream {
    let tokens: proc_macro2::TokenStream = {
        /* transform input */
        input.to_token_stream()
    };
    tokens
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
    attr_ident: &'static str,
) -> Result<syn::Lit, ExtractAttributeError> {
    if let syn::Data::Struct(ref data) = ast.data {
        for field in &data.fields {
            if field.ident.as_ref().expect("Field must have an identifier") != field_ident {
                continue;
            }
            for attr in &field.attrs {
                //todo if the field has attribute not equal to attr_ident, skip
                //todo this is somewhat problematic since syn1.0 now regards attributes as tokenstreams
                //todo so for now, we just use good old contains with strings but this is NOT GOOD
                // println!(
                //     "attr token includes attr ident: {:?}",
                //     attr.into_token_stream().to_string().contains(attr_ident)
                // );
                if !attr.into_token_stream().to_string().contains(attr_ident) {
                    continue;
                }
                // println!("attr ident: {:?}", attr_ident);
                // let attr = attr.parse_args::<TokenStream>(); //this has to be s
                // if attr.tokens.to_string() != attr_ident {
                //     continue;
                // }
                // if attr.name() != attr_ident {
                //     continue;
                // }
                match attr.parse_meta() {
                    Ok(meta) => {
                        if let syn::Meta::NameValue(meta) = meta {
                            return Ok(meta.lit);
                        } else {
                            return Err(ExtractAttributeError::AttributeNotNameValue);
                        }
                    }
                    Err(_) => {
                        return Err(ExtractAttributeError::AttributeNotNameValue);
                    }
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
    let lit = (extract_attribute_value(ast, field_ident, attr_ident))?;

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
    let lit = (extract_attribute_value(ast, field_ident, attr_ident))?;

    if let syn::Lit::Int(int) = lit {
        match int.base10_parse() {
            Ok(value) => {
                return Ok(value);
            }
            Err(_) => {
                return Err(ExtractAttributeError::AttributeValueWrongType);
            }
        }
    }

    Err(ExtractAttributeError::AttributeValueWrongType)
}

#[proc_macro_derive(BuildMessage, attributes(message_type))]
pub fn build_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let source = input.to_string();
    let ast: DeriveInput = syn::parse(input).unwrap();

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
    let build_message_name = str_to_ident(&build_message_name[..]);
    let message_type_header = str_to_tokens(&message_type_header[..]);

    let mut tokens = quote! {
        impl #message_name {
            fn msg_type_header() -> &'static [u8] {
                (#message_type_header).as_bytes()
            }
        }

        pub struct #build_message_name {
            cache: crate::message::BuildMessageInternalCache,
        }

        impl #build_message_name {
            fn new() -> #build_message_name {
                #build_message_name {
                    cache: crate::message::BuildMessageInternalCache {
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

            fn new_into_box() -> Box<dyn crate::message::BuildMessage + Send> {
                Box::new(#build_message_name::new())
            }
        }

        impl crate::message::BuildMessage for #build_message_name {
            fn first_field(&self, version: crate::message_version::MessageVersion) -> crate::field_tag::FieldTag {
                #message_name::first_field(version)
            }

            fn field_count(&self, version: crate::message_version::MessageVersion) -> usize {
                #message_name::field_count(version)
            }

            fn fields(&mut self,version: crate::message_version::MessageVersion) -> crate::message::FieldHashMap {
                fn get_or_set_fields(option_fields: &mut Option<crate::message::FieldHashMap>,
                                     version: crate::message_version::MessageVersion) -> crate::message::FieldHashMap {
                    if option_fields.is_none() {
                        let fields = #message_name::fields(version);
                        *option_fields = Some(fields);
                    }

                    option_fields.as_ref().unwrap().clone()
                }

                match version {
                    crate::message_version::MessageVersion::FIX40 => get_or_set_fields(&mut self.cache.fields_fix40,version),
                    crate::message_version::MessageVersion::FIX41 => get_or_set_fields(&mut self.cache.fields_fix41,version),
                    crate::message_version::MessageVersion::FIX42 => get_or_set_fields(&mut self.cache.fields_fix42,version),
                    crate::message_version::MessageVersion::FIX43 => get_or_set_fields(&mut self.cache.fields_fix43,version),
                    crate::message_version::MessageVersion::FIX44 => get_or_set_fields(&mut self.cache.fields_fix44,version),
                    crate::message_version::MessageVersion::FIX50 => get_or_set_fields(&mut self.cache.fields_fix50,version),
                    crate::message_version::MessageVersion::FIX50SP1 => get_or_set_fields(&mut self.cache.fields_fix50sp1,version),
                    crate::message_version::MessageVersion::FIX50SP2 => get_or_set_fields(&mut self.cache.fields_fix50sp2,version),
                }
            }

            fn required_fields(&self,version: crate::message_version::MessageVersion) -> crate::message::FieldHashSet {
                #message_name::required_fields(version)
            }

            fn new_into_box(&self) -> Box<dyn crate::message::BuildMessage + Send> {
                #build_message_name::new_into_box()
            }

            fn build(&self) -> Box<dyn crate::message::Message + Send> {
                Box::new(#message_name::new())
            }
        }

        impl crate::message::MessageBuildable for #message_name {
            fn builder(&self) -> Box<dyn crate::message::BuildMessage + Send> {
                #build_message_name::new_into_box()
            }

            fn builder_func(&self) -> fn() -> Box<dyn crate::message::BuildMessage + Send> {
                #build_message_name::new_into_box
            }
        }
    };

    if is_fixt_message {
        let extra_tokens = quote! {
            impl crate::fixt::message::BuildFIXTMessage for #build_message_name {
                fn new_into_box(&self) -> Box<dyn crate::fixt::message::BuildFIXTMessage + Send> {
                    Box::new(#build_message_name::new())
                }

                fn build(&self) -> Box<dyn crate::fixt::message::FIXTMessage + Send> {
                    Box::new(#message_name::new())
                }
            }

            impl crate::fixt::message::FIXTMessageBuildable for #message_name {
                fn builder(&self) -> Box<dyn crate::fixt::message::BuildFIXTMessage + Send> {
                    Box::new(#build_message_name::new())
                }
            }
        };
        tokens.extend(extra_tokens);
    }

    proc_macro::TokenStream::from(tokens)
}

#[proc_macro_derive(BuildField, attributes(tag))]
pub fn build_field(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // println!("input is:{:?}", input);
    let ast: DeriveInput = syn::parse(input).unwrap();
    // println!("reached here start");
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
    // println!("tag: {}", tag);
    let tag_u64 = tag;
    let tag = tag.to_string();

    let mut tag_bytes = "b\"".to_string();
    tag_bytes += &tag[..];
    tag_bytes += "\"";

    let field_name = ast.ident.to_token_stream(); //todo is this the write way?
                                                  // println!("field_name is:{}", field_name);
    let tag = str_to_tokens(&tag[..]);
    let tag_bytes = str_to_tokens(&tag_bytes[..]);
    // println!("tag bytes: {}",tag_bytes);
    // let tag_bytes = "aa".as_bytes();
    let tokens = quote! {
        impl #field_name {
            fn tag_bytes() -> &'static [u8] {
                (#tag_bytes).as_bytes()
            }

            fn tag() -> crate::field_tag::FieldTag {
                crate::field_tag::FieldTag(#tag_u64)
            }
        }
    };
    // println!("reached here end");
    proc_macro::TokenStream::from(tokens)
}
