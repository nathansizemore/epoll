// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.


extern crate libc;
extern crate time;
extern crate simple_slab;
#[macro_use] extern crate bitflags;


use std::io::{self, Error, ErrorKind};
use std::os::unix::io::{RawFd, AsRawFd};

use simple_slab::Slab;

pub use interest::Interest;

mod interest;



bitflags! {
    pub flags ControlOptions: libc::c_int {
        const EPOLL_CTL_ADD = libc::EPOLL_CTL_ADD,
        const EPOLL_CTL_MOD = libc::EPOLL_CTL_MOD,
        const EPOLL_CTL_DEL = libc::EPOLL_CTL_DEL
    }
}

bitflags! {
    pub flags Events: libc::c_int {
        const EPOLLET      = libc::EPOLLET,
        const EPOLLIN      = libc::EPOLLIN,
        const EPOLLERR     = libc::EPOLLERR,
        const EPOLLHUP     = libc::EPOLLHUP,
        const EPOLLMSG     = libc::EPOLLMSG,
        const EPOLLOUT     = libc::EPOLLOUT,
        const EPOLLPRI     = libc::EPOLLPRI,
        const EPOLLRDHUP   = libc::EPOLLRDHUP,
        const EPOLLRDBAND  = libc::EPOLLRDBAND,
        const EPOLLRDNORM  = libc::EPOLLRDNORM,
        const EPOLLWAKEUP  = libc::EPOLLWAKEUP,
        const EPOLLWRBAND  = libc::EPOLLWRBAND,
        const EPOLLWRNORM  = libc::EPOLLWRNORM,
        const EPOLLONESHOT = libc::EPOLLONESHOT
    }
}


pub struct EpollInstance {
    fd: libc::c_int,
    interest_list: Slab<Interest>
}

impl EpollInstance {

    pub fn new() -> io::Result<EpollInstance> {
        let epfd = unsafe {
            let fd = try!(cvt(libc::epoll_create(1)));
            let mut flags = try!(cvt(libc::fcntl(fd, libc::F_GETFD)));
            flags |= libc::FD_CLOEXEC;
            try!(cvt(libc::fcntl(fd, libc::F_SETFD, flags)));
            fd
        };

        Ok(EpollInstance {
            fd: epfd,
            interest_list: Slab::<Interest>::new()
        })
    }

    pub fn add_interest(&mut self, interest: &mut Interest) -> io::Result<()> {
        try!(ctl(self.fd,
                 EPOLL_CTL_ADD.bits(),
                 interest.as_raw_fd(),
                 interest.event_mask_mut() as *mut libc::epoll_event));
        self.interest_list.insert(interest.clone());
        Ok(())
    }

    pub fn mod_interest(&mut self, interest: &mut Interest) -> io::Result<()> {

    }
}

fn ctl(epfd: libc::c_int,
       op: libc::c_int,
       fd: libc::c_int,
       event: *mut libc::epoll_event)
       -> io::Result<()>
{
    unsafe { try!(cvt(libc::epoll_ctl(epfd, op, fd, event))) };
    Ok(())
}

fn cvt(result: libc::c_int) -> io::Result<libc::c_int> {
    if result < 0 {
        Err(Error::last_os_error())
    } else {
        Ok(result)
    }
}
