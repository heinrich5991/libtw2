extern crate arrayvec;
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

use arrayvec::ArrayVec;
use event_loop::Addr;
use event_loop::Application;
use event_loop::Loop;
use event_loop::SocketLoop;
use event_loop::Timeout;
use gamenet::VERSION;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
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
        match msg {
            SystemOrGame::System(System::Info(info)) => {
                if info.version == VERSION {
                    if info.password == Some(b"foobar") {
                        self.loop_.sends(pid, system::MapChange {
                            name: b"dm1",
                            crc: 0xf2159e6e_u32 as i32,
                            size: 5805,
                        });
                    } else {
                        self.loop_.disconnect(pid, b"Wrong password");
                    }
                } else {
                    unimplemented!();
                }
            }
            _ => {},
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
                    self.loop_.flush(pid);
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
