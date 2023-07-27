use bitflags::bitflags;
use liburing_sys::{
    io_uring_cqe, io_uring_cqe_get_data64, IORING_CQE_F_BUFFER, IORING_CQE_F_MORE,
    IORING_CQE_F_NOTIF, IORING_CQE_F_SOCK_NONEMPTY,
};

bitflags! {
    /// Carries request-specific information
    pub struct CqeFlags: u32 {
        /// If set, the upper 16 bits of the flags field carries the buffer Id that was chosen for
        /// this request.
        const Buffer = IORING_CQE_F_BUFFER;
        /// If set, the application should expect more completions from the request.
        const More = IORING_CQE_F_MORE;
        /// If set, upon receiving the data from the socket in the current request, the socket
        /// still had data left on completion of this request.
        const SockNonEmpty = IORING_CQE_F_SOCK_NONEMPTY;
        /// Set for notification CQEs, as seen with the zero-copy networking send and receive
        /// support.
        const Notification = IORING_CQE_F_NOTIF;
    }
}

/// TODO
pub struct CompletionEntry<'a> {
    inner: &'a io_uring_cqe,
}

impl<'a> CompletionEntry<'a> {
    pub(crate) fn new(cqe: &'a io_uring_cqe) -> Self {
        Self { inner: cqe }
    }

    /// Retrieve the user_data set on the corresponding SQE request.
    pub fn get_user_data(&self) -> u64 {
        unsafe { io_uring_cqe_get_data64(self.inner) }
    }

    /// Get flags set on the CQE, which carry request-specific information.
    pub fn flags(&self) -> CqeFlags {
        CqeFlags::from_bits(self.inner.flags).expect("Supported flags are defined")
    }

    /// Get the result of the operation. This is equivalent to the return value of the syscall
    /// represented by the SQE request.
    pub fn result(&self) -> i32 {
        self.inner.res
    }
}
