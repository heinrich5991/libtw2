use rustc_serialize;
use serverbrowse::protocol;

use std::fmt;
use std::io::net::ip::IpAddr;

/// Protocol version of the `SERVERBROWSE_GETINFO` packet.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd, RustcEncodable, Show)]
pub enum ProtocolVersion {
    /// `SERVERBROWSE_GETINFO_5`.
    V5,
    /// `SERVERBROWSE_GETINFO_6`.
    V6,
}

/// Server address. Can currently store IPv4 and IPv6 addresses including a UDP
/// port number. Use as an opaque struct.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Addr(protocol::Addr);

impl Addr {
    /// Creates a new `Addr` from a given IP address and a UDP port.
    pub fn new(ip_addr: IpAddr, port: u16) -> Addr {
        Addr(protocol::Addr { ip_address: ip_addr, port: port })
    }
    /// Converts a serverbrowse address to an `Addr`.
    pub fn from_srvbrowse_addr(addr: protocol::Addr) -> Addr {
        Addr(addr)
    }
    /// Converts the address into a serverbrowse address.
    pub fn to_srvbrowse_addr(self) -> protocol::Addr {
        let Addr(inner) = self;
	inner
    }
}

/// Server address including protocol version.
#[derive(Clone, Copy, Eq, Hash, PartialEq, RustcEncodable)]
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

impl fmt::String for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Show::fmt(self, f)
    }
}

impl fmt::Show for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Addr(ref inner) = self;
        fmt::Show::fmt(inner, f)
    }
}

impl fmt::String for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &Addr(ref inner) = self;
        fmt::String::fmt(inner, f)
    }
}

impl rustc_serialize::Encodable for Addr {
    fn encode<S:rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(),S::Error> {
        s.emit_str(self.to_string().as_slice())
    }
}

impl fmt::Show for ServerAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}", self.version, self.addr)
    }
}

impl fmt::String for ServerAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Show::fmt(self, f)
    }
}
