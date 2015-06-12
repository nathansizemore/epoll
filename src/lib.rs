// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// Licensed under the Mozilla Public License, Version 2.0
// This file may not be copied, modified, or distributed
// except according to those terms.


extern crate libc;
extern crate errno;

use util::{CreateError, CtrlError, WaitError};


mod epoll;
mod util;


/// Represents the result of calling epoll_create1
pub type CreateResult = Result<i32, CreateError>;

/// Represents the result of calling epoll_ctrl
pub type CtrlResult = Result<i32, CtrlError>;

/// Represents the result of calling epoll_wait
pub type WaitResult = Result<i32, WaitError>;
