use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net;

/// Looks up a hostname and returns the first associated IP address.
///
/// If an error occurs during the lookup, it is returned, if no address is
/// found, `None` is returned.
///
/// Note that this function might block.
pub fn lookup_host(domain: &str) -> io::Result<Option<IpAddr>> {
    for maybe_addr in try!(net::lookup_host(domain)) {
        let socket_addr: SocketAddr = try!(maybe_addr);
        return Ok(Some(socket_addr.ip()));
    }
    Ok(None)
}
