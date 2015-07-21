// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible
// With Secondary Licenses", as defined by
// the Mozilla Public License, v. 2.0.


extern crate libc;
extern crate errno;


use std::result::Result;
use std::os::unix::io::RawFd;

use errno::errno;
use libc::c_int;
use libc::consts::os::posix88;

use util::*;
mod util;


/// Represents the result of calling epoll_create1
pub type CreateResult = Result<RawFd, CreateError>;

/// Represents the result of calling epoll_ctrl
pub type CtlResult = Result<(), CtlError>;

/// Represents the result of calling epoll_wait
pub type WaitResult = Result<u32, WaitError>;


#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
pub struct EpollEvent {
    events: u32,
    data: u64
}

#[cfg(not(target_arch = "x86_64"))]
#[repr(C)]
pub struct EpollEvent {
    events: u32,
    data: u64
}

extern "C" {
    fn epoll_create1(flags: c_int) -> c_int;
    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int,
        event: *const EpollEvent) -> c_int;
    fn epoll_wait(epfd: c_int, event: *mut EpollEvent,
        maxevents: c_int, timeout: c_int) -> c_int;
}

/// Attempts to create a new epoll instance
#[inline]
pub fn create1(flags: u32) -> CreateResult {
    let mut epoll_fd;
    unsafe {
        epoll_fd = epoll_create1(flags as c_int);
    }

    if epoll_fd == -1 {
        let errno = errno().0 as i32;
        return match errno {
            posix88::EINVAL     => Err(CreateError::EINVAL),
            posix88::EMFILE     => Err(CreateError::EMFILE),
            posix88::ENFILE     => Err(CreateError::ENFILE),
            posix88::ENOMEM     => Err(CreateError::ENOMEM),
            _ => panic!("Unexpected errno: {}", errno)
        }
    }

    Ok(epoll_fd)
}

/// Calls epoll_ctl(2) with supplied params
#[inline]
pub fn ctl(epoll_fd: RawFd, op: CtlOp,
           socket_fd: RawFd, event: Box<EpollEvent>) -> CtlResult {
    let mut x;
    unsafe {
        x = epoll_ctl(epoll_fd as c_int,
            op as c_int,
            socket_fd as c_int,
            &*event);
    }

    if x == -1 {
        let errno = errno().0 as i32;
        return match errno {
            posix88::EBADF      => Err(CtlError::EBADF),
            posix88::EEXIST     => Err(CtlError::EEXIST),
            posix88::EINVAL     => Err(CtlError::EINVAL),
            posix88::ENOENT     => Err(CtlError::ENOENT),
            posix88::ENOSPC     => Err(CtlError::ENOSPC),
            posix88::EPERM      => Err(CtlError::EPERM),
            _ => panic!("Unexpected errno: {}", errno)
        }
    }

    Ok(())
}

/// Calls epoll_wait(1) with supplied params
#[inline]
pub fn wait(epoll_fd: RawFd, events: &mut [EpollEvent],
    timeout: u32) -> WaitResult {

    println!("events.len(): {}", events.len());

    let mut num_fds_ready;
    unsafe {
        num_fds_ready = epoll_wait(epoll_fd as c_int,
            events.as_mut_ptr(),
            events.len() as c_int,
            timeout as c_int);
    }

    if num_fds_ready == -1 {
        let errno = errno().0 as i32;
        return match errno {
            posix88::EBADF  => Err(WaitError::EBADF),
            posix88::EFAULT => Err(WaitError::EFAULT),
            posix88::EINTR  => Err(WaitError::EINTR),
            posix88::EINVAL => Err(WaitError::EINVAL),
            _ => panic!("Unexpected errno: {}", errno)
        }
    }

    Ok(num_fds_ready as u32)
}
