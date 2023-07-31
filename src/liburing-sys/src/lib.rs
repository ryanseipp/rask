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
    ptr,
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use libc::{c_void, mode_t, msghdr, sockaddr, socklen_t, timespec, uintptr_t};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const IO_URING_OP_SUPPORTED: u32 = 1u32 << 0;

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
// #[inline]
// pub unsafe fn io_uring_opcode_supported(p: *const io_uring_probe, op: i32) -> i32 {
//     if op as u8 > (*p).last_op {
//         return 0;
//     }
//
//     ((*p).ops.1[op as usize].flags & IO_URING_OP_SUPPORTED as u16).into()
// }

#[inline]
pub fn io_uring_opcode_supported(probe: &io_uring_probe, op: i32) -> i32 {
    if op as u8 > probe.last_op {
        return 0;
    }

    (probe.ops.1[op as usize].flags & IO_URING_OP_SUPPORTED as u16).into()
}

/// Marks `seen` IO completions belonging to CQ as consumed.
///
/// After the caller has submitted a request with [`io_uring_submit()`], the application can
/// retrieve the complition with [`io_uring_wait_cqe()`], [`io_uring_peek_cqe()`], or any of the
/// other CQE retrieval helpers, and mark it as consumed with [`io_uring_cqe_seen()`]. The function
/// `io_uring_cqe_seen` calls this function.
///
/// Completions must be marked as seen, so that their slot can be reused. Failure to do so will
/// result in the same completion being returned on the next invocation.
///
/// Must be called after [`io_uring_for_each_cqe()`].
#[inline]
pub fn io_uring_cq_advance(cq: &mut io_uring_cq, seen: u32) {
    unsafe {
        if seen > 0 {
            let head = *cq.khead + seen;
            let khead: *mut AtomicU32 = cq.khead.cast();
            (*khead).store(head, Ordering::Release);
        }
    }
}

/// Marks the IO completion `cqe` as consumed.
///
/// After the caller has submitted a request with [`io_uring_submit()`], the application can
/// retrieve the complition with [`io_uring_wait_cqe()`], [`io_uring_peek_cqe()`], or any of the
/// other CQE retrieval helpers, and mark it as consumed with [`io_uring_cqe_seen()`].
///
/// Completions must be marked as completed, so that their slot can be reused.
///
/// Must be called after [`io_uring_peek_cqe()`] or [`io_uring_wait_cqe()`] after the cqe has been
/// processed by the application.
#[inline]
pub fn io_uring_cqe_seen(cq: &mut io_uring_cq, _cqe: io_uring_cqe) {
    io_uring_cq_advance(cq, 1);
}

/// Stores a pointer with the submission queue entry.
///
/// After the caller has requested a submission queue entry (SQE) with [`io_uring_get_sqe`], they
/// can associate a data pointer or value with the SQE. Once the completion arrives, the function
/// [`io_uring_cqe_get_data`] can be called to retrieve the data pointer or value associated with
/// the submitted request.
#[inline]
pub fn io_uring_sqe_set_data<T>(sqe: &mut io_uring_sqe, data: *mut T) {
    sqe.user_data = data as u64;
}

/// Returns the [`user_data`](io_uring_cqe::user_data) with the completion queue entry as a data
/// pointer.
///
/// After the caller has received a cempletion queue entry (CQE) with [`io_uring_wait_cqe()`], the
/// application can call [`io_uring_cqe_get_data()`] to retrieve the user_data value.
///
/// # Safety
/// Requires that [`user_data`](io_uring_cqe::user_data) must have been set earlier with
/// [`io_uring_sqe_set_data()`]. Otherwise the return value is undefined. The caller is responsible
/// for using the same type `T` with [`io_uring_sqe_set_data()`] and this function.
#[inline]
pub unsafe fn io_uring_cqe_get_data<T>(cqe: &io_uring_cqe) -> *mut T {
    cqe.user_data as *mut T
}

/// Stores a 64-bit data value with the submission queue entry.
///
/// After the caller has requested a submission queue entry (SQE) with [`io_uring_get_sqe()`], they
/// can associate a data pointer or value with the SQE. Once the completion arrives, the function
/// [`io_uring_cqe_get_data64()`] can be called to retrieve the data pointer or value associated
/// with the submitted request.
#[inline]
pub fn io_uring_sqe_set_data64(sqe: &mut io_uring_sqe, data: u64) {
    sqe.user_data = data;
}

/// Returns the [`user_data`](io_uring_cqe::user_data) with the completion queue entry as a 64-bit
/// value.
///
/// After the caller has received a completion queue entry (CQE) with [`io_uring_wait_cqe()`], the
/// application can all [`io_uring_cqe_get_data64()`] to retrieve the `user_data` value.
///
/// # Safety
/// Requires that [`user_data`](io_uring_cqe::user_data) has been set earlier with the function
/// [`io_uring_sqe_set_data64()`]. Otherwise, the return value is undefined.
#[inline]
pub unsafe fn io_uring_cqe_get_data64(cqe: *const io_uring_cqe) -> u64 {
    (*cqe).user_data
}

/// Allows the caller to change the behavior of the submission queue entry by specifying flags. It
/// enables the flags beloning to the sqe submission queue entry param.
///
/// `flags` is a bit mask of 0 or more of the following values ORed together:
/// * [`IOSQE_FIXED_FILE`]
///   * The file descriptor in the SQE refers to the index of a previously registered file or direct
///   file descriptor, not a normal file descriptor.
/// * [`IOSQE_ASYNC`]
///   * Normal operation for io_uring is to try and issue an sqe as non-blocking first, and if that
///   fails, execute it in an async manner.
///   * To support more efficient overlapped operation of requests that the application
///   knows/assumes will always (or most of the time) block, the application can ask for an sqe to
///   be issued async from the start.
///   * Note that this flag immediately causes the SQE to be offloaded to an async helper thread
///   with no internal non-blocking attempt. This may be less efficient and should not be used
///   liberally or without understanding the performance and efficiency tradeoffs.
/// * [`IOSQE_IO_LINK`]
///   * When this flag is specified, the SQE forms a link with the next SQE in the submission ring.
///   That next SQE will not be started before the previous request completes. This, in effect,
///   forms a chain of SQEs, which can be arbitrarily long.
///   * The tail of the chain is denoted by the first SQE that does not have this flag set.
///   * Chains are not supported across submission boundaries.
///   * Even if the last SQE in a submission has this flag set, it will still terminate the current
///   chain.
///   * This flag has no effect on previous SQE submissions, nor does it impact SQEs that are
///   outside of the chain tail. This means that multiple chains can be executing in parallel, or
///   chains and individual SQEs.
///   * Only members inside the chain are serialized. A chain of SQEs will be broken if any request
///   in that chain ends in error.
/// * [`IOSQE_IO_HARDLINK`]
///   * Like [`IOSQE_IO_LINK`], except the links aren't severed if an error or unexpected result
///   occurs.
/// * [`IOSQE_IO_DRAIN`]
///   * When this flag is specified, the SQE will not be started before previously submitted SQEs
///   have completed, and new SQEs will not be started before this one completes.
/// * [`IOSQE_CQE_SKIP_SUCCESS`]
///   * Request that no CQE be generated for this request, if it completes successfully. This can
///   be useful in cases where the application doesn't need to know when a specific request
///   completed, if it completed successfully.
/// * [`IOSQE_BUFFER_SELECT`]
///   * If set, and if the request type supports it, select an IO buffer from the indicated buffer
///   group.
///   * This can be used with requests that read or receive data from a file or socket, where
///   buffer selection is deferred until the kernel is ready to transfer data, instead of when the
///   IO is originally submitted.
///   * The application must also set the buf_group field in the SQE, indicating which previously
///   registered buffer group to select a buffer from.
#[inline]
pub fn io_uring_sqe_set_flags(sqe: &mut io_uring_sqe, flags: u32) {
    sqe.flags = flags as u8
}

#[inline]
fn io_uring_prep_rw(
    op: u32,
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: *const (),
    len: u32,
    offset: u64,
) {
    sqe.opcode = op as u8;
    sqe.flags = 0;
    sqe.ioprio = 0;
    sqe.fd = fd;
    sqe.__bindgen_anon_1.off = offset;
    sqe.__bindgen_anon_2.addr = addr as u64;
    sqe.len = len;
}

/// Prepares a splice request
///
/// The submission queue entry is setup to use as input the file descriptor `fd_id` at offset
/// `off_in`, splicing data to the file descriptor `fd_out` at offset `off_out`. `nbytes` bytes of
/// data should be spliced between the two descriptors. `splice_flags` are modifier flags for the
/// operation. See [`splice(2)`](https://man.archlinux.org/man/splice.2) for the generic splice
/// flags.
#[inline]
pub fn io_uring_prep_splice(
    sqe: &mut io_uring_sqe,
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

    sqe.__bindgen_anon_2.splice_off_in = off_in as u64;
    sqe.__bindgen_anon_5.splice_fd_in = fd_in;
    sqe.__bindgen_anon_3.splice_flags = splice_flags;
}

/// Prepares a tee request
///
/// The submission queue entry is setup to use as input the file descriptor `fd_in` and as output
/// the file descriptor `fd_out` duplicating `nbytes` bytes worth of data. `splice_flags` are
/// modifier flags for the operation. See [`tee(2)`](https://man.archlinux.org/man/tee.2) for the
/// generic splice flags.
#[inline]
pub fn io_uring_prep_tee(
    sqe: &mut io_uring_sqe,
    fd_in: i32,
    fd_out: i32,
    nbytes: u32,
    splice_flags: u32,
) {
    io_uring_prep_rw(IORING_OP_TEE, sqe, fd_out, ptr::null(), nbytes, 0);

    sqe.__bindgen_anon_2.splice_off_in = 0;
    sqe.__bindgen_anon_5.splice_fd_in = fd_in;
    sqe.__bindgen_anon_3.splice_flags = splice_flags;
}

/// Prepares a vectored IO read request
///
/// On files that support seeking, if the offset is set to -1, the read operation commences at the
/// file offset, and the file offset is incremented by the number of bytes read. See
/// [`read(2)`](https://man.archlinux.org/man/read.2) for more details. Note that for an async API,
/// reading and updating the current file offset may result in unpredictable behavior, unless
/// access to the file is serialized. It is not encouraged to use this feature, if it's possible to
/// provide the desired IO offset from the application or library.
///
/// On files that are not capable of seeking, the offset must be 0 or -1. After the read has been
/// prepared it can be submitted with one of the submit functions.
///
/// # Notes
/// Unless an application explicitly needs to pass in more than one iovec, it is more efficient to
/// use [`io_uring_prep_read`] rather than this function, as no state has to be maintained for a
/// non-vectored IO request. As with any request that passes in data in a struct, that data must
/// remain valid until the request has been successfully submitted. It need not remain valid until
/// completion.
///
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_readv(sqe: &mut io_uring_sqe, fd: i32, iovecs: &[iovec], offset: u64) {
    io_uring_prep_rw(
        IORING_OP_READV,
        sqe,
        fd,
        iovecs.as_ptr().cast(),
        iovecs.len() as u32,
        offset,
    )
}

/// Prepares a vectored IO read request
///
/// Similar to [`io_uring_prep_readv`] with the addition of flags. Supported values for flags are:
/// * `RWF_HIPRI`: High priority request, poll if possible
/// * `RWF_DSYNC`: per-IO O_DSYNC
/// * `RWF_SYNC`: per-IO O_SYNC
/// * `RWF_NOWAIT`: per-IO, return -EAGAIN if operation would block
/// * `RWF_APPEND`: per-IO O_APPEND
///
/// # Notes
/// Unless an application explicitly needs to pass in more than one iovec, it is more efficient to
/// use [`io_uring_prep_read`] rather than this function, as no state has to be maintained for a
/// non-vectored IO request. As with any request that passes in data in a struct, that data must
/// remain valid until the request has been successfully submitted. It need not remain valid until
/// completion.
///
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_readv2(
    sqe: &mut io_uring_sqe,
    fd: i32,
    iovecs: &[iovec],
    offset: u64,
    flags: i32,
) {
    io_uring_prep_readv(sqe, fd, iovecs, offset);
    sqe.__bindgen_anon_3.rw_flags = flags;
}

/// Prepares an IO read request with a previously registered IO buffer
///
/// This works just like [`io_uring_prep_read`] except i requires the use of buffers that have been
/// registered with [`io_uring_register_buffers`]. The `buf` argument must fall within a region
/// specified by `buf_index` in the previously registered buffer. The buffer need not be aligned
/// with the start of the registered buffer.
#[inline]
pub fn io_uring_prep_read_fixed(
    sqe: &mut io_uring_sqe,
    fd: i32,
    buf: &mut [u8],
    offset: u64,
    buf_index: i32,
) {
    io_uring_prep_rw(
        IORING_OP_READ_FIXED,
        sqe,
        fd,
        buf.as_ptr().cast(),
        buf.len() as u32,
        offset,
    );
    sqe.__bindgen_anon_4.buf_index = buf_index as u16;
}

/// Prepares a vectored IO write request
///
/// On files that support seeking, if the offset is set to -1, the write operation commences at the
/// file offset, and the file offset is incremented by the number of bytes written. See
/// [`write(2)`](https://man.archlinux.org/man/write.2) for more details. Note that for an async
/// API, reading and updating the current file offset may result in unpredictable behavior, unless
/// access to the file is serialized. It is not encouraged to use this feature, if it's possible to
/// provide the desired IO offset from the application or library.
///
/// On files that are not capable of seeking, the offset must be 0 or -1. After the write has been
/// prepared it can be submitted with one of the submit functions.
///
/// # Notes
/// Unless an application explicitly needs to pass in more than one iovec, it is more efficient to
/// use [`io_uring_prep_read`] rather than this function, as no state has to be maintained for a
/// non-vectored IO request. As with any request that passes in data in a struct, that data must
/// remain valid until the request has been successfully submitted. It need not remain valid until
/// completion.
///
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_writev(sqe: &mut io_uring_sqe, fd: i32, iovecs: &[iovec], offset: u64) {
    io_uring_prep_rw(
        IORING_OP_WRITEV,
        sqe,
        fd,
        iovecs.as_ptr().cast(),
        iovecs.len() as u32,
        offset,
    )
}

/// Prepares a vectored IO write request
///
/// Similar to [`io_uring_prep_writev`] with the addition of flags. Supported values for flags are:
/// * `RWF_HIPRI`: High priority request, poll if possible
/// * `RWF_DSYNC`: per-IO O_DSYNC
/// * `RWF_SYNC`: per-IO O_SYNC
/// * `RWF_NOWAIT`: per-IO, return -EAGAIN if operation would block
/// * `RWF_APPEND`: per-IO O_APPEND
///
/// # Notes
/// Unless an application explicitly needs to pass in more than one iovec, it is more efficient to
/// use [`io_uring_prep_write`] rather than this function, as no state has to be maintained for a
/// non-vectored IO request. As with any request that passes in data in a struct, that data must
/// remain valid until the request has been successfully submitted. It need not remain valid until
/// completion.
///
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_writev2(
    sqe: &mut io_uring_sqe,
    fd: i32,
    iovecs: &[iovec],
    offset: u64,
    flags: i32,
) {
    io_uring_prep_writev(sqe, fd, iovecs, offset);
    sqe.__bindgen_anon_3.rw_flags = flags;
}

/// Prepares an IO write request with a previously registered IO buffer
///
/// This works just like [`io_uring_prep_write`] except i requires the use of buffers that have been
/// registered with [`io_uring_register_buffers`]. The `buf` argument must fall within a region
/// specified by `buf_index` in the previously registered buffer. The buffer need not be aligned
/// with the start of the registered buffer.
#[inline]
pub fn io_uring_prep_write_fixed(
    sqe: &mut io_uring_sqe,
    fd: i32,
    buf: &mut [u8],
    offset: u64,
    buf_index: i32,
) {
    io_uring_prep_rw(
        IORING_OP_WRITE_FIXED,
        sqe,
        fd,
        buf.as_mut_ptr().cast(),
        buf.len() as u32,
        offset,
    );
    sqe.__bindgen_anon_4.buf_index = buf_index as u16;
}

/// Prepares a recvmsg request.
///
/// The submission queue entry is setup to use the file descriptor `fd` to start receiving the data
/// indicated by `msg` with the [`recvmsg(2)`](https://man.archlinux.org/man/recvmsg.2) defined
/// flags in the `flags` argument.
///
/// # Notes
/// As with any request that passes in data in a struct, that data must remain valid until the
/// request has been successfully submitted. It need not remain valid until completion.
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_recvmsg(sqe: &mut io_uring_sqe, fd: i32, msg: *mut msghdr, flags: u32) {
    io_uring_prep_rw(IORING_OP_RECVMSG, sqe, fd, msg.cast(), 1, 0);
    sqe.__bindgen_anon_3.msg_flags = flags;
}

/// Prepares a multishot recvmsg request.
///
/// The submission queue entry is setup to use the file descriptor `fd` to start receiving the data
/// indicated by `msg` with the [`recvmsg(2)`](https://man.archlinux.org/man/recvmsg.2) defined
/// flags in the `flags` argument.
///
/// Allows the application to issue a single receive request, which repeatedly posts a CQE when
/// data is available. It requires the `IOSQE_BUFFER_SELECT` flag to be set and no `MSG_WAITALL`
/// flag to be set. Therefore each CQE will take a buffer out of a provided buffer pool for
/// receiving. The application should check the flags of each CQE, regardless of its result. If a
/// posted CQE does not have the `IORING_CQE_F_MORE` flag set then the multishot receive will be
/// done and the application should issue a new request.
///
/// Unlike [`recvmsg(2)`](https://man.archlinux.org/man/recvmsg.2), multishot recvmsg will prepend
/// a struct [`io_uring_recvmsg_out`] which describes the layout of the rest of the buffer in
/// combination with the intitial struct [`msghdr`] submitted with the request. See
/// [`io_uring_recvmsg_out(3)`](https://man.archlinux.org/man/io_uring_recvmsg_out.3) for more
/// information on accessing the data.
///
/// # Notes
/// As with any request that passes in data in a struct, that data must remain valid until the
/// request has been successfully submitted. It need not remain valid until completion.
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_recvmsg_multishot(
    sqe: &mut io_uring_sqe,
    fd: i32,
    msg: *mut msghdr,
    flags: u32,
) {
    io_uring_prep_recvmsg(sqe, fd, msg, flags);
    sqe.ioprio |= IORING_RECV_MULTISHOT as u16;
}

/// Prepares a sendmsg request
///
/// The submission queue entry is setup to use the file descriptor fd to start sending the data
/// indicated by `msg` with the [`sendmsg(2)`](https://man.archlinux.org/man/sendmsg.2) defined
/// flags in the `flags` argument.
///
/// # Notes
/// Using `IOSQE_IO_LINK` with this request type requires the setting of `MSG_WAITALL` in the flags
/// argument, as a short send isn't considered an error condition without that being set.
///
/// As with any request that passes in data in a struct, that data must remain valid until the
/// request has been successfully submitted. It need not remain valid until completion.
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the `IORING_FEAT_SUBMIT_STABLE` flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_sendmsg(sqe: &mut io_uring_sqe, fd: i32, msg: *const msghdr, flags: u32) {
    io_uring_prep_rw(IORING_OP_SENDMSG, sqe, fd, msg.cast(), 1, 0);
    sqe.__bindgen_anon_3.msg_flags = flags;
}

#[inline]
pub fn io_uring_prep_poll_add(sqe: &mut io_uring_sqe, fd: i32, poll_mask: u32) {
    io_uring_prep_rw(IORING_OP_POLL_ADD, sqe, fd, ptr::null(), 0, 0);
    sqe.__bindgen_anon_3.poll32_events = poll_mask.to_le();
}

#[inline]
pub fn io_uring_prep_poll_multishot(sqe: &mut io_uring_sqe, fd: i32, poll_mask: u32) {
    io_uring_prep_poll_add(sqe, fd, poll_mask);
    sqe.len = IORING_POLL_ADD_MULTI;
}

#[inline]
pub fn io_uring_prep_poll_remove(sqe: &mut io_uring_sqe, user_data: u64) {
    io_uring_prep_rw(IORING_OP_POLL_REMOVE, sqe, -1, ptr::null(), 0, 0);
    sqe.__bindgen_anon_2.addr = user_data
}

#[inline]
pub fn io_uring_prep_poll_update(
    sqe: &mut io_uring_sqe,
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
    sqe.__bindgen_anon_2.addr = old_user_data;
    sqe.__bindgen_anon_3.poll32_events = poll_mask.to_le();
}

#[inline]
pub fn io_uring_prep_fsync(sqe: &mut io_uring_sqe, fd: i32, fsync_flags: u32) {
    io_uring_prep_rw(IORING_OP_FSYNC, sqe, fd, ptr::null(), 0, 0);
    sqe.__bindgen_anon_3.fsync_flags = fsync_flags;
}

#[inline]
pub fn io_uring_prep_nop(sqe: &mut io_uring_sqe) {
    io_uring_prep_rw(IORING_OP_NOP, sqe, -1, ptr::null(), 0, 0);
}

#[inline]
pub fn io_uring_prep_timeout(sqe: &mut io_uring_sqe, ts: *mut timespec, count: u32, flags: u32) {
    io_uring_prep_rw(IORING_OP_TIMEOUT, sqe, -1, ts.cast(), 1, count as u64);
    sqe.__bindgen_anon_3.timeout_flags = flags;
}

#[inline]
pub fn io_uring_prep_timeout_remove(sqe: &mut io_uring_sqe, user_data: u64, flags: u32) {
    io_uring_prep_rw(IORING_OP_TIMEOUT_REMOVE, sqe, -1, ptr::null(), 0, 0);
    sqe.__bindgen_anon_2.addr = user_data;
    sqe.__bindgen_anon_3.timeout_flags = flags;
}

#[inline]
pub fn io_uring_prep_timeout_update(
    sqe: &mut io_uring_sqe,
    ts: *mut timespec,
    user_data: u64,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_TIMEOUT_REMOVE, sqe, -1, ptr::null(), 0, ts as u64);
    sqe.__bindgen_anon_2.addr = user_data;
    sqe.__bindgen_anon_3.timeout_flags = flags | IORING_TIMEOUT_UPDATE;
}

#[inline]
pub fn io_uring_prep_accept(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
) {
    io_uring_prep_rw(IORING_OP_ACCEPT, sqe, fd, addr.cast(), 0, addrlen as u64);
    sqe.__bindgen_anon_3.accept_flags = flags;
}

#[inline]
fn io_uring_set_target_fixed_file(sqe: &mut io_uring_sqe, file_index: u32) {
    sqe.__bindgen_anon_5.file_index = file_index + 1;
}

#[inline]
pub fn io_uring_prep_accept_direct(
    sqe: &mut io_uring_sqe,
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

#[inline]
pub fn io_uring_prep_multishot_accept(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
) {
    io_uring_prep_accept(sqe, fd, addr, addrlen, flags);
    sqe.ioprio |= IORING_ACCEPT_MULTISHOT as u16;
}

#[inline]
pub fn io_uring_prep_multishot_accept_direct(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: *mut sockaddr,
    addrlen: *mut socklen_t,
    flags: u32,
) {
    io_uring_prep_multishot_accept(sqe, fd, addr, addrlen, flags);
    io_uring_set_target_fixed_file(sqe, (IORING_FILE_INDEX_ALLOC - 1) as u32);
}

#[inline]
pub fn io_uring_prep_cancel64(sqe: &mut io_uring_sqe, user_data: u64, flags: i32) {
    io_uring_prep_rw(IORING_OP_ASYNC_CANCEL, sqe, -1, ptr::null(), 0, 0);
    sqe.__bindgen_anon_2.addr = user_data;
    sqe.__bindgen_anon_3.cancel_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_cancel(sqe: &mut io_uring_sqe, user_data: *mut c_void, flags: i32) {
    io_uring_prep_cancel64(sqe, user_data as u64, flags);
}

#[inline]
pub fn io_uring_prep_cancel_fd(sqe: &mut io_uring_sqe, fd: i32, flags: u32) {
    io_uring_prep_rw(IORING_OP_ASYNC_CANCEL, sqe, fd, ptr::null(), 0, 0);
    sqe.__bindgen_anon_3.cancel_flags = flags | IORING_ASYNC_CANCEL_FD;
}

#[inline]
pub fn io_uring_prep_link_timeout(sqe: &mut io_uring_sqe, ts: *mut timespec, flags: u32) {
    io_uring_prep_rw(IORING_OP_LINK_TIMEOUT, sqe, -1, ts.cast(), 1, 0);
    sqe.__bindgen_anon_3.timeout_flags = flags;
}

#[inline]
pub fn io_uring_prep_connect(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: *const sockaddr,
    addrlen: socklen_t,
) {
    io_uring_prep_rw(IORING_OP_CONNECT, sqe, fd, addr.cast(), 0, addrlen as u64);
}

#[inline]
pub fn io_uring_prep_files_update(sqe: &mut io_uring_sqe, fds: &mut [i32], offset: i32) {
    io_uring_prep_rw(
        IORING_OP_FILES_UPDATE,
        sqe,
        -1,
        fds.as_mut_ptr().cast(),
        fds.len() as u32,
        offset as u64,
    );
}

#[inline]
pub fn io_uring_prep_fallocate(sqe: &mut io_uring_sqe, fd: i32, mode: i32, offset: u64, len: u64) {
    io_uring_prep_rw(
        IORING_OP_FALLOCATE,
        sqe,
        fd,
        ptr::null(),
        mode as u32,
        offset,
    );
    sqe.__bindgen_anon_2.addr = len;
}

#[inline]
pub fn io_uring_prep_openat(
    sqe: &mut io_uring_sqe,
    dfd: i32,
    path: *const char,
    flags: i32,
    mode: mode_t,
) {
    io_uring_prep_rw(IORING_OP_OPENAT, sqe, dfd, path.cast(), mode, 0);
    sqe.__bindgen_anon_3.open_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_openat_direct(
    sqe: &mut io_uring_sqe,
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

#[inline]
pub fn io_uring_prep_close(sqe: &mut io_uring_sqe, fd: i32) {
    io_uring_prep_rw(IORING_OP_CLOSE, sqe, fd, ptr::null(), 0, 0);
}

#[inline]
pub fn io_uring_prep_close_direct(sqe: &mut io_uring_sqe, file_index: u32) {
    io_uring_prep_close(sqe, 0);
    io_uring_set_target_fixed_file(sqe, file_index);
}

#[inline]
pub fn io_uring_prep_read(sqe: &mut io_uring_sqe, fd: i32, buf: *mut (), nbytes: u32, offset: u64) {
    io_uring_prep_rw(IORING_OP_READ, sqe, fd, buf, nbytes, offset);
}

#[inline]
pub fn io_uring_prep_write(
    sqe: &mut io_uring_sqe,
    fd: i32,
    buf: *mut (),
    nbytes: u32,
    offset: u64,
) {
    io_uring_prep_rw(IORING_OP_WRITE, sqe, fd, buf, nbytes, offset);
}

// TODO: statx fadvise madvise

#[inline]
pub fn io_uring_prep_send(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: *const (),
    len: usize,
    flags: i32,
) {
    io_uring_prep_rw(IORING_OP_SEND, sqe, sockfd, buf, len as u32, 0);
    sqe.__bindgen_anon_3.msg_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_send_set_addr(
    sqe: &mut io_uring_sqe,
    dest_addr: *const sockaddr,
    addr_len: u16,
) {
    sqe.__bindgen_anon_1.addr2 = dest_addr as u64;
    sqe.__bindgen_anon_5.__bindgen_anon_1.addr_len = addr_len;
}

#[inline]
pub fn io_uring_prep_sendto(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: *const (),
    len: usize,
    flags: i32,
    addr: *const sockaddr,
    addr_len: u16,
) {
    io_uring_prep_send(sqe, sockfd, buf, len, flags);
    io_uring_prep_send_set_addr(sqe, addr, addr_len);
}

#[inline]
pub fn io_uring_prep_send_zc(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: *const (),
    len: usize,
    flags: i32,
    zc_flags: u32,
) {
    io_uring_prep_rw(IORING_OP_SEND_ZC, sqe, sockfd, buf, len as u32, 0);
    sqe.__bindgen_anon_3.msg_flags = flags as u32;
    sqe.ioprio = zc_flags as u16;
}

#[inline]
pub fn io_uring_prep_send_zc_fixed(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: *const (),
    len: usize,
    flags: i32,
    zc_flags: u32,
    buf_index: u32,
) {
    io_uring_prep_send_zc(sqe, sockfd, buf, len, flags, zc_flags);
    sqe.ioprio |= IORING_RECVSEND_FIXED_BUF as u16;
    sqe.__bindgen_anon_4.buf_index = buf_index as u16;
}

#[inline]
pub fn io_uring_prep_sendmsg_zc(sqe: &mut io_uring_sqe, fd: i32, msg: *const msghdr, flags: u32) {
    io_uring_prep_sendmsg(sqe, fd, msg, flags);
    sqe.opcode = IORING_OP_SENDMSG_ZC as u8;
}

#[inline]
pub fn io_uring_prep_recv(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: *mut (),
    len: usize,
    flags: i32,
) {
    io_uring_prep_rw(IORING_OP_RECV, sqe, sockfd, buf, len as u32, 0);
    sqe.__bindgen_anon_3.msg_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_recv_multishot(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: *mut (),
    len: usize,
    flags: i32,
) {
    io_uring_prep_recv(sqe, sockfd, buf, len, flags);
    sqe.ioprio |= IORING_RECV_MULTISHOT as u16;
}

// TODO: recvmsg helpers, openat2, epollctl, provide_buffers, remove_buffers, shutdown, unlink,
// rename, sync_file_range, mkdir, symlink, link

#[inline]
pub fn io_uring_prep_msg_ring_cqe_flags(
    sqe: &mut io_uring_sqe,
    fd: i32,
    len: u32,
    data: u64,
    flags: u32,
    cqe_flags: u32,
) {
    io_uring_prep_rw(IORING_OP_MSG_RING, sqe, fd, ptr::null(), len, data);
    sqe.__bindgen_anon_3.msg_ring_flags = IORING_MSG_RING_FLAGS_PASS | flags;
    sqe.__bindgen_anon_5.file_index = cqe_flags;
}

#[inline]
pub fn io_uring_prep_msg_ring(sqe: &mut io_uring_sqe, fd: i32, len: u32, data: u64, flags: u32) {
    io_uring_prep_rw(IORING_OP_MSG_RING, sqe, fd, ptr::null(), len, data);
    sqe.__bindgen_anon_3.msg_ring_flags = flags;
}

#[inline]
pub fn io_uring_prep_msg_ring_fd(
    sqe: &mut io_uring_sqe,
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
        IORING_MSG_SEND_FD as uintptr_t as _,
        0,
        data,
    );
    unsafe { sqe.__bindgen_anon_6.__bindgen_anon_1.as_mut().addr3 = source_fd as u64 };
    let target_fd = if target_fd == IORING_FILE_INDEX_ALLOC {
        target_fd - 1
    } else {
        target_fd
    };
    io_uring_set_target_fixed_file(sqe, target_fd as u32);
    sqe.__bindgen_anon_3.msg_ring_flags = flags;
}

#[inline]
pub fn io_uring_prep_msg_ring_fd_alloc(
    sqe: &mut io_uring_sqe,
    fd: i32,
    source_fd: i32,
    data: u64,
    flags: u32,
) {
    io_uring_prep_msg_ring_fd(sqe, fd, source_fd, IORING_FILE_INDEX_ALLOC, data, flags);
}

// TODO: xattr

#[inline]
pub fn io_uring_prep_socket(
    sqe: &mut io_uring_sqe,
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
    sqe.__bindgen_anon_3.rw_flags = flags as i32;
}

#[inline]
pub fn io_uring_prep_socket_direct(
    sqe: &mut io_uring_sqe,
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

#[inline]
pub fn io_uring_prep_socket_direct_alloc(
    sqe: &mut io_uring_sqe,
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
            // io_uring_cq_advance(ring, 1);

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
    cq: &mut io_uring_cq,
    br: &mut io_uring_buf_ring,
    cq_count: i32,
    buf_count: i32,
) {
    br.__bindgen_anon_1.__bindgen_anon_1.as_mut().tail += buf_count as u16;
    io_uring_cq_advance(cq, cq_count as u32);
}

/// # Safety
/// `br` must point to a valid and initialized `io_uring_buf_ring`
#[inline]
pub unsafe fn io_uring_buf_ring_cq_advance(
    cq: &mut io_uring_cq,
    br: &mut io_uring_buf_ring,
    count: i32,
) {
    io_uring_buf_ring_cq_advance_internal(cq, br, count, count)
}
