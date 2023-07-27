use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, ToSocketAddrs};
use std::os::fd::{FromRawFd, RawFd};
use std::{io, mem, ptr};

const SOCK_BACKLOG: i32 = 1024;

macro_rules! syscall {
    ($fn: ident($($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub fn bind(addr: impl ToSocketAddrs) -> io::Result<Vec<TcpListener>> {
    let mut listeners = Vec::new();

    for address in addr.to_socket_addrs()? {
        listeners.push(create_tcp_listener(address)?);
    }

    Ok(listeners)
}

fn create_tcp_listener(addr: SocketAddr) -> io::Result<TcpListener> {
    let sock_fd = match addr.ip() {
        IpAddr::V4(ip4_addr) => create_ip_socket(ip4_addr, addr.port()),
        IpAddr::V6(ip6_addr) => create_ip6_socket(ip6_addr, addr.port()),
    }?;

    syscall!(listen(sock_fd, SOCK_BACKLOG))?;
    let listener = unsafe { TcpListener::from_raw_fd(sock_fd) };

    Ok(listener)
}

fn create_ip_socket(addr: Ipv4Addr, port: u16) -> io::Result<RawFd> {
    let sock_fd = syscall!(socket(libc::AF_INET, libc::SOCK_STREAM, 0))?;
    set_reuse_addr(sock_fd)?;
    set_reuse_port(sock_fd)?;

    let socket_address = libc::sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: port.to_be(),
        sin_addr: libc::in_addr {
            s_addr: u32::from(addr).to_be(),
        },
        sin_zero: [0u8; 8],
    };

    syscall!(bind(
        sock_fd,
        ptr::addr_of!(socket_address).cast(),
        mem::size_of::<libc::sockaddr_in>() as u32
    ))?;

    Ok(sock_fd)
}

fn create_ip6_socket(addr: Ipv6Addr, port: u16) -> io::Result<RawFd> {
    let sock_fd = syscall!(socket(libc::AF_INET6, libc::SOCK_STREAM, 0))?;
    set_reuse_addr(sock_fd)?;
    set_reuse_port(sock_fd)?;

    let socket_address = libc::sockaddr_in6 {
        sin6_family: libc::AF_INET6 as u16,
        sin6_port: port.to_be(),
        sin6_flowinfo: 0,
        sin6_addr: libc::in6_addr {
            s6_addr: addr.octets(),
        },
        sin6_scope_id: 0,
    };

    syscall!(bind(
        sock_fd,
        ptr::addr_of!(socket_address).cast(),
        mem::size_of::<libc::sockaddr_in6>() as u32
    ))?;
    Ok(sock_fd)
}

fn set_reuse_addr(socket: RawFd) -> io::Result<()> {
    let value: libc::c_int = 1;
    syscall!(setsockopt(
        socket,
        libc::SOL_SOCKET,
        libc::SO_REUSEADDR,
        ptr::addr_of!(value).cast(),
        mem::size_of::<libc::c_int>() as libc::socklen_t,
    ))?;

    Ok(())
}

fn set_reuse_port(socket: RawFd) -> io::Result<()> {
    let value: libc::c_int = 1;
    syscall!(setsockopt(
        socket,
        libc::SOL_SOCKET,
        libc::SO_REUSEPORT,
        ptr::addr_of!(value).cast(),
        mem::size_of::<libc::c_int>() as libc::socklen_t,
    ))?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        io, mem,
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
        os::fd::AsRawFd,
        ptr,
    };

    use super::bind;

    #[test]
    fn binds_to_multiple_addrs() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
        let addr2 = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);
        let addrs = vec![addr1, addr2];

        let listeners = bind(&addrs[..]).unwrap();

        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].local_addr().unwrap(), addr1);
        assert_eq!(listeners[1].local_addr().unwrap(), addr2);

        for listener in listeners {
            let mut reuseaddr = 0;
            let mut len: libc::socklen_t = mem::size_of::<libc::socklen_t>() as u32;
            syscall!(getsockopt(
                listener.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                ptr::addr_of_mut!(reuseaddr).cast(),
                ptr::addr_of_mut!(len)
            ))
            .unwrap();
            assert_eq!(reuseaddr, 1);

            let mut reuseport = 0;
            let mut len: libc::socklen_t = mem::size_of::<libc::socklen_t>() as u32;
            syscall!(getsockopt(
                listener.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_REUSEPORT,
                ptr::addr_of_mut!(reuseport).cast(),
                ptr::addr_of_mut!(len)
            ))
            .unwrap();
            assert_eq!(reuseport, 1);
        }
    }
}
