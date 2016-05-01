extern crate arrayvec;
extern crate buffer;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
extern crate mio;
extern crate net;

use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use gamenet::msg::System;
use gamenet::msg::system::Info;
use gamenet::msg::system::MapChange;
use gamenet::msg::system::MapData;
use gamenet::msg::system::RequestMapData;
use gamenet::packer::Unpacker;
use gamenet::packer::with_packer;
use hexdump::hexdump;
use itertools::Itertools;
use mio::udp::UdpSocket;
use net::net::Chunk;
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use net::net::ChunkType;
use net::net::Net;
use net::net::PeerId;
use std::collections::HashMap;
use std::env;
use std::io;
use std::mem;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const VERSION: &'static [u8] = b"0.6 626fce9a778df4d4";

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Addr {
    ip: IpAddr,
    port: u16,
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

struct Socket {
    last_tick: Instant,
    v4: UdpSocket,
    v6: UdpSocket,
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
    fn new() -> io::Result<Socket> {
        Ok(Socket {
            last_tick: Instant::now(),
            v4: try!(udp_socket("0.0.0.0:0")),
            v6: try!(udp_socket("[::]:0")),
        })
    }
    fn next_tick_delta(&mut self) -> Duration {
        let now = Instant::now();
        now.duration_since(mem::replace(&mut self.last_tick, now))
    }
    fn receive<'a, B: Buffer<'a>>(&mut self, buf: B)
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
        Some(result.map(|(len, addr)| unsafe {
            buf.advance(len);
            (Addr::from(addr), buf.initialized())
        }))
    }
}

impl net::net::Callback<Addr> for Socket {
    type Error = io::Error;
    fn send(&mut self, addr: Addr, data: &[u8]) -> Result<(), io::Error> {
        let sock_addr = SocketAddr::new(addr.ip, addr.port);
        let socket = if let IpAddr::V4(..) = addr.ip {
            &mut self.v4
        } else {
            &mut self.v6
        };
        swap(socket.send_to(data, &sock_addr))
            .unwrap_or_else(|| Err(io::Error::new(io::ErrorKind::WouldBlock, "write would block")))
            .map(|s| assert!(data.len() == s))
    }
    fn time_since_tick(&mut self) -> Duration {
        self.last_tick.elapsed()
    }
}

fn parse_connections<'a, I: Iterator<Item=String>>(iter: I) -> Option<Vec<Addr>> {
    iter.map(|s| Addr::from_str(&s).ok()).collect()
}

enum Peer {
    Connecting,
    SentInfo,
}

struct Main {
    socket: Socket,
    peers: HashMap<PeerId, Peer>,
    net: Net<Addr>,
    version_msg: ArrayVec<[u8; 32]>,
}

impl Main {
    fn init(addresses: &[Addr]) -> Main {
        let mut version_msg = ArrayVec::new();
        with_packer(&mut version_msg, |p| System::Info(Info {
            version: VERSION,
            password: Some(b""),
        }).encode_complete(p).unwrap());
        let mut main = Main {
            socket: Socket::new().unwrap(),
            peers: HashMap::with_capacity(addresses.len()),
            net: Net::new(),
            version_msg: version_msg,
        };
        for &addr in addresses {
            let (pid, err) = main.net.connect(&mut main.socket, addr);
            err.unwrap();
            main.peers.insert(pid, Peer::Connecting);
        }
        main
    }
    fn process_connected_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) {
        let mut buf: ArrayVec<[u8; 32]> = ArrayVec::new();
        match System::decode_complete(&mut Unpacker::new(data)).unwrap() {
            System::MapChange(MapChange { .. }) => {
                with_packer(&mut buf, |p| System::RequestMapData(RequestMapData {
                    chunk: 0,
                }).encode_complete(p));
                self.net.send(&mut self.socket, Chunk {
                    data: &buf,
                    addr: ChunkAddr::Peer(pid, ChunkType::Vital),
                }).unwrap();
                self.net.flush(&mut self.socket, pid).unwrap();
            }
            System::MapData(MapData { last, chunk, data, .. }) => {
                hexdump(data);
                if last == 0 {
                    with_packer(&mut buf, |p| System::RequestMapData(RequestMapData {
                        chunk: chunk + 1,
                    }).encode_complete(p));
                    self.net.send(&mut self.socket, Chunk {
                        data: &buf,
                        addr: ChunkAddr::Peer(pid, ChunkType::Vital),
                    }).unwrap();
                    self.net.flush(&mut self.socket, pid).unwrap();
                } else {
                    self.net.disconnect(&mut self.socket, pid, b"done downloading");
                }
            }
            c => println!("{:?}", c),
        }
    }
    fn process_event(&mut self, chunk: ChunkOrEvent<Addr>) {
        match chunk {
            ChunkOrEvent::Ready(pid) => {
                let p = self.peers.get_mut(&pid).expect("invalid pid");
                *p = Peer::SentInfo;
                self.net.send(&mut self.socket, Chunk {
                    data: &self.version_msg,
                    addr: ChunkAddr::Peer(pid, ChunkType::Vital)
                }).unwrap();
                self.net.flush(&mut self.socket, pid).unwrap();
            }
            ChunkOrEvent::Chunk(Chunk {
                addr: ChunkAddr::Peer(pid, type_),
                data,
            }) => {
                if type_ != ChunkType::Connless {
                    self.process_connected_packet(pid, type_ == ChunkType::Vital, data);
                }
            }
            _ => {}
        }
    }
    fn run(&mut self) {
        let mut buf1: ArrayVec<[u8; 4096]> = ArrayVec::new();
        let mut buf2: ArrayVec<[u8; 4096]> = ArrayVec::new();
        while self.net.needs_tick() {
            while let Some(res) = { buf1.clear(); self.socket.receive(&mut buf1) } {
                let (addr, data) = res.unwrap();
                buf2.clear();
                let (iter, res) = self.net.feed(&mut self.socket, addr, data, &mut buf2);
                res.unwrap();
                for chunk in iter {
                    self.process_event(chunk);
                }
            }
            let delta = self.socket.next_tick_delta();
            self.net.tick(&mut self.socket, delta).foreach(|e| panic!("{:?}", e));
            thread::sleep(Duration::from_millis(50));
        }
    }
}

fn main() {
    let args = env::args().dropping(1);
    let addresses = parse_connections(args).expect("invalid addresses");
    Main::init(&addresses).run();
}
