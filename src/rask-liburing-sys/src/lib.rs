#![deny(
    // missing_docs,
    // missing_debug_implementations,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

//! Rust binding for liburing

use std::{
    ffi::OsString,
    ptr::{self, NonNull},
    sync::atomic::{AtomicU16, AtomicU32, Ordering},
};

use libc::{c_void, mode_t, msghdr, sockaddr, socklen_t, timespec};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const IO_URING_OP_SUPPORTED: u32 = 1u32 << 0;

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
    if seen > 0 {
        let head = unsafe { *cq.khead } + seen;
        let khead: &mut AtomicU32 = unsafe { &mut *cq.khead.cast() };
        khead.store(head, Ordering::Release);
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
pub fn io_uring_sqe_set_data<T>(sqe: &mut io_uring_sqe, data: NonNull<T>) {
    sqe.user_data = data.as_ptr() as u64;
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
pub unsafe fn io_uring_cqe_get_data<T>(cqe: &io_uring_cqe) -> Option<&T> {
    (cqe.user_data as *mut T).as_ref()
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
pub unsafe fn io_uring_cqe_get_data64(cqe: &io_uring_cqe) -> u64 {
    cqe.user_data
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
fn io_uring_prep_rw<T>(
    op: u32,
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: Option<&T>,
    len: u32,
    offset: u64,
) {
    sqe.opcode = op as u8;
    sqe.flags = 0;
    sqe.ioprio = 0;
    sqe.fd = fd;
    sqe.__bindgen_anon_1.off = offset;
    sqe.__bindgen_anon_2.addr = addr.map_or(ptr::null(), |a| a) as u64;
    sqe.len = len;
}

#[inline]
fn io_uring_prep_rw_buf<T>(
    op: u32,
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: Option<&[T]>,
    offset: u64,
) {
    sqe.opcode = op as u8;
    sqe.flags = 0;
    sqe.ioprio = 0;
    sqe.fd = fd;
    sqe.__bindgen_anon_1.off = offset;
    sqe.__bindgen_anon_2.addr = addr.map_or(ptr::null(), |a| a.as_ptr()) as u64;
    sqe.len = addr.map_or(0, |a| a.len()) as u32;
}

#[inline]
fn io_uring_prep_rw_null(op: u32, sqe: &mut io_uring_sqe, fd: i32, len: u32, offset: u64) {
    sqe.opcode = op as u8;
    sqe.flags = 0;
    sqe.ioprio = 0;
    sqe.fd = fd;
    sqe.__bindgen_anon_1.off = offset;
    sqe.__bindgen_anon_2.addr = ptr::null::<usize>() as u64;
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
    io_uring_prep_rw_null(IORING_OP_SPLICE, sqe, fd_out, nbytes, off_out as u64);

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
    io_uring_prep_rw_null(IORING_OP_TEE, sqe, fd_out, nbytes, 0);

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
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_readv(sqe: &mut io_uring_sqe, fd: i32, iovecs: &[iovec], offset: u64) {
    io_uring_prep_rw_buf(IORING_OP_READV, sqe, fd, Some(iovecs), offset)
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
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
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
    io_uring_prep_rw_buf(IORING_OP_READ_FIXED, sqe, fd, Some(buf), offset);
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
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_writev(sqe: &mut io_uring_sqe, fd: i32, iovecs: &[iovec], offset: u64) {
    io_uring_prep_rw_buf(IORING_OP_WRITEV, sqe, fd, Some(iovecs), offset)
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
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
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
    io_uring_prep_rw_buf(IORING_OP_WRITE_FIXED, sqe, fd, Some(buf), offset);
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
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_recvmsg(sqe: &mut io_uring_sqe, fd: i32, msg: &mut msghdr, flags: u32) {
    io_uring_prep_rw(IORING_OP_RECVMSG, sqe, fd, Some(msg), 1, 0);
    sqe.__bindgen_anon_3.msg_flags = flags;
}

/// Prepares a multishot recvmsg request.
///
/// The submission queue entry is setup to use the file descriptor `fd` to start receiving the data
/// indicated by `msg` with the [`recvmsg(2)`](https://man.archlinux.org/man/recvmsg.2) defined
/// flags in the `flags` argument.
///
/// Allows the application to issue a single receive request, which repeatedly posts a CQE when
/// data is available. It requires the [`IOSQE_BUFFER_SELECT`] flag to be set and no `MSG_WAITALL`
/// flag to be set. Therefore each CQE will take a buffer out of a provided buffer pool for
/// receiving. The application should check the flags of each CQE, regardless of its result. If a
/// posted CQE does not have the [`IORING_CQE_F_MORE`] flag set then the multishot receive will be
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
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_recvmsg_multishot(
    sqe: &mut io_uring_sqe,
    fd: i32,
    msg: &mut msghdr,
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
/// Using [`IOSQE_IO_LINK`] with this request type requires the setting of [`MSG_WAITALL`] in the
/// flags argument, as a short send isn't considered an error condition without that being set.
///
/// As with any request that passes in data in a struct, that data must remain valid until the
/// request has been successfully submitted. It need not remain valid until completion.
/// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
/// earlier) required state to be stable until the completion occurred. Applications can test for
/// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
/// [`io_uring_queue_init_params`].
#[inline]
pub fn io_uring_prep_sendmsg(sqe: &mut io_uring_sqe, fd: i32, msg: &msghdr, flags: u32) {
    io_uring_prep_rw(IORING_OP_SENDMSG, sqe, fd, Some(msg), 1, 0);
    sqe.__bindgen_anon_3.msg_flags = flags;
}

/// Prepares a poll request
///
/// The submission queue entry is setup to use the file descriptor `fd` that should get polled,
/// with the events desired specified in the `poll_mask` argument.
///
/// When the specified event has triggered, a completion CQE is posted and no more events will be
/// generated by the poll request.
#[inline]
pub fn io_uring_prep_poll_add(sqe: &mut io_uring_sqe, fd: i32, poll_mask: u32) {
    io_uring_prep_rw_null(IORING_OP_POLL_ADD, sqe, fd, 0, 0);
    sqe.__bindgen_anon_3.poll32_events = poll_mask.to_le();
}

/// Prepares a poll request
///
/// The submission queue entry is setup to use the file descriptor `fd` that should get polled,
/// with the events desired specified in the `poll_mask` argument.
///
/// Behaves identically to [`io_uring_prep_poll_add()`] in terms of events, but persists across
/// notifications and will repeatedly post notifications for the same registration. A CQE posted
/// from a multishot poll request will have [`IORING_CQE_F_MORE`] set in the CQE flags member,
/// indicating that the application should expect more completions from this request. If the
/// multishot poll request gets terminater or experiences an error, this flag will not be set in
/// the CQE. If this happens, the application should not expect further CQEs from the original
/// request and must reissue a new one if it still wishes to get notifications on this file
/// descriptor.
#[inline]
pub fn io_uring_prep_poll_multishot(sqe: &mut io_uring_sqe, fd: i32, poll_mask: u32) {
    io_uring_prep_poll_add(sqe, fd, poll_mask);
    sqe.len = IORING_POLL_ADD_MULTI;
}

/// Prepares a poll removal request
///
/// The submission queue entry is setup to remove a poll request identified by `user_data`.
///
/// Works like [`io_uring_prep_cancel()`] except only looks for poll requests. Apart from that,
/// behavior is identical.
#[inline]
pub fn io_uring_prep_poll_remove(sqe: &mut io_uring_sqe, user_data: u64) {
    io_uring_prep_rw_null(IORING_OP_POLL_REMOVE, sqe, -1, 0, 0);
    sqe.__bindgen_anon_2.addr = user_data
}

/// Prepares a poll update request
///
/// The submission queue entry is setup to update a poll request identified by `old_user_data`,
/// replacing it with the `new_user_data` information. The `poll_mask` arguments contains the new
/// mask to use for the poll request, and flags argument contains modifier flags telling io_uring
/// what fields to update.
#[inline]
pub fn io_uring_prep_poll_update(
    sqe: &mut io_uring_sqe,
    old_user_data: u64,
    new_user_data: u64,
    poll_mask: u32,
    flags: u32,
) {
    io_uring_prep_rw_null(IORING_OP_POLL_REMOVE, sqe, -1, flags, new_user_data);
    sqe.__bindgen_anon_2.addr = old_user_data;
    sqe.__bindgen_anon_3.poll32_events = poll_mask.to_le();
}

/// Prepares an fsync request
///
/// The submission queue entry is setup to use the file descriptor `fd` that should get synced,
/// with the modifier flags indicated by the `flags` argument.
///
/// By default, this acts like an [`fsync(2)`](https://man.archlinux.org/man/fsync.2) operation,
/// which is the default behavior. If [`IORING_FSYNC_DATASYNC`] is set in the flags argument, then
/// it behaves like [`fdatasync(2)`](https://man.archlinux.org/man/fdatasync.2). If no range is
/// specified, the fd will be synced from 0 to end-of-file.
///
/// It's possible to specify a range to sync, if one is desired. If the `off` field of the SQE is
/// set to non-zero, then that indicates the offset to start syncing at. If len is set in the SQE,
/// then that indicates the size in bytes to sync from the offset. Note that these fields are not
/// accepted by this helper, so they have to be set manually in the SQE after calling this prep
/// helper.
#[inline]
pub fn io_uring_prep_fsync(sqe: &mut io_uring_sqe, fd: i32, fsync_flags: u32) {
    io_uring_prep_rw_null(IORING_OP_FSYNC, sqe, fd, 0, 0);
    sqe.__bindgen_anon_3.fsync_flags = fsync_flags;
}

/// Prepares a nop (no operation) request
///
/// The submission queue entry sqe does not require any additional setup.
#[inline]
pub fn io_uring_prep_nop(sqe: &mut io_uring_sqe) {
    io_uring_prep_rw_null(IORING_OP_NOP, sqe, -1, 0, 0);
}

/// Prepares a timeout request
///
/// The submission queue entry is setup to arm a timeout specified by `ts` and with a timeout count
/// of `count` completion entries. The `flags` argument holds modifier flags for the request.
///
/// This request type can be used as a timeout waking anyone sleeping for events on the CQ ring.
/// The flags argument may contain:
/// * [`IORING_TIMEOUT_ABS`]: The value specified in `ts` is an absolute value rather than a
/// relative one.
/// * [`IORING_TIMEOUT_BOOTTIME`]: The boottime clock source should be used.
/// * [`IORING_TIMEOUT_REALTIME`]: The realtime clock source should be used.
/// * [`IORING_TIMEOUT_ETIME_SUCCESS`]: Consider an expired timeout a success in terms of the
/// posted completion. Normally a timeout that triggers would return in a -ETIME CQE res value.
/// * [`IORING_TIMEOUT_MULTISHOT`]: The request will return multiple timeout completions. The
/// completion flag [`IORING_CQE_F_MORE`] is set if more timeouts are expected. The value specified
/// in `count` is the number of repeats. A value of 0 means the timeout is indefinite and can only
/// be stopped by a removal request.
///
/// The timeout completion event will trigger if either the specified timeout has occurred, or the
/// specified number of events to wait for have been posted to the CQ ring.
#[inline]
pub fn io_uring_prep_timeout(sqe: &mut io_uring_sqe, ts: &mut timespec, count: u32, flags: u32) {
    io_uring_prep_rw(IORING_OP_TIMEOUT, sqe, -1, Some(ts), 1, count as u64);
    sqe.__bindgen_anon_3.timeout_flags = flags;
}

/// Cancels an existing timeout request
///
/// The submission queue entry is setup to arm a timeout removal specified by `user_data` and with
/// modifier flags given by `flags`.
///
/// The timeout remove command does not currently accept any flags.
#[inline]
pub fn io_uring_prep_timeout_remove(sqe: &mut io_uring_sqe, user_data: u64, flags: u32) {
    io_uring_prep_rw_null(IORING_OP_TIMEOUT_REMOVE, sqe, -1, 0, 0);
    sqe.__bindgen_anon_2.addr = user_data;
    sqe.__bindgen_anon_3.timeout_flags = flags;
}

/// Prepares a timeout update request
///
/// The submission queue entry is setup to arm a timeout update specified by `user_data` and with
/// modifier flags given by `flags`. Additionally the update request includes a `ts` structure,
/// which contains new timeout information.
///
/// The flags member may contain a bitmask of the following values:
/// * [`IORING_TIMEOUT_ABS`]: The value specified in `ts` is an absolute value rather than a
/// relative one.
/// * [`IORING_TIMEOUT_BOOTTIME`]: The boottime clock source should be used.
/// * [`IORING_TIMEOUT_REALTIME`]: The realtime clock source should be used.
/// * [`IORING_TIMEOUT_ETIME_SUCCESS`]: Consider an expired timeout a success in terms of the
/// posted completion. Normally a timeout that triggers would return in a -ETIME CQE res value.
#[inline]
pub fn io_uring_prep_timeout_update(
    sqe: &mut io_uring_sqe,
    ts: &mut timespec,
    user_data: u64,
    flags: u32,
) {
    io_uring_prep_rw_null(
        IORING_OP_TIMEOUT_REMOVE,
        sqe,
        -1,
        0,
        ts as *mut timespec as u64,
    );
    sqe.__bindgen_anon_2.addr = user_data;
    sqe.__bindgen_anon_3.timeout_flags = flags | IORING_TIMEOUT_UPDATE;
}

/// Prepares an accept request
///
/// The submission queue entry is setup to use the file descriptor `fd` to start accepting a
/// connection request described by the socket address at `addr` and of structure length `addrlen`
/// and using modifier flags in `flags`.
///
/// See the man page [`accept4(2)`](https://man.archlinux.org/man/accept4.2) for more details of
/// the accept function itself.
#[inline]
pub fn io_uring_prep_accept(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: Option<&mut sockaddr>,
    addrlen: Option<&mut socklen_t>,
    flags: u32,
) {
    io_uring_prep_rw(
        IORING_OP_ACCEPT,
        sqe,
        fd,
        addr.as_ref(),
        0,
        addrlen.map_or(ptr::null_mut(), |a| a as *mut socklen_t) as u64,
    );
    sqe.__bindgen_anon_3.accept_flags = flags;
}

#[inline]
fn io_uring_set_target_fixed_file(sqe: &mut io_uring_sqe, file_index: u32) {
    sqe.__bindgen_anon_5.file_index = file_index + 1;
}

/// Prepares an accept request
///
/// The submission queue entry is setup to use the file descriptor `fd` to start accepting a
/// connection request described by the socket address at `addr` and of structure length `addrlen`
/// and using modifier flags in `flags`.
///
/// Direct descriptors are [`io_uring`] private file descriptors. They avoid some of the overhead
/// associated with thread shared file tables and can be used in any [`io_uring`] request that
/// takes a file descriptor. This function creates such direct descriptors. Subsequent to their
/// creation, they can be used by setting [`IOSQE_FIXED_FILE`] in the SQE flags member, and setting
/// the SQE `fd` field to the direct descriptor value rather than the regular file descriptor.
/// Direct descriptors are managed like registered files.
///
/// To use this function, the application must have first registered a file table of a desired size
/// using [`io_uring_register_files()`] or [`io_uring_register_files_sparse()`]. Once registered,
/// this allows an entry in that table to be specifically selected through the `file_index`
/// argument. If the specified entry already contains a file, the file will first be removed from
/// the table and closed, consistent with the behavior of updating an existing file with
/// [`io_uring_register_files_update()`]. `file_index` can also be set to
/// [`IORING_FILE_INDEX_ALLOC`] for this variant and an unused table index will be dynamically
/// chosen and returned. If both forms of direct selection will be employed, specific and dynamic,
/// see [`io_uring_register_file_alloc_range()`] for setting up the table so dynamically chosen
/// entries are made against a different range than that targeted by specific requests.
///
/// When a direct descriptor accept request asks for a table slot to be dynamically chosen but
/// there are no free entries, `-ENFILE` is returned as the CQE res.
///
/// See the man page [`accept4(2)`](https://man.archlinux.org/man/accept4.2) for more details of
/// the accept function itself.
#[inline]
pub fn io_uring_prep_accept_direct(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: Option<&mut sockaddr>,
    addrlen: Option<&mut socklen_t>,
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

/// Prepares a multishot accept request
///
/// The submission queue entry is setup to use the file descriptor `fd` to start accepting a
/// connection request described by the socket address at `addr` and of structure length `addrlen`
/// and using modifier flags in `flags`.
///
/// This function allows an application to issue a single accept request, which will repeatedly
/// trigger a CQE when a connection request comes in. Like other multishot type requests, the
/// application should look at the CQE flags and see if [`IORING_CQE_F_MORE`] is set on completion
/// as an indication of whether or not the accept request will generate further CQEs. Note that
/// setting `addr` and `addrlen` may not make a lot of sense, as the same value would be used for
/// every accepted connection. This means that the data written to `addr` may be overwritten by a
/// new connection before the application has had time to process a past connection. If the
/// application knows that a new connection cannot come in before a previous one has been
/// processed, it may be used as expected.
///
/// See the man page [`accept4(2)`](https://man.archlinux.org/man/accept4.2) for more details of
/// the accept function itself.
#[inline]
pub fn io_uring_prep_multishot_accept(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: Option<&mut sockaddr>,
    addrlen: Option<&mut socklen_t>,
    flags: u32,
) {
    io_uring_prep_accept(sqe, fd, addr, addrlen, flags);
    sqe.ioprio |= IORING_ACCEPT_MULTISHOT as u16;
}

/// Prepares an accept request
///
/// The submission queue entry is setup to use the file descriptor `fd` to start accepting a
/// connection request described by the socket address at `addr` and of structure length `addrlen`
/// and using modifier flags in `flags`.
///
/// Direct descriptors are [`io_uring`] private file descriptors. They avoid some of the overhead
/// associated with thread shared file tables and can be used in any [`io_uring`] request that
/// takes a file descriptor. This function creates such direct descriptors. Subsequent to their
/// creation, they can be used by setting [`IOSQE_FIXED_FILE`] in the SQE flags member, and setting
/// the SQE `fd` field to the direct descriptor value rather than the regular file descriptor.
/// Direct descriptors are managed like registered files.
///
/// To use this function, the application must have first registered a file table of a desired size
/// using [`io_uring_register_files()`] or [`io_uring_register_files_sparse()`]. Once registered,
/// this allows an entry in that table to be specifically selected through the `file_index`
/// argument. If the specified entry already contains a file, the file will first be removed from
/// the table and closed, consistent with the behavior of updating an existing file with
/// [`io_uring_register_files_update()`]. `file_index` can also be set to
/// [`IORING_FILE_INDEX_ALLOC`] for this variant and an unused table index will be dynamically
/// chosen and returned. This function will have an unused table index dynamically chosen and
/// returned for each connection accepted. If both forms of direct selection will be employed,
/// specific and dynamic, see [`io_uring_register_file_alloc_range()`] for setting up the table so
/// dynamically chosen entries are made against a different range than that targeted by specific
/// requests.
///
/// When a direct descriptor accept request asks for a table slot to be dynamically chosen but
/// there are no free entries, `-ENFILE` is returned as the CQE res.
///
/// This function allows an application to issue a single accept request, which will repeatedly
/// trigger a CQE when a connection request comes in. Like other multishot type requests, the
/// application should look at the CQE flags and see if [`IORING_CQE_F_MORE`] is set on completion
/// as an indication of whether or not the accept request will generate further CQEs. Note that
/// setting `addr` and `addrlen` may not make a lot of sense, as the same value would be used for
/// every accepted connection. This means that the data written to `addr` may be overwritten by a
/// new connection before the application has had time to process a past connection. If the
/// application knows that a new connection cannot come in before a previous one has been
/// processed, it may be used as expected.
///
/// See the man page [`accept4(2)`](https://man.archlinux.org/man/accept4.2) for more details of
/// the accept function itself.
#[inline]
pub fn io_uring_prep_multishot_accept_direct(
    sqe: &mut io_uring_sqe,
    fd: i32,
    addr: Option<&mut sockaddr>,
    addrlen: Option<&mut socklen_t>,
    flags: u32,
) {
    io_uring_prep_multishot_accept(sqe, fd, addr, addrlen, flags);
    io_uring_set_target_fixed_file(sqe, (IORING_FILE_INDEX_ALLOC - 1) as u32);
}

/// Prepares a cancellation request
///
/// The submission queue entry is prepared to cancel an existing request identified by `user_data`.
/// For the `flags` argument, see below.
///
/// This function is identical to [`io_uring_prep_cancel()`], except it takes a 64-bit integer
/// rather than a pointer type.
///
/// The cancellation request will attempt to find the previously issued request identified by
/// `user_data` and cancel it. The identifier is what the previously issued request has in their
/// `user_data` field in the SQE.
///
/// By default, the first request matching the criteria given will be cancelled. This can be
/// modified with any of the following flags passed in:
/// * [`IORING_ASYNC_CANCEL_ALL`]: Cancel all requests that match the given criteria, rather than
/// just cancelling the first one found.
/// * [`IORING_ASYNC_CANCEL_FD`]: Match based on the file descriptor used in the original request
/// rather than the `user_data`. This is what [`io_uring_prep_cancel_fd()`] sets up.
/// * [`IORING_ASYNC_CANCEL_ANY`]: Match any request in the ring, regardless of `user_data` or file
/// descriptor. Can be used to cancel any pending request in the ring.
#[inline]
pub fn io_uring_prep_cancel64(sqe: &mut io_uring_sqe, user_data: u64, flags: i32) {
    io_uring_prep_rw_null(IORING_OP_ASYNC_CANCEL, sqe, -1, 0, 0);
    sqe.__bindgen_anon_2.addr = user_data;
    sqe.__bindgen_anon_3.cancel_flags = flags as u32;
}

/// Prepares a cancellation request
///
/// The submission queue entry is prepared to cancel an existing request identified by `user_data`.
/// For the `flags` argument, see below.
///
/// This function is identical to [`io_uring_prep_cancel64()`], except it takes a pointer type
/// rather than a 64-bit integer
///
/// The cancellation request will attempt to find the previously issued request identified by
/// `user_data` and cancel it. The identifier is what the previously issued request has in their
/// `user_data` field in the SQE.
///
/// By default, the first request matching the criteria given will be cancelled. This can be
/// modified with any of the following flags passed in:
/// * [`IORING_ASYNC_CANCEL_ALL`]: Cancel all requests that match the given criteria, rather than
/// just cancelling the first one found.
/// * [`IORING_ASYNC_CANCEL_FD`]: Match based on the file descriptor used in the original request
/// rather than the `user_data`. This is what [`io_uring_prep_cancel_fd()`] sets up.
/// * [`IORING_ASYNC_CANCEL_ANY`]: Match any request in the ring, regardless of `user_data` or file
/// descriptor. Can be used to cancel any pending request in the ring.
#[inline]
pub fn io_uring_prep_cancel<T>(sqe: &mut io_uring_sqe, user_data: NonNull<T>, flags: i32) {
    io_uring_prep_cancel64(sqe, user_data.as_ptr() as u64, flags);
}

/// Prepares a cancellation request
///
/// The submission queue entry is prepared to cancel an existing request that used the file
/// descriptor `fd`. For the `flags` argument, see below.
///
/// The cancellation request will attempt to find the previously issued request that used `fd` as
/// the file descriptor and cancel it.
///
/// By default, the first request matching the criteria given will be cancelled. This can be
/// modified with any of the following flags passed in:
/// * [`IORING_ASYNC_CANCEL_ALL`]: Cancel all requests that match the given criteria, rather than
/// just cancelling the first one found.
/// * [`IORING_ASYNC_CANCEL_FD`]: Match based on the file descriptor used in the original request
/// rather than the `user_data`. This is what [`io_uring_prep_cancel_fd()`] sets up.
/// * [`IORING_ASYNC_CANCEL_ANY`]: Match any request in the ring, regardless of `user_data` or file
/// descriptor. Can be used to cancel any pending request in the ring.
#[inline]
pub fn io_uring_prep_cancel_fd(sqe: &mut io_uring_sqe, fd: i32, flags: u32) {
    io_uring_prep_rw_null(IORING_OP_ASYNC_CANCEL, sqe, fd, 0, 0);
    sqe.__bindgen_anon_3.cancel_flags = flags | IORING_ASYNC_CANCEL_FD;
}

/// Prepares a timeout request for linked SQEs
///
/// The submission queue entry sets up a timeout specified by `ts`. The `flags` argument holds
/// modifier flags for the timeout behavior of the request.
///
/// The `ts` argument must be filled in with the appropriate information for the timeout.
///
/// The flags argument may contain:
/// * [`IORING_TIMEOUT_ABS`]: The value specified in `ts` is an absolute value rather than a
/// relative one.
/// * [`IORING_TIMEOUT_BOOTTIME`]: The boottime clock source should be used.
/// * [`IORING_TIMEOUT_REALTIME`]: The realtime clock source should be used.
/// * [`IORING_TIMEOUT_ETIME_SUCCESS`]: Consider an expired timeout a success in terms of the
/// posted completion. Normally a timeout that triggers would return in a -ETIME CQE res value.
///
/// It is invalid to create a chain (linked SQEs) consisting only of a link timeout request. If all
/// the requests in the chain are completed before timeout, then the link timeout request gets
/// cancelled. Upon timeout, all the uncompleted requests in the chain get cancelled.
#[inline]
pub fn io_uring_prep_link_timeout(sqe: &mut io_uring_sqe, ts: &mut timespec, flags: u32) {
    io_uring_prep_rw(IORING_OP_LINK_TIMEOUT, sqe, -1, Some(ts), 1, 0);
    sqe.__bindgen_anon_3.timeout_flags = flags;
}

/// Prepares a connect request
///
/// The submission queue entry is setup to use the file descriptor `fd` to start connecting to the
/// destination described by the socket address at `addr` and of structure length `addrlen`.
#[inline]
pub fn io_uring_prep_connect(sqe: &mut io_uring_sqe, fd: i32, addr: &sockaddr, addrlen: socklen_t) {
    io_uring_prep_rw(IORING_OP_CONNECT, sqe, fd, Some(addr), 0, addrlen as u64);
}

#[inline]
pub fn io_uring_prep_files_update(sqe: &mut io_uring_sqe, fds: &mut [i32], offset: i32) {
    io_uring_prep_rw_buf(IORING_OP_FILES_UPDATE, sqe, -1, Some(fds), offset as u64);
}

#[inline]
pub fn io_uring_prep_fallocate(sqe: &mut io_uring_sqe, fd: i32, mode: i32, offset: u64, len: u64) {
    io_uring_prep_rw_null(IORING_OP_FALLOCATE, sqe, fd, mode as u32, offset);
    sqe.__bindgen_anon_2.addr = len;
}

#[inline]
pub fn io_uring_prep_openat(
    sqe: &mut io_uring_sqe,
    dfd: i32,
    path: OsString,
    flags: i32,
    mode: mode_t,
) {
    io_uring_prep_rw(IORING_OP_OPENAT, sqe, dfd, Some(&path), mode, 0);
    sqe.__bindgen_anon_3.open_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_openat_direct(
    sqe: &mut io_uring_sqe,
    dfd: i32,
    path: OsString,
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
    io_uring_prep_rw_null(IORING_OP_CLOSE, sqe, fd, 0, 0);
}

#[inline]
pub fn io_uring_prep_close_direct(sqe: &mut io_uring_sqe, file_index: u32) {
    io_uring_prep_close(sqe, 0);
    io_uring_set_target_fixed_file(sqe, file_index);
}

#[inline]
pub fn io_uring_prep_read(sqe: &mut io_uring_sqe, fd: i32, buf: &mut [u8], offset: u64) {
    io_uring_prep_rw_buf(IORING_OP_READ, sqe, fd, Some(buf), offset);
}

#[inline]
pub fn io_uring_prep_write(sqe: &mut io_uring_sqe, fd: i32, buf: &[u8], offset: u64) {
    io_uring_prep_rw_buf(IORING_OP_WRITE, sqe, fd, Some(buf), offset);
}

// TODO: statx fadvise madvise

#[inline]
pub fn io_uring_prep_send(sqe: &mut io_uring_sqe, sockfd: i32, buf: &[u8], flags: i32) {
    io_uring_prep_rw_buf(IORING_OP_SEND, sqe, sockfd, Some(buf), 0);
    sqe.__bindgen_anon_3.msg_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_send_set_addr(
    sqe: &mut io_uring_sqe,
    dest_addr: NonNull<sockaddr>,
    addr_len: u16,
) {
    sqe.__bindgen_anon_1.addr2 = dest_addr.as_ptr() as u64;
    sqe.__bindgen_anon_5.__bindgen_anon_1.addr_len = addr_len;
}

#[inline]
pub fn io_uring_prep_sendto(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: &mut [u8],
    flags: i32,
    addr: NonNull<sockaddr>,
    addr_len: u16,
) {
    io_uring_prep_send(sqe, sockfd, buf, flags);
    io_uring_prep_send_set_addr(sqe, addr, addr_len);
}

#[inline]
pub fn io_uring_prep_send_zc(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: &mut [u8],
    flags: i32,
    zc_flags: u32,
) {
    io_uring_prep_rw_buf(IORING_OP_SEND_ZC, sqe, sockfd, Some(buf), 0);
    sqe.__bindgen_anon_3.msg_flags = flags as u32;
    sqe.ioprio = zc_flags as u16;
}

#[inline]
pub fn io_uring_prep_send_zc_fixed(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: &mut [u8],
    flags: i32,
    zc_flags: u32,
    buf_index: u32,
) {
    io_uring_prep_send_zc(sqe, sockfd, buf, flags, zc_flags);
    sqe.ioprio |= IORING_RECVSEND_FIXED_BUF as u16;
    sqe.__bindgen_anon_4.buf_index = buf_index as u16;
}

#[inline]
pub fn io_uring_prep_sendmsg_zc(sqe: &mut io_uring_sqe, fd: i32, msg: &msghdr, flags: u32) {
    io_uring_prep_sendmsg(sqe, fd, msg, flags);
    sqe.opcode = IORING_OP_SENDMSG_ZC as u8;
}

#[inline]
pub fn io_uring_prep_recv(sqe: &mut io_uring_sqe, sockfd: i32, buf: &mut [u8], flags: i32) {
    io_uring_prep_rw_buf(IORING_OP_RECV, sqe, sockfd, Some(buf), 0);
    sqe.__bindgen_anon_3.msg_flags = flags as u32;
}

#[inline]
pub fn io_uring_prep_recv_multishot(
    sqe: &mut io_uring_sqe,
    sockfd: i32,
    buf: &mut [u8],
    flags: i32,
) {
    io_uring_prep_recv(sqe, sockfd, buf, flags);
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
    io_uring_prep_rw_null(IORING_OP_MSG_RING, sqe, fd, len, data);
    sqe.__bindgen_anon_3.msg_ring_flags = IORING_MSG_RING_FLAGS_PASS | flags;
    sqe.__bindgen_anon_5.file_index = cqe_flags;
}

#[inline]
pub fn io_uring_prep_msg_ring(sqe: &mut io_uring_sqe, fd: i32, len: u32, data: u64, flags: u32) {
    io_uring_prep_rw_null(IORING_OP_MSG_RING, sqe, fd, len, data);
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
        Some(&(IORING_MSG_SEND_FD as *const usize)),
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
    io_uring_prep_rw_null(
        IORING_OP_SOCKET,
        sqe,
        domain,
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
pub fn io_uring_sq_ready(ring: &io_uring) -> u32 {
    let mut khead: u32 = unsafe { *ring.sq.khead };

    if ring.flags & IORING_SETUP_SQPOLL == 0 {
        let khead_ptr: &mut AtomicU32 = unsafe { &mut *ring.sq.khead.cast() };
        khead = khead_ptr.load(Ordering::Acquire);
    }

    ring.sq.sqe_tail - khead
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_sq_space_left(ring: &io_uring) -> u32 {
    ring.sq.ring_entries - io_uring_sq_ready(ring)
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_sqring_wait(ring: &mut io_uring) -> i32 {
    if ring.flags & IORING_SETUP_SQPOLL == 0 || io_uring_sq_space_left(ring) > 0 {
        0
    } else {
        __io_uring_sqring_wait(ring)
    }
}

#[inline]
pub fn io_uring_cq_ready(cq: &io_uring_cq) -> u32 {
    // SAFETY: io_uring_cq was initialized by kernel and has valid references for khead and ktail
    let tail: &AtomicU32 = unsafe { &*cq.ktail.cast() };
    tail.load(Ordering::Acquire) - unsafe { *cq.khead }
}

#[inline]
pub fn io_uring_cq_has_overflown(sq: &io_uring_sq) -> bool {
    let kflags: &AtomicU32 = unsafe { &*sq.kflags.cast() };

    // SAFETY: io_uring_sq was initialized by kernel and has valid references for kflags
    kflags.load(Ordering::Relaxed) & IORING_SQ_CQ_OVERFLOW > 0
}

/// # Safety
/// honestly idk just be valid
#[inline]
pub unsafe fn io_uring_wait_cqe_nr(
    ring: &mut io_uring,
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

    let tail_ptr: &mut AtomicU32 = &mut *(*ring).cq.ktail.cast();
    loop {
        let tail = tail_ptr.load(Ordering::Acquire);
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
pub unsafe fn io_uring_peek_cqe(ring: &mut io_uring, cqe_ptr: *mut *mut io_uring_cqe) -> i32 {
    if io_uring_peek_cqe_internal(ring, cqe_ptr, ptr::null_mut()) == 0 && !cqe_ptr.is_null() {
        return 0;
    }

    io_uring_wait_cqe_nr(ring, cqe_ptr, 0)
}

/// # Safety
/// `ring` must point to a valid and initialized `io_uring`
#[inline]
pub unsafe fn io_uring_wait_cqe(ring: &mut io_uring, cqe_ptr: *mut *mut io_uring_cqe) -> i32 {
    if io_uring_peek_cqe_internal(ring, cqe_ptr, ptr::null_mut()) == 0 && !cqe_ptr.is_null() {
        return 0;
    }

    io_uring_wait_cqe_nr(ring, cqe_ptr, 1)
}

#[inline]
pub fn io_uring_get_sqe(ring: &mut io_uring) -> Option<&mut io_uring_sqe> {
    unsafe {
        let sq: &mut io_uring_sq = &mut ring.sq;
        let next = sq.sqe_tail + 1;
        let mut shift = 0;

        if ring.flags & IORING_SETUP_SQE128 > 0 {
            shift = 1;
        }

        let head = if ring.flags & IORING_SETUP_SQPOLL == 0 {
            let khead_ptr: &AtomicU32 = &*sq.khead.cast();
            khead_ptr.load(Ordering::Relaxed)
        } else {
            let khead_ptr: &AtomicU32 = &*sq.khead.cast();
            khead_ptr.load(Ordering::Acquire)
        };

        if next - head <= sq.ring_entries {
            let sqe = sq
                .sqes
                .offset(((sq.sqe_tail & sq.ring_mask) << shift) as isize);

            sq.sqe_tail = next;

            return Some(&mut *sqe);
        }

        None
    }
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
// TODO: find better type for addr
#[inline]
pub unsafe fn io_uring_buf_ring_add(
    br: &mut io_uring_buf_ring,
    addr: *mut c_void,
    len: u32,
    bid: u16,
    mask: i32,
    buf_offset: i32,
) {
    let buf: *mut io_uring_buf = ptr::addr_of_mut!(
        br.__bindgen_anon_1.bufs.as_mut()[((br.__bindgen_anon_1.__bindgen_anon_1.as_ref().tail
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
pub unsafe fn io_uring_buf_ring_advance(br: &mut io_uring_buf_ring, count: i32) {
    let new_tail = br.__bindgen_anon_1.__bindgen_anon_1.as_ref().tail + count as u16;
    let tail: &mut AtomicU16 =
        &mut *ptr::addr_of_mut!(br.__bindgen_anon_1.__bindgen_anon_1.as_mut().tail).cast();

    tail.store(new_tail, Ordering::Release);
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
