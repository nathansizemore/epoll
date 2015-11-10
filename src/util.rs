// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.


//use std;
use std::{fmt, error};


pub mod ctl_op {
    /// Register the target file descriptor fd on the epoll instance
    /// referred to by the file descriptor epfd and associate the
    /// event with the internal file linked to fd.
    pub const ADD: u32 = 1;
    /// Change the event event associated with the target file descriptor fd.
    pub const MOD: u32 = 2;
    /// Remove (deregister) the target file descriptor fd from the epoll
    /// instance referred to by epfd. The event is ignored and can be NULL.
    pub const DEL: u32 = 3;
}

pub mod event_type {
    /// The associated file is available for read(2) operations.
    pub const EPOLLIN:          u32 = 0x001;
    /// The associated file is available for write(2) operations.
    pub const EPOLLOUT:         u32 = 0x004;
    /// Stream socket peer closed connection, or shut down writing
    /// half of connection.
    pub const EPOLLRDHUP:       u32 = 0x2000;
    /// There is urgent data available for read(2) operations.
    pub const EPOLLPRI:         u32 = 0x002;
    /// Error condition happened on the associated file descriptor.
    /// epoll_wait(2) will always wait for this event; it is not
    /// necessary to set it in events.
    pub const EPOLLERR:         u32 = 0x008;
    /// Hang up happened on the associated file descriptor. epoll_wait(2)
    /// will always wait for this event; it is not necessary to set it
    /// in events.
    pub const EPOLLHUP:         u32 = 0x010;
    /// Sets the Edge Triggered behavior for the associated file descriptor.
    /// The default behavior for epoll is Level Triggered.
    pub const EPOLLET:          u32 = (1 << 31);
    /// Sets the one-shot behavior for the associated file descriptor.
    /// This means that after an event is pulled out with epoll_wait(2)
    /// the associated file descriptor is internally disabled and no other
    /// events will be reported by the epoll interface. The user must call
    /// epoll_ctl() with EPOLL_CTL_MOD to rearm the file descriptor with a
    /// new event mask.
    pub const EPOLLONESHOT:     u32 = (1 << 30);
    /// If EPOLLONESHOT and EPOLLET are clear and the process has the
    /// CAP_BLOCK_SUSPEND capability, ensure that the system does not
    /// enter "suspend" or "hibernate" while this event is pending or
    /// being processed.  The event is considered as being "processed"
    /// from the time when it is returned by a call to epoll_wait(2)
    /// until the next call to epoll_wait(2) on the same epoll(7) file
    /// descriptor, the closure of that file descriptor, the removal
    /// of the event file descriptor with EPOLL_CTL_DEL, or the
    /// clearing of EPOLLWAKEUP for the event file descriptor with
    /// EPOLL_CTL_MOD.
    pub const EPOLLWAKEUP:      u32 = (1 << 29);
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CtlError {
    /// fd is not a valid file descriptor.
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

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl error::Error for CreateError {
    fn description(&self) -> &str {
        match *self {
            CreateError::EINVAL => "Invalid flags",
            CreateError::EMFILE => "/proc/sys/fs/epoll/max_user_instances limit reached",
            CreateError::ENFILE => "NOFILE limit has been reached",
            CreateError::ENOMEM => "Insufficient memory"
        }
    }
}

impl error::Error for CtlError {
    fn description(&self) -> &str {
        match *self {
            CtlError::EBADF => "Invalid file descriptor",
            CtlError::EEXIST => "File descriptor already registered",
            CtlError::EINVAL => "epoll_fd is not an epoll file descriptor",
            CtlError::ENOENT => "File descriptor not registered",
            CtlError::ENOSPC => "Insufficient memory",
            CtlError::EPERM => "/proc/sys/fs/epoll/max_user_watches limit reached"
        }
    }
}

impl error::Error for WaitError {
    fn description(&self) -> &str {
        match *self {
            WaitError::EBADF => "Invalid file descriptor",
            WaitError::EFAULT => "Memory pointed to by events is not accessible with write permissions",
            WaitError::EINTR => "Interrupted by a signal before any requested events occurred or timeout expired",
            WaitError::EINVAL => "epoll_fd not an epoll file descriptor, or maxevents is less than or equal to zero."
        }
    }
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

impl fmt::Display for CtlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CtlError::EBADF => "EBADF".fmt(f),
            CtlError::EEXIST => "EEXIST".fmt(f),
            CtlError::EINVAL => "EINVAL".fmt(f),
            CtlError::ENOENT => "ENOENT".fmt(f),
            CtlError::ENOSPC => "ENOSPC".fmt(f),
            CtlError::EPERM => "EPERM".fmt(f)
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
