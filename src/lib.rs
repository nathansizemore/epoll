// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// Licensed under the Mozilla Public License, Version 2.0
// This file may not be copied, modified, or distributed
// except according to those terms.


extern crate libc;
extern crate errno;

use std::os::unix::io::RawFd;

use self::errno::errno;
use self::libc::consts::os::posix88;
use self::libc::{size_t, c_void, c_int, ssize_t};

use util::{CreateError, CtrlError, WaitError, CtlOp};


mod util;


/// Represents the result of calling epoll_create1
pub type CreateResult = Result<RawFd, CreateError>;

/// Represents the result of calling epoll_ctrl
pub type CtlResult = Result<RawFd, CtlError>;

/// Represents the result of calling epoll_wait
pub type WaitResult = Result<RawFd, WaitError>;


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
pub fn create1(flags: u32) -> CreateResult {
    let mut epoll_fd;
    unsafe {
        epoll_fd = epoll::create1(flags as c_int);
    }

    if epoll_fd == -1 {
        let errno = errno().0 as i32;
        return match errno {
            posix88::EINVAL     => Err(SetFdError::EINVAL),
            posix88::EMFILE     => Err(SetFdError::EMFILE),
            posix88::ENFILE     => Err(SetFdError::ENFILE),
            posix88::ENOMEM     => Err(SetFdError::ENOMEM),
            _ => panic!("Unexpected errno: {}", errno)
    }

    Ok(epoll_fd)
}

/// asdf
pub fn ctl(epoll_fd: RawFd, op: CtlOp,
           socket_fd: RawFd, event: Box<EpollEvent>) -> CtlResult {

}
