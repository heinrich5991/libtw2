extern crate buffer;
#[macro_use] extern crate common;
extern crate hexdump;
extern crate itertools;
extern crate libc;
#[macro_use] extern crate log;
extern crate mio;
extern crate net;
extern crate num;
extern crate rand;

use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use mio::udp::UdpSocket;
use mio::EventSet;
use mio::Token;
use net::Timestamp;
use net::net::Callback;
use num::ToPrimitive;
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
use std::u32;

mod system;

trait DurationToMs {
    fn to_milliseconds_saturating(&self) -> u32;
}

impl DurationToMs for Duration {
    fn to_milliseconds_saturating(&self) -> u32 {
        (self.as_secs()
            .to_u32().unwrap_or(u32::max_value())
            .to_u64().unwrap()
            * 1000
            + self.subsec_nanos().to_u64().unwrap() / 1000 / 1000
        ).to_u32().unwrap_or(u32::max_value())
    }
}

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
        let sock_addr: SocketAddr = try!(s.parse());
        Ok(Addr::from(sock_addr))
    }
}

pub struct Socket {
    start: Instant,
    time_cached: Timestamp,
    poll: mio::Poll,
    v4: UdpSocket,
    v6: UdpSocket,
    check_v4: bool,
    check_v6: bool,
    loss_rate: f32,
}

fn udp_socket(bindaddr: &SocketAddr) -> io::Result<UdpSocket> {
    debug!("binding to {}", bindaddr);
    let result;
    match *bindaddr {
        SocketAddr::V4(..) => result = try!(UdpSocket::v4()),
        SocketAddr::V6(..) => {
            result = try!(UdpSocket::v6());
            try!(system::set_ipv6_only(&result));
        }
    }
    try!(result.bind(bindaddr));
    Ok(result)
}

fn swap<T, E>(res: Result<Option<T>, E>) -> Option<Result<T, E>> {
    match res {
        Ok(Some(x)) => Some(Ok(x)),
        Ok(None) => None,
        Err(x) => Some(Err(x)),
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
            poll.register(socket, Token(token), EventSet::readable(), PollOpt::level())
        }

        let addr_v4 = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let addr_v6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));

        // TODO: Handle the error if either of these doesn't exist:
        let v4 = try!(udp_socket(&SocketAddr::new(addr_v4, port)));
        let v6 = try!(udp_socket(&SocketAddr::new(addr_v6, port)));
        let mut poll = try!(mio::Poll::new());
        try!(register(&mut poll, 4, &v4));
        try!(register(&mut poll, 6, &v6));
        Ok(Socket {
            start: Instant::now(),
            time_cached: Timestamp::from_secs_since_epoch(0),
            poll: poll,
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
                if let Some(r) = swap(self.v6.recv_from(buf_slice)) {
                    result = Some(r);
                    self.check_v6 = false;
                }
            }
            if result.is_none() && self.check_v4 {
                if let Some(r) = swap(self.v4.recv_from(buf_slice)) {
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
        let ms = duration.map(|d| d.to_milliseconds_saturating().to_usize().unwrap());
        try!(self.poll.poll(ms));
        // TODO: Add a verification that this also works with
        // ```
        // try!(self.poll.poll(None));
        // ```
        // on loss-free networks.
        for ev in self.poll.events() {
            assert!(ev.kind == EventSet::readable());
            match ev.token {
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
    fn send(&mut self, addr: Addr, data: &[u8]) -> Result<(), io::Error> {
        if self.loss() {
            return Ok(());
        }
        dump(Direction::Send, addr, data);
        let sock_addr = SocketAddr::new(addr.ip, addr.port);
        let socket = if let IpAddr::V4(..) = addr.ip {
            &mut self.v4
        } else {
            &mut self.v6
        };
        swap(socket.send_to(data, &sock_addr))
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
