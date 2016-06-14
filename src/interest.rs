// Copyright 2016 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.


use std::os::unix::io::{RawFd, AsRawFd};

use libc;
use time::PreciseTime;


#[derive(Clone)]
pub struct Interest {
    fd: libc::c_int,
    event_mask: libc::epoll_event,
    last_event: PreciseTime
}

impl Interest {

    pub fn new(fd: libc::c_int, flags: ::Events, data: u64) -> Interest {
        Interest {
            fd: fd,
            event_mask: libc::epoll_event {
                events: flags.bits() as u32,
                u64: data
            },
            last_event: PreciseTime::now()
        }
    }

    pub fn event_mask<'a>(&'a self) -> &'a libc::epoll_event {
        &self.event_mask
    }

    pub fn event_mask_mut<'a>(&'a mut self) -> &'a mut libc::epoll_event {
        &mut self.event_mask
    }
}

impl AsRawFd for Interest {
    fn as_raw_fd(&self) -> RawFd { self.fd }
}
