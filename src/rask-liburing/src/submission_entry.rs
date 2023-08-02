use std::{error::Error, fmt, os::fd::AsRawFd, ptr};

use rask_liburing_sys::{
    io_uring_prep_accept, io_uring_prep_close, io_uring_prep_multishot_accept, io_uring_prep_recv,
    io_uring_prep_send, io_uring_sqe, io_uring_sqe_set_data64,
};

/// Indicates the SubmissionQueue is full. Either the kernel needs to be notified of new SQEs, or
/// should be given time to process requests.
///
/// See [io_uring_enter(2)](https://man.archlinux.org/man/io_uring_enter.2)
#[derive(Debug)]
pub struct SQFullError;

impl Error for SQFullError {}

impl fmt::Display for SQFullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubmissionQueue is full. Notify the kernel of new entries via `io_uring_enter` or give it time to process requests.")
    }
}

/// TODO
pub struct SubmissionEntry<'a> {
    inner: &'a mut io_uring_sqe,
}

impl<'a> SubmissionEntry<'a> {
    pub(crate) fn new(sqe: &'a mut io_uring_sqe) -> Self {
        Self { inner: sqe }
    }

    /// Associate data with a SQE. This data can be retrieved from the corresponding CQE
    pub fn set_user_data(&mut self, data: u64) -> &mut Self {
        io_uring_sqe_set_data64(self.inner, data);
        self
    }

    /// Prepare the entry for an accept request.
    ///
    /// `fd` should be a file descriptor to a connection-based socket, i.e. [`TcpListener`]
    ///
    /// See [accept(2)](https://man.archlinux.org/man/accept.2)
    pub fn prep_accept(&mut self, fd: impl AsRawFd) -> &mut Self {
        io_uring_prep_accept(
            self.inner,
            fd.as_raw_fd(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        );

        self
    }

    /// Prepare the entry for an accept request.
    ///
    /// `fd` should be a file descriptor to a connection-based socket, i.e. [`TcpListener`]
    /// This is similar to `prep_accept`, except that this SQE will repeatedly trigger a CQE when
    /// a connection request is received. When a CQE is received, the flags should be checked for
    /// IORING_CQE_F_MORE to determine if this accept request will continue to produce CQEs. If
    /// the flag is not set, this should be called again.
    ///
    /// See [accept(2)](https://man.archlinux.org/man/accept.2) and [io_uring_prep_multishot_accept(3)](https://man.archlinux.org/man/io_uring_prep_multishot_accept.3)
    pub fn prep_accept_multi(&mut self, fd: &impl AsRawFd) -> &mut Self {
        io_uring_prep_multishot_accept(
            self.inner,
            fd.as_raw_fd(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
        );

        self
    }

    /// Prepare the entry for a receive request.
    ///
    /// The caller must guarantee `buffer` lives long enough to be used by the kernel, and when
    /// handling the corresponding CQE.
    ///
    /// See [recv(2)](https://man.archlinux.org/man/recv.2)
    pub fn prep_recv(&mut self, fd: impl AsRawFd, buffer: &mut [u8]) -> &mut Self {
        io_uring_prep_recv(self.inner, fd.as_raw_fd(), buffer, 0);

        self
    }

    /// Prepare the entry for a send request.
    ///
    /// The caller must guarantee `buffer` lives long enough to be used by the kernel.
    ///
    /// See [send(2)](https://man.archlinux.org/man/send.2)
    pub fn prep_send(&mut self, fd: impl AsRawFd, buffer: &[u8]) -> &mut Self {
        io_uring_prep_send(self.inner, fd.as_raw_fd(), buffer, 0);

        self
    }

    /// Prepare the entry for a close request.
    ///
    /// See [close(2)](https://man.archlinux.org/man/close.2)
    pub fn prep_close(&mut self, fd: impl AsRawFd) -> &mut Self {
        io_uring_prep_close(self.inner, fd.as_raw_fd());
        self
    }
}
