use mio::udp::UdpSocket;
use std::io;

// FIXME: Remove this module once the Rust standard library does this.

#[cfg(unix)]
fn set_ipv6_only_impl(socket: &UdpSocket) -> io::Result<()> {
    use libc;
    use std::mem;
    use std::os::unix::io::AsRawFd;

    let one = true as libc::c_int;
    let socket = socket.as_raw_fd();
    let res;
    unsafe {
        res = libc::setsockopt(
            socket,
            libc::IPPROTO_IPV6,
            libc::IPV6_V6ONLY,
            &one as *const _ as *const _,
            mem::size_of_val(&one) as libc::socklen_t,
        );
    }
    if res == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn set_ipv6_only_impl(socket: &UdpSocket) -> io::Result<()> {
    Ok(())
}

pub fn set_ipv6_only(socket: &UdpSocket) -> io::Result<()> {
    set_ipv6_only_impl(socket)
}
