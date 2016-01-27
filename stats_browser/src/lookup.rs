use std::io;
use std::net::SocketAddr;
use std::net;
use addr::Addr;

/// Looks up a hostname and returns the first associated IP address.
///
/// If an error occurs during the lookup, it is returned, if no address is
/// found, `None` is returned.
///
/// Note that this function might block.
pub fn lookup_host(domain: &str, port: u16) -> io::Result<Option<Addr>> {
    for maybe_addr in try!(net::lookup_host(domain)) {
        let socket_addr: SocketAddr = try!(maybe_addr);
        let socket_addr = match socket_addr {
            net::SocketAddr::V4(a) =>
                net::SocketAddr::V4(net::SocketAddrV4::new(*a.ip(), port)),
            net::SocketAddr::V6(a) =>
                net::SocketAddr::V6(net::SocketAddrV6::new(*a.ip(), port, a.flowinfo(), a.scope_id())),
        };
        return Ok(Some(Addr::from_socket_addr(socket_addr)));
    }
    Ok(None)
}
