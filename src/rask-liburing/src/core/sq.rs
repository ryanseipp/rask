use std::sync::atomic::AtomicU32;

use super::sqe::IoUringSqe;

#[repr(C)]
pub struct IoUringSq<'a, const N: usize> {
    pub(crate) khead: &'a AtomicU32,
    pub(crate) ktail: &'a AtomicU32,
    #[deprecated(note = "Use ring_mask instead")]
    pub(crate) kring_mask: &'a u32,
    #[deprecated(note = "Use ring_entries instead")]
    pub(crate) kring_entries: &'a u32,
    pub(crate) kflags: &'a u32,
    pub(crate) kdropped: &'a u32,
    pub(crate) array: &'a [u32; N],
    pub(crate) sqes: &'a mut [IoUringSqe; N],

    pub(crate) sqe_head: u32,
    pub(crate) sqe_tail: u32,

    pub(crate) ring_sz: usize,
    pub(crate) ring_ptr: *const libc::c_void,

    pub(crate) ring_mask: u32,
    pub(crate) ring_entries: u32,

    pub(crate) pad: [u32; 2],
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, size_of};

    use crate::core::sq::IoUringSq;

    #[test]
    fn iouringsq_has_correct_layout() {
        assert_eq!(
            size_of::<IoUringSq<'_, 0>>(),
            104,
            "Size of: {}",
            size_of::<IoUringSq<'_, 0>>()
        );

        assert_eq!(
            size_of::<IoUringSq<'_, 1024>>(),
            104,
            "Size of: {}",
            size_of::<IoUringSq<'_, 1024>>()
        );

        assert_eq!(
            align_of::<IoUringSq<'_, 0>>(),
            8,
            "Alignment of: {}",
            align_of::<IoUringSq<'_, 0>>()
        );

        assert_eq!(
            align_of::<IoUringSq<'_, 1024>>(),
            8,
            "Alignment of: {}",
            align_of::<IoUringSq<'_, 1024>>()
        );
    }
}
