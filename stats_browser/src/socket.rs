extern crate mio;

use self::mio::net::UdpSocket as MioUdpSocket;

use std::fmt;
use std::io;

use addr::Addr;

/// An unconnected non-blocking UDP socket.
pub struct UdpSocket(MioUdpSocket);

fn non_block<T>(res: io::Result<T>) -> io::Result<Option<T>> {
    match res {
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
        Ok(x) => Ok(Some(x)),
    }
}

impl UdpSocket {
    /// Opens a UDP socket.
    pub fn open() -> SockResult<UdpSocket> {
        MioUdpSocket::bind(&"[::]:0".parse().unwrap())
            .map(|s| UdpSocket(s))
            .map_err(|e| SockError(e))
    }
    /// Sends a UDP packet to the specified address. Non-blocking.
    pub fn send_to(&mut self, buf: &[u8], dst: Addr) -> SockResult<NonBlock<()>> {
        let &mut UdpSocket(ref mut std_sock) = self;
        match non_block(std_sock.send_to(buf, &dst.to_socket_addr())) {
            Ok(Some(len)) => {
                assert!(len == buf.len(), "short send: {} out of {}", len, buf.len());
                Ok(Ok(()))
            },
            Ok(None) => Ok(Err(WouldBlock)),
            Err(e) => Err(SockError(e)),
        }
    }
    /// Receives a UDP packet. Non-blocking.
    ///
    /// Returns number number of bytes read and the source address.
    pub fn recv_from(&mut self, buf: &mut [u8]) -> SockResult<NonBlock<(usize, Addr)>> {
        let &mut UdpSocket(ref mut std_sock) = self;
        match non_block(std_sock.recv_from(buf)) {
            Ok(Some((len, sockaddr))) => {
                let from = Addr::from_socket_addr(sockaddr);
                Ok(Ok((len, from)))
            },
            Ok(None) => Ok(Err(WouldBlock)),
            Err(x) => Err(SockError(x)),
        }
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
pub struct SockError(io::Error);

/// Socket result alias.
pub type SockResult<T> = Result<T,SockError>;
/// Non-blocking result alias.
pub type NonBlock<T> = Result<T,WouldBlock>;

/// Returned when an operation can't succeed without blocking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WouldBlock;

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl fmt::Debug for SockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let SockError(ref inner) = *self;
        fmt::Debug::fmt(inner, f)
    }
}
