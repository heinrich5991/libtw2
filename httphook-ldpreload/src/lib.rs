#![cfg(all(unix, not(target_os = "macos")))]

#[macro_use]
extern crate log;

use std::cell::Cell;
use std::mem;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;
use std::slice;

use self::leaky_vec::LeakyVec;

mod leaky_vec;

static INTERCEPT_SOCKETS: LeakyVec<i32> = LeakyVec::new();

fn on_udp_socket(sockfd: i32, addr: SocketAddr) {
    if !INTERCEPT_SOCKETS.contains(&sockfd) {
        INTERCEPT_SOCKETS.push_and_commit(sockfd);
    }
    info!("registering {addr}");
    libtw2_httphook::register_server_6(addr.port());
}

fn on_udp_packet(sockfd: i32, packet: &[u8]) {
    if !INTERCEPT_SOCKETS.contains(&sockfd) {
        return;
    }
    libtw2_httphook::on_packet(packet);
}

fn usize(i: u32) -> usize {
    i as usize
}

unsafe fn from_sockaddr(addr: *const libc::sockaddr, addrlen: u32) -> Option<SocketAddr> {
    fn port(network_port: u16) -> u16 {
        // The bytes are big-endian in memory, but in a weird type…
        u16::from_be_bytes(network_port.to_ne_bytes())
    }

    let addrlen = usize(addrlen);
    match unsafe { i32::from((*addr).sa_family) } {
        libc::AF_INET if addrlen >= mem::size_of::<libc::sockaddr_in>() => {
            let addr: libc::sockaddr_in = unsafe { *addr.cast() };
            // The bytes are big-endian in memory, but in a weird type…
            let ip_addr = Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes());
            return Some(SocketAddrV4::new(ip_addr, port(addr.sin_port)).into());
        }
        libc::AF_INET6 if addrlen >= mem::size_of::<libc::sockaddr_in6>() => {
            let addr: libc::sockaddr_in6 = unsafe { *addr.cast() };
            return Some(
                SocketAddrV6::new(
                    Ipv6Addr::from(addr.sin6_addr.s6_addr),
                    port(addr.sin6_port),
                    addr.sin6_flowinfo,
                    addr.sin6_scope_id,
                )
                .into(),
            );
        }
        _ => {}
    }
    None
}

thread_local! {
    pub static LAST_UDP_SOCKET: Cell<i32> = Cell::new(-1);
}

redhook::hook! {
    unsafe fn socket(domain: i32, type_: i32, protocol: i32) -> i32 => socket_wrapper {
        let result = redhook::real!(socket)(domain, type_, protocol);
        if result < 0 {
            return result;
        }
        if type_ == libc::SOCK_DGRAM && protocol == libc::IPPROTO_IP {
            // TODO (MSRV 1.63): Remove `with`.
            LAST_UDP_SOCKET.with(|s| s.set(result));
        }
        result
    }
}

redhook::hook! {
    unsafe fn bind(sockfd: i32, addr: *const libc::sockaddr, addrlen: u32) -> i32 => bind_wrapper {
        let result = redhook::real!(bind)(sockfd, addr, addrlen);
        if result != 0 {
            return result;
        }
        // TODO (MSRV 1.63): Remove `with`.
        let last_udp_socket = LAST_UDP_SOCKET.with(|s| s.replace(-1));
        if let Some(addr) = unsafe { from_sockaddr(addr, addrlen) } {
            if last_udp_socket == sockfd {
                on_udp_socket(sockfd, addr);
            }
        }
        result
    }
}

redhook::hook! {
    unsafe fn recvmmsg(sockfd: i32, msgvec: *mut libc::mmsghdr, vlen: u32, flags: i32, timeout: *mut libc::timespec) -> i32 => recvmmsg_wrapper {
        {
            let msgvec = slice::from_raw_parts_mut(msgvec, vlen as usize);
            for msg in &mut *msgvec {
                if msg.msg_hdr.msg_iovlen > 1 {
                    error!("WARNING: shortening msg_iovlen from {} to supported size of 1", msg.msg_hdr.msg_iovlen);
                    msg.msg_hdr.msg_iovlen = 1;
                }
            }
            if msgvec.len() == 0 {
                return redhook::real!(recvmmsg)(sockfd, msgvec.as_mut_ptr(), vlen, flags, timeout);
            }
        }
        let result = redhook::real!(recvmmsg)(sockfd, msgvec, vlen, flags, timeout);
        if result <= 0 {
            return result;
        }
        let msgvec = slice::from_raw_parts_mut(msgvec, result as usize);
        for msg in msgvec {
            let msg_ptr = (*msg.msg_hdr.msg_iov).iov_base as *mut u8;
            let msg_len = msg.msg_len as usize;
            on_udp_packet(sockfd, slice::from_raw_parts(msg_ptr, msg_len));
        }
        result
    }
}

redhook::hook! {
    unsafe fn recvfrom(sockfd: i32, buf: *mut u8, size: usize, flags: i32, address: *mut libc::sockaddr, address_len: *mut libc::socklen_t) -> isize => recvfrom_wrapper {
        let result = redhook::real!(recvfrom)(sockfd, buf, size, flags, address, address_len);
        if result <= 0 {
            return result;
        }
        on_udp_packet(sockfd, slice::from_raw_parts(buf, result as usize));
        result
    }
}
