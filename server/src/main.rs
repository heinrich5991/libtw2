extern crate arrayvec;
extern crate common;
extern crate event_loop;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
extern crate net;
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
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::game;
use gamenet::msg::system;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use net::net::Chunk;
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use net::net::ChunkType;
use net::net::PeerId;
use packer::Unpacker;
use packer::with_packer;
use std::fmt;
use std::fmt::Write;

fn hexdump(level: LogLevel, data: &[u8]) {
    if log_enabled!(level) {
        hexdump_iter(data).foreach(|s| log!(level, "{}", s));
    }
}

// TODO: Attach peer ids to warnings.
struct Warn<'a>(&'a [u8]);

impl<'a, W: fmt::Debug> warn::Warn<W> for Warn<'a> {
    fn warn(&mut self, w: W) {
        warn!("{:?}", w);
        hexdump(LogLevel::Warn, self.0);
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
        let msg = match SystemOrGame::decode(&mut Warn(data), &mut Unpacker::new(data)) {
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
    fn process_event(&mut self, event: ChunkOrEvent<Addr>) {
        match event {
            ChunkOrEvent::Connect(pid) => {
                self.loop_.accept(pid);
            },
            ChunkOrEvent::Chunk(chunk) => match chunk {
                Chunk { addr: ChunkAddr::Peer(_, ChunkType::Connless), .. } => {},
                Chunk { addr: ChunkAddr::Peer(pid, type_), data } => {
                    self.process_client_packet(pid, type_ == ChunkType::Vital, data);
                }
                _ => {},
            },
            _ => {},
        }
    }
}

fn main() {
    logger::init();
    Server::run::<SocketLoop>();
}
