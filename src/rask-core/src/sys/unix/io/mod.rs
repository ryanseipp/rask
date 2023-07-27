#[cfg(target_os = "linux")]
mod io_uring;

#[cfg(not(target_os = "linux"))]
mod kqueue;
