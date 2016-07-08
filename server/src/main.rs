extern crate arrayvec;
extern crate common;
extern crate event_loop;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
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
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use net::net::ChunkType;
use net::net::PeerId;
use num::ToPrimitive;
use packer::Unpacker;
use packer::with_packer;
use std::fmt::Write;
use std::fmt;

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
                data: &buf,
                addr: ChunkAddr::Peer(pid, ChunkType::Vital),
            })
        }
        inner(msg.into(), pid, self)
    }
    fn sendg<'a, G: Into<Game<'a>>>(&mut self, pid: PeerId, msg: G) {
        fn inner<L: Loop+?Sized>(msg: Game, pid: PeerId, loop_: &mut L) {
            let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            loop_.send(Chunk {
                data: &buf,
                addr: ChunkAddr::Peer(pid, ChunkType::Vital),
            })
        }
        inner(msg.into(), pid, self)
    }
    fn sendc<'a, C: Into<Connless<'a>>>(&mut self, addr: Addr, msg: C) {
        fn inner<L: Loop+?Sized>(msg: Connless, addr: Addr, loop_: &mut L) {
            let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            loop_.send(Chunk {
                data: &buf,
                addr: ChunkAddr::NonPeerConnless(addr),
            })
        }
        inner(msg.into(), addr, self)
    }
}
impl<L: Loop> LoopExt for L { }

struct Server;

struct ServerLoop<'a, L: Loop+'a> {
    loop_: &'a mut L,
}

impl<L: Loop> Application<L> for Server {
    fn needs_tick(&mut self) -> Timeout { Timeout::inactive() }
    fn on_tick(&mut self, _: &mut L) { }

    fn on_packet(&mut self, loop_: &mut L, event: ChunkOrEvent<Addr>) {
        ServerLoop { loop_: loop_ }.process_event(event);
    }
}

impl Server {
    fn run<L: Loop>() {
        L::accept_connections_on_port(8303).run(Server);
    }
}
impl<'a, L: Loop> ServerLoop<'a, L> {
    fn process_client_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) {
        let msg = match SystemOrGame::decode(&mut Warn(pid, data), &mut Unpacker::new(data)) {
            Ok(m) => m,
            Err(err) => {
                warn!("decode error {:?}:", err);
                hexdump(LogLevel::Warn, data);
                return;
            }
        };
        if !vital {
            warn!("non-vital message {:?}", msg);
            return;
        }
        let mut processed = false;
        match msg {
            SystemOrGame::System(System::Info(info)) => {
                if info.version == VERSION {
                    if info.password == Some(b"foobar") {
                        self.loop_.sends(pid, system::MapChange {
                            name: b"dm1",
                            crc: 0xf2159e6e_u32 as i32,
                            size: 5805,
                        });
                        self.loop_.flush(pid);
                    } else {
                        self.loop_.disconnect(pid, b"Wrong password");
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
                }
                processed = true;
            }
            SystemOrGame::System(System::Ready(system::Ready)) => {
                self.loop_.sendg(pid, game::SvMotd {
                    message: b"Hello World!",
                });
                self.loop_.sends(pid, system::ConReady);
                self.loop_.flush(pid);
                processed = true;
            }
            SystemOrGame::Game(Game::ClStartInfo(info)) => {
                info!("{:?}:{} enters the game", pid, AlmostString::new(info.name));
                self.loop_.sendg(pid, game::SvVoteClearOptions);
                self.loop_.sendg(pid, game::SV_TUNE_PARAMS_DEFAULT);
                self.loop_.sendg(pid, game::SvReadyToEnter);
                self.loop_.flush(pid);
                processed = true;
            }
            SystemOrGame::System(System::EnterGame(system::EnterGame)) => {
                for i in 0..3 {
                    self.loop_.sends(pid, system::SnapEmpty {
                        tick: i,
                        delta_tick: i - (-1),
                    });
                }
                self.loop_.flush(pid);
                processed = true;
            }
            _ => {},
        }
        if !processed {
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
                    flags: 0,
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
                self.loop_.accept(pid);
            },
            ChunkOrEvent::Chunk(chunk) => match chunk {
                Chunk { addr: ChunkAddr::Peer(_, ChunkType::Connless), .. }
                    => unimplemented!(),
                Chunk { addr: ChunkAddr::Peer(pid, type_), data } => {
                    self.process_client_packet(pid, type_ == ChunkType::Vital, data);
                },
                Chunk { addr: ChunkAddr::NonPeerConnless(addr), data } => {
                    self.process_connless_packet(addr, data);
                },
            },
            _ => {},
        }
    }
}

fn main() {
    logger::init();
    Server::run::<SocketLoop>();
}
