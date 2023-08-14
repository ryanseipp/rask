const IORING_FILE_INDEX_ALLOC: i32 = -1;
const IORING_SETUP_IOPOLL: u32 = 1;
const IORING_SETUP_SQPOLL: u32 = 2;
const IORING_SETUP_SQ_AFF: u32 = 4;
const IORING_SETUP_CQSIZE: u32 = 8;
const IORING_SETUP_CLAMP: u32 = 16;
const IORING_SETUP_ATTACH_WQ: u32 = 32;
const IORING_SETUP_R_DISABLED: u32 = 64;
const IORING_SETUP_SUBMIT_ALL: u32 = 128;
const IORING_SETUP_COOP_TASKRUN: u32 = 256;
const IORING_SETUP_TASKRUN_FLAG: u32 = 512;
const IORING_SETUP_SQE128: u32 = 1024;
const IORING_SETUP_CQE32: u32 = 2048;
const IORING_SETUP_SINGLE_ISSUER: u32 = 4096;
const IORING_SETUP_DEFER_TASKRUN: u32 = 8192;
const IORING_URING_CMD_FIXED: u32 = 1;
const IORING_FSYNC_DATASYNC: u32 = 1;
const IORING_TIMEOUT_ABS: u32 = 1;
const IORING_TIMEOUT_UPDATE: u32 = 2;
const IORING_TIMEOUT_BOOTTIME: u32 = 4;
const IORING_TIMEOUT_REALTIME: u32 = 8;
const IORING_LINK_TIMEOUT_UPDATE: u32 = 16;
const IORING_TIMEOUT_ETIME_SUCCESS: u32 = 32;
const IORING_TIMEOUT_MULTISHOT: u32 = 64;
const IORING_TIMEOUT_CLOCK_MASK: u32 = 12;
const IORING_TIMEOUT_UPDATE_MASK: u32 = 18;
const IORING_POLL_ADD_MULTI: u32 = 1;
const IORING_POLL_UPDATE_EVENTS: u32 = 2;
const IORING_POLL_UPDATE_USER_DATA: u32 = 4;
const IORING_POLL_ADD_LEVEL: u32 = 8;
const IORING_ASYNC_CANCEL_ALL: u32 = 1;
const IORING_ASYNC_CANCEL_FD: u32 = 2;
const IORING_ASYNC_CANCEL_ANY: u32 = 4;
const IORING_ASYNC_CANCEL_FD_FIXED: u32 = 8;
const IORING_RECVSEND_POLL_FIRST: u32 = 1;
const IORING_RECV_MULTISHOT: u32 = 2;
const IORING_RECVSEND_FIXED_BUF: u32 = 4;
const IORING_SEND_ZC_REPORT_USAGE: u32 = 8;
const IORING_NOTIF_USAGE_ZC_COPIED: u32 = 2147483648;
const IORING_ACCEPT_MULTISHOT: u32 = 1;
const IORING_MSG_RING_CQE_SKIP: u32 = 1;
const IORING_MSG_RING_FLAGS_PASS: u32 = 2;
const IORING_CQE_F_BUFFER: u32 = 1;
const IORING_CQE_F_MORE: u32 = 2;
const IORING_CQE_F_SOCK_NONEMPTY: u32 = 4;
const IORING_CQE_F_NOTIF: u32 = 8;
const IORING_OFF_SQ_RING: u32 = 0;
const IORING_OFF_CQ_RING: u32 = 134217728;
const IORING_OFF_SQES: u32 = 268435456;
const IORING_OFF_PBUF_RING: u32 = 2147483648;
const IORING_OFF_PBUF_SHIFT: u32 = 16;
const IORING_OFF_MMAP_MASK: u32 = 4160749568;
const IORING_SQ_NEED_WAKEUP: u32 = 1;
const IORING_SQ_CQ_OVERFLOW: u32 = 2;
const IORING_SQ_TASKRUN: u32 = 4;
const IORING_CQ_EVENTFD_DISABLED: u32 = 1;
const IORING_ENTER_GETEVENTS: u32 = 1;
const IORING_ENTER_SQ_WAKEUP: u32 = 2;
const IORING_ENTER_SQ_WAIT: u32 = 4;
const IORING_ENTER_EXT_ARG: u32 = 8;
const IORING_ENTER_REGISTERED_RING: u32 = 16;
const IORING_FEAT_SINGLE_MMAP: u32 = 1;
const IORING_FEAT_NODROP: u32 = 2;
const IORING_FEAT_SUBMIT_STABLE: u32 = 4;
const IORING_FEAT_RW_CUR_POS: u32 = 8;
const IORING_FEAT_CUR_PERSONALITY: u32 = 16;
const IORING_FEAT_FAST_POLL: u32 = 32;
const IORING_FEAT_POLL_32BITS: u32 = 64;
const IORING_FEAT_SQPOLL_NONFIXED: u32 = 128;
const IORING_FEAT_EXT_ARG: u32 = 256;
const IORING_FEAT_NATIVE_WORKERS: u32 = 512;
const IORING_FEAT_RSRC_TAGS: u32 = 1024;
const IORING_FEAT_CQE_SKIP: u32 = 2048;
const IORING_FEAT_LINKED_FILE: u32 = 4096;
const IORING_FEAT_REG_REG_RING: u32 = 8192;
const IORING_RSRC_REGISTER_SPARSE: u32 = 1;
const IORING_REGISTER_FILES_SKIP: i32 = -2;

pub mod sqe;