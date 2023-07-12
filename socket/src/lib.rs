extern crate buffer;
#[macro_use] extern crate common;
extern crate hexdump;
extern crate itertools;
extern crate libc;
#[macro_use] extern crate log;
extern crate mio;
extern crate net;
extern crate net2;
extern crate rand;

use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use mio::Ready;
use mio::Token;
use mio::net::UdpSocket;
use net2::UdpBuilder;
use net::Timestamp;
use net::net::Callback;
use rand::RngCore as _;
use rand::thread_rng;
use std::error;
use std::fmt;
use std::io;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::str;
use std::time::Duration;
use std::time::Instant;

#[derive(Debug)]
enum Direction {
    Send,
    Receive,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Direction::Send => "->",
            Direction::Receive => "<-",
        }.fmt(f)
    }
}

fn hexdump(level: LogLevel, data: &[u8]) {
    if log_enabled!(level) {
        hexdump_iter(data).foreach(|s| log!(level, "{}", s));
    }
}

fn dump(dir: Direction, addr: Addr, data: &[u8]) {
    debug!("{} {}", dir, addr);
    hexdump(LogLevel::Debug, data);
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Addr {
    pub ip: IpAddr,
    pub port: u16,
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        SocketAddr::new(self.ip, self.port).fmt(f)
    }
}

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<SocketAddr> for Addr {
    fn from(sock_addr: SocketAddr) -> Addr {
        Addr {
            ip: sock_addr.ip(),
            port: sock_addr.port(),
        }
    }
}

impl FromStr for Addr {
    type Err = std::net::AddrParseError;
    fn from_str(s: &str) -> Result<Addr, std::net::AddrParseError> {
        let sock_addr: SocketAddr = s.parse()?;
        Ok(Addr::from(sock_addr))
    }
}

#[derive(Debug)]
pub struct NoAddressFamiliesSupported(());

impl error::Error for NoAddressFamiliesSupported {}

impl fmt::Display for NoAddressFamiliesSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("neither IPv4 nor IPv6 supported on this system")
    }
}

#[derive(Debug)]
pub struct AddressFamilyNotSupported(());

impl error::Error for AddressFamilyNotSupported {}

impl fmt::Display for AddressFamilyNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            "destination address family (IPv4 or IPv6) not supported on this system"
        )
    }
}

pub struct Socket {
    start: Instant,
    time_cached: Timestamp,
    poll: mio::Poll,
    events: mio::Events,
    v4: Option<UdpSocket>,
    v6: Option<UdpSocket>,
    check_v4: bool,
    check_v6: bool,
    loss_rate: f32,
}

fn udp_socket(bindaddr: &SocketAddr) -> io::Result<Option<UdpSocket>> {
    debug!("binding to {}", bindaddr);
    let builder;
    match *bindaddr {
        SocketAddr::V4(..) => builder = UdpBuilder::new_v4(),
        SocketAddr::V6(..) => builder = UdpBuilder::new_v6(),
    }
    let builder = match builder {
        Err(ref e) if e.raw_os_error() == Some(libc::EAFNOSUPPORT) =>
            return Ok(None), // Address family not supported.
        b => b?,
    };
    if let SocketAddr::V6(..) = *bindaddr {
        builder.only_v6(true)?;
    }
    Ok(Some(UdpSocket::from_socket(builder.bind(bindaddr)?)?))
}

fn non_block<T>(res: io::Result<T>) -> Option<io::Result<T>> {
    match res {
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => None,
        x => Some(x),
    }
}

impl Socket {
    pub fn new() -> io::Result<Socket> {
        Socket::construct(None, 0.0)
    }
    pub fn with_loss_rate(loss_rate: f32) -> io::Result<Socket> {
        Socket::construct(None, loss_rate)
    }
    pub fn bound(port: u16) -> io::Result<Socket> {
        Socket::construct(Some(port), 0.0)
    }
    pub fn bound_with_loss_rate(port: u16, loss_rate: f32) -> io::Result<Socket> {
        Socket::construct(Some(port), loss_rate)
    }
    pub fn construct(port: Option<u16>, loss_rate: f32) -> io::Result<Socket> {
        assert!(port != Some(0));
        let port = port.unwrap_or(0);
        assert!(0.0 <= loss_rate && loss_rate <= 1.0);

        fn register(poll: &mut mio::Poll, token: usize, socket: &UdpSocket) -> io::Result<()> {
            use mio::PollOpt;
            poll.register(socket, Token(token), Ready::readable(), PollOpt::level())
        }

        let addr_v4 = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let addr_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));

        let v4 = udp_socket(&SocketAddr::new(addr_v4, port))?;
        let v6 = udp_socket(&SocketAddr::new(addr_v6, port))?;

        if v4.is_none() && v6.is_none() {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      NoAddressFamiliesSupported(())));
        }

        let mut poll = mio::Poll::new()?;
        v4.as_ref().map(|v4| register(&mut poll, 4, &v4)).unwrap_or(Ok(()))?;
        v6.as_ref().map(|v6| register(&mut poll, 6, &v6)).unwrap_or(Ok(()))?;
        Ok(Socket {
            start: Instant::now(),
            time_cached: Timestamp::from_secs_since_epoch(0),
            poll: poll,
            events: mio::Events::with_capacity(2),
            v4: v4,
            v6: v6,
            check_v4: false,
            check_v6: false,
            loss_rate: loss_rate,
        })
    }
    fn loss(&self) -> bool {
        self.loss_rate != 0.0 && rand::random::<f32>() < self.loss_rate
    }
    pub fn receive<'a, B: Buffer<'a>>(&mut self, buf: B)
        -> Option<Result<(Addr, &'a [u8]), io::Error>>
    {
        with_buffer(buf, |b| self.receive_impl(b))
    }
    fn receive_impl<'d, 's>(&mut self, mut buf: BufferRef<'d, 's>)
        -> Option<Result<(Addr, &'d [u8]), io::Error>>
    {
        let mut result = None;
        {
            let buf_slice = unsafe { buf.uninitialized_mut() };
            if result.is_none() && self.check_v6 {
                if let Some(r) = non_block(self.v6.as_ref().unwrap().recv_from(buf_slice)) {
                    result = Some(r);
                    self.check_v6 = false;
                }
            }
            if result.is_none() && self.check_v4 {
                if let Some(r) = non_block(self.v4.as_ref().unwrap().recv_from(buf_slice)) {
                    result = Some(r);
                    self.check_v4 = false;
                }
            }
        }
        let result = unwrap_or_return!(result);
        if self.loss() {
            return self.receive_impl(buf);
        }
        Some(result.map(|(len, addr)| unsafe {
            let addr = Addr::from(addr);
            buf.advance(len);
            let initialized = buf.initialized();
            dump(Direction::Receive, addr, initialized);
            (addr, initialized)
        }))
    }
    pub fn sleep(&mut self, duration: Option<Duration>) -> io::Result<()> {
        self.poll.poll(&mut self.events, duration)?;
        // TODO: Add a verification that this also works with
        // ```
        // self.poll.poll(None)?;
        // ```
        // on loss-free networks.
        for ev in &self.events {
            assert!(ev.readiness() == Ready::readable());
            match ev.token() {
                Token(4) => self.check_v4 = true,
                Token(6) => self.check_v6 = true,
                _ => unreachable!(),
            }
        }
        self.update_time_cached();
        Ok(())
    }
    pub fn update_time_cached(&mut self) {
        self.time_cached = Timestamp::from_secs_since_epoch(0) + self.start.elapsed();
    }
}

impl Callback<Addr> for Socket {
    type Error = io::Error;
    fn secure_random(&mut self, buffer: &mut [u8]) {
        thread_rng().fill_bytes(buffer)
    }
    fn send(&mut self, addr: Addr, data: &[u8]) -> Result<(), io::Error> {
        if self.loss() {
            return Ok(());
        }
        dump(Direction::Send, addr, data);
        let sock_addr = SocketAddr::new(addr.ip, addr.port);
        let maybe_socket = if let IpAddr::V4(..) = addr.ip {
            &self.v4
        } else {
            &self.v6
        };
        let socket;
        if let Some(ref s) = *maybe_socket {
            socket = s;
        } else {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      AddressFamilyNotSupported(())));
        }
        non_block(socket.send_to(data, &sock_addr))
            .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::WouldBlock, "write would block")))
            .map(|s| assert!(data.len() == s))
        // TODO: Check for these errors and decide what to do with them
        // EHOSTUNREACH
        // ENETDOWN
        // ENTUNREACH
        // EAGAIN EWOULDBLOCK
    }
    fn time(&mut self) -> Timestamp {
        self.time_cached
    }
}
