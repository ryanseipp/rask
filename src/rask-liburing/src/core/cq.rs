use std::sync::atomic::{AtomicU32, Ordering};

use super::{cqe::IoUringCqe, IORING_SQ_CQ_OVERFLOW};

#[repr(C)]
pub struct IoUringCq<'a, const N: usize> {
    khead: &'a AtomicU32,
    ktail: &'a AtomicU32,
    #[deprecated(note = "Use ring_mask instead")]
    kring_mask: &'a u32,
    #[deprecated(note = "Use ring_entries instead")]
    kring_entries: &'a u32,
    kflags: &'a AtomicU32,
    koverflow: &'a u32,
    cqes: &'a [IoUringCqe; N],

    ring_sz: usize,
    ring_ptr: *const libc::c_void,

    ring_mask: u32,
    ring_entries: u32,

    pad: [u32; 2],
}

impl<'a, const N: usize> IoUringCq<'a, N> {
    /// Gets the number of unconsumed, ready entries
    pub fn ready(&self) -> u32 {
        self.ktail.load(Ordering::Acquire) - self.khead.load(Ordering::Relaxed)
    }

    /// Determines if ready entries have been dropped due to a full CQ
    pub fn has_overflown(&self) -> bool {
        self.kflags.load(Ordering::Relaxed) & IORING_SQ_CQ_OVERFLOW > 0
    }
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    use crate::core::cq::IoUringCq;

    #[test]
    fn iouringcq_has_correct_layout() {
        assert_eq!(
            size_of::<IoUringCq<'_, 0>>(),
            88,
            "Size of: {}",
            size_of::<IoUringCq<'_, 0>>()
        );

        assert_eq!(
            size_of::<IoUringCq<'_, 1024>>(),
            88,
            "Size of: {}",
            size_of::<IoUringCq<'_, 1024>>()
        );

        assert_eq!(
            align_of::<IoUringCq<'_, 0>>(),
            8,
            "Alignment of: {}",
            align_of::<IoUringCq<'_, 0>>()
        );

        assert_eq!(
            align_of::<IoUringCq<'_, 1024>>(),
            8,
            "Alignment of: {}",
            align_of::<IoUringCq<'_, 1024>>()
        );
    }
}
