extern crate arrayvec;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
extern crate net;
extern crate socket;
extern crate warn;

use arrayvec::ArrayVec;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use net::Net;
use net::net::Callback;
use net::time::Timestamp;
use socket::Socket;
use std::cmp;
use std::fmt;

pub type Addr = socket::Addr;
pub type Chunk<'a> = net::net::Chunk<'a>;
pub type ChunkOrEvent<'a> = net::net::ChunkOrEvent<'a, Addr>;
pub type PeerId = net::net::PeerId;
pub type Timeout = net::Timeout;

pub trait Loop {
    fn accept_connections_on_port(port: u16) -> Self;
    fn client() -> Self;
    fn run<A: Application<Self>>(self, application: A) where Self: Sized;

    fn time(&mut self) -> Timestamp;
    fn connect(&mut self, addr: Addr) -> PeerId;
    fn disconnect(&mut self, pid: PeerId, reason: &[u8]);
    fn send_connless(&mut self, addr: Addr, data: &[u8]);
    fn send(&mut self, chunk: Chunk);
    fn flush(&mut self, pid: PeerId);
    fn ignore(&mut self, pid: PeerId);
    fn accept(&mut self, pid: PeerId);
}

pub trait Application<L: Loop> {
    fn needs_tick(&mut self) -> Timeout;
    fn on_tick(&mut self, loop_: &mut L);
    fn on_packet(&mut self, loop_: &mut L, event: ChunkOrEvent);
}

pub struct SocketLoop {
    socket: Socket,
    net: Net<Addr>,
    server: bool,
}

impl Loop for SocketLoop {
    fn accept_connections_on_port(port: u16) -> SocketLoop {
        SocketLoop {
            socket: Socket::bound(port).unwrap(),
            net: Net::server(),
            server: true,
        }
    }
    fn client() -> SocketLoop {
        SocketLoop {
            socket: Socket::new().unwrap(),
            net: Net::client(),
            server: false,
        }
    }
    fn run<A: Application<SocketLoop>>(mut self, mut application: A) {
        let mut buf1: ArrayVec<[u8; 4096]> = ArrayVec::new();
        let mut buf2: ArrayVec<[u8; 4096]> = ArrayVec::new();

        loop {
            self.net.tick(&mut self.socket).foreach(|e| panic!("{:?}", e));
            application.on_tick(&mut self);

            let sleep_timeout = cmp::min(self.net.needs_tick(), application.needs_tick());
            let sleep_duration = sleep_timeout.time_from(self.socket.time());
            if !self.server && sleep_duration.is_none() {
                break;
            }
            self.socket.sleep(sleep_duration).unwrap();

            while let Some(res) = { buf1.clear(); self.socket.receive(&mut buf1) } {
                let (addr, data) = res.unwrap();
                buf2.clear();
                let (iter, res) = self.net.feed(&mut self.socket, &mut Warn(addr, data), addr, data, &mut buf2);
                res.unwrap();
                for mut chunk in iter {
                    if self.net.is_receive_chunk_still_valid(&mut chunk) {
                        application.on_packet(&mut self, chunk);
                    }
                }
            }
        }
    }
    fn time(&mut self) -> Timestamp {
        self.socket.time()
    }
    fn connect(&mut self, addr: Addr) -> PeerId {
        let (pid, res) = self.net.connect(&mut self.socket, addr);
        res.unwrap();
        pid
    }
    fn disconnect(&mut self, pid: PeerId, reason: &[u8]) {
        self.net.disconnect(&mut self.socket, pid, reason).unwrap();
    }
    fn send_connless(&mut self, addr: Addr, data: &[u8]) {
        self.net.send_connless(&mut self.socket, addr, data).unwrap();
    }
    fn send(&mut self, chunk: Chunk) {
        self.net.send(&mut self.socket, chunk).unwrap();
    }
    fn flush(&mut self, pid: PeerId) {
        // TODO: Only flush at the end of the tick.
        self.net.flush(&mut self.socket, pid).unwrap();
    }
    fn ignore(&mut self, pid: PeerId) {
        self.net.ignore(pid);
    }
    fn accept(&mut self, pid: PeerId) {
        self.net.accept(&mut self.socket, pid).unwrap();
    }
}

fn hexdump(level: LogLevel, data: &[u8]) {
    if log_enabled!(level) {
        hexdump_iter(data).foreach(|s| log!(level, "{}", s));
    }
}

struct Warn<'a>(Addr, &'a [u8]);

impl<'a, W: fmt::Debug> warn::Warn<W> for Warn<'a> {
    fn warn(&mut self, w: W) {
        warn!("{}: {:?}", self.0, w);
        hexdump(LogLevel::Warn, self.1);
    }
}
