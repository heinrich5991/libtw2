use serverbrowse::protocol;
use serverbrowse::protocol::IpAddr;

use std::fmt;
use std::net;

/// Protocol version of the `SERVERBROWSE_GETINFO` packet.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProtocolVersion {
    /// `SERVERBROWSE_GETINFO_5`.
    V5,
    /// `SERVERBROWSE_GETINFO_6`.
    V6,
    /// `SERVERBROWSE_GETINFO_7`.
    V7,
}

pub const ALL_PROTOCOL_VERSIONS: &'static [ProtocolVersion] = &[
    ProtocolVersion::V5,
    ProtocolVersion::V6,
    ProtocolVersion::V7,
];

/// Server address. Can currently store IPv4 and IPv6 addresses including a UDP
/// port number. Use as an opaque struct.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Addr(protocol::Addr);

impl Addr {
    /// Converts a serverbrowse address to an `Addr`.
    pub fn from_srvbrowse_addr(addr: protocol::Addr) -> Addr {
        Addr(addr)
    }
    /// Converts the address into a serverbrowse address.
    pub fn to_srvbrowse_addr(self) -> protocol::Addr {
        let Addr(inner) = self;
        inner
    }
    /// Converts the address to a socket address.
    pub fn to_socket_addr(self) -> net::SocketAddr {
        let srvbrowse_addr = self.to_srvbrowse_addr();
        match srvbrowse_addr.ip_address {
            IpAddr::V4(x) =>
                net::SocketAddr::V4(net::SocketAddrV4::new(x, srvbrowse_addr.port)),
            IpAddr::V6(x) =>
                net::SocketAddr::V6(net::SocketAddrV6::new(x, srvbrowse_addr.port, 0, 0)),
        }
    }
    /// Converts a socket address to an `Addr`.
    pub fn from_socket_addr(addr: net::SocketAddr) -> Addr {
        let (ip_addr, port) = match addr {
            net::SocketAddr::V4(a) => (IpAddr::V4(*a.ip()), a.port()),
            net::SocketAddr::V6(a) => {
                let mut ip = IpAddr::V6(*a.ip());
                // TODO: switch to `to_ipv4_mapped` in the future.
                if let Some(v4) = a.ip().to_ipv4() {
                    if !a.ip().is_loopback() {
                        ip = IpAddr::V4(v4);
                    }
                }
                (ip, a.port())
            }
        };
        Addr(protocol::Addr { ip_address: ip_addr, port: port })
    }
    /// Returns the current address, replacing the port with the given one.
    pub fn with_port(&self, port: u16) -> Addr {
        Addr(protocol::Addr { ip_address: self.0.ip_address, port: port })
    }
}

/// Server address including protocol version.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ServerAddr {
    /// The protocol version of the listening server.
    pub version: ProtocolVersion,
    /// The actual address of the server.
    pub addr: Addr,
}

impl ServerAddr {
    /// Creates a `ServerAddress` from a version and an address.
    pub fn new(version: ProtocolVersion, addr: Addr) -> ServerAddr {
        ServerAddr {
            version: version,
            addr: addr,
        }
    }
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Addr(ref inner) = self;
        fmt::Debug::fmt(inner, f)
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Addr(ref inner) = self;
        fmt::Display::fmt(inner, f)
    }
}

impl fmt::Debug for ServerAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.version, self.addr)
    }
}

impl fmt::Display for ServerAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
