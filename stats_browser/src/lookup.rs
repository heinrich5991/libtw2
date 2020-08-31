use addr::Addr;
use std::io;
use std::net::ToSocketAddrs;

/// Looks up a hostname and returns the first associated IP address.
///
/// If an error occurs during the lookup, it is returned, if no address is
/// found, `None` is returned.
///
/// Note that this function might block.
pub fn lookup_host(domain: &str, port: u16) -> io::Result<Option<Addr>> {
    for socket_addr in (domain, port).to_socket_addrs()? {
        return Ok(Some(Addr::from_socket_addr(socket_addr)));
    }
    Ok(None)
}
