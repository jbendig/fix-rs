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

use mio::{Evented,Poll,PollOpt,Ready,Registration,SetReadiness,Token};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;

pub struct NetworkReadRetry {
    tokens_to_retry: VecDeque<Token>,
    registration: RefCell<Option<Registration>>,
    set_readiness: RefCell<Option<SetReadiness>>,
}

impl NetworkReadRetry {
    pub fn new() -> NetworkReadRetry {
        NetworkReadRetry {
            tokens_to_retry: VecDeque::new(),
            registration: RefCell::new(None),
            set_readiness: RefCell::new(None),
        }
    }

    pub fn queue(&mut self,token: Token) {
        if self.tokens_to_retry.contains(&token) {
            return;
        }

        self.tokens_to_retry.push_back(token);

        if let Some(ref mut set_readiness) = *self.set_readiness.borrow_mut() {
            let _ = set_readiness.set_readiness(Ready::readable());
        }
    }

    pub fn poll(&mut self) -> Option<Token> {
        self.remove_by_index(0)
    }

    pub fn remove_all(&mut self,token: Token) {
        let mut x = 0;
        while x < self.tokens_to_retry.len() {
            if self.tokens_to_retry[x] == token {
                self.remove_by_index(x);
                continue;
            }

            x += 1;
        }
    }

    pub fn remove_by_index(&mut self,index: usize) -> Option<Token> {
        if let Some(ref mut set_readiness) = *self.set_readiness.borrow_mut() {
            let _ = set_readiness.set_readiness(Ready::empty());
        }

        self.tokens_to_retry.remove(index)
    }
}

impl Evented for NetworkReadRetry {
    fn register(&self,poll: &Poll,token: Token,interest: Ready,opts: PollOpt) -> io::Result<()> {
        if self.registration.borrow().is_some() {
            return Err(io::Error::new(io::ErrorKind::Other,"NetworkReadRetry already registered"));
        }

        let (registration,set_readiness) = Registration::new2();
        registration.register(poll,token,interest,opts)?;
        *self.registration.borrow_mut() = Some(registration);
        *self.set_readiness.borrow_mut() = Some(set_readiness);

        Ok(())
    }

    fn reregister(&self,poll: &Poll,token: Token,interest: Ready,opts: PollOpt) -> io::Result<()> {
        if let Some(ref mut registration) = *self.registration.borrow_mut() {
            return poll.reregister(registration,token,interest,opts);
        }

        Err(io::Error::new(io::ErrorKind::Other,"NetworkReadRetry not registered"))
    }

    fn deregister(&self,poll: &Poll) -> io::Result<()> {
        if let Some(ref mut registration) = *self.registration.borrow_mut() {
            return poll.deregister(registration);
        }

        Err(io::Error::new(io::ErrorKind::Other,"NetworkReadRetry not registered"))
    }
}
