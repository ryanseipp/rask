#![deny(
    // missing_docs,
    // missing_debug_implementations,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! Rust binding for liburing

use std::{
    mem, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use libc::{c_void, mode_t, msghdr, sockaddr, socklen_t, timespec, uintptr_t};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const IO_URING_OP_SUPPORTED: u32 = 1u32 << 0;

// #[inline]
// unsafe fn io_uring_write_once<T>(p: *mut T, v: T) {
//     let mut v = v;
//     let p = AtomicPtr::from(p);
//     p.store(ptr::addr_of_mut!(v), Ordering::Relaxed);
// }

#[inline]
unsafe fn io_uring_read_once<T>(p: *mut T) -> T {
    let p = AtomicPtr::from(p);
    p.load(Ordering::Relaxed).read()
}

#[inline]
fn io_uring_smp_store_release<T>(p: *mut T, v: T) {
    let mut v = v;
    let p = AtomicPtr::from(p);
    p.store(ptr::addr_of_mut!(v), Ordering::Release);
}

#[inline]
unsafe fn io_uring_smp_load_acquire<T>(p: *mut T) -> T {
    let p = AtomicPtr::from(p);
    p.load(Ordering::Acquire).read()
}

/// # Safety
/// `p` must be a valid and initialized `io_uring_probe`
#[inline]
pub unsafe fn io_uring_opcode_supported(p: *const io_uring_probe, op: i32) -> i32 {
    if op as u8 > (*p).last_op {
        return 0;
    }

    ((*p).ops.1[op as usize].flags & IO_URING_OP_SUPPORTED as u16).into()
}

/// # Safety
/// Must be called after io_uring_for_each_cqe()
#[inline]
pub unsafe fn io_uring_cq_advance(ring: *mut io_uring, nr: u32) {
    if nr > 0 {
        let cq = &mut (*ring).cq;
        io_uring_smp_store_release(cq.khead, *cq.khead + nr);
    }
}

/// # Safety
/// Must be called after io_uring_{peek,wait}_cqe() after the cqe has been processed by the
/// application.
#[inline]
pub unsafe fn io_uring_cqe_seen(ring: *mut io_uring, cqe: *mut io_uring_cqe) {
    if !cqe.is_null() {
        io_uring_cq_advance(ring, 1);
    }
}

/// Associate pointer @data with the sqe, for later retrieval from the cqe at command completion
/// time with io_uring_cqe_get_data().
///
/// # Safety
/// `sqe` must reference a valid and initialized `io_uring_sqe`
/// Anything referenced by `data` must live long enough to be retrieved by `io_uring_cqe_get_data()`
#[inline]
pub unsafe fn io_uring_sqe_set_data(sqe: *mut io_uring_sqe, data: *mut c_void) {
    (*sqe).user_data = data as u64;
}

/// # Safety
/// `cqe` must reference a valid and initialized `io_uring_cqe`
/// User is responsible for ensuring anything referenced by user_data lived long enough
#[inline]
pub unsafe fn io_uring_cqe_get_data(cqe: *const io_uring_cqe) -> *mut c_void {
    (*cqe).user_data as *mut c_void
}

/// Assing a 64-bit value to this sqe, which can get retrieved at completion time with
/// `io_uring_cqe_get_data64`. Just like the non-64 variants, except these store a 64-bit type
/// rather than a data pointer.
///
/// # Safety
/// `sqe` must reference a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_sqe_set_data64(sqe: *mut io_uring_sqe, data: u64) {
    (*sqe).user_data = data;
}

/// # Safety
/// `cqe` must reference a valid and initialized `io_uring_cqe`
#[inline]
pub unsafe fn io_uring_cqe_get_data64(cqe: *const io_uring_cqe) -> u64 {
    (*cqe).user_data
}

/// # Safety
/// `sqe` must reference a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_sqe_set_flags(sqe: *mut io_uring_sqe, flags: u32) {
    (*sqe).flags = flags as u8
}

/// # Safety
/// `sqe` must reference a valid and initialized `io_uring_sqe`
/// `addr`, if non-null, must reference a valid buffer or iovecs
#[inline]
unsafe fn io_uring_prep_rw(
    op: u32,
    sqe: *mut io_uring_sqe,
    fd: i32,
    addr: *const c_void,
    len: u32,
    offset: u64,
) {
    (*sqe).opcode = op as u8;
    (*sqe).flags = 0;
    (*sqe).ioprio = 0;
    (*sqe).fd = fd;
    (*sqe).__bindgen_anon_1.off = offset;
    (*sqe).__bindgen_anon_2.addr = addr as u64;
    (*sqe).len = len;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_splice(
    sqe: *mut io_uring_sqe,
    fd_in: i32,
    off_in: i64,
    fd_out: i32,
    off_out: i64,
    nbytes: u32,
    splice_flags: u32,
) {
    io_uring_prep_rw(
        IORING_OP_SPLICE,
        sqe,
        fd_out,
        ptr::null(),
        nbytes,
        off_out as u64,
    );

    (*sqe).__bindgen_anon_2.splice_off_in = off_in as u64;
    (*sqe).__bindgen_anon_5.splice_fd_in = fd_in;
    (*sqe).__bindgen_anon_3.splice_flags = splice_flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_tee(
    sqe: *mut io_uring_sqe,
    fd_in: i32,
    fd_out: i32,
    nbytes: u32,
    splice_flags: u32,
) {
    io_uring_prep_rw(IORING_OP_TEE, sqe, fd_out, ptr::null(), nbytes, 0);

    (*sqe).__bindgen_anon_2.splice_off_in = 0;
    (*sqe).__bindgen_anon_5.splice_fd_in = fd_in;
    (*sqe).__bindgen_anon_3.splice_flags = splice_flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `iovecs` must point to an array of `iovec` with length `nr_vecs`
#[inline]
pub unsafe fn io_uring_prep_readv(
    sqe: *mut io_uring_sqe,
    fd: i32,
    iovecs: *const iovec,
    nr_vecs: u32,
    offset: u64,
) {
    io_uring_prep_rw(
        IORING_OP_READV,
        sqe,
        fd,
        mem::transmute(iovecs),
        nr_vecs,
        offset,
    )
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `iovecs` must point to an array of `iovec` with length `nr_vecs`
#[inline]
pub unsafe fn io_uring_prep_readv2(
    sqe: *mut io_uring_sqe,
    fd: i32,
    iovecs: *const iovec,
    nr_vecs: u32,
    offset: u64,
    flags: i32,
) {
    io_uring_prep_readv(sqe, fd, iovecs, nr_vecs, offset);
    (*sqe).__bindgen_anon_3.rw_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `buf` must point to a valid array with length at least `nbytes`
#[inline]
pub unsafe fn io_uring_prep_read_fixed(
    sqe: *mut io_uring_sqe,
    fd: i32,
    buf: *mut c_void,
    nbytes: u32,
    offset: u64,
    buf_index: i32,
) {
    io_uring_prep_rw(IORING_OP_READ_FIXED, sqe, fd, buf, nbytes, offset);
    (*sqe).__bindgen_anon_4.buf_index = buf_index as u16;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `iovecs` must point to an array of `iovec` with length `nr_vecs`
#[inline]
pub unsafe fn io_uring_prep_writev(
    sqe: *mut io_uring_sqe,
    fd: i32,
    iovecs: *const iovec,
    nr_vecs: u32,
    offset: u64,
) {
    io_uring_prep_rw(IORING_OP_WRITEV, sqe, fd, iovecs.cast(), nr_vecs, offset)
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `iovecs` must point to an array of `iovec` with length `nr_vecs`
#[inline]
pub unsafe fn io_uring_prep_writev2(
    sqe: *mut io_uring_sqe,
    fd: i32,
    iovecs: *const iovec,
    nr_vecs: u32,
    offset: u64,
    flags: i32,
) {
    io_uring_prep_writev(sqe, fd, iovecs, nr_vecs, offset);
    (*sqe).__bindgen_anon_3.rw_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `buf` must point to a valid array with length at least `nbytes`
#[inline]
pub unsafe fn io_uring_prep_write_fixed(
    sqe: *mut io_uring_sqe,
    fd: i32,
    buf: *mut c_void,
    nbytes: u32,
    offset: u64,
    buf_index: i32,
) {
    io_uring_prep_rw(IORING_OP_WRITE_FIXED, sqe, fd, buf, nbytes, offset);
    (*sqe).__bindgen_anon_4.buf_index = buf_index as u16;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `msg` must point to a valid and initialized `msghdr`
#[inline]
pub unsafe fn io_uring_prep_recvmsg(sqe: *mut io_uring_sqe, fd: i32, msg: *mut msghdr, flags: u32) {
    io_uring_prep_rw(IORING_OP_RECVMSG, sqe, fd, msg.cast(), 1, 0);
    (*sqe).__bindgen_anon_3.msg_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `msg` must point to a valid and initialized `msghdr`
#[inline]
pub unsafe fn io_uring_prep_recvmsg_multishot(
    sqe: *mut io_uring_sqe,
    fd: i32,
    msg: *mut msghdr,
    flags: u32,
) {
    io_uring_prep_recvmsg(sqe, fd, msg, flags);
    (*sqe).ioprio |= IORING_RECV_MULTISHOT as u16;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `msg` must point to a valid and initialized `msghdr`
#[inline]
pub unsafe fn io_uring_prep_sendmsg(
    sqe: *mut io_uring_sqe,
    fd: i32,
    msg: *const msghdr,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_SENDMSG, sqe, fd, msg.cast(), 1, 0);
    (*sqe).__bindgen_anon_3.msg_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_poll_add(sqe: *mut io_uring_sqe, fd: i32, poll_mask: u32) {
    io_uring_prep_rw(IORING_OP_POLL_ADD, sqe, fd, ptr::null(), 0, 0);
    (*sqe).__bindgen_anon_3.poll32_events = poll_mask.to_le();
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_poll_multishot(sqe: *mut io_uring_sqe, fd: i32, poll_mask: u32) {
    io_uring_prep_poll_add(sqe, fd, poll_mask);
    (*sqe).len = IORING_POLL_ADD_MULTI;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_poll_remove(sqe: *mut io_uring_sqe, user_data: u64) {
    io_uring_prep_rw(IORING_OP_POLL_REMOVE, sqe, -1, ptr::null(), 0, 0);
    (*sqe).__bindgen_anon_2.addr = user_data
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_poll_update(
    sqe: *mut io_uring_sqe,
    old_user_data: u64,
    new_user_data: u64,
    poll_mask: u32,
    flags: u32,
) {
    io_uring_prep_rw(
        IORING_OP_POLL_REMOVE,
        sqe,
        -1,
        ptr::null(),
        flags,
        new_user_data,
    );
    (*sqe).__bindgen_anon_2.addr = old_user_data;
    (*sqe).__bindgen_anon_3.poll32_events = poll_mask.to_le();
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_fsync(sqe: *mut io_uring_sqe, fd: i32, fsync_flags: u32) {
    io_uring_prep_rw(IORING_OP_FSYNC, sqe, fd, ptr::null(), 0, 0);
    (*sqe).__bindgen_anon_3.fsync_flags = fsync_flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_nop(sqe: *mut io_uring_sqe) {
    io_uring_prep_rw(IORING_OP_NOP, sqe, -1, ptr::null(), 0, 0);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `ts` must point to a valid and initialized `timespec`
#[inline]
pub unsafe fn io_uring_prep_timeout(
    sqe: *mut io_uring_sqe,
    ts: *mut timespec,
    count: u32,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_TIMEOUT, sqe, -1, ts.cast(), 1, count as u64);
    (*sqe).__bindgen_anon_3.timeout_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_timeout_remove(sqe: *mut io_uring_sqe, user_data: u64, flags: u32) {
    io_uring_prep_rw(IORING_OP_TIMEOUT_REMOVE, sqe, -1, ptr::null(), 0, 0);
    (*sqe).__bindgen_anon_2.addr = user_data;
    (*sqe).__bindgen_anon_3.timeout_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `ts` must point to a valid and initialized `timespec`
#[inline]
pub unsafe fn io_uring_prep_timeout_update(
    sqe: *mut io_uring_sqe,
    ts: *mut timespec,
    user_data: u64,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_TIMEOUT_REMOVE, sqe, -1, ptr::null(), 0, ts as u64);
    (*sqe).__bindgen_anon_2.addr = user_data;
    (*sqe).__bindgen_anon_3.timeout_flags = flags | IORING_TIMEOUT_UPDATE;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `addr` must point to a valid and initialized `sockaddr`
/// `addrlen` must point to a valid and initialized `socklen_t`
#[inline]
pub unsafe fn io_uring_prep_accept(
    sqe: *mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_ACCEPT, sqe, fd, addr.cast(), 0, addrlen as u64);
    (*sqe).__bindgen_anon_3.accept_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
unsafe fn io_uring_set_target_fixed_file(sqe: *mut io_uring_sqe, file_index: u32) {
    (*sqe).__bindgen_anon_5.file_index = file_index + 1;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `addr` must point to a valid and initialized `sockaddr`
/// `addrlen` must point to a valid and initialized `socklen_t`
#[inline]
pub unsafe fn io_uring_prep_accept_direct(
    sqe: *mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
    file_index: u32,
) {
    io_uring_prep_accept(sqe, fd, addr, addrlen, flags);
    let file_index = if file_index as i32 == IORING_FILE_INDEX_ALLOC {
        file_index - 1
    } else {
        file_index
    };

    io_uring_set_target_fixed_file(sqe, file_index);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `addr` must point to a valid and initialized `sockaddr`
/// `addrlen` must point to a valid and initialized `socklen_t`
#[inline]
pub unsafe fn io_uring_prep_multishot_accept(
    sqe: *mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
) {
    io_uring_prep_accept(sqe, fd, addr, addrlen, flags);
    (*sqe).ioprio |= IORING_ACCEPT_MULTISHOT as u16;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
/// `addr` must point to a valid and initialized `sockaddr`
/// `addrlen` must point to a valid and initialized `socklen_t`
#[inline]
pub unsafe fn io_uring_prep_multishot_accept_direct(
    sqe: *mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
) {
    io_uring_prep_multishot_accept(sqe, fd, addr, addrlen, flags);
    io_uring_set_target_fixed_file(sqe, (IORING_FILE_INDEX_ALLOC - 1) as u32);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_cancel64(sqe: *mut io_uring_sqe, user_data: u64, flags: i32) {
    io_uring_prep_rw(IORING_OP_ASYNC_CANCEL, sqe, -1, ptr::null(), 0, 0);
    (*sqe).__bindgen_anon_2.addr = user_data;
    (*sqe).__bindgen_anon_3.cancel_flags = flags as u32;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_cancel(sqe: *mut io_uring_sqe, user_data: *mut c_void, flags: i32) {
    io_uring_prep_cancel64(sqe, user_data as u64, flags);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_cancel_fd(sqe: *mut io_uring_sqe, fd: i32, flags: u32) {
    io_uring_prep_rw(IORING_OP_ASYNC_CANCEL, sqe, fd, ptr::null(), 0, 0);
    (*sqe).__bindgen_anon_3.cancel_flags = flags | IORING_ASYNC_CANCEL_FD;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_link_timeout(sqe: *mut io_uring_sqe, ts: *mut timespec, flags: u32) {
    io_uring_prep_rw(IORING_OP_LINK_TIMEOUT, sqe, -1, ts.cast(), 1, 0);
    (*sqe).__bindgen_anon_3.timeout_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_connect(
    sqe: *mut io_uring_sqe,
    fd: i32,
    addr: *const sockaddr,
    addrlen: socklen_t,
) {
    io_uring_prep_rw(IORING_OP_CONNECT, sqe, fd, addr.cast(), 0, addrlen as u64);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_files_update(
    sqe: *mut io_uring_sqe,
    fds: *mut i32,
    nr_fds: u32,
    offset: i32,
) {
    io_uring_prep_rw(
        IORING_OP_FILES_UPDATE,
        sqe,
        -1,
        fds.cast(),
        nr_fds,
        offset as u64,
    );
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_fallocate(
    sqe: *mut io_uring_sqe,
    fd: i32,
    mode: i32,
    offset: u64,
    len: u64,
) {
    io_uring_prep_rw(
        IORING_OP_FALLOCATE,
        sqe,
        fd,
        ptr::null(),
        mode as u32,
        offset,
    );
    (*sqe).__bindgen_anon_2.addr = len;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_openat(
    sqe: *mut io_uring_sqe,
    dfd: i32,
    path: *const char,
    flags: i32,
    mode: mode_t,
) {
    io_uring_prep_rw(IORING_OP_OPENAT, sqe, dfd, path.cast(), mode, 0);
    (*sqe).__bindgen_anon_3.open_flags = flags as u32;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_openat_direct(
    sqe: *mut io_uring_sqe,
    dfd: i32,
    path: *const char,
    flags: i32,
    mode: mode_t,
    file_index: u32,
) {
    io_uring_prep_openat(sqe, dfd, path, flags, mode);
    let file_index = if file_index as i32 == IORING_FILE_INDEX_ALLOC {
        file_index - 1
    } else {
        file_index
    };

    io_uring_set_target_fixed_file(sqe, file_index);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_close(sqe: *mut io_uring_sqe, fd: i32) {
    io_uring_prep_rw(IORING_OP_CLOSE, sqe, fd, ptr::null(), 0, 0);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_close_direct(sqe: *mut io_uring_sqe, file_index: u32) {
    io_uring_prep_close(sqe, 0);
    io_uring_set_target_fixed_file(sqe, file_index);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_read(
    sqe: *mut io_uring_sqe,
    fd: i32,
    buf: *mut c_void,
    nbytes: u32,
    offset: u64,
) {
    io_uring_prep_rw(IORING_OP_READ, sqe, fd, buf, nbytes, offset);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_write(
    sqe: *mut io_uring_sqe,
    fd: i32,
    buf: *mut c_void,
    nbytes: u32,
    offset: u64,
) {
    io_uring_prep_rw(IORING_OP_WRITE, sqe, fd, buf, nbytes, offset);
}

// TODO: statx fadvise madvise

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_send(
    sqe: *mut io_uring_sqe,
    sockfd: i32,
    buf: *const c_void,
    len: usize,
    flags: i32,
) {
    io_uring_prep_rw(IORING_OP_SEND, sqe, sockfd, buf, len as u32, 0);
    (*sqe).__bindgen_anon_3.msg_flags = flags as u32;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_send_set_addr(
    sqe: *mut io_uring_sqe,
    dest_addr: *const sockaddr,
    addr_len: u16,
) {
    (*sqe).__bindgen_anon_1.addr2 = dest_addr as u64;
    (*sqe).__bindgen_anon_5.__bindgen_anon_1.addr_len = addr_len;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_sendto(
    sqe: *mut io_uring_sqe,
    sockfd: i32,
    buf: *const c_void,
    len: usize,
    flags: i32,
    addr: *const sockaddr,
    addr_len: u16,
) {
    io_uring_prep_send(sqe, sockfd, buf, len, flags);
    io_uring_prep_send_set_addr(sqe, addr, addr_len);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_send_zc(
    sqe: *mut io_uring_sqe,
    sockfd: i32,
    buf: *const c_void,
    len: usize,
    flags: i32,
    zc_flags: u32,
) {
    io_uring_prep_rw(IORING_OP_SEND_ZC, sqe, sockfd, buf, len as u32, 0);
    (*sqe).__bindgen_anon_3.msg_flags = flags as u32;
    (*sqe).ioprio = zc_flags as u16;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_send_zc_fixed(
    sqe: *mut io_uring_sqe,
    sockfd: i32,
    buf: *const c_void,
    len: usize,
    flags: i32,
    zc_flags: u32,
    buf_index: u32,
) {
    io_uring_prep_send_zc(sqe, sockfd, buf, len, flags, zc_flags);
    (*sqe).ioprio |= IORING_RECVSEND_FIXED_BUF as u16;
    (*sqe).__bindgen_anon_4.buf_index = buf_index as u16;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_sendmsg_zc(
    sqe: *mut io_uring_sqe,
    fd: i32,
    msg: *const msghdr,
    flags: u32,
) {
    io_uring_prep_sendmsg(sqe, fd, msg, flags);
    (*sqe).opcode = IORING_OP_SENDMSG_ZC as u8;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_recv(
    sqe: *mut io_uring_sqe,
    sockfd: i32,
    buf: *mut c_void,
    len: usize,
    flags: i32,
) {
    io_uring_prep_rw(IORING_OP_RECV, sqe, sockfd, buf, len as u32, 0);
    (*sqe).__bindgen_anon_3.msg_flags = flags as u32;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_recv_multishot(
    sqe: *mut io_uring_sqe,
    sockfd: i32,
    buf: *mut c_void,
    len: usize,
    flags: i32,
) {
    io_uring_prep_recv(sqe, sockfd, buf, len, flags);
    (*sqe).ioprio |= IORING_RECV_MULTISHOT as u16;
}

// TODO: recvmsg helpers, openat2, epollctl, provide_buffers, remove_buffers, shutdown, unlink,
// rename, sync_file_range, mkdir, symlink, link

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_msg_ring_cqe_flags(
    sqe: *mut io_uring_sqe,
    fd: i32,
    len: u32,
    data: u64,
    flags: u32,
    cqe_flags: u32,
) {
    io_uring_prep_rw(IORING_OP_MSG_RING, sqe, fd, ptr::null(), len, data);
    (*sqe).__bindgen_anon_3.msg_ring_flags = IORING_MSG_RING_FLAGS_PASS | flags;
    (*sqe).__bindgen_anon_5.file_index = cqe_flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_msg_ring(
    sqe: *mut io_uring_sqe,
    fd: i32,
    len: u32,
    data: u64,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_MSG_RING, sqe, fd, ptr::null(), len, data);
    (*sqe).__bindgen_anon_3.msg_ring_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_msg_ring_fd(
    sqe: *mut io_uring_sqe,
    fd: i32,
    source_fd: i32,
    target_fd: i32,
    data: u64,
    flags: u32,
) {
    io_uring_prep_rw(
        IORING_OP_MSG_RING,
        sqe,
        fd,
        IORING_MSG_SEND_FD as uintptr_t as *mut c_void,
        0,
        data,
    );
    (*sqe).__bindgen_anon_6.__bindgen_anon_1.as_mut().addr3 = source_fd as u64;
    let target_fd = if target_fd == IORING_FILE_INDEX_ALLOC {
        target_fd - 1
    } else {
        target_fd
    };
    io_uring_set_target_fixed_file(sqe, target_fd as u32);
    (*sqe).__bindgen_anon_3.msg_ring_flags = flags;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_msg_ring_fd_alloc(
    sqe: *mut io_uring_sqe,
    fd: i32,
    source_fd: i32,
    data: u64,
    flags: u32,
) {
    io_uring_prep_msg_ring_fd(sqe, fd, source_fd, IORING_FILE_INDEX_ALLOC, data, flags);
}

// TODO: xattr

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_socket(
    sqe: *mut io_uring_sqe,
    domain: i32,
    r#type: i32,
    protocol: i32,
    flags: u32,
) {
    io_uring_prep_rw(
        IORING_OP_SOCKET,
        sqe,
        domain,
        ptr::null(),
        protocol as u32,
        r#type as u64,
    );
    (*sqe).__bindgen_anon_3.rw_flags = flags as i32;
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_socket_direct(
    sqe: *mut io_uring_sqe,
    domain: i32,
    r#type: i32,
    protocol: i32,
    file_index: u32,
    flags: u32,
) {
    io_uring_prep_socket(sqe, domain, r#type, protocol, flags);
    let file_index = if file_index as i32 == IORING_FILE_INDEX_ALLOC {
        file_index - 1
    } else {
        file_index
    };
    io_uring_set_target_fixed_file(sqe, file_index);
}

/// # Safety
/// `sqe` must point to a valid and initialized `io_uring_sqe`
#[inline]
pub unsafe fn io_uring_prep_socket_direct_alloc(
    sqe: *mut io_uring_sqe,
    domain: i32,
    r#type: i32,
    protocol: i32,
    flags: u32,
) {
    io_uring_prep_socket(sqe, domain, r#type, protocol, flags);
    io_uring_set_target_fixed_file(sqe, (IORING_FILE_INDEX_ALLOC - 1) as u32);
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_sq_ready(ring: *const io_uring) -> u32 {
    let mut khead: u32 = (*ring).sq.khead.read_volatile();

    if (*ring).flags & IORING_SETUP_SQPOLL == 0 {
        khead = io_uring_smp_load_acquire((*ring).sq.khead);
    }

    (*ring).sq.sqe_tail - khead
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_sq_space_left(ring: *const io_uring) -> u32 {
    (*ring).sq.ring_entries - io_uring_sq_ready(ring)
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_sqring_wait(ring: *mut io_uring) -> i32 {
    if (*ring).flags & IORING_SETUP_SQPOLL == 0 || io_uring_sq_space_left(ring) > 0 {
        0
    } else {
        __io_uring_sqring_wait(ring)
    }
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_cq_ready(ring: *const io_uring) -> u32 {
    io_uring_smp_load_acquire((*ring).cq.ktail) - *(*ring).cq.khead
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_cq_has_overflown(ring: *const io_uring) -> bool {
    io_uring_read_once((*ring).sq.kflags) & IORING_SQ_CQ_OVERFLOW > 0
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_wait_cqe_nr(
    ring: *mut io_uring,
    cqe_ptr: *mut *mut io_uring_cqe,
    wait_nr: u32,
) -> i32 {
    __io_uring_get_cqe(ring, cqe_ptr, 0, wait_nr, ptr::null_mut())
}

const LIBURING_UDATA_TIMEOUT: u64 = u64::MAX;

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
unsafe fn io_uring_peek_cqe_internal(
    ring: *mut io_uring,
    cqe_ptr: *mut *mut io_uring_cqe,
    nr_available: *mut u32,
) -> i32 {
    let mut cqe: *mut io_uring_cqe;
    let mut err: i32 = 0;
    let mut available;
    let mask = (*ring).cq.ring_mask;

    let shift: i32 = if (*ring).flags & IORING_SETUP_CQE32 > 0 {
        1
    } else {
        0
    };

    loop {
        let tail = io_uring_smp_load_acquire((*ring).cq.ktail);
        let head = (*ring).cq.khead.read();

        cqe = ptr::null_mut();
        available = tail - head;

        if available == 0 {
            break;
        }

        cqe = (*ring).cq.cqes.offset(((head & mask) << shift) as isize);
        if (*ring).features & IORING_FEAT_EXT_ARG == 0 && (*cqe).user_data == LIBURING_UDATA_TIMEOUT
        {
            if (*cqe).res < 0 {
                err = (*cqe).res;
            }
            io_uring_cq_advance(ring, 1);

            if err == 0 {
                continue;
            }
            cqe = ptr::null_mut();
        }

        break;
    }

    *cqe_ptr = cqe;
    if !nr_available.is_null() {
        *nr_available = available;
    }

    err
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_peek_cqe(ring: *mut io_uring, cqe_ptr: *mut *mut io_uring_cqe) -> i32 {
    if io_uring_peek_cqe_internal(ring, cqe_ptr, ptr::null_mut()) == 0 && !cqe_ptr.is_null() {
        return 0;
    }

    io_uring_wait_cqe_nr(ring, cqe_ptr, 0)
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_wait_cqe(ring: *mut io_uring, cqe_ptr: *mut *mut io_uring_cqe) -> i32 {
    if io_uring_peek_cqe_internal(ring, cqe_ptr, ptr::null_mut()) == 0 && !cqe_ptr.is_null() {
        return 0;
    }

    io_uring_wait_cqe_nr(ring, cqe_ptr, 1)
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_get_sqe(ring: *mut io_uring) -> *mut io_uring_sqe {
    let sq: *mut io_uring_sq = ptr::addr_of_mut!((*ring).sq);
    let next = (*sq).sqe_tail + 1;
    let mut shift = 0;

    if (*ring).flags & IORING_SETUP_SQE128 > 0 {
        shift = 1;
    }
    let head = if (*ring).flags & IORING_SETUP_SQPOLL == 0 {
        io_uring_read_once((*sq).khead)
    } else {
        io_uring_smp_load_acquire((*sq).khead)
    };

    if next - head <= (*sq).ring_entries {
        let sqe = (*sq)
            .sqes
            .offset((((*sq).sqe_tail & (*sq).ring_mask) << shift) as isize);

        (*sq).sqe_tail = next;

        return sqe;
    }

    ptr::null_mut()
}

pub fn io_uring_buf_ring_mask(ring_entries: u32) -> i32 {
    (ring_entries - 1) as i32
}

/// # Safety
/// `br` must point to a valid and initialized `io_uring_buf_ring`
#[inline]
pub unsafe fn io_uring_buf_ring_init(br: *mut io_uring_buf_ring) {
    (*br).__bindgen_anon_1.__bindgen_anon_1.as_mut().tail = 0;
}

/// # Safety
/// `br` must point to a valid and initialized `io_uring_buf_ring`
#[inline]
pub unsafe fn io_uring_buf_ring_add(
    br: *mut io_uring_buf_ring,
    addr: *mut c_void,
    len: u32,
    bid: u16,
    mask: i32,
    buf_offset: i32,
) {
    let buf: *mut io_uring_buf = ptr::addr_of_mut!(
        (*br).__bindgen_anon_1.bufs.as_mut()[(((*br).__bindgen_anon_1.__bindgen_anon_1.as_ref().tail
            as i32
            + buf_offset)
            & mask) as usize]
    );

    (*buf).addr = addr as u64;
    (*buf).len = len;
    (*buf).bid = bid;
}

/// # Safety
/// `br` must point to a valid and initialized `io_uring_buf_ring`
#[inline]
pub unsafe fn io_uring_buf_ring_advance(br: *mut io_uring_buf_ring, count: i32) {
    let new_tail = (*br).__bindgen_anon_1.__bindgen_anon_1.as_ref().tail + count as u16;
    io_uring_smp_store_release(
        ptr::addr_of_mut!((*br).__bindgen_anon_1.__bindgen_anon_1.as_mut().tail),
        new_tail,
    );
}

/// # Safety
/// `br` must point to a valid and initialized `io_uring_buf_ring`
#[inline]
unsafe fn io_uring_buf_ring_cq_advance_internal(
    ring: *mut io_uring,
    br: *mut io_uring_buf_ring,
    cq_count: i32,
    buf_count: i32,
) {
    (*br).__bindgen_anon_1.__bindgen_anon_1.as_mut().tail += buf_count as u16;
    io_uring_cq_advance(ring, cq_count as u32);
}

/// # Safety
/// `br` must point to a valid and initialized `io_uring_buf_ring`
#[inline]
pub unsafe fn io_uring_buf_ring_cq_advance(
    ring: *mut io_uring,
    br: *mut io_uring_buf_ring,
    count: i32,
) {
    io_uring_buf_ring_cq_advance_internal(ring, br, count, count)
}
