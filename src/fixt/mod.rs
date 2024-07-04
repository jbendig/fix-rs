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

pub mod engine;
mod engine_thread;
#[macro_use]
pub mod message;

pub mod tests {
    pub use super::engine_thread::{
        AUTO_DISCONNECT_AFTER_INBOUND_RESEND_REQUEST_LOOP_COUNT,
        AUTO_DISCONNECT_AFTER_NO_LOGON_RECEIVED_SECONDS, INBOUND_BYTES_BUFFER_CAPACITY,
        INBOUND_MESSAGES_BUFFER_LEN_MAX,
    };
}
