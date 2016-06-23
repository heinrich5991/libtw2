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
use net::net::Chunk;
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use socket::Addr;
use socket::Socket;
use std::fmt;
use std::time::Duration;

fn hexdump(level: LogLevel, data: &[u8]) {
    if log_enabled!(level) {
        hexdump_iter(data).foreach(|s| log!(level, "{}", s));
    }
}

struct Warn<'a>(&'a [u8]);

impl<'a, W: fmt::Debug> warn::Warn<W> for Warn<'a> {
    fn warn(&mut self, w: W) {
        warn!("{:?}", w);
        hexdump(LogLevel::Warn, self.0);
    }
}

struct Server {
    socket: Socket,
    net: Net<Addr>,
}

impl Server {
    fn init() -> Server {
        Server {
            socket: Socket::bound(8303).unwrap(),
            net: Net::server(),
        }
    }
    fn run(&mut self) {
        let mut buf1: ArrayVec<[u8; 4096]> = ArrayVec::new();
        let mut buf2: ArrayVec<[u8; 4096]> = ArrayVec::new();

        loop {
            let sleep_time;
            if self.net.needs_tick() {
                self.net.tick(&mut self.socket).foreach(|e| panic!("{:?}", e));
                sleep_time = Some(Duration::from_millis(50));
            } else {
                sleep_time = None;
            }
            self.socket.sleep(sleep_time).unwrap();

            while let Some(res) = { buf1.clear(); self.socket.receive(&mut buf1) } {
                let (addr, data) = res.unwrap();
                buf2.clear();
                let (iter, res) = self.net.feed(&mut self.socket, &mut Warn(data), addr, data, &mut buf2);
                res.unwrap();
                for chunk in iter {
                    match chunk {
                        ChunkOrEvent::Connect(pid) => {
                            self.net.disconnect(&mut self.socket, pid, b"unwanted").unwrap();
                        }
                        x @ ChunkOrEvent::Chunk(Chunk {
                            addr: ChunkAddr::NonPeerConnless(..), ..
                        }) => warn!("unknown connless packet {:?}", x),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

fn main() {
    logger::init();
    Server::init().run();
}
