#![deny(
    missing_docs,
    // missing_debug_implementations,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
// temporary
#![allow(dead_code)]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

//! An idiomatic Rust wrapper around liburing-sys

mod completion_entry;
pub use completion_entry::*;

mod submission_entry;
pub use submission_entry::*;

use std::{io, mem::MaybeUninit, ptr};

use liburing_sys::{
    io_uring, io_uring_enter, io_uring_get_sqe, io_uring_queue_exit, io_uring_queue_init,
    io_uring_sq,
};

/// io_uring is a Linux-specific API for asynchronous I/O. It allows the user to submit one or more I/O requests,
/// which are processed asynchronously without blocking the calling process. io_uring gets its name from ring
/// buffers which are shared between user space and kernel space. This arrangement allows for efficient I/O, while
/// avoiding the overhead of copying buffers between them, where possible. This interface makes io_uring different
/// from other UNIX I/O APIs, wherein, rather than just communicate between kernel and user space with system calls,
/// ring buffers are used as the main mode of communication.
#[derive(Debug)]
pub struct IoUring {
    inner: io_uring,
    unsubmitted: u32,
    in_flight: u32,
}

impl IoUring {
    /// Initializes a new io_uring instance, including the SQ and CQ ring buffers.
    pub fn new(entries: u32) -> io::Result<Self> {
        let mut ring: MaybeUninit<io_uring> = MaybeUninit::zeroed();
        let res = unsafe { io_uring_queue_init(entries, ring.as_mut_ptr(), 0) };
        let ring = unsafe { ring.assume_init() };

        if res < 0 {
            return Err(io::Error::from_raw_os_error(-res));
        }

        Ok(Self {
            inner: ring,
            unsubmitted: 0,
            in_flight: 0,
        })
    }

    /// Get the next available submission queue entry from the submission queue belonging to this ring.
    pub fn get_sqe(&mut self) -> Result<SubmissionEntry<'_>, SQFullError> {
        let entry = unsafe { io_uring_get_sqe(ptr::addr_of_mut!(self.inner)) };
        let entry = unsafe { entry.as_mut() }.ok_or(SQFullError)?;

        self.unsubmitted += 1;

        Ok(SubmissionEntry::new(entry))
    }

    /// Informs the kernel of new SQEs, but waits for no CQEs before continuing.
    ///
    /// Returns the number of SQEs that were successfully submitted.
    ///
    /// See [io_uring_enter(2)](https://man.archlinux.org/man/io_uring_enter.2)
    pub fn enter(&mut self) -> io::Result<u32> {
        self.enter_and_wait(0)
    }

    /// Informs the kernel of new SQEs, and waits for the given number of CQEs
    ///
    /// Returns the number of SQEs that were successfully submitted.
    ///
    /// See [io_uring_enter(2)](https://man.archlinux.org/man/io_uring_enter.2)
    pub fn enter_and_wait(&mut self, wait_for: u32) -> io::Result<u32> {
        let accepted = unsafe {
            io_uring_enter(
                self.inner.ring_fd as u32,
                self.unsubmitted,
                wait_for,
                0,
                ptr::null_mut(),
            )
        };

        if accepted < 0 {
            return Err(io::Error::last_os_error());
        }

        let accepted = accepted as u32;
        self.unsubmitted -= accepted;
        self.in_flight += accepted;

        Ok(accepted)
    }
}

impl Drop for IoUring {
    fn drop(&mut self) {
        unsafe { io_uring_queue_exit(ptr::addr_of_mut!(self.inner)) }
    }
}

/// TODO
#[derive(Debug)]
pub struct SubmissionQueue<'a> {
    inner: &'a io_uring_sq,
}

impl<'a> SubmissionQueue<'a> {
    fn new(sq: &'a io_uring_sq) -> Self {
        Self { inner: sq }
    }
}
