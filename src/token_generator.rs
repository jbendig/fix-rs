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
use std::collections::HashSet;

pub struct TokenGenerator {
    seed: usize,
    //All created tokens will have a number >= reserved_end.
    reserved_end: usize,
    //Let user set some arbitrary token limit for resource reasons. For example, only so many ports
    //may be reserved for use.
    max_tokens: usize,
    active_tokens: HashSet<Token>,
}

impl TokenGenerator {
    pub fn new(reserved_end: usize, max_tokens: Option<usize>) -> TokenGenerator {
        TokenGenerator {
            seed: reserved_end,
            reserved_end: reserved_end,
            max_tokens: max_tokens.unwrap_or(usize::max_value() - reserved_end),
            active_tokens: HashSet::new(),
        }
    }

    pub fn create(&mut self) -> Option<Token> {
        if self.active_tokens.len() == self.max_tokens {
            return None;
        }

        loop {
            let token_id = self.seed;
            self.seed = self.seed.overflowing_add(1).0;

            let token = Token(token_id);
            if !self.active_tokens.contains(&token) && token_id >= self.reserved_end {
                self.active_tokens.insert(token);
                return Some(token);
            }
        }
    }

    pub fn remove(&mut self, token: Token) {
        self.active_tokens.remove(&token);
    }
}
