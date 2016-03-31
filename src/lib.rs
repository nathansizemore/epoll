// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.


extern crate libc;
extern crate errno;


use std::result::Result;
use std::os::unix::io::RawFd;

use errno::errno;
use libc::c_int;

use util::*;
pub mod util;


/// Represents the result of calling epoll_create1
pub type CreateResult = Result<RawFd, CreateError>;

/// Represents the result of calling epoll_ctrl
pub type CtlResult = Result<(), CtlError>;

/// Represents the result of calling epoll_wait
pub type WaitResult = Result<usize, WaitError>;


#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64
}

#[cfg(not(target_arch = "x86_64"))]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EpollEvent {
    pub events: u32,
    pub data: u64
}

extern "C" {
    fn epoll_create1(flags: c_int) -> c_int;
    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int, event: *mut EpollEvent) -> c_int;
    fn epoll_wait(epfd: c_int, event: *mut EpollEvent, maxevents: c_int, timeout: c_int) -> c_int;
}

/// Attempts to create a new epoll instance
#[inline]
pub fn create1(flags: u32) -> CreateResult {
    let epoll_fd;
    unsafe {
        epoll_fd = epoll_create1(flags as c_int);
    }

    if epoll_fd == -1 {
        let errno = errno().0 as i32;
        return match errno {
            libc::EINVAL     => Err(CreateError::EINVAL),
            libc::EMFILE     => Err(CreateError::EMFILE),
            libc::ENFILE     => Err(CreateError::ENFILE),
            libc::ENOMEM     => Err(CreateError::ENOMEM),
            _ => panic!("Unexpected errno: {}", errno)
        }
    }

    Ok(epoll_fd)
}

/// Calls epoll_ctl(2) with supplied params
#[inline]
pub fn ctl(epoll_fd: RawFd, op: u32,
           socket_fd: RawFd, event: &mut EpollEvent) -> CtlResult {
    let x;
    unsafe {
        x = epoll_ctl(epoll_fd as c_int,
            op as c_int,
            socket_fd as c_int,
            event as *mut EpollEvent);
    }

    if x == -1 {
        let errno = errno().0 as i32;
        return match errno {
            libc::EBADF      => Err(CtlError::EBADF),
            libc::EEXIST     => Err(CtlError::EEXIST),
            libc::EINVAL     => Err(CtlError::EINVAL),
            libc::ENOENT     => Err(CtlError::ENOENT),
            libc::ENOSPC     => Err(CtlError::ENOSPC),
            libc::EPERM      => Err(CtlError::EPERM),
            _ => panic!("Unexpected errno: {}", errno)
        }
    }

    Ok(())
}

/// Calls epoll_wait(1) with supplied params
#[inline]
pub fn wait(epoll_fd: RawFd, events: &mut [EpollEvent],
    timeout: i32) -> WaitResult {

    let num_fds_ready;
    unsafe {
        num_fds_ready = epoll_wait(epoll_fd as c_int,
            events.as_mut_ptr(),
            events.len() as c_int,
            timeout as c_int);
    }

    if num_fds_ready == -1 {
        let errno = errno().0 as i32;
        return match errno {
            libc::EBADF  => Err(WaitError::EBADF),
            libc::EFAULT => Err(WaitError::EFAULT),
            libc::EINTR  => Err(WaitError::EINTR),
            libc::EINVAL => Err(WaitError::EINVAL),
            _ => panic!("Unexpected errno: {}", errno)
        }
    }

    Ok(num_fds_ready as usize)
}
