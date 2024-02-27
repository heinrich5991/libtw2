#[macro_use]
extern crate log;

use arrayvec::ArrayVec;
use hexdump::hexdump_iter;
use itertools::Itertools;
use libtw2_common::Takeable;
use libtw2_net::collections::PeerMap;
use libtw2_net::collections::PeerSet;
use libtw2_net::net::Callback;
use libtw2_net::Net;
use libtw2_socket::Socket;
use log::LogLevel;
use std::cmp;
use std::fmt;

pub use libtw2_net::collections;
pub use libtw2_net::net::PeerId;
pub use libtw2_net::Timeout;
pub use libtw2_net::Timestamp;
pub use libtw2_socket::Addr;

pub type Chunk<'a> = libtw2_net::net::Chunk<'a>;
pub type ConnlessChunk<'a> = libtw2_net::net::ConnlessChunk<'a, Addr>;

pub trait Loop {
    fn accept_connections_on_port(port: u16) -> Self;
    fn client() -> Self;
    fn run<A: Application<Self>>(self, application: A)
    where
        Self: Sized;

    fn time(&mut self) -> Timestamp;
    fn connect(&mut self, addr: Addr) -> PeerId;
    fn disconnect(&mut self, pid: PeerId, reason: &[u8]);
    fn send_connless(&mut self, addr: Addr, data: &[u8]);
    fn send(&mut self, chunk: Chunk);
    fn force_flush(&mut self, pid: PeerId);
    fn flush(&mut self, pid: PeerId);
    fn ignore(&mut self, pid: PeerId);
    fn accept(&mut self, pid: PeerId);
    fn reject(&mut self, pid: PeerId, reason: &[u8]);
}

pub trait Application<L: Loop> {
    fn needs_tick(&mut self) -> Timeout;
    fn on_tick(&mut self, loop_: &mut L);
    fn on_packet(&mut self, loop_: &mut L, chunk: Chunk);
    fn on_connless_packet(&mut self, loop_: &mut L, chunk: ConnlessChunk);
    fn on_connect(&mut self, loop_: &mut L, pid: PeerId);
    fn on_ready(&mut self, loop_: &mut L, pid: PeerId);
    fn on_disconnect(&mut self, loop_: &mut L, pid: PeerId, remote: bool, reason: &[u8]);
}

pub struct SocketLoop {
    socket: Socket,
    net: Net<Addr>,
    want_to_flush: PeerSet,
    disconnected: Takeable<PeerMap<ArrayVec<[u8; 1024]>>>,
    server: bool,
}

impl Loop for SocketLoop {
    fn accept_connections_on_port(port: u16) -> SocketLoop {
        SocketLoop {
            socket: Socket::bound(port).unwrap(),
            net: Net::server(),
            want_to_flush: PeerSet::new(),
            disconnected: Default::default(),
            server: true,
        }
    }
    fn client() -> SocketLoop {
        SocketLoop {
            socket: Socket::new().unwrap(),
            net: Net::client(),
            want_to_flush: PeerSet::new(),
            disconnected: Default::default(),
            server: false,
        }
    }
    fn run<A: Application<SocketLoop>>(mut self, mut application: A) {
        let mut buf1: ArrayVec<[u8; 4096]> = ArrayVec::new();
        let mut buf2: ArrayVec<[u8; 4096]> = ArrayVec::new();

        loop {
            self.net
                .tick(&mut self.socket)
                .foreach(|e| panic!("{:?}", e));
            application.on_tick(&mut self);

            for pid in self.want_to_flush.drain() {
                self.net.flush(&mut self.socket, pid).unwrap();
            }

            let mut disconnected = self.disconnected.take();
            for (pid, reason) in disconnected.drain() {
                application.on_disconnect(&mut self, pid, false, &reason);
            }
            self.disconnected.restore(disconnected);

            let sleep_timeout = cmp::min(self.net.needs_tick(), application.needs_tick());
            let sleep_duration = sleep_timeout.time_from(self.socket.time());
            if !self.server && sleep_duration.is_none() {
                break;
            }
            self.socket.sleep(sleep_duration).unwrap();

            while let Some(res) = {
                buf1.clear();
                self.socket.receive(&mut buf1)
            } {
                let (addr, data) = res.unwrap();
                buf2.clear();
                let (iter, res) = self.net.feed(
                    &mut self.socket,
                    &mut Warn(addr, data),
                    addr,
                    data,
                    &mut buf2,
                );
                res.unwrap();
                for mut chunk in iter {
                    if !self.net.is_receive_chunk_still_valid(&mut chunk) {
                        continue;
                    }
                    use libtw2_net::net::ChunkOrEvent::*;
                    match chunk {
                        Chunk(c) => application.on_packet(&mut self, c),
                        Connless(c) => application.on_connless_packet(&mut self, c),
                        Connect(pid) => application.on_connect(&mut self, pid),
                        Ready(pid) => application.on_ready(&mut self, pid),
                        Disconnect(pid, r) => application.on_disconnect(&mut self, pid, true, r),
                    }
                }
            }

            let mut disconnected = self.disconnected.take();
            for (pid, reason) in disconnected.drain() {
                application.on_disconnect(&mut self, pid, false, &reason);
            }
            self.disconnected.restore(disconnected);
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
        if self.want_to_flush.contains(pid) {
            self.net.flush(&mut self.socket, pid).unwrap();
            self.want_to_flush.remove(pid);
        }
        self.disconnected
            .insert(pid, reason.iter().cloned().collect());
        self.net.disconnect(&mut self.socket, pid, reason).unwrap();
    }
    fn send_connless(&mut self, addr: Addr, data: &[u8]) {
        self.net
            .send_connless(&mut self.socket, addr, data)
            .unwrap();
    }
    fn send(&mut self, chunk: Chunk) {
        self.net.send(&mut self.socket, chunk).unwrap();
    }
    fn force_flush(&mut self, pid: PeerId) {
        if self.want_to_flush.contains(pid) {
            self.want_to_flush.remove(pid);
        }
        self.net.flush(&mut self.socket, pid).unwrap();
    }
    fn flush(&mut self, pid: PeerId) {
        self.want_to_flush.insert(pid);
    }
    fn ignore(&mut self, pid: PeerId) {
        self.net.ignore(pid);
    }
    fn accept(&mut self, pid: PeerId) {
        self.net.accept(&mut self.socket, pid).unwrap();
    }
    fn reject(&mut self, pid: PeerId, reason: &[u8]) {
        self.net.reject(&mut self.socket, pid, reason).unwrap();
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
