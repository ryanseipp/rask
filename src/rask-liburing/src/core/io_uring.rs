use std::{os::fd::RawFd, sync::atomic::Ordering};

use super::{
    cq::IoUringCq, /* cqe::IoUringCqe, */ sq::IoUringSq, sqe::IoUringSqe, IORING_SETUP_SQE128,
    IORING_SETUP_SQPOLL,
};

#[repr(C)]
pub struct IoUring<'a, const N: usize> {
    sq: IoUringSq<'a, N>,
    cq: IoUringCq<'a, N>,
    flags: u32,
    ring_fd: RawFd,
    features: u32,
    enter_ring_fd: RawFd,
    int_flags: u8,
    pad: [u8; 3],
    pad2: u32,
}

impl<'a, const N: usize> IoUring<'a, N> {
    /// Obtain the next available SQL to prepare an IO request
    ///
    /// If [`Option::None`] is returned, the submission queue is full and entries must be submitted to the
    /// kernel for processing before new ones can be prepared.
    pub fn get_sqe(&mut self) -> Option<&mut IoUringSqe> {
        let next = self.sq.sqe_tail + 1;
        let mut shift = 0;

        if self.flags & IORING_SETUP_SQE128 > 0 {
            shift = 1;
        }

        let head = if self.flags & IORING_SETUP_SQPOLL == 0 {
            self.sq.khead.load(Ordering::Relaxed)
        } else {
            self.sq.khead.load(Ordering::Acquire)
        };

        if next - head <= self.sq.ring_entries {
            let sqe = &mut self.sq.sqes[((self.sq.sqe_tail & self.sq.ring_mask) << shift) as usize];
            self.sq.sqe_tail = next;

            return Some(sqe);
        }

        None
    }

    // pub fn wait_cqe_nr(&mut self, wait: u32) -> &'a [IoUringCqe] {}
    //
    // pub fn wait_cqe(&mut self) -> &'a [IoUringCqe] {}
    //
    // pub fn peek_cqe(&mut self) -> &'a [IoUringCqe] {}
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    use crate::core::io_uring::IoUring;

    #[test]
    fn iouring_has_correct_layout() {
        assert_eq!(
            size_of::<IoUring<'_, 0>>(),
            216,
            "Size of: {}",
            size_of::<IoUring<'_, 0>>()
        );

        assert_eq!(
            size_of::<IoUring<'_, 1024>>(),
            216,
            "Size of: {}",
            size_of::<IoUring<'_, 1024>>()
        );

        assert_eq!(
            align_of::<IoUring<'_, 0>>(),
            8,
            "Alignment of: {}",
            align_of::<IoUring<'_, 0>>()
        );

        assert_eq!(
            align_of::<IoUring<'_, 1024>>(),
            8,
            "Alignment of: {}",
            align_of::<IoUring<'_, 1024>>()
        );
    }
}
