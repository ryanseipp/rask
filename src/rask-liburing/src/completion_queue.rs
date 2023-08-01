use std::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use rask_liburing_sys::io_uring_cq;

use crate::CompletionEntry;

/// An iterator over CQEs, automatically advancing the ring buffer when iteration completes
pub struct CompletionQueue<'a> {
    inner: &'a io_uring_cq,
    count: usize,
    head: u32,
}

impl<'a> CompletionQueue<'a> {
    pub(crate) fn new(cq: &'a io_uring_cq) -> Self {
        let head = AtomicPtr::from(cq.khead);
        Self {
            inner: cq,
            count: 0,
            head: unsafe { head.load(Ordering::Acquire).read() },
        }
    }
}

impl<'a> Iterator for CompletionQueue<'a> {
    type Item = CompletionEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let tail = AtomicPtr::from(self.inner.ktail);
        if self.head == unsafe { tail.load(Ordering::Acquire).read() } {
            None
        } else {
            let index = (self.head & self.inner.ring_mask) << 1;
            Some(CompletionEntry::new(unsafe {
                self.inner.cqes.offset(index as isize).as_ref().unwrap()
            }))
        }

        // let item = ptr::null_mut();
        // match unsafe { io_uring_peek_cqe(self.inner, item) } {
        //     0 => None,
        //     _ => {
        //         self.count += 1;
        //         unsafe { item.read().as_ref().map(CompletionEntry::new) }
        //     }
        // }
    }
}

impl Drop for CompletionQueue<'_> {
    fn drop(&mut self) {
        if self.count > 0 {
            let khead = AtomicPtr::from(self.inner.khead);
            let mut value = unsafe { *self.inner.khead + self.count as u32 };
            khead.store(ptr::addr_of_mut!(value), Ordering::Release);
        }
    }
}
