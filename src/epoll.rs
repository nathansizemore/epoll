// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// Licensed under the Mozilla Public License, Version 2.0
// This file may not be copied, modified, or distributed
// except according to those terms.


//! epoll crate


use super::libc::{size_t, c_void, c_int, ssize_t};


#[cfg(target_arch = "x86_64")]
#[repr(C, packed)]
pub struct epoll_event {
    events: u32,
    data: u64
}

#[cfg(not(target_arch = "x86_64"))]
#[repr(C)]
pub struct epoll_event {
    events: i32,
    data: u64
}

extern "C" {
    fn epoll_create1(flags: c_int) -> c_int;
    fn epoll_ctl(epfd: c_int, op: c_int, fd: c_int,
        event: *const epoll_event) -> c_int;
    fn epoll_wait(epfd: c_int, event: *mut epoll_event,
        maxevents: c_int, timeout: c_int) -> c_int;
}
