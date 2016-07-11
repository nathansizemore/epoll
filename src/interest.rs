// Copyright 2016 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.


use std::os::unix::io::{RawFd, AsRawFd};

use libc;


#[derive(Clone)]
pub struct Interest {
    fd: RawFd,
    events: ::Events,
    data: u64
}

impl Interest {

    /// Creates a new Interest.
    pub fn new(fd: RawFd, flags: ::Events, data: u64) -> Interest {
        Interest {
            fd: fd,
            events: flags,
            data: data
        }
    }

    /// Copy of `Event` flags associated with this `Interest`.
    pub fn events(&self) -> ::Events {
        self.events
    }

    /// Returns a mutable reference to its `Events` flags.
    pub fn events_mut<'a>(&'a mut self) -> &'a mut ::Events {
        &mut self.events
    }

    /// Replaces all associated `Event` flags with `events`.
    pub fn set_events(&mut self, events: ::Events) {
        self.events = events;
    }

    /// Copy of arbitrary data associated with this `Interest`.
    pub fn data(&self) -> u64 {
        self.data
    }

    /// Returns a mutable reference to its arbitrary data.
    pub fn data_mut<'a>(&'a mut self) -> &'a mut u64 {
        &mut self.data
    }

    /// Replaces arbitrary data with `data`.
    pub fn set_data(&mut self, data: u64) {
        self.data = data;
    }
}

impl AsRawFd for Interest {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

pub fn int2event(interest: &Interest) -> libc::epoll_event {
    libc::epoll_event {
        events: interest.events().bits() as u32,
        u64: interest.data()
    }
}
