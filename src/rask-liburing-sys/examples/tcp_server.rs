use std::{io, mem::MaybeUninit, net::TcpListener, os::fd::AsRawFd};

use rask_liburing_sys::{
    io_uring_get_sqe, io_uring_prep_multishot_accept, io_uring_queue_init, io_uring_submit_and_wait,
};

const IO_URING_QUEUE_ENTRIES: u32 = 2048;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("::1:8000")?;
    println!("Server listening on {}", listener.local_addr()?);

    let mut ring = unsafe {
        let mut ring = MaybeUninit::zeroed();
        io_uring_queue_init(IO_URING_QUEUE_ENTRIES, ring.as_mut_ptr(), 0);
        ring.assume_init()
    };

    let sqe = io_uring_get_sqe(&mut ring).unwrap();
    io_uring_prep_multishot_accept(sqe, listener.as_raw_fd(), None, None, 0);

    loop {
        unsafe { io_uring_submit_and_wait(&mut ring, 1) };
    }
}
