use std::sync::atomic::AtomicU32;

use super::sqe::IoUringSqe;

#[repr(C)]
pub struct IoUringSq<'a, const N: usize> {
    khead: &'a AtomicU32,
    ktail: &'a AtomicU32,
    #[deprecated(note = "Use ring_mask instead")]
    kring_mask: &'a u32,
    #[deprecated(note = "Use ring_entries instead")]
    kring_entries: &'a u32,
    kflags: &'a u32,
    kdropped: &'a u32,
    array: &'a [u32; N],
    sqes: &'a [IoUringSqe; N],

    sqe_head: u32,
    sqe_tail: u32,

    ring_sz: usize,
    ring_ptr: *const libc::c_void,

    ring_mask: u32,
    ring_entries: u32,

    pad: [u32; 2],
}
