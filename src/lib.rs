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

#![feature(attr_literals)]
#![feature(plugin)]
#![feature(proc_macro)]

#![plugin(phf_macros)]

#![allow(unknown_lints)]

extern crate chrono;
#[macro_use]
extern crate fix_rs_macros;
extern crate mio;
extern crate phf;
extern crate time;

#[cfg(feature="load-testing")]
pub mod byte_buffer;
#[cfg(not(feature="load-testing"))]
mod byte_buffer;
#[macro_use]
pub mod fixt;
pub mod constant;
#[macro_use]
pub mod field;
pub mod field_tag;
pub mod field_type;
pub mod fix;
pub mod fix_version;
pub mod hash;
#[macro_use]
pub mod message;
pub mod message_version;
mod network_read_retry;
pub mod rule;
mod token_generator;

//Dictionary is put last because it needs the above macros.
#[macro_use]
pub mod dictionary;
