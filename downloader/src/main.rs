extern crate arrayvec;
extern crate buffer;
extern crate env_logger;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate mio;
extern crate net;
extern crate num;
extern crate rand;
extern crate warn;

use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use gamenet::msg::System;
use gamenet::msg::system::Info;
use gamenet::msg::system::Input;
use gamenet::msg::system::MapChange;
use gamenet::msg::system::MapData;
use gamenet::msg::system::Ready;
use gamenet::msg::system::RequestMapData;
use gamenet::msg::system::Snap;
use gamenet::msg::system::SnapEmpty;
use gamenet::msg::system::SnapSingle;
use gamenet::msg::system;
use gamenet::packer::Unpacker;
use gamenet::packer::with_packer;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use mio::udp::UdpSocket;
use net::net::Chunk;
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use net::net::ChunkType;
use net::net::Net;
use net::net::PeerId;
use num::ToPrimitive;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io::Write;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use std::time::Instant;
use std::u32;

const NETWORK_LOSS_RATE: f32 = 0.0;
const VERSION: &'static [u8] = b"0.6 626fce9a778df4d4";

fn loss() -> bool {
    assert!(0.0 <= NETWORK_LOSS_RATE && NETWORK_LOSS_RATE <= 1.0);
    NETWORK_LOSS_RATE != 0.0 && rand::random::<f32>() < NETWORK_LOSS_RATE
}

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
struct Addr {
    ip: IpAddr,
    port: u16,
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

struct Socket {
    start: Instant,
    time_cached: Duration,
    poll: mio::Poll,
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
        })
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
    fn sleep(&mut self, duration: Duration) -> io::Result<()> {
        let milliseconds = duration.to_milliseconds_saturating().to_usize().unwrap();
        try!(self.poll.poll(Some(milliseconds)));
        // TODO: Add a verification that this also works with
        // ```
        // try!(self.poll.poll(None));
        // ```
        // on loss-free networks.
        Ok(())
    }
    fn update_time_cached(&mut self) {
        self.time_cached = self.start.elapsed()
    }
}

impl net::net::Callback<Addr> for Socket {
    type Error = io::Error;
    fn send(&mut self, addr: Addr, data: &[u8]) -> Result<(), io::Error> {
        if loss() {
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
    }
    fn time(&mut self) -> Duration {
        self.time_cached
    }
}

struct Warn<'a>(&'a [u8]);

impl<'a, W: fmt::Debug> warn::Warn<W> for Warn<'a> {
    fn warn(&mut self, w: W) {
        warn!("{:?}", w);
        hexdump(LogLevel::Warn, self.0);
    }
}

fn parse_connections<'a, I: Iterator<Item=String>>(iter: I) -> Option<Vec<Addr>> {
    iter.map(|s| Addr::from_str(&s).ok()).collect()
}

#[derive(Clone, Debug)]
struct Peer {
    state: PeerState,
}

impl Peer {
    fn new() -> Peer {
        Peer {
            state: PeerState::Connecting,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PeerState {
    Connecting,
    SentInfo,
    // DownloadingMap(dummy, crc, chunk)
    DownloadingMap(bool, i32, i32),
    // SentReady(dummy, num_chunks)
    SentReady(bool, u32),
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
        }).encode(p).unwrap());
        let mut main = Main {
            socket: Socket::new().unwrap(),
            peers: HashMap::with_capacity(addresses.len()),
            net: Net::client(),
            version_msg: version_msg,
        };
        for &addr in addresses {
            let (pid, err) = main.net.connect(&mut main.socket, addr);
            err.unwrap();
            main.peers.insert(pid, Peer::new());
        }
        main
    }
    fn process_connected_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) -> bool {
        fn send(msg: System, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
            let mut buf: ArrayVec<[u8; 32]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            net.send(socket, Chunk {
                data: &buf,
                addr: ChunkAddr::Peer(pid, ChunkType::Vital),
            }).unwrap();
            net.flush(socket, pid).unwrap();
        }
        let _ = vital;
        let msg;
        if let Ok(m) = System::decode(&mut Warn(data), &mut Unpacker::new(data)) {
            msg = m;
        } else {
            if data.len() >= 1 && data[0] == b'\x02' {
                // MOTD message, we successfully connected.
                return true;
            }
            warn!("decode error:");
            hexdump(LogLevel::Warn, data);
            return false;
        }
        debug!("{:?}", msg);
        let mut request_chunk = None;
        let mut processed = false;
        {
            let state = &mut self.peers.get_mut(&pid).expect("invalid pid").state;
            if let System::MapChange(MapChange { crc, size, name }) = msg {
                if let Some(_) = size.to_usize() {
                    request_chunk = Some(0);
                    match *state {
                        PeerState::SentInfo => {}
                        PeerState::SentReady(true, _) => info!("now getting real map"),
                        _ => warn!("map change from state {:?}", *state),
                    }
                    let dummy = name == b"dummy" && crc as u32 == 0xbeae0b9f;
                    *state = PeerState::DownloadingMap(dummy, crc, 0);
                    info!("map change: {:?}", String::from_utf8_lossy(name));
                    processed = true;
                }
            }
            match *state {
                PeerState::Connecting => unreachable!(),
                PeerState::SentInfo => {}, // Handled above.
                PeerState::DownloadingMap(dummy, cur_crc, cur_chunk) => {
                    if let System::MapData(MapData { last, crc, chunk, .. }) = msg {
                        if cur_crc == crc && cur_chunk == chunk {
                            if last != 0 {
                                *state = PeerState::SentReady(dummy, 0);
                                let m = System::Ready(Ready);
                                send(m, pid, &mut self.net, &mut self.socket);
                                info!("finished");
                            } else {
                                let cur_chunk = cur_chunk.checked_add(1).unwrap();
                                *state = PeerState::DownloadingMap(dummy, cur_crc, cur_chunk);
                                request_chunk = Some(cur_chunk);
                                print!("{}\r", cur_chunk);
                                io::stdout().flush().unwrap();
                            }
                        }
                        processed = true;
                    }
                }
                PeerState::SentReady(dummy, num_snaps) => {
                    match msg {
                        System::ConReady(..) => {
                            processed = true;
                        }
                        System::Snap(Snap { tick, .. })
                        | System::SnapEmpty(SnapEmpty { tick, .. })
                        | System::SnapSingle(SnapSingle { tick, .. })
                        => {
                            let num_snaps = num_snaps.checked_add(1).unwrap();
                            *state = PeerState::SentReady(dummy, num_snaps);
                            if num_snaps == 3 {
                                send(System::Input(Input {
                                    ack_snapshot: tick,
                                    intended_tick: tick,
                                    input: system::INPUT_DATA_EMPTY,
                                }), pid, &mut self.net, &mut self.socket);
                            }
                            processed = true;
                        }
                        _ => {},
                    }
                },
            }
        }
        if !processed {
            warn!("unprocessed message {:?}", msg);
        }
        request_chunk.map(|c| {
            let m = System::RequestMapData(RequestMapData { chunk: c });
            send(m, pid, &mut self.net, &mut self.socket);
        });
        false
    }
    fn process_event(&mut self, chunk: ChunkOrEvent<Addr>) -> bool {
        match chunk {
            ChunkOrEvent::Ready(pid) => {
                let p = self.peers.get_mut(&pid).expect("invalid pid");
                p.state = PeerState::SentInfo;
                self.net.send(&mut self.socket, Chunk {
                    data: &self.version_msg,
                    addr: ChunkAddr::Peer(pid, ChunkType::Vital)
                }).unwrap();
                self.net.flush(&mut self.socket, pid).unwrap();
                false
            }
            ChunkOrEvent::Chunk(Chunk {
                addr: ChunkAddr::Peer(pid, type_),
                data,
            }) => {
                if type_ != ChunkType::Connless {
                    self.process_connected_packet(pid, type_ == ChunkType::Vital, data)
                } else {
                    false
                }
            }
            ChunkOrEvent::Chunk(..) => false,
            ChunkOrEvent::Disconnect(pid, reason) => {
                error!("disconnected pid={:?} error={:?}", pid, String::from_utf8_lossy(reason));
                false
            },
            ChunkOrEvent::Connect(..) => unreachable!(),
        }
    }
    fn run(&mut self) {
        let mut buf1: ArrayVec<[u8; 4096]> = ArrayVec::new();
        let mut buf2: ArrayVec<[u8; 4096]> = ArrayVec::new();
        while self.net.needs_tick() {
            self.net.tick(&mut self.socket).foreach(|e| panic!("{:?}", e));
            self.socket.sleep(Duration::from_millis(50)).unwrap();
            self.socket.update_time_cached();

            while let Some(res) = { buf1.clear(); self.socket.receive(&mut buf1) } {
                if loss() {
                    continue;
                }
                let (addr, data) = res.unwrap();
                dump(Direction::Receive, addr, data);
                buf2.clear();
                let (iter, res) = self.net.feed(&mut self.socket, &mut Warn(data), addr, data, &mut buf2);
                res.unwrap();
                for chunk in iter {
                    if self.process_event(chunk) {
                        let pid = match chunk {
                            ChunkOrEvent::Chunk(Chunk {
                                addr: ChunkAddr::Peer(pid, _), ..
                            }) => pid,
                            ChunkOrEvent::Connect(pid) => pid,
                            ChunkOrEvent::Disconnect(pid, _) => pid,
                            ChunkOrEvent::Ready(pid) => pid,
                            _ => unreachable!(),
                        };
                        self.net.disconnect(&mut self.socket, pid, b"maps").unwrap();
                        break;
                    }
                }
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    let args = env::args().dropping(1);
    let addresses = parse_connections(args).expect("invalid addresses");
    Main::init(&addresses).run();
}
