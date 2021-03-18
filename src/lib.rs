// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate bitflags;
extern crate libc;

use std::{
    collections::hash_map::{Entry, HashMap},
    convert::TryInto,
    fmt::{self, Debug, Formatter},
    io::{self, ErrorKind},
    marker::PhantomData,
    mem::MaybeUninit,
    num::TryFromIntError,
    os::unix::io::RawFd,
    ptr::null_mut,
};

#[derive(Copy, Clone)]
union RawData {
    ptr: *mut libc::c_void,
    fd: RawFd,
    _u32: u32,
    _u64: u64,
}

/// Regroup DakaKind type
pub trait DataKind {}

// TODO write a macro_delc! for Ptr Fd U32 and U64

/// This represent Ptr mode
#[derive(Debug, Copy, Clone)]
pub struct Ptr<T> {
    phantom: PhantomData<*mut T>,
}
impl<T> DataKind for Ptr<T> {}

/// This represent Fd mode
#[derive(Debug, Copy, Clone)]
pub struct Fd;
impl DataKind for Fd {}

/// This represent U32 mode
#[derive(Debug, Copy, Clone)]
pub struct U32;
impl DataKind for U32 {}

/// This represent U64 mode
#[derive(Debug, Copy, Clone)]
pub struct U64;
impl DataKind for U64 {}

/// Data is used to represent user data in EPoll
/// You can only choice from 4 types Ptr<T>, Fd, U32, U64
/// use the appropriate function to create them
pub struct Data<T: DataKind> {
    raw: RawData,
    data_kind: PhantomData<T>,
}

impl Data<Fd> {
    pub fn new_fd(fd: RawFd) -> Self {
        Self {
            raw: RawData { fd },
            data_kind: PhantomData,
        }
    }

    pub fn fd(&self) -> RawFd {
        unsafe { self.raw.fd }
    }
}

impl Clone for Data<Fd> {
    fn clone(&self) -> Self {
        Self::new_fd(self.fd())
    }
}

impl Debug for Data<Fd> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data<Fd>")
            .field("raw", &self.fd())
            .field("data_kind", &self.data_kind)
            .finish()
    }
}

impl<T> Data<Ptr<T>> {
    pub fn new_ptr(t: T) -> Self
    where
        T: Into<Box<T>>,
    {
        let ptr = Box::into_raw(t.into()) as *mut _;
        Self {
            raw: RawData { ptr },
            data_kind: PhantomData,
        }
    }

    pub fn ptr(&self) -> &T {
        unsafe { &*(self.raw.ptr as *const T) }
    }

    pub fn ptr_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.raw.ptr as *mut T) }
    }

    pub fn into_inner(self) -> Box<T> {
        unsafe { Box::from_raw(self.raw.ptr as *mut T) }
    }
}

impl<T: Debug> Debug for Data<Ptr<T>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data<Ptr<T>>")
            .field("raw", &self.ptr())
            .field("data_kind", &self.data_kind)
            .finish()
    }
}

impl<T: Clone> Clone for Data<Ptr<T>> {
    fn clone(&self) -> Self {
        Self::new_ptr(self.ptr().clone())
    }
}

impl Data<U32> {
    pub fn new_u32(_u32: u32) -> Self {
        Self {
            raw: RawData { _u32 },
            data_kind: PhantomData,
        }
    }

    pub fn _u32(&self) -> u32 {
        unsafe { self.raw._u32 }
    }
}

impl Clone for Data<U32> {
    fn clone(&self) -> Self {
        Self::new_u32(self._u32())
    }
}

impl Debug for Data<U32> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data<U32>")
            .field("raw", &self._u32())
            .field("data_kind", &self.data_kind)
            .finish()
    }
}

impl Data<U64> {
    pub fn new_u64(_u64: u64) -> Self {
        Self {
            raw: RawData { _u64 },
            data_kind: PhantomData,
        }
    }

    pub fn _u64(&self) -> u64 {
        unsafe { self.raw._u64 }
    }
}

impl Clone for Data<U64> {
    fn clone(&self) -> Self {
        Self::new_u64(self._u64())
    }
}

impl Debug for Data<U64> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Data<U64>")
            .field("raw", &self._u64())
            .field("data_kind", &self.data_kind)
            .finish()
    }
}

#[repr(i32)]
#[allow(non_camel_case_types)]
enum ControlOptions {
    /// Indicates an addition to the interest list.
    EPOLL_CTL_ADD = libc::EPOLL_CTL_ADD,
    /// Indicates a modification of flags for an interest already in list.
    EPOLL_CTL_MOD = libc::EPOLL_CTL_MOD,
    /// Indicates a removal of an interest from the list.
    EPOLL_CTL_DEL = libc::EPOLL_CTL_DEL,
}

bitflags! {
    /// BitFlags that allow to configure events to listen from a file descriptor
    pub struct Events: u32 {
        /// Sets the Edge Triggered behavior for the associated file descriptor.
        ///
        /// The default behavior for epoll is Level Triggered.
        const EPOLLET      = libc::EPOLLET as u32;
        /// The associated file is available for read operations.
        const EPOLLIN      = libc::EPOLLIN as u32;
        /// Error condition happened on the associated file descriptor.
        ///
        /// `wait` will always wait for this event; is not necessary to set it in events.
        const EPOLLERR     = libc::EPOLLERR as u32;
        /// Hang up happened on the associated file descriptor.
        ///
        /// `wait` will always wait for this event; it is not necessary to set it in events.
        /// Note that when reading from a channel such as a pipe or a stream socket, this event
        /// merely indicates that the peer closed its end of the channel. Subsequent reads from
        /// the channel will return 0 (end of file) only after all outstanding data in the
        /// channel has been consumed.
        const EPOLLHUP     = libc::EPOLLHUP as u32;
        /// The associated file is available for write operations.
        const EPOLLOUT     = libc::EPOLLOUT as u32;
        /// There is urgent data available for read operations.
        const EPOLLPRI     = libc::EPOLLPRI as u32;
        /// Stream socket peer closed connection, or shut down writing half of connection.
        ///
        /// This flag is especially useful for writing simple code to detect peer shutdown when
        /// using Edge Triggered monitoring.
        const EPOLLRDHUP   = libc::EPOLLRDHUP as u32;
        /// If `EPOLLONESHOT` and `EPOLLET` are clear and the process has the `CAP_BLOCK_SUSPEND`
        /// capability, ensure that the system does not enter "suspend" or "hibernate" while this
        /// event is pending or being processed.
        ///
        /// The event is considered as being "processed" from the time when it is returned by
        /// a call to `wait` until the next call to `wait` on the same `EpollInstance`
        /// descriptor, the closure of that file descriptor, the removal of the event file
        /// descriptor with `EPOLL_CTL_DEL`, or the clearing of `EPOLLWAKEUP` for the event file
        /// descriptor with `EPOLL_CTL_MOD`.
        const EPOLLWAKEUP  = libc::EPOLLWAKEUP as u32;
        /// Sets the one-shot behavior for the associated file descriptor.
        ///
        /// This means that after an event is pulled out with `wait` the associated file
        /// descriptor is internally disabled and no other events will be reported by the epoll
        /// interface.  The user must call `ctl` with `EPOLL_CTL_MOD` to rearm the file
        /// descriptor with a new event mask.
        const EPOLLONESHOT = libc::EPOLLONESHOT as u32;
        /// Sets an exclusive wakeup mode for the epoll file descriptor that is being attached to
        /// the target file descriptor, `fd`. When a wakeup event occurs and multiple epoll file
        /// descriptors are attached to the same target file using `EPOLLEXCLUSIVE`, one or more of
        /// the epoll file descriptors will receive an event with `wait`. The default in this
        /// scenario (when `EPOLLEXCLUSIVE` is not set) is for all epoll file descriptors to
        /// receive an event. `EPOLLEXCLUSIVE` is thus useful for avoiding thundering herd problems
        /// in certain scenarios.
        ///
        /// If the same file descriptor is in multiple epoll instances, some with the
        /// `EPOLLEXCLUSIVE` flag, and others without, then events will be provided to all epoll
        /// instances that did not specify `EPOLLEXCLUSIVE`, and at least one of the epoll
        /// instances that did specify `EPOLLEXCLUSIVE`.
        ///
        /// The following values may be specified in conjunction with `EPOLLEXCLUSIVE`: `EPOLLIN`,
        /// `EPOLLOUT`, `EPOLLWAKEUP`, and `EPOLLET`. `EPOLLHUP` and `EPOLLERR` can also be
        /// specified, but this is not required: as usual, these events are always reported if they
        /// occur, regardless of whether they are specified in `Events`. Attempts to specify other
        /// values in `Events` yield the error `EINVAL`.
        ///
        /// `EPOLLEXCLUSIVE` may be used only in an `EPOLL_CTL_ADD` operation; attempts to employ
        /// it with `EPOLL_CTL_MOD` yield an error. If `EPOLLEXCLUSIVE` has been set using `ctl`,
        /// then a subsequent `EPOLL_CTL_MOD` on the same `epfd`, `fd` pair yields an error. A call
        /// to `ctl` that specifies `EPOLLEXCLUSIVE` in `Events` and specifies the target file
        /// descriptor `fd` as an epoll instance will likewise fail. The error in all of these
        /// cases is `EINVAL`.
        ///
        /// The `EPOLLEXCLUSIVE` flag is an input flag for the `Event.events` field when calling
        /// `ctl`; it is never returned by `wait`.
        const EPOLLEXCLUSIVE = libc::EPOLLEXCLUSIVE as u32;
    }
}

/// 'libc::epoll_event' equivalent.
#[repr(C)]
#[cfg_attr(
    any(
        all(
            target_arch = "x86",
            not(target_env = "musl"),
            not(target_os = "android")
        ),
        target_arch = "x86_64"
    ),
    repr(packed)
)]
pub struct Event<T: DataKind> {
    events: Events,
    data: Data<T>,
}

static_assertions::assert_eq_size!(
    libc::epoll_event,
    Event<Ptr<()>>,
    Event<Fd>,
    Event<U32>,
    Event<U64>,
);

impl<T: DataKind> Event<T> {
    pub fn new(events: Events, data: Data<T>) -> Self {
        Self { events, data }
    }

    pub fn events(&self) -> Events {
        self.events
    }

    pub fn data(&self) -> &Data<T> {
        // https://github.com/rust-lang/rust/issues/46043
        // it's safe cause Event align is 1
        static_assertions::assert_eq_align!(
            u8,
            libc::epoll_event,
            Event<Ptr<()>>,
            Event<Fd>,
            Event<U32>,
            Event<U64>,
        );
        unsafe { &self.data }
    }
}

impl<T: DataKind> Clone for Event<T>
where
    Data<T>: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.events, self.data().clone())
    }
}

impl<T: DataKind> Debug for Event<T>
where
    Data<T>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("events", &self.events())
            .field("data", self.data())
            .finish()
    }
}

/// This represent an EPoll instance
/// You will need to choice between 4 datas types
/// RawFd, u32, u64, Ptr<T>
/// This is enforced cause epoll doesn't allow to diffenciate
/// the union its use internally to stock user data
/// and anyway mix between data type don't make much sense
///
/// This will disallow any miss use about the union at compile time
///
/// Notice that while this is safe this currently can't prevent leak
/// You will need to handle this a little yourself by calling `into_inner()`
/// when you use the Ptr<T> type
pub struct EPoll<T: DataKind> {
    fd: RawFd,
    datas: HashMap<RawFd, Event<T>>,
    buffer: Vec<MaybeUninit<Event<T>>>,
}

impl<T: DataKind> Debug for EPoll<T>
where
    Data<T>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EPoll")
            .field("fd", &self.fd)
            .field("datas", &self.datas)
            .finish()
    }
}

impl<T: DataKind> EPoll<T> {
    /// Creates a new epoll file descriptor.
    ///
    /// If `cloexec` is true, `FD_CLOEXEC` will be set on the returned file descriptor.
    ///
    /// ## Notes
    ///
    /// * `epoll_create1()` is the underlying syscall.
    pub fn create(cloexec: bool, max_events: i32) -> io::Result<Self> {
        let flags = if cloexec { libc::EPOLL_CLOEXEC } else { 0 };
        let fd = Self::cvt(unsafe { libc::epoll_create1(flags) })?;
        let max_events = max_events
            .try_into()
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;

        Ok(Self {
            fd,
            datas: Default::default(),
            buffer: Vec::with_capacity(max_events),
        })
    }

    /// Safe wrapper to add an event for `libc::epoll_ctl`
    pub fn ctl_add(&mut self, fd: RawFd, event: Event<T>) -> io::Result<()> {
        match self.datas.entry(fd) {
            Entry::Occupied(_) => Err(ErrorKind::AlreadyExists.into()),
            Entry::Vacant(v) => {
                let event = v.insert(event) as *mut _ as *mut libc::epoll_event;
                let op = ControlOptions::EPOLL_CTL_ADD as i32;

                Self::cvt(unsafe { libc::epoll_ctl(self.fd, op, fd, event) })?;

                Ok(())
            }
        }
    }

    /// Safe wrapper to modify an event for `libc::epoll_ctl`
    /// return the old value
    pub fn ctl_mod(
        &mut self,
        fd: RawFd,
        mut event: Event<T>,
    ) -> io::Result<Event<T>> {
        match self.datas.entry(fd) {
            Entry::Occupied(mut o) => {
                let new = &mut event as *mut _ as *mut libc::epoll_event;
                let op = ControlOptions::EPOLL_CTL_MOD as i32;

                Self::cvt(unsafe { libc::epoll_ctl(self.fd, op, fd, new) })?;

                Ok(o.insert(event))
            }
            Entry::Vacant(_) => Err(ErrorKind::NotFound.into()),
        }
    }

    /// Safe wrapper to delete an event for `libc::epoll_ctl`
    pub fn ctl_del(&mut self, fd: RawFd) -> io::Result<Event<T>> {
        match self.datas.entry(fd) {
            Entry::Occupied(o) => {
                let event = null_mut() as *mut libc::epoll_event;
                let op = ControlOptions::EPOLL_CTL_DEL as i32;

                Self::cvt(unsafe { libc::epoll_ctl(self.fd, op, fd, event) })?;

                Ok(o.remove())
            }
            Entry::Vacant(_) => Err(ErrorKind::NotFound.into()),
        }
    }

    /// Safe wrapper for `libc::epoll_wait`
    /// The timeout argument is in milliseconds
    ///
    /// ## Notes
    ///
    /// * If `timeout` is negative, it will block until an event is received.
    pub fn wait(&mut self, timeout: i32) -> io::Result<&[Event<T>]> {
        let timeout = if timeout < 0 { -1 } else { timeout };

        let num_events = unsafe {
            Self::cvt(libc::epoll_wait(
                self.fd,
                self.buffer.as_mut_ptr() as *mut libc::epoll_event,
                self.buffer.capacity() as i32,
                timeout,
            ))? as usize
        };

        unsafe {
            self.buffer.set_len(num_events);
        }

        // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#method.slice_assume_init_ref
        Ok(unsafe {
            &*(self.buffer.as_slice() as *const _ as *const [Event<T>])
        })
    }

    /// This resize the buffer used to recieve event
    /// as epoll use a i32 we also take an i32 and convert it to usize
    /// this do at best
    pub fn resize_buffer(
        &mut self,
        max_events: i32,
    ) -> Result<(), TryFromIntError> {
        let max_events = max_events.try_into()?;
        self.buffer.resize_with(max_events, MaybeUninit::uninit);
        self.buffer.shrink_to_fit();

        Ok(())
    }

    /// Safe wrapper for `libc::close`
    /// this will return the datas
    /// For Event<Ptr<T>> only if you want to free ressource
    /// you will need to call `Event<Ptr<T>>::into_inner()`
    /// This could be improve if we could specialize Drop
    /// https://github.com/rust-lang/rust/issues/46893
    pub fn close(self) -> io::Result<HashMap<RawFd, Event<T>>> {
        Self::cvt(unsafe { libc::close(self.fd) })?;

        Ok(self.datas)
    }

    /// This return all data associed with this epoll fd
    /// You CAN'T modify direclt Event<T> the only thing you can modify
    /// is Event<Ptr<T>> because it's a reference
    /// if you want modify the direct value of Event<T>
    /// you will need to use `ctl_mod()`
    pub fn get_data(&self) -> &HashMap<RawFd, Event<T>> {
        &self.datas
    }

    fn cvt(result: libc::c_int) -> io::Result<libc::c_int> {
        if result < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {}
