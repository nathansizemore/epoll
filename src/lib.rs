// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.


extern crate libc;
extern crate time;
extern crate simple_slab;
#[macro_use] extern crate bitflags;


use std::ops::{Add, Drop};
use std::sync::Mutex;
use std::io::{self, Error};
use std::os::unix::io::{RawFd, AsRawFd};

use time::{Duration, PreciseTime};
use simple_slab::Slab;
pub use interest::Interest;

mod interest;


bitflags! {
    flags ControlOptions: i32 {
        const EPOLL_CTL_ADD = libc::EPOLL_CTL_ADD,
        const EPOLL_CTL_MOD = libc::EPOLL_CTL_MOD,
        const EPOLL_CTL_DEL = libc::EPOLL_CTL_DEL
    }
}

bitflags! {
    pub flags Events: u32 {
        /// Sets the Edge Triggered behavior for the associated file descriptor.
        ///
        /// The default behavior for epoll is Level Triggered.
        const EPOLLET      = libc::EPOLLET as u32,
        /// The associated file is available for read operations.
        const EPOLLIN      = libc::EPOLLIN as u32,
        /// Error condition happened on the associated file descriptor.
        ///
        /// `wait` will always wait for this event; is not necessary to set it in events.
        const EPOLLERR     = libc::EPOLLERR as u32,
        /// Hang up happened on the associated file descriptor.
        ///
        /// `wait` will always wait for this event; it is not necessary to set it in events.
        /// Note that when reading from a channel such as a pipe or a stream socket, this event
        /// merely indicates that the peer closed its end of the channel. Subsequent reads from
        /// the channel will return 0 (end of file) only after all outstanding data in the
        /// channel has been consumed.
        const EPOLLHUP     = libc::EPOLLHUP as u32,
        /// The associated file is available for write operations.
        const EPOLLOUT     = libc::EPOLLOUT as u32,
        /// There is urgent data available for read operations.
        const EPOLLPRI     = libc::EPOLLPRI as u32,
        /// Stream socket peer closed connection, or shut down writing half of connection.
        ///
        /// This flag is especially useful for writing simple code to detect peer shutdown when
        /// using Edge Triggered monitoring.
        const EPOLLRDHUP   = libc::EPOLLRDHUP as u32,
        /// If `EPOLLONESHOT` and `EPOLLET` are clear and the process has the `CAP_BLOCK_SUSPEND`
        /// capability, ensure that the system does not enter "suspend" or "hibernate" while this
        /// event is pending or being processed.
        ///
        /// The event is considered as being "processed" from the time when it is returned by
        /// a call to `wait` until the next call to `wait` on the same `EpollInstance`
        /// descriptor, the closure of that file descriptor, the removal of the event file
        /// descriptor with `EPOLL_CTL_DEL`, or the clearing of `EPOLLWAKEUP` for the event file
        /// descriptor with `EPOLL_CTL_MOD`.
        const EPOLLWAKEUP  = libc::EPOLLWAKEUP as u32,
        /// Sets the one-shot behavior for the associated file descriptor.
        ///
        /// This means that after an event is pulled out with `wait` the associated file
        /// descriptor is internally disabled and no other events will be reported by the epoll
        /// interface.  The user must call `ctl` with `EPOLL_CTL_MOD` to rearm the file
        /// descriptor with a new event mask.
        const EPOLLONESHOT = libc::EPOLLONESHOT as u32
    }
}


/// Thread safe abstraction around the returned `fd` from `libc::epoll_create(1)`
pub struct EpollInstance {
    fd: libc::c_int,
    interest_mutex: Mutex<Slab<Interest>>,
    events: u64,
    wait: Duration
}

impl EpollInstance {

    /// Creates a new `EpollInstance`.
    ///
    /// ## Notes
    /// * `FD_CLOEXEC` flag is set on the underlying fd returned.
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
            interest_mutex: Mutex::new(Slab::<Interest>::new()),
            events: 0,
            wait: Duration::zero()
        })
    }

    /// Register an initial `Interest` with this instance.
    ///
    /// ## Panics
    ///
    /// Panics if the interior Mutex has been poisoned.
    pub fn add_interest(&mut self, interest: Interest) -> io::Result<()> {
        let mut event_mask = libc::epoll_event {
            events: interest.events().bits() as u32,
            u64: interest.data()
        };
        try!(ctl(self.fd,
                 EPOLL_CTL_ADD.bits(),
                 interest.as_raw_fd(),
                 &mut event_mask as *mut libc::epoll_event));

        let mut list = self.interest_mutex.lock().unwrap();
        (*list).insert(interest);

        Ok(())
    }

    /// Modify the original `Interest`, identified by its `RawFd`, to the passed
    /// interest's events and data fields.
    ///
    /// ## Panics
    ///
    /// Panics if the interior Mutex has been poisoned.
    pub fn mod_interest(&mut self, interest: &Interest) -> io::Result<()> {
        let mut event_mask = libc::epoll_event {
            events: interest.events().bits() as u32,
            u64: interest.data()
        };
        try!(ctl(self.fd,
                 EPOLL_CTL_MOD.bits(),
                 interest.as_raw_fd(),
                 &mut event_mask as *mut libc::epoll_event));

        let mut list = self.interest_mutex.lock().unwrap();
        for ref mut _interest in (*list).iter_mut() {
            if _interest.as_raw_fd() == interest.as_raw_fd() {
                _interest.set_events(interest.events());
                _interest.set_data(interest.data());
                break;
            }
        }

        Ok(())
    }

    /// Remove the passed `Interest`, identified by its `RawFd`, from this instance.
    ///
    /// ## Panics
    ///
    /// Panics if the interior Mutex has been poisoned.
    pub fn del_interest(&mut self, interest: &Interest) -> io::Result<()> {
        // In kernel versions before 2.6.9, the EPOLL_CTL_DEL operation required a non-null
        // pointer in event, even though this argument is ignored.
        let mut event_mask = libc::epoll_event {
            events: 0u32,
            u64: 0u64
        };
        try!(ctl(self.fd,
                 EPOLL_CTL_DEL.bits(),
                 interest.as_raw_fd(),
                 &mut event_mask as *mut libc::epoll_event));

        let mut offset: usize = 0;
        let mut list = self.interest_mutex.lock().unwrap();
        for ref mut _interest in (*list).iter() {
            if _interest.as_raw_fd() == interest.as_raw_fd() {
                break;
            }
            offset += 1
        }
        (*list).remove(offset);

        Ok(())
    }

    /// Waits for events on this instance for at most `timeout` milliseconds and returns at most
    /// `max_returned` `Interests`.
    ///
    /// ## Notes
    ///
    /// * If `timeout` is negative, it will block until an event is received.
    /// * `max_returned` must be greater than zero.
    ///
    /// ## Panics
    ///
    /// Panics if the interior Mutex has been poisoned.
    pub fn wait(&mut self, timeout: i32, max_returned: usize) -> io::Result<Vec<Interest>> {
        let timeout = if timeout < -1 { -1 } else { timeout };
        let mut ret_buf = Vec::<Interest>::with_capacity(max_returned);
        let mut buf = Vec::<libc::epoll_event>::with_capacity(max_returned);

        let (start, end) = unsafe {
            let start = PreciseTime::now();
            let num_events = try!(cvt(libc::epoll_wait(self.fd,
                                                       buf.as_mut_ptr(),
                                                       max_returned as i32,
                                                       timeout))) as usize;
            let end = PreciseTime::now();
            buf.set_len(num_events);
            (start, end)
        };

        self.events += buf.len() as u64;
        self.wait.add(start.to(end));

        let mut list = self.interest_mutex.lock().unwrap();
        for ref event in buf.iter() {
            for ref mut interest in (*list).iter_mut() {
                if interest.data() == event.u64 {
                    (*interest.events_mut()) = Events::from_bits(event.events).unwrap();
                    ret_buf.push(interest.clone());
                    break;
                }
            }
        }

        Ok((ret_buf))
    }

    /// Returns the total number of events reported.
    pub fn events(&self) -> u64 {
        self.events
    }

    /// Returns the total time spent in a blocked, waiting state.
    pub fn wait_time(&self) -> std::time::Duration {
        self.wait.to_std().unwrap()
    }
}

impl AsRawFd for EpollInstance {
    fn as_raw_fd(&self) -> RawFd { self.fd }
}

impl Drop for EpollInstance {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}

unsafe impl Send for EpollInstance {}
unsafe impl Sync for EpollInstance {}


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
