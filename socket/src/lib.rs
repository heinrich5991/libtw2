extern crate buffer;
extern crate hexdump;
extern crate itertools;
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
use net::net::Callback;
use num::ToPrimitive;
use std::fmt;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::str;
use std::time::Duration;
use std::time::Instant;
use std::u32;

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
    time_cached: Duration,
    poll: mio::Poll,
    v4: UdpSocket,
    v6: UdpSocket,
    loss_rate: f32,
}

fn udp_socket(bindaddr: &str) -> io::Result<UdpSocket> {
    match bindaddr {
        "0.0.0.0:0" => UdpSocket::v4(),
        "[::]:0" => UdpSocket::v6(),
        _ => panic!("invalid bindaddr {}", bindaddr),
    }
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
        Socket::with_loss_rate(0.0)
    }
    pub fn with_loss_rate(loss_rate: f32) -> io::Result<Socket> {
        assert!(0.0 <= loss_rate && loss_rate <= 1.0);

        fn register(poll: &mut mio::Poll, socket: &UdpSocket) -> io::Result<()> {
            use mio::EventSet;
            use mio::PollOpt;
            use mio::Token;
            poll.register(socket, Token(0), EventSet::readable(), PollOpt::level())
        }

        let v4 = try!(udp_socket("0.0.0.0:0"));
        let v6 = try!(udp_socket("[::]:0"));
        let mut poll = try!(mio::Poll::new());
        try!(register(&mut poll, &v4));
        try!(register(&mut poll, &v6));
        Ok(Socket {
            start: Instant::now(),
            time_cached: Duration::from_millis(0),
            poll: poll,
            v4: v4,
            v6: v6,
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
        let result;
        {
            let buf_slice = unsafe { buf.uninitialized_mut() };
            if let Some(r) = swap(self.v4.recv_from(buf_slice)) {
                result = r;
            } else if let Some(r) = swap(self.v6.recv_from(buf_slice)) {
                result = r;
            } else {
                return None;
            }
        }
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
    pub fn sleep(&mut self, duration: Duration) -> io::Result<()> {
        let milliseconds = duration.to_milliseconds_saturating().to_usize().unwrap();
        try!(self.poll.poll(Some(milliseconds)));
        // TODO: Add a verification that this also works with
        // ```
        // try!(self.poll.poll(None));
        // ```
        // on loss-free networks.
        Ok(())
    }
    pub fn update_time_cached(&mut self) {
        self.time_cached = self.start.elapsed()
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
    fn time(&mut self) -> Duration {
        self.time_cached
    }
}
