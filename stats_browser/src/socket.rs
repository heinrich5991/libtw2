extern crate mio;

use self::mio::MioError;
use self::mio::buf::Buf;
use self::mio::buf::MutSliceBuf;
use self::mio::buf::SliceBuf;
use self::mio::net::UnconnectedSocket;
use self::mio::net::udp::UdpSocket as MioUdpSocket;

use std::fmt;

use addr::Addr;

/// An unconnected non-blocking UDP socket.
pub struct UdpSocket(MioUdpSocket);

impl UdpSocket {
    /// Opens a UDP socket.
    pub fn open() -> SockResult<UdpSocket> {
        MioUdpSocket::v4()
            .map(|s| UdpSocket(s))
            .map_err(|e| SockError(e))
    }
    /// Sends a UDP packet to the specified address. Non-blocking.
    pub fn send_to(&mut self, buf: &[u8], dst: Addr) -> SockResult<NonBlock<()>> {
        let &mut UdpSocket(ref mut mio_sock) = self;
        let mut mio_buf = SliceBuf::wrap(buf);
        match mio_sock.send_to(&mut mio_buf, &addr_to_sockaddr(dst)) {
            Ok(mio::NonBlock::Ready(())) => Ok(Ok(())),
            Ok(mio::NonBlock::WouldBlock) => Ok(Err(WouldBlock)),
            Err(e) => Err(SockError(e)),
        }
    }
    /// Receives a UDP packet. Non-blocking.
    ///
    /// Returns number number of bytes read and the source address.
    pub fn recv_from(&mut self, buf: &mut [u8]) -> SockResult<NonBlock<(uint, Addr)>> {
        let &mut UdpSocket(ref mut mio_sock) = self;
        let buf_len = buf.len();
        let mut mio_buf = MutSliceBuf::wrap(buf);
        match mio_sock.recv_from(&mut mio_buf) {
            Ok(mio::NonBlock::Ready(sockaddr)) => {
                let from = sockaddr_to_addr(sockaddr);
                let remaining = mio_buf.remaining();
                let read_len = buf_len - remaining;
                Ok(Ok((read_len, from)))
            },
            Ok(mio::NonBlock::WouldBlock) => Ok(Err(WouldBlock)),
            Err(x) => { panic!("socket error, {:?}", x); },
        }
    }
}

/// Converts the address to a socket address.
pub fn addr_to_sockaddr(addr: Addr) -> mio::net::SockAddr {
    let srvbrowse_addr = addr.to_srvbrowse_addr();
    mio::net::SockAddr::InetAddr(srvbrowse_addr.ip_address, srvbrowse_addr.port)
}
/// Converts a socket address to an `Addr`.
pub fn sockaddr_to_addr(addr: mio::net::SockAddr) -> Addr {
    match addr {
        mio::net::SockAddr::InetAddr(ip_address, port) => Addr::new(ip_address, port),
        x => { panic!("Invalid sockaddr: {:?}", x); }
    }
}

/// Extension trait providing the `would_block` function for `NonBlock`.
pub trait NonBlockExt { fn would_block(&self) -> bool; }
impl<T> NonBlockExt for NonBlock<T> {
    /// Returns `true` if the operation would block.
    fn would_block(&self) -> bool {
        if let &Err(WouldBlock) = self {
            true
        } else {
            false
        }
    }
}

/// Socket error. Opaque struct.
#[derive(Clone, Copy, PartialEq)]
pub struct SockError(MioError);

/// Socket result alias.
pub type SockResult<T> = Result<T,SockError>;
/// Non-blocking result alias.
pub type NonBlock<T> = Result<T,WouldBlock>;

/// Returned when an operation can't succeed without blocking.
#[derive(Clone, Copy, Eq, PartialEq, Show)]
pub struct WouldBlock;

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl Eq for SockError {}

impl fmt::Show for SockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let SockError(ref inner) = *self;
	fmt::Show::fmt(inner, f)
    }
}
