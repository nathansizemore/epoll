// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// Licensed under the Mozilla Public License, Version 2.0
// This file may not be copied, modified, or distributed
// except according to those terms.


use std::fmt;


#[derive(Debug, Clone)]
pub enum CreateError {
    /// Invalid value specified in flags.
    EINVAL,
    /// The per-user limit on the number of epoll instances
    /// imposed by /proc/sys/fs/epoll/max_user_instances was encountered.
    EMFILE,
    /// The system limit on the total number of open files has been reached.
    ENFILE,
    /// There was insufficient memory to create the kernel object.
    ENOMEM
}

#[derive(Debug, Clone)]
pub enum CtrlError {
    ///  fd is not a valid file descriptor.
    EBADF,
    /// op was EPOLL_CTL_ADD, and the supplied file descriptor fd is already
    /// registered with this epoll instance.
    EEXIST,
    /// epfd is not an epoll file descriptor, or fd is the same as epfd,
    /// or the requested operation op is not supported by this interface.
    EINVAL,
    /// op was EPOLL_CTL_MOD or EPOLL_CTL_DEL, and fd is not registered
    /// with this epoll instance.
    ENOENT,
    /// There was insufficient memory to handle the requested op control
    /// operation.
    ENOSPC,
    /// The limit imposed by /proc/sys/fs/epoll/max_user_watches was
    /// encountered while trying to register (EPOLL_CTL_ADD) a new
    /// file descriptor on an epoll instance.
    EPERM
}

#[derive(Debug, Clone)]
pub enum WaitError {
    /// epfd is not a valid file descriptor.
    EBADF,
    /// The memory area pointed to by events is not accessible
    /// with write permissions.
    EFAULT,
    /// The call was interrupted by a signal handler before
    /// either any of the requested events occurred or the timeout expired.
    EINTR,
    /// epfd is not an epoll file descriptor, or maxevents is less than
    /// or equal to zero.
    EINVAL
}


impl fmt::Display for CreateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CreateError::EINVAL => "EINVAL".fmt(f),
            CreateError::EMFILE => "EMFILE".fmt(f),
            CreateError::ENFILE => "ENFILE".fmt(f),
            CreateError::ENOMEM => "ENOMEM".fmt(f)
        }
    }
}

impl fmt::Display for CtrlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CtrlError::EBADF => "EBADF".fmt(f),
            CtrlError::EEXIST => "EEXIST".fmt(f),
            CtrlError::EINVAL => "EINVAL".fmt(f),
            CtrlError::ENOENT => "ENOENT".fmt(f),
            CtrlError::ENOSPC => "ENOSPC".fmt(f),
            CtrlError::EPERM => "EPERM".fmt(f)
        }
    }
}

impl fmt::Display for WaitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WaitError::EBADF => "EBADF".fmt(f),
            WaitError::EFAULT => "EFAULT".fmt(f),
            WaitError::EINTR => "EINTR".fmt(f),
            WaitError::EINVAL => "EINVAL".fmt(f)
        }
    }
}
