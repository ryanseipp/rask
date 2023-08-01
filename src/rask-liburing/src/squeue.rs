use std::sync::atomic::AtomicU32;

use rask_liburing_sys::io_uring_sqe;

pub struct SubmissionQueue {
    khead: *const AtomicU32,
    ktail: *const AtomicU32,
    kdropped: *const AtomicU32,
    sqes: *const io_uring_sqe,
    ring_mask: u32,
    ring_entries: u32,
}
