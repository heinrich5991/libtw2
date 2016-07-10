extern crate arrayvec;
extern crate common;
extern crate event_loop;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
#[macro_use] extern crate matches;
extern crate net;
extern crate num;
extern crate packer;
extern crate socket;
extern crate warn;

use arrayvec::ArrayString;
use arrayvec::ArrayVec;
use common::pretty::AlmostString;
use event_loop::Addr;
use event_loop::Application;
use event_loop::Loop;
use event_loop::SocketLoop;
use event_loop::Timeout;
use gamenet::VERSION;
use gamenet::msg::Connless;
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::connless;
use gamenet::msg::game;
use gamenet::msg::system;
use gamenet::msg;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use net::net::Chunk;
use net::net::ChunkOrEvent;
use net::net::ConnlessChunk;
use net::net::PeerId;
use net::time::Timestamp;
use num::ToPrimitive;
use packer::Unpacker;
use packer::with_packer;
use std::collections::HashMap;
use std::collections::hash_map;
use std::fmt::Write;
use std::fmt;
use std::ops;
use std::time::Duration;

const TICKS_PER_SECOND: u32 = 50;

fn hexdump(level: LogLevel, data: &[u8]) {
    if log_enabled!(level) {
        hexdump_iter(data).foreach(|s| log!(level, "{}", s));
    }
}

struct Warn<'a, T: fmt::Debug>(T, &'a [u8]);

impl<'a, T: fmt::Debug, W: fmt::Debug> warn::Warn<W> for Warn<'a, T> {
    fn warn(&mut self, w: W) {
        warn!("{:?}: {:?}", self.0, w);
        hexdump(LogLevel::Warn, self.1);
    }
}

trait LoopExt: Loop {
    fn sends<'a, S: Into<System<'a>>>(&mut self, pid: PeerId, msg: S) {
        fn inner<L: Loop+?Sized>(msg: System, pid: PeerId, loop_: &mut L) {
            let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            loop_.send(Chunk {
                pid: pid,
                vital: true,
                data: &buf,
            })
        }
        inner(msg.into(), pid, self)
    }
    fn sendg<'a, G: Into<Game<'a>>>(&mut self, pid: PeerId, msg: G) {
        fn inner<L: Loop+?Sized>(msg: Game, pid: PeerId, loop_: &mut L) {
            let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            loop_.send(Chunk {
                pid: pid,
                vital: true,
                data: &buf,
            })
        }
        inner(msg.into(), pid, self)
    }
    fn sendc<'a, C: Into<Connless<'a>>>(&mut self, addr: Addr, msg: C) {
        fn inner<L: Loop+?Sized>(msg: Connless, addr: Addr, loop_: &mut L) {
            let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            loop_.send_connless(addr, &buf)
        }
        inner(msg.into(), addr, self)
    }
}
impl<L: Loop> LoopExt for L { }

#[derive(Default)]
struct Server {
    peers: Peers,
    game_start: Timestamp,
    game_tick: u32,
}

impl Server {
    fn game_tick_time(&self, tick: u32) -> Timestamp {
        let millis = tick.to_u64().unwrap() * 1000 / TICKS_PER_SECOND.to_u64().unwrap();
        self.game_start + Duration::from_millis(millis)
    }
}

#[derive(Default)]
struct Peers {
    peers: HashMap<PeerId, Peer>,
}

impl Peers {
    fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
    fn insert(&mut self, pid: PeerId, peer: Peer) {
        self.peers.insert(pid, peer);
    }
    fn entry(&mut self, pid: PeerId) -> hash_map::OccupiedEntry<PeerId, Peer> {
        match self.peers.entry(pid) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => panic!("invalid pid"),
        }
    }
    fn remove(&mut self, pid: PeerId) {
        self.peers.remove(&pid).expect("invalid pid");
    }
}

impl ops::Index<PeerId> for Peers {
    type Output = Peer;
    fn index(&self, pid: PeerId) -> &Peer {
        self.peers.get(&pid).expect("invalid pid")
    }
}

impl ops::IndexMut<PeerId> for Peers {
    fn index_mut(&mut self, pid: PeerId) -> &mut Peer {
        self.peers.get_mut(&pid).expect("invalid pid")
    }
}

#[derive(Default)]
struct Peer {
    state: PeerState,
}

impl Default for PeerState {
    fn default() -> PeerState {
        PeerState::SystemInfo
    }
}

enum PeerState {
    SystemInfo,
    SystemReady,
    GameInfo,
    SystemEnterGame,
    Ingame,
}

struct ServerLoop<'a, L: Loop+'a> {
    loop_: &'a mut L,
    server: &'a mut Server,
}

impl<L: Loop> Application<L> for Server {
    fn needs_tick(&mut self) -> Timeout {
        if !self.peers.is_empty() {
            Timeout::active(self.game_tick_time(self.game_tick + 1) + Duration::from_millis(1))
        } else {
            Timeout::inactive()
        }
    }
    fn on_tick(&mut self, loop_: &mut L) {
        ServerLoop { server: self, loop_: loop_ }.tick();
    }

    fn on_packet(&mut self, loop_: &mut L, event: ChunkOrEvent<Addr>) {
        ServerLoop { server: self, loop_: loop_ }.process_event(event);
    }
}

impl Server {
    fn run<L: Loop>() {
        L::accept_connections_on_port(8303).run(Server::default());
    }
}
impl<'a, L: Loop> ServerLoop<'a, L> {
    fn process_client_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) {
        use PeerState::*;

        let msg = match SystemOrGame::decode(&mut Warn(pid, data), &mut Unpacker::new(data)) {
            Ok(m) => m,
            Err(err) => {
                warn!("decode error {:?}:", err);
                hexdump(LogLevel::Warn, data);
                return;
            }
        };
        if !vital && !matches!(msg, SystemOrGame::System(System::Input(..))) {
            warn!("non-vital message {:?}", msg);
            return;
        }
        let mut ignored = false;
        let mut processed = false;
        let mut peer = self.server.peers.entry(pid);
        match (&peer.get().state, msg) {
            (&SystemInfo, SystemOrGame::System(System::Info(info))) => {
                if info.version == VERSION {
                    if info.password == Some(b"foobar") {
                        self.loop_.sends(pid, system::MapChange {
                            name: b"dm1",
                            crc: 0xf2159e6e_u32 as i32,
                            size: 5805,
                        });
                        self.loop_.flush(pid);
                        peer.get_mut().state = SystemReady;
                    } else {
                        self.loop_.disconnect(pid, b"Wrong password");
                        peer.remove();
                    }
                } else {
                    let mut buf: ArrayString<[u8; 128]> = ArrayString::new();
                    write!(
                        &mut buf,
                        "Wrong version. Server is running '{}' and client '{}'",
                        AlmostString::new(VERSION),
                        AlmostString::new(info.version),
                    ).unwrap_or_else(|_| {
                        buf.clear();
                        write!(
                            &mut buf,
                            "Wrong version. Server is running '{}' and client version is too long",
                            AlmostString::new(VERSION)
                        )
                    }.unwrap());
                    self.loop_.disconnect(pid, buf.as_bytes());
                    peer.remove();
                }
                processed = true;
            }
            (&SystemReady, SystemOrGame::System(System::Ready(system::Ready))) => {
                self.loop_.sendg(pid, game::SvMotd {
                    message: b"Hello World!",
                });
                self.loop_.sends(pid, system::ConReady);
                self.loop_.flush(pid);
                peer.get_mut().state = GameInfo;
                processed = true;
            }
            (&GameInfo, SystemOrGame::Game(Game::ClStartInfo(info))) => {
                info!("{:?}:{} enters the game", pid, AlmostString::new(info.name));
                self.loop_.sendg(pid, game::SvVoteClearOptions);
                self.loop_.sendg(pid, game::SV_TUNE_PARAMS_DEFAULT);
                self.loop_.sendg(pid, game::SvReadyToEnter);
                self.loop_.flush(pid);
                peer.get_mut().state = SystemEnterGame;
                processed = true;
            }
            (&SystemEnterGame, SystemOrGame::System(System::EnterGame(system::EnterGame))) => {
                for i in 0..3 {
                    self.loop_.sends(pid, system::SnapEmpty {
                        tick: i,
                        delta_tick: i - (-1),
                    });
                }
                self.loop_.flush(pid);
                peer.get_mut().state = Ingame;
                processed = true;
            }
            (&Ingame, SystemOrGame::System(System::Input(..))) => {
                ignored = true;
            }
            _ => {},
        }
        if !processed && !ignored {
            warn!("unprocessed message {:?}", msg);
        }
    }
    fn process_connless_packet(&mut self, addr: Addr, data: &[u8]) {
        let msg = match Connless::decode(&mut Warn(addr, data), &mut Unpacker::new(data)) {
            Ok(m) => m,
            Err(err) => {
                warn!("decode error {:?}:", err);
                hexdump(LogLevel::Warn, data);
                return;
            },
        };
        let mut processed = false;
        match msg {
            Connless::RequestInfo(request) => {
                processed = true;
                self.loop_.sendc(addr, connless::Info {
                    token: request.token.to_i32().unwrap(),
                    version: VERSION,
                    name: b"Rust Teeworlds Server",
                    game_type: b"DM",
                    map: b"dm1",
                    flags: 1,
                    num_players: 0,
                    max_players: 16,
                    num_clients: 0,
                    max_clients: 16,
                    clients: msg::CLIENTS_DATA_NONE,
                });
            },
            _ => {},
        }
        if !processed {
            warn!("unprocessed message {:?}", msg);
        }
    }
    fn process_event(&mut self, event: ChunkOrEvent<Addr>) {
        match event {
            ChunkOrEvent::Connect(pid) => {
                if self.server.peers.is_empty() {
                    self.server.game_start = self.loop_.time();
                    self.server.game_tick = 0;
                }
                self.loop_.accept(pid);
                self.server.peers.insert(pid, Peer::default());
            },
            ChunkOrEvent::Chunk(Chunk { pid, vital, data }) => {
                self.process_client_packet(pid, vital, data);
            },
            ChunkOrEvent::Connless(ConnlessChunk { addr, data, .. }) => {
                self.process_connless_packet(addr, data);
            },
            ChunkOrEvent::Ready(..) => unreachable!(),
            ChunkOrEvent::Disconnect(pid, reason) => {
                if !reason.is_empty() {
                    info!("{:?} leaves the game ({})", pid, AlmostString::new(reason));
                } else {
                    info!("{:?} leaves the game", pid);
                }
                self.server.peers.remove(pid);
            },
        }
    }
    fn tick(&mut self) {
        while self.server.game_tick_time(self.server.game_tick + 1) <= self.loop_.time() {
            // TODO: Do tick. :)
            self.server.game_tick += 1;
        }
    }
}

fn main() {
    logger::init();
    Server::run::<SocketLoop>();
}
