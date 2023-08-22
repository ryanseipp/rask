use libc::{iovec, mode_t, msghdr, sockaddr, socklen_t, timespec};
use std::{
    ffi::OsString,
    ptr::{self, NonNull},
};

use super::{
    IORING_ACCEPT_MULTISHOT, IORING_ASYNC_CANCEL_FD, IORING_FILE_INDEX_ALLOC,
    IORING_MSG_RING_FLAGS_PASS, IORING_POLL_ADD_MULTI, IORING_RECVSEND_FIXED_BUF,
    IORING_RECV_MULTISHOT, IORING_TIMEOUT_UPDATE,
};

const IOSQE_FIXED_FILE_BIT: u32 = 0;
const IOSQE_IO_DRAIN_BIT: u32 = 1;
const IOSQE_IO_LINK_BIT: u32 = 2;
const IOSQE_IO_HARDLINK_BIT: u32 = 3;
const IOSQE_ASYNC_BIT: u32 = 4;
const IOSQE_BUFFER_SELECT_BIT: u32 = 5;
const IOSQE_CQE_SKIP_SUCCESS_BIT: u32 = 6;
const IORING_OP_NOP: u8 = 0;
const IORING_OP_READV: u8 = 1;
const IORING_OP_WRITEV: u8 = 2;
const IORING_OP_FSYNC: u8 = 3;
const IORING_OP_READ_FIXED: u8 = 4;
const IORING_OP_WRITE_FIXED: u8 = 5;
const IORING_OP_POLL_ADD: u8 = 6;
const IORING_OP_POLL_REMOVE: u8 = 7;
const IORING_OP_SYNC_FILE_RANGE: u8 = 8;
const IORING_OP_SENDMSG: u8 = 9;
const IORING_OP_RECVMSG: u8 = 10;
const IORING_OP_TIMEOUT: u8 = 11;
const IORING_OP_TIMEOUT_REMOVE: u8 = 12;
const IORING_OP_ACCEPT: u8 = 13;
const IORING_OP_ASYNC_CANCEL: u8 = 14;
const IORING_OP_LINK_TIMEOUT: u8 = 15;
const IORING_OP_CONNECT: u8 = 16;
const IORING_OP_FALLOCATE: u8 = 17;
const IORING_OP_OPENAT: u8 = 18;
const IORING_OP_CLOSE: u8 = 19;
const IORING_OP_FILES_UPDATE: u8 = 20;
const IORING_OP_STATX: u8 = 21;
const IORING_OP_READ: u8 = 22;
const IORING_OP_WRITE: u8 = 23;
const IORING_OP_FADVISE: u8 = 24;
const IORING_OP_MADVISE: u8 = 25;
const IORING_OP_SEND: u8 = 26;
const IORING_OP_RECV: u8 = 27;
const IORING_OP_OPENAT2: u8 = 28;
const IORING_OP_EPOLL_CTL: u8 = 29;
const IORING_OP_SPLICE: u8 = 30;
const IORING_OP_PROVIDE_BUFFERS: u8 = 31;
const IORING_OP_REMOVE_BUFFERS: u8 = 32;
const IORING_OP_TEE: u8 = 33;
const IORING_OP_SHUTDOWN: u8 = 34;
const IORING_OP_RENAMEAT: u8 = 35;
const IORING_OP_UNLINKAT: u8 = 36;
const IORING_OP_MKDIRAT: u8 = 37;
const IORING_OP_SYMLINKAT: u8 = 38;
const IORING_OP_LINKAT: u8 = 39;
const IORING_OP_MSG_RING: u8 = 40;
const IORING_OP_FSETXATTR: u8 = 41;
const IORING_OP_SETXATTR: u8 = 42;
const IORING_OP_FGETXATTR: u8 = 43;
const IORING_OP_GETXATTR: u8 = 44;
const IORING_OP_SOCKET: u8 = 45;
const IORING_OP_URING_CMD: u8 = 46;
const IORING_OP_SEND_ZC: u8 = 47;
const IORING_OP_SENDMSG_ZC: u8 = 48;
const IORING_OP_LAST: u8 = 49;
const IORING_MSG_DATA: u8 = 0;
const IORING_MSG_SEND_FD: u8 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
struct Union1S {
    cmd_op: u32,
    pad1: u32,
}

#[repr(C)]
union Union1 {
    off: u64,
    addr2: u64,
    op: Union1S,
}

#[repr(C)]
union Union2 {
    addr: u64,
    splice_off_in: u64,
}

#[repr(C)]
union Flags {
    rw_flags: i32,
    fsync_flags: u32,
    poll_events: u16,
    poll32_events: u32,
    sync_range_flags: u32,
    msg_flags: u32,
    timeout_flags: u32,
    accept_flags: u32,
    cancel_flags: u32,
    open_flags: u32,
    statx_flags: u32,
    fadvise_advice: u32,
    splice_flags: u32,
    rename_flags: u32,
    unlink_flags: u32,
    hardlink_flags: u32,
    xattr_flags: u32,
    msg_ring_flags: u32,
    uring_cmd_flags: u32,
}

#[repr(C, packed)]
union BufferSelect {
    buf_index: u16,
    buf_group: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Union5S {
    addr_len: u16,
    pad3: [u16; 1],
}

#[repr(C)]
union Union5 {
    splice_fd_in: i32,
    file_index: u32,
    addr_len: Union5S,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Union6S {
    addr3: u64,
    pad2: [u64; 1],
}

union Union6 {
    addr: Union6S,
    cmd: [u8; 0],
}

#[repr(C)]
pub struct IoUringSqe {
    opcode: u8,
    flags: u8,
    ioprio: u16,
    fd: i32,
    union1: Union1,
    union2: Union2,
    len: u32,
    op_flags: Flags,
    user_data: u64,
    buffer_select: BufferSelect,
    personality: u16,
    union5: Union5,
    union6: Union6,
}

impl IoUringSqe {
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
    pub fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    /// Stores a pointer with the submission queue entry.
    ///
    /// After the caller has requested a submission queue entry (SQE) with [`io_uring_get_sqe`], they
    /// can associate a data pointer or value with the SQE. Once the completion arrives, the function
    /// [`io_uring_cqe_get_data`] can be called to retrieve the data pointer or value associated with
    /// the submitted request.
    #[inline]
    pub fn set_data<T>(&mut self, data: NonNull<T>) {
        self.user_data = data.as_ptr() as u64;
    }

    /// Stores a 64-bit data value with the submission queue entry.
    ///
    /// After the caller has requested a submission queue entry (SQE) with [`io_uring_get_sqe()`], they
    /// can associate a data pointer or value with the SQE. Once the completion arrives, the function
    /// [`io_uring_cqe_get_data64()`] can be called to retrieve the data pointer or value associated
    /// with the submitted request.
    #[inline]
    pub fn set_data64(&mut self, data: u64) {
        self.user_data = data;
    }

    #[inline]
    fn prep_rw<T>(&mut self, op: u8, fd: i32, addr: Option<&T>, len: u32, offset: u64) {
        self.opcode = op;
        self.flags = 0;
        self.ioprio = 0;
        self.fd = fd;
        self.union1.off = offset;
        self.union2.addr = addr.map_or(ptr::null(), |a| a) as u64;
        self.len = len;
    }

    #[inline]
    fn prep_rw_buf<T>(&mut self, op: u8, fd: i32, addr: Option<&[T]>, offset: u64) {
        self.opcode = op;
        self.flags = 0;
        self.ioprio = 0;
        self.fd = fd;
        self.union1.off = offset;
        self.union2.addr = addr.map_or(ptr::null(), |a| a.as_ptr()) as u64;
        self.len = addr.map_or(0, |a| a.len()) as u32;
    }

    #[inline]
    fn prep_rw_null(&mut self, op: u8, fd: i32, len: u32, offset: u64) {
        self.opcode = op;
        self.flags = 0;
        self.ioprio = 0;
        self.fd = fd;
        self.union1.off = offset;
        self.union2.addr = ptr::null::<usize>() as u64;
        self.len = len;
    }

    /// Prepares a splice request
    ///
    /// The submission queue entry is setup to use as input the file descriptor `fd_id` at offset
    /// `off_in`, splicing data to the file descriptor `fd_out` at offset `off_out`. `nbytes` bytes of
    /// data should be spliced between the two descriptors. `splice_flags` are modifier flags for the
    /// operation. See [`splice(2)`](https://man.archlinux.org/man/splice.2) for the generic splice
    /// flags.
    #[inline]
    pub fn prep_splice(
        &mut self,
        fd_in: i32,
        off_in: i64,
        fd_out: i32,
        off_out: i64,
        nbytes: u32,
        splice_flags: u32,
    ) {
        self.prep_rw_null(IORING_OP_SPLICE, fd_out, nbytes, off_out as u64);

        self.union2.splice_off_in = off_in as u64;
        self.union5.splice_fd_in = fd_in;
        self.op_flags.splice_flags = splice_flags;
    }

    /// Prepares a tee request
    ///
    /// The submission queue entry is setup to use as input the file descriptor `fd_in` and as output
    /// the file descriptor `fd_out` duplicating `nbytes` bytes worth of data. `splice_flags` are
    /// modifier flags for the operation. See [`tee(2)`](https://man.archlinux.org/man/tee.2) for the
    /// generic splice flags.
    #[inline]
    pub fn prep_tee(&mut self, fd_in: i32, fd_out: i32, nbytes: u32, splice_flags: u32) {
        self.prep_rw_null(IORING_OP_TEE, fd_out, nbytes, 0);

        self.union2.splice_off_in = 0;
        self.union5.splice_fd_in = fd_in;
        self.op_flags.splice_flags = splice_flags;
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
    /// use [`prep_read`] rather than this function, as no state has to be maintained for a
    /// non-vectored IO request. As with any request that passes in data in a struct, that data must
    /// remain valid until the request has been successfully submitted. It need not remain valid until
    /// completion.
    ///
    /// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
    /// earlier) required state to be stable until the completion occurred. Applications can test for
    /// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
    /// [`io_uring_queue_init_params`].
    #[inline]
    pub fn prep_readv(&mut self, fd: i32, iovecs: &[iovec], offset: u64) {
        self.prep_rw_buf(IORING_OP_READV, fd, Some(iovecs), offset)
    }

    /// Prepares a vectored IO read request
    ///
    /// Similar to [`prep_readv`] with the addition of flags. Supported values for flags are:
    /// * `RWF_HIPRI`: High priority request, poll if possible
    /// * `RWF_DSYNC`: per-IO O_DSYNC
    /// * `RWF_SYNC`: per-IO O_SYNC
    /// * `RWF_NOWAIT`: per-IO, return -EAGAIN if operation would block
    /// * `RWF_APPEND`: per-IO O_APPEND
    ///
    /// # Notes
    /// Unless an application explicitly needs to pass in more than one iovec, it is more efficient to
    /// use [`prep_read`] rather than this function, as no state has to be maintained for a
    /// non-vectored IO request. As with any request that passes in data in a struct, that data must
    /// remain valid until the request has been successfully submitted. It need not remain valid until
    /// completion.
    ///
    /// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
    /// earlier) required state to be stable until the completion occurred. Applications can test for
    /// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
    /// [`io_uring_queue_init_params`].
    #[inline]
    pub fn prep_readv2(&mut self, fd: i32, iovecs: &[iovec], offset: u64, flags: i32) {
        self.prep_readv(fd, iovecs, offset);
        self.op_flags.rw_flags = flags;
    }

    /// Prepares an IO read request with a previously registered IO buffer
    ///
    /// This works just like [`prep_read`] except i requires the use of buffers that have been
    /// registered with [`io_uring_register_buffers`]. The `buf` argument must fall within a region
    /// specified by `buf_index` in the previously registered buffer. The buffer need not be aligned
    /// with the start of the registered buffer.
    #[inline]
    pub fn prep_read_fixed(&mut self, fd: i32, buf: &mut [u8], offset: u64, buf_index: i32) {
        self.prep_rw_buf(IORING_OP_READ_FIXED, fd, Some(buf), offset);
        self.buffer_select.buf_index = buf_index as u16;
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
    /// use [`prep_read`] rather than this function, as no state has to be maintained for a
    /// non-vectored IO request. As with any request that passes in data in a struct, that data must
    /// remain valid until the request has been successfully submitted. It need not remain valid until
    /// completion.
    ///
    /// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
    /// earlier) required state to be stable until the completion occurred. Applications can test for
    /// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
    /// [`io_uring_queue_init_params`].
    #[inline]
    pub fn prep_writev(&mut self, fd: i32, iovecs: &[iovec], offset: u64) {
        self.prep_rw_buf(IORING_OP_WRITEV, fd, Some(iovecs), offset)
    }

    /// Prepares a vectored IO write request
    ///
    /// Similar to [`prep_writev`] with the addition of flags. Supported values for flags are:
    /// * `RWF_HIPRI`: High priority request, poll if possible
    /// * `RWF_DSYNC`: per-IO O_DSYNC
    /// * `RWF_SYNC`: per-IO O_SYNC
    /// * `RWF_NOWAIT`: per-IO, return -EAGAIN if operation would block
    /// * `RWF_APPEND`: per-IO O_APPEND
    ///
    /// # Notes
    /// Unless an application explicitly needs to pass in more than one iovec, it is more efficient to
    /// use [`prep_write`] rather than this function, as no state has to be maintained for a
    /// non-vectored IO request. As with any request that passes in data in a struct, that data must
    /// remain valid until the request has been successfully submitted. It need not remain valid until
    /// completion.
    ///
    /// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
    /// earlier) required state to be stable until the completion occurred. Applications can test for
    /// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
    /// [`io_uring_queue_init_params`].
    #[inline]
    pub fn prep_writev2(&mut self, fd: i32, iovecs: &[iovec], offset: u64, flags: i32) {
        self.prep_writev(fd, iovecs, offset);
        self.op_flags.rw_flags = flags;
    }

    /// Prepares an IO write request with a previously registered IO buffer
    ///
    /// This works just like [`prep_write`] except i requires the use of buffers that have been
    /// registered with [`io_uring_register_buffers`]. The `buf` argument must fall within a region
    /// specified by `buf_index` in the previously registered buffer. The buffer need not be aligned
    /// with the start of the registered buffer.
    #[inline]
    pub fn prep_write_fixed(&mut self, fd: i32, buf: &mut [u8], offset: u64, buf_index: i32) {
        self.prep_rw_buf(IORING_OP_WRITE_FIXED, fd, Some(buf), offset);
        self.buffer_select.buf_index = buf_index as u16;
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
    pub fn prep_recvmsg(&mut self, fd: i32, msg: &mut msghdr, flags: u32) {
        self.prep_rw(IORING_OP_RECVMSG, fd, Some(msg), 1, 0);
        self.op_flags.msg_flags = flags;
    }

    /// Prepares a multishot recvmsg request.
    ///
    /// The submission queue entry is setup to use the file descriptor `fd` to start receiving the data
    /// indicated by `msg` with the [`recvmsg(2)`](https://man.archlinux.org/man/recvmsg.2) defined
    /// flags in the `flags` argument.
    ///
    /// Allows the application to issue a single receive request, which repeatedly posts a CQE when
    /// data is available. It requires the [`IOself_BUFFER_SELECT`] flag to be set and no `MSG_WAITALL`
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
    pub fn prep_recvmsg_multishot(&mut self, fd: i32, msg: &mut msghdr, flags: u32) {
        self.prep_recvmsg(fd, msg, flags);
        self.ioprio |= IORING_RECV_MULTISHOT as u16;
    }

    /// Prepares a sendmsg request
    ///
    /// The submission queue entry is setup to use the file descriptor fd to start sending the data
    /// indicated by `msg` with the [`sendmsg(2)`](https://man.archlinux.org/man/sendmsg.2) defined
    /// flags in the `flags` argument.
    ///
    /// # Notes
    /// Using [`IOself_IO_LINK`] with this request type requires the setting of [`MSG_WAITALL`] in the
    /// flags argument, as a short send isn't considered an error condition without that being set.
    ///
    /// As with any request that passes in data in a struct, that data must remain valid until the
    /// request has been successfully submitted. It need not remain valid until completion.
    /// Once a request has been submitted, the in-kernel state is stable. Very early kernels (5.4 and
    /// earlier) required state to be stable until the completion occurred. Applications can test for
    /// this behavior by inspecting the [`IORING_FEAT_SUBMIT_STABLE`] flag passed back from
    /// [`io_uring_queue_init_params`].
    #[inline]
    pub fn prep_sendmsg(&mut self, fd: i32, msg: &msghdr, flags: u32) {
        self.prep_rw(IORING_OP_SENDMSG, fd, Some(msg), 1, 0);
        self.op_flags.msg_flags = flags;
    }

    /// Prepares a poll request
    ///
    /// The submission queue entry is setup to use the file descriptor `fd` that should get polled,
    /// with the events desired specified in the `poll_mask` argument.
    ///
    /// When the specified event has triggered, a completion CQE is posted and no more events will be
    /// generated by the poll request.
    #[inline]
    pub fn prep_poll_add(&mut self, fd: i32, poll_mask: u32) {
        self.prep_rw_null(IORING_OP_POLL_ADD, fd, 0, 0);
        self.op_flags.poll32_events = poll_mask.to_le();
    }

    /// Prepares a poll request
    ///
    /// The submission queue entry is setup to use the file descriptor `fd` that should get polled,
    /// with the events desired specified in the `poll_mask` argument.
    ///
    /// Behaves identically to [`self.prep_poll_add(`] in terms of events, but persists across
    /// notifications and will repeatedly post notifications for the same registration. A CQE posted
    /// from a multishot poll request will have [`IORING_CQE_F_MORE`] set in the CQE flags member,
    /// indicating that the application should expect more completions from this request. If the
    /// multishot poll request gets terminater or experiences an error, this flag will not be set in
    /// the CQE. If this happens, the application should not expect further CQEs from the original
    /// request and must reissue a new one if it still wishes to get notifications on this file
    /// descriptor.
    #[inline]
    pub fn prep_poll_multishot(&mut self, fd: i32, poll_mask: u32) {
        self.prep_poll_add(fd, poll_mask);
        self.len = IORING_POLL_ADD_MULTI;
    }

    /// Prepares a poll removal request
    ///
    /// The submission queue entry is setup to remove a poll request identified by `user_data`.
    ///
    /// Works like [`self.prep_cancel(`] except only looks for poll requests. Apart from that,
    /// behavior is identical.
    #[inline]
    pub fn prep_poll_remove(&mut self, user_data: u64) {
        self.prep_rw_null(IORING_OP_POLL_REMOVE, -1, 0, 0);
        self.union2.addr = user_data
    }

    /// Prepares a poll update request
    ///
    /// The submission queue entry is setup to update a poll request identified by `old_user_data`,
    /// replacing it with the `new_user_data` information. The `poll_mask` arguments contains the new
    /// mask to use for the poll request, and flags argument contains modifier flags telling io_uring
    /// what fields to update.
    #[inline]
    pub fn prep_poll_update(
        &mut self,
        old_user_data: u64,
        new_user_data: u64,
        poll_mask: u32,
        flags: u32,
    ) {
        self.prep_rw_null(IORING_OP_POLL_REMOVE, -1, flags, new_user_data);
        self.union2.addr = old_user_data;
        self.op_flags.poll32_events = poll_mask.to_le();
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
    /// It's possible to specify a range to sync, if one is desired. If the `off` field of the self is
    /// set to non-zero, then that indicates the offset to start syncing at. If len is set in the self,
    /// then that indicates the size in bytes to sync from the offset. Note that these fields are not
    /// accepted by this helper, so they have to be set manually in the self after calling this prep
    /// helper.
    #[inline]
    pub fn prep_fsync(&mut self, fd: i32, fsync_flags: u32) {
        self.prep_rw_null(IORING_OP_FSYNC, fd, 0, 0);
        self.op_flags.fsync_flags = fsync_flags;
    }

    /// Prepares a nop (no operation) request
    ///
    /// The submission queue entry self does not require any additional setup.
    #[inline]
    pub fn prep_nop(&mut self) {
        self.prep_rw_null(IORING_OP_NOP, -1, 0, 0);
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
    pub fn prep_timeout(&mut self, ts: &mut timespec, count: u32, flags: u32) {
        self.prep_rw(IORING_OP_TIMEOUT, -1, Some(ts), 1, count as u64);
        self.op_flags.timeout_flags = flags;
    }

    /// Cancels an existing timeout request
    ///
    /// The submission queue entry is setup to arm a timeout removal specified by `user_data` and with
    /// modifier flags given by `flags`.
    ///
    /// The timeout remove command does not currently accept any flags.
    #[inline]
    pub fn prep_timeout_remove(&mut self, user_data: u64, flags: u32) {
        self.prep_rw_null(IORING_OP_TIMEOUT_REMOVE, -1, 0, 0);
        self.union2.addr = user_data;
        self.op_flags.timeout_flags = flags;
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
    pub fn prep_timeout_update(&mut self, ts: &mut timespec, user_data: u64, flags: u32) {
        self.prep_rw_null(IORING_OP_TIMEOUT_REMOVE, -1, 0, ts as *mut timespec as u64);
        self.union2.addr = user_data;
        self.op_flags.timeout_flags = flags | IORING_TIMEOUT_UPDATE;
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
    pub fn prep_accept(
        &mut self,
        fd: i32,
        addr: Option<&mut sockaddr>,
        addrlen: Option<&mut socklen_t>,
        flags: u32,
    ) {
        self.prep_rw(
            IORING_OP_ACCEPT,
            fd,
            addr.as_ref(),
            0,
            addrlen.map_or(ptr::null_mut(), |a| a as *mut socklen_t) as u64,
        );
        self.op_flags.accept_flags = flags;
    }

    #[inline]
    fn io_uring_set_target_fixed_file(&mut self, file_index: u32) {
        self.union5.file_index = file_index + 1;
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
    /// creation, they can be used by setting [`IOself_FIXED_FILE`] in the self flags member, and setting
    /// the self `fd` field to the direct descriptor value rather than the regular file descriptor.
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
    pub fn prep_accept_direct(
        &mut self,
        fd: i32,
        addr: Option<&mut sockaddr>,
        addrlen: Option<&mut socklen_t>,
        flags: u32,
        file_index: u32,
    ) {
        self.prep_accept(fd, addr, addrlen, flags);
        let file_index = if file_index as i32 == IORING_FILE_INDEX_ALLOC {
            file_index - 1
        } else {
            file_index
        };

        self.io_uring_set_target_fixed_file(file_index);
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
    pub fn prep_multishot_accept(
        &mut self,
        fd: i32,
        addr: Option<&mut sockaddr>,
        addrlen: Option<&mut socklen_t>,
        flags: u32,
    ) {
        self.prep_accept(fd, addr, addrlen, flags);
        self.ioprio |= IORING_ACCEPT_MULTISHOT as u16;
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
    /// creation, they can be used by setting [`IOself_FIXED_FILE`] in the self flags member, and setting
    /// the self `fd` field to the direct descriptor value rather than the regular file descriptor.
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
    pub fn prep_multishot_accept_direct(
        &mut self,
        fd: i32,
        addr: Option<&mut sockaddr>,
        addrlen: Option<&mut socklen_t>,
        flags: u32,
    ) {
        self.prep_multishot_accept(fd, addr, addrlen, flags);
        self.io_uring_set_target_fixed_file((IORING_FILE_INDEX_ALLOC - 1) as u32);
    }

    /// Prepares a cancellation request
    ///
    /// The submission queue entry is prepared to cancel an existing request identified by `user_data`.
    /// For the `flags` argument, see below.
    ///
    /// This function is identical to [`self.prep_cancel(`], except it takes a 64-bit integer
    /// rather than a pointer type.
    ///
    /// The cancellation request will attempt to find the previously issued request identified by
    /// `user_data` and cancel it. The identifier is what the previously issued request has in their
    /// `user_data` field in the self.
    ///
    /// By default, the first request matching the criteria given will be cancelled. This can be
    /// modified with any of the following flags passed in:
    /// * [`IORING_ASYNC_CANCEL_ALL`]: Cancel all requests that match the given criteria, rather than
    /// just cancelling the first one found.
    /// * [`IORING_ASYNC_CANCEL_FD`]: Match based on the file descriptor used in the original request
    /// rather than the `user_data`. This is what [`self.prep_cancel_fd(`] sets up.
    /// * [`IORING_ASYNC_CANCEL_ANY`]: Match any request in the ring, regardless of `user_data` or file
    /// descriptor. Can be used to cancel any pending request in the ring.
    #[inline]
    pub fn prep_cancel64(&mut self, user_data: u64, flags: i32) {
        self.prep_rw_null(IORING_OP_ASYNC_CANCEL, -1, 0, 0);
        self.union2.addr = user_data;
        self.op_flags.cancel_flags = flags as u32;
    }

    /// Prepares a cancellation request
    ///
    /// The submission queue entry is prepared to cancel an existing request identified by `user_data`.
    /// For the `flags` argument, see below.
    ///
    /// This function is identical to [`self.prep_cancel64(`], except it takes a pointer type
    /// rather than a 64-bit integer
    ///
    /// The cancellation request will attempt to find the previously issued request identified by
    /// `user_data` and cancel it. The identifier is what the previously issued request has in their
    /// `user_data` field in the self.
    ///
    /// By default, the first request matching the criteria given will be cancelled. This can be
    /// modified with any of the following flags passed in:
    /// * [`IORING_ASYNC_CANCEL_ALL`]: Cancel all requests that match the given criteria, rather than
    /// just cancelling the first one found.
    /// * [`IORING_ASYNC_CANCEL_FD`]: Match based on the file descriptor used in the original request
    /// rather than the `user_data`. This is what [`self.prep_cancel_fd(`] sets up.
    /// * [`IORING_ASYNC_CANCEL_ANY`]: Match any request in the ring, regardless of `user_data` or file
    /// descriptor. Can be used to cancel any pending request in the ring.
    #[inline]
    pub fn prep_cancel<T>(&mut self, user_data: NonNull<T>, flags: i32) {
        self.prep_cancel64(user_data.as_ptr() as u64, flags);
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
    /// rather than the `user_data`. This is what [`self.prep_cancel_fd(`] sets up.
    /// * [`IORING_ASYNC_CANCEL_ANY`]: Match any request in the ring, regardless of `user_data` or file
    /// descriptor. Can be used to cancel any pending request in the ring.
    #[inline]
    pub fn prep_cancel_fd(&mut self, fd: i32, flags: u32) {
        self.prep_rw_null(IORING_OP_ASYNC_CANCEL, fd, 0, 0);
        self.op_flags.cancel_flags = flags | IORING_ASYNC_CANCEL_FD;
    }

    /// Prepares a timeout request for linked selfs
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
    /// It is invalid to create a chain (linked selfs) consisting only of a link timeout request. If all
    /// the requests in the chain are completed before timeout, then the link timeout request gets
    /// cancelled. Upon timeout, all the uncompleted requests in the chain get cancelled.
    #[inline]
    pub fn prep_link_timeout(&mut self, ts: &mut timespec, flags: u32) {
        self.prep_rw(IORING_OP_LINK_TIMEOUT, -1, Some(ts), 1, 0);
        self.op_flags.timeout_flags = flags;
    }

    /// Prepares a connect request
    ///
    /// The submission queue entry is setup to use the file descriptor `fd` to start connecting to the
    /// destination described by the socket address at `addr` and of structure length `addrlen`.
    ///
    /// See [`connect(2)`](https://man.archlinux.org/man/connect.2)
    #[inline]
    pub fn prep_connect(&mut self, fd: i32, addr: &sockaddr, addrlen: socklen_t) {
        self.prep_rw(IORING_OP_CONNECT, fd, Some(addr), 0, addrlen as u64);
    }

    /// Prepares a request for updating a number of previously registered file descriptors
    ///
    /// The submission queue entry is setup to use the file descriptor array `fds` to update that
    /// amount of previously registered files starting at `offset`.
    ///
    /// Once a previously registered file is updated with a new one, the existing entry is updated and
    /// then removed from the table. This operation is equivalent to first unregistering that entry and
    /// then inserting a new one, just bundled into one combined operation.
    ///
    /// If `offset` is specified as [`IORING_FILE_INDEX_ALLOC`], io_uring will allocate free direct
    /// descriptors instead of having the application to pass, and store allocated direct descriptors
    /// into `fds`. The CQE's result will return the number of direct descriptors allocated.
    #[inline]
    pub fn prep_files_update(&mut self, fds: &mut [i32], offset: i32) {
        self.prep_rw_buf(IORING_OP_FILES_UPDATE, -1, Some(fds), offset as u64);
    }

    /// Prepares a fallocate request
    ///
    /// The submission queue entry is setup to use the file descriptor pointed to by `fd` to start a
    /// fallocate operation described by `mode` at `offset` and `len` length in bytes.
    ///
    /// See [`fallocate(2)`](https://man.archlinux.org/man/fallocate.2) for more details.
    #[inline]
    pub fn prep_fallocate(&mut self, fd: i32, mode: i32, offset: u64, len: u64) {
        self.prep_rw_null(IORING_OP_FALLOCATE, fd, mode as u32, offset);
        self.union2.addr = len;
    }

    #[inline]
    pub fn prep_openat(&mut self, dfd: i32, path: OsString, flags: i32, mode: mode_t) {
        self.prep_rw(IORING_OP_OPENAT, dfd, Some(&path), mode, 0);
        self.op_flags.open_flags = flags as u32;
    }

    #[inline]
    pub fn prep_openat_direct(
        &mut self,
        dfd: i32,
        path: OsString,
        flags: i32,
        mode: mode_t,
        file_index: u32,
    ) {
        self.prep_openat(dfd, path, flags, mode);
        let file_index = if file_index as i32 == IORING_FILE_INDEX_ALLOC {
            file_index - 1
        } else {
            file_index
        };

        self.io_uring_set_target_fixed_file(file_index);
    }

    /// Prepares a [`close`](https://man.archlinux.org/man/close.2) request
    ///
    /// The submission queue entry is setup to close the file descriptor indicated by `fd`.
    #[inline]
    pub fn prep_close(&mut self, fd: i32) {
        self.prep_rw_null(IORING_OP_CLOSE, fd, 0, 0);
    }

    /// Prepares a [`close`](https://man.archlinux.org/man/close.2) request
    ///
    /// For a direct descriptor close request, the offset is specified by the `file_index` argument
    /// instead of the `fd`. This is identical to unregistering the direct descriptor, and is provided
    /// as a convenience.
    #[inline]
    pub fn prep_close_direct(&mut self, file_index: u32) {
        self.prep_close(0);
        self.io_uring_set_target_fixed_file(file_index);
    }

    /// Prepares an IO [`read`](https://man.archlinux.org/man/read.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `fd` to start reading enough
    /// bytes to fill the buffer `buf` at the specified offset.
    ///
    /// On files that support seeking, if the offset is set to -1, the read operation commences at the
    /// file offset, and the file offset is incremented by the number of bytes read. Not that for an
    /// async API, reading and updating the current file offset may result in unpredictable behavior,
    /// unless access to the file is serialized. It is not encouraged to use this feature, if it's
    /// possible to provide the desired IO offset from the application or library.
    ///
    /// On files that are not capable of seeking, the offset must be 0 or -1.
    #[inline]
    pub fn prep_read(&mut self, fd: i32, buf: &mut [u8], offset: u64) {
        self.prep_rw_buf(IORING_OP_READ, fd, Some(buf), offset);
    }

    /// Prepares an IO [`write`](https://man.archlinux.org/man/write.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `fd` to start writing the entire
    /// buffer `buf` at the specified offset.
    ///
    /// On files that support seeking, if the offset is set to -1, the read operation commences at the
    /// file offset, and the file offset is incremented by the number of bytes read. Not that for an
    /// async API, reading and updating the current file offset may result in unpredictable behavior,
    /// unless access to the file is serialized. It is not encouraged to use this feature, if it's
    /// possible to provide the desired IO offset from the application or library.
    ///
    /// On files that are not capable of seeking, the offset must be 0 or -1.
    #[inline]
    pub fn prep_write(&mut self, fd: i32, buf: &[u8], offset: u64) {
        self.prep_rw_buf(IORING_OP_WRITE, fd, Some(buf), offset);
    }

    // TODO: statx fadvise madvise

    /// Prepares a [`send`](https://man.archlinux.org/man/send.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `sockfd` to start sending the
    /// data from `buf` with modifier flags `flags`.
    ///
    /// Note that using [`IOself_IO_LINK`] with this trype requires the setting of [`MSG_WAITALL`] in
    /// the flags argument, as a short send isn't considered an error condition without that being set.
    ///
    /// See the man page linked above for more details on the underlying request.
    #[inline]
    pub fn prep_send(&mut self, sockfd: i32, buf: &[u8], flags: i32) {
        self.prep_rw_buf(IORING_OP_SEND, sockfd, Some(buf), 0);
        self.op_flags.msg_flags = flags as u32;
    }

    /// Sets a socket destination address specified by `dest_addr` and its length using `addr_len`
    /// parameters. It can be used once `self` is prepared using any of the
    /// [`send(2)`](https://man.archlinux.org/man/send.2) io_uring helpers.
    #[inline]
    pub fn prep_send_set_addr(&mut self, dest_addr: NonNull<sockaddr>, addr_len: u16) {
        self.union1.addr2 = dest_addr.as_ptr() as u64;
        self.union5.addr_len.addr_len = addr_len;
    }

    /// Prepares a [`sendto`](https://man.archlinux.org/man/sendto.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `sockfd` to start sending the
    /// data from `buf` with modifier flags `flags`. The destination address is specified by `addr` and
    /// `addr_len` and must be a valid address for the socket type.
    ///
    /// Note that using [`IOself_IO_LINK`] with this trype requires the setting of [`MSG_WAITALL`] in
    /// the flags argument, as a short send isn't considered an error condition without that being set.
    ///
    /// See the man page linked above for more details on the underlying request.
    #[inline]
    pub fn prep_sendto(
        &mut self,
        sockfd: i32,
        buf: &mut [u8],
        flags: i32,
        addr: NonNull<sockaddr>,
        addr_len: u16,
    ) {
        self.prep_send(sockfd, buf, flags);
        self.prep_send_set_addr(addr, addr_len);
    }

    /// Prepares a zerocopy [`send`](https://man.archlinux.org/man/send.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `sockfd` to start sending data
    /// from `buf` with send modifier flags `flags` and zerocopy modifier flags `zc_flags`.
    ///
    /// This prepares an async zerocopy [`send(2)`](https://man.archlinux.org/man/send.2) request. See
    /// that man page for details. For details on the zerocopy nature of it, see
    /// [`io_uring_enter(2)`](https://man.archlinux.org/man/io_uring_enter.2).
    #[inline]
    pub fn prep_send_zc(&mut self, sockfd: i32, buf: &mut [u8], flags: i32, zc_flags: u32) {
        self.prep_rw_buf(IORING_OP_SEND_ZC, sockfd, Some(buf), 0);
        self.op_flags.msg_flags = flags as u32;
        self.ioprio = zc_flags as u16;
    }

    /// Prepares a zerocopy [`send`](https://man.archlinux.org/man/send.2) request
    ///
    /// This function works just like [`self.prep_send_zc(`] except it requires the use of buffers
    /// that have been registered with [`io_uring_register_buffers()`]. The `buf` argument must fall
    /// within a region specified by `buf_index` in the previously registered buffer. The buffer need
    /// not be aligned with the start of the registered buffer.
    ///
    /// Note that using [`IOself_IO_LINK`] with this trype requires the setting of [`MSG_WAITALL`] in
    /// the flags argument, as a short send isn't considered an error condition without that being set.
    #[inline]
    pub fn prep_send_zc_fixed(
        &mut self,
        sockfd: i32,
        buf: &mut [u8],
        flags: i32,
        zc_flags: u32,
        buf_index: u32,
    ) {
        self.prep_send_zc(sockfd, buf, flags, zc_flags);
        self.ioprio |= IORING_RECVSEND_FIXED_BUF as u16;
        self.buffer_select.buf_index = buf_index as u16;
    }

    /// Prepares a [`sendmsg`](https://man.archlinux.org/man/sendmsg.2) request
    ///
    /// This function accepts the same parameters as [`self.prep_sendmsg(`] but prepares a zerocopy
    /// sendmsg request.
    ///
    /// Note that using [`IOself_IO_LINK`] with this trype requires the setting of [`MSG_WAITALL`] in
    /// the flags argument, as a short send isn't considered an error condition without that being set.
    #[inline]
    pub fn prep_sendmsg_zc(&mut self, fd: i32, msg: &msghdr, flags: u32) {
        self.prep_sendmsg(fd, msg, flags);
        self.opcode = IORING_OP_SENDMSG_ZC;
    }

    /// Prepares a [`recv`](https://man.archlinux.org/man/recv.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `sockfd` to start receiving the
    /// data into the destination buffer `buf` and with modifier flags `flags`.
    ///
    /// After calling this function, additional io_uring internal modifier flags may be set in the self
    /// [`ioprio`](io_uring_self::ioprio) field. The following flags are supported:
    /// * [`IORING_RECVSEND_POLL_FIRST`]
    ///   * If set, io_uring will assume the socket is currently empty and attempting to receive data
    ///   will be unsuccessful. For this case, io_uring will arm internal poll and trigger a receive of
    ///   the data when the socket has data to read. This initial receive attempt can be wasteful for
    ///   the case where the socket is expected to be empty, setting this flag will bypass the initial
    ///   receive attempt and go straight to arming poll. If poll does indicate that data is ready to be
    ///   received, the operation will proceed.
    ///   * Can be used with the CQE [`IORING_CQE_F_SOCK_NONEMPTY`] flag, which io_uring will set on
    ///   CQEs after a [`recv(2)`](https://man.archlinux.org/man/recv.2) or
    ///   [recvmsg(2)](https://man.archlinux.org/man/recvmsg.2) operation. If set, the socket still had
    ///   data to be read after the operation completed. Both these flags are available since 5.19.
    #[inline]
    pub fn prep_recv(&mut self, sockfd: i32, buf: &mut [u8], flags: i32) {
        self.prep_rw_buf(IORING_OP_RECV, sockfd, Some(buf), 0);
        self.op_flags.msg_flags = flags as u32;
    }

    /// Prepares a [`recv`](https://man.archlinux.org/man/recv.2) request
    ///
    /// The submission queue entry is setup to use the file descriptor `sockfd` to start receiving the
    /// data into the destination buffer `buf` and with modifier flags `flags`.
    ///
    /// The multishot version allows the application to issue a single receive request, which
    /// repeatedly posts a CQE when data is available. It requires `length` to be 0, the
    /// [`IOself_BUFFER_SELECT`] flag to be set, and no [`MSG_WAITALL`] flag to be set. Therefore each
    /// CQE will take a buffer out of a provided buffer pool for receiving. The application should
    /// check the flags of each CQE, regardless of its result. If a posted CQE does not have the
    /// [`IORING_CQE_F_MORE`] flag set then the multishot receive will be done and the application
    /// should issue a new request. Multishot variants are available since kernel 6.0.
    ///
    /// After calling this function, additional io_uring internal modifier flags may be set in the self
    /// [`ioprio`](io_uring_self::ioprio) field. The following flags are supported:
    /// * [`IORING_RECVSEND_POLL_FIRST`]
    ///   * If set, io_uring will assume the socket is currently empty and attempting to receive data
    ///   will be unsuccessful. For this case, io_uring will arm internal poll and trigger a receive of
    ///   the data when the socket has data to read. This initial receive attempt can be wasteful for
    ///   the case where the socket is expected to be empty, setting this flag will bypass the initial
    ///   receive attempt and go straight to arming poll. If poll does indicate that data is ready to be
    ///   received, the operation will proceed.
    ///   * Can be used with the CQE [`IORING_CQE_F_SOCK_NONEMPTY`] flag, which io_uring will set on
    ///   CQEs after a [`recv(2)`](https://man.archlinux.org/man/recv.2) or
    ///   [`recvmsg(2)`](https://man.archlinux.org/man/recvmsg.2) operation. If set, the socket still had
    ///   data to be read after the operation completed. Both these flags are available since 5.19.
    #[inline]
    pub fn prep_recv_multishot(&mut self, sockfd: i32, buf: &mut [u8], flags: i32) {
        self.prep_recv(sockfd, buf, flags);
        self.ioprio |= IORING_RECV_MULTISHOT as u16;
    }

    // TODO: recvmsg helpers, openat2, epollctl, provide_buffers, remove_buffers, shutdown, unlink,
    // rename, sync_file_range, mkdir, symlink, link

    #[inline]
    pub fn prep_msg_ring_cqe_flags(
        &mut self,
        fd: i32,
        len: u32,
        data: u64,
        flags: u32,
        cqe_flags: u32,
    ) {
        self.prep_rw_null(IORING_OP_MSG_RING, fd, len, data);
        self.op_flags.msg_ring_flags = IORING_MSG_RING_FLAGS_PASS | flags;
        self.union5.file_index = cqe_flags;
    }

    #[inline]
    pub fn prep_msg_ring(&mut self, fd: i32, len: u32, data: u64, flags: u32) {
        self.prep_rw_null(IORING_OP_MSG_RING, fd, len, data);
        self.op_flags.msg_ring_flags = flags;
    }

    #[inline]
    pub fn prep_msg_ring_fd(
        &mut self,
        fd: i32,
        source_fd: i32,
        target_fd: i32,
        data: u64,
        flags: u32,
    ) {
        self.prep_rw(
            IORING_OP_MSG_RING,
            fd,
            Some(&(IORING_MSG_SEND_FD as *const usize)),
            0,
            data,
        );
        self.union6.addr.addr3 = source_fd as u64;
        let target_fd = if target_fd == IORING_FILE_INDEX_ALLOC {
            target_fd - 1
        } else {
            target_fd
        };
        self.io_uring_set_target_fixed_file(target_fd as u32);
        self.op_flags.msg_ring_flags = flags;
    }

    #[inline]
    pub fn prep_msg_ring_fd_alloc(&mut self, fd: i32, source_fd: i32, data: u64, flags: u32) {
        self.prep_msg_ring_fd(fd, source_fd, IORING_FILE_INDEX_ALLOC, data, flags);
    }

    // TODO: xattr

    #[inline]
    pub fn prep_socket(&mut self, domain: i32, r#type: i32, protocol: i32, flags: u32) {
        self.prep_rw_null(IORING_OP_SOCKET, domain, protocol as u32, r#type as u64);
        self.op_flags.rw_flags = flags as i32;
    }

    #[inline]
    pub fn prep_socket_direct(
        &mut self,
        domain: i32,
        r#type: i32,
        protocol: i32,
        file_index: u32,
        flags: u32,
    ) {
        self.prep_socket(domain, r#type, protocol, flags);
        let file_index = if file_index as i32 == IORING_FILE_INDEX_ALLOC {
            file_index - 1
        } else {
            file_index
        };
        self.io_uring_set_target_fixed_file(file_index);
    }

    #[inline]
    pub fn prep_socket_direct_alloc(
        &mut self,
        domain: i32,
        r#type: i32,
        protocol: i32,
        flags: u32,
    ) {
        self.prep_socket(domain, r#type, protocol, flags);
        self.io_uring_set_target_fixed_file((IORING_FILE_INDEX_ALLOC - 1) as u32);
    }
}
