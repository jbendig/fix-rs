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

use mio::Token;

pub const NO_INBOUND_TIMEOUT_PADDING_MS: u64 = 250;
pub const INBOUND_MESSAGES_BUFFER_LEN_MAX: usize = 10;
pub const INBOUND_BYTES_BUFFER_CAPACITY: usize = 2048;
pub const CONNECTION_COUNT_MAX: usize = 65536;

pub const BASE_CONNECTION_TOKEN: Token = Token(3);
