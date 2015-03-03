extern crate mio;

use self::mio::buf::Buf;
use self::mio::buf::MutBuf;
use self::mio::buf::MutSliceBuf;
use self::mio::buf::SliceBuf;
use self::mio::net::TryRecv;
use self::mio::net::TrySend;
use self::mio::net::udp::UdpSocket as MioUdpSocket;

use std::fmt;
use std::io;

use addr::Addr;

/// An unconnected non-blocking UDP socket.
pub struct UdpSocket(MioUdpSocket);

impl UdpSocket {
    /// Opens a UDP socket.
    pub fn open() -> SockResult<UdpSocket> {
        MioUdpSocket::bind("localhost")
            .map(|s| UdpSocket(s))
            .map_err(|e| SockError(e))
    }
    /// Sends a UDP packet to the specified address. Non-blocking.
    pub fn send_to(&mut self, buf: &[u8], dst: Addr) -> SockResult<NonBlock<()>> {
        let &mut UdpSocket(ref mut mio_sock) = self;
        let mut mio_buf = SliceBuf::wrap(buf);
        match TrySend::send_to(mio_sock, &mut mio_buf, &dst.to_socket_addr()) {
            Ok(mio::NonBlock::Ready(())) => Ok(Ok(())),
            Ok(mio::NonBlock::WouldBlock) => Ok(Err(WouldBlock)),
            Err(e) => Err(SockError(e)),
        }
    }
    /// Receives a UDP packet. Non-blocking.
    ///
    /// Returns number number of bytes read and the source address.
    pub fn recv_from(&mut self, buf: &mut [u8]) -> SockResult<NonBlock<(usize, Addr)>> {
        let &mut UdpSocket(ref mut mio_sock) = self;
        let buf_len = buf.len();
        let mut mio_buf = MutSliceBuf::wrap(buf);
        match TryRecv::recv_from(mio_sock, &mut mio_buf) {
            Ok(mio::NonBlock::Ready(sockaddr)) => {
                let from = Addr::from_socket_addr(sockaddr);
                let remaining = mio_buf.remaining();
                let read_len = buf_len - remaining;
                Ok(Ok((read_len, from)))
            },
            Ok(mio::NonBlock::WouldBlock) => Ok(Err(WouldBlock)),
            Err(x) => { panic!("socket error, {:?}", x); },
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
#[derive(Clone, PartialEq)]
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

impl Eq for SockError {}

impl fmt::Debug for SockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let SockError(ref inner) = *self;
        fmt::Debug::fmt(inner, f)
    }
}
