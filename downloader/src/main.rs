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
use gamenet::bytes::PrettyBytes;
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::game::ClCallVote;
use gamenet::msg::game::ClStartInfo;
use gamenet::msg::game::SvVoteOptionAdd;
use gamenet::msg::game::SvVoteOptionRemove;
use gamenet::msg::game;
use gamenet::msg::system::EnterGame;
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
use net::net::Callback;
use net::net::Chunk;
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use net::net::ChunkType;
use net::net::Net;
use net::net::PeerId;
use num::ToPrimitive;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::io::Write;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops;
use std::str::FromStr;
use std::str;
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

impl Callback<Addr> for Socket {
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
    visited_votes: HashSet<Vec<u8>>,
    current_votes: HashSet<Vec<u8>>,
    list_votes: HashSet<Vec<u8>>,
    completed_list_votes: HashSet<Vec<u8>>,
    previous_list_vote: Option<Vec<u8>>,
    previous_vote: Option<Vec<u8>>,
    num_snaps_since_reset: u32,
    state: PeerState,
}

impl Peer {
    fn new() -> Peer {
        Peer {
            visited_votes: HashSet::new(),
            current_votes: HashSet::new(),
            list_votes: HashSet::new(),
            completed_list_votes: HashSet::new(),
            previous_list_vote: None,
            previous_vote: None,
            num_snaps_since_reset: 0,
            state: PeerState::Connection,
        }
    }
    fn vote(&mut self, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) -> bool {
        fn send_vote(visited_votes: &mut HashSet<Vec<u8>>, vote: &[u8], pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
            sendg(Game::ClCallVote(ClCallVote {
                type_: game::CL_CALL_VOTE_TYPE_OPTION,
                value: vote,
                reason: b"downloader",
            }), pid, net, socket);
            visited_votes.insert(vote.to_owned());
        }
        self.previous_vote = self.current_votes.difference(&self.visited_votes).cloned().next();
        if let Some(ref vote) = self.previous_vote {
            send_vote(&mut self.visited_votes, vote, pid, net, socket);
            info!("voting for {:?}", PrettyBytes::new(vote));
        } else {
            self.previous_vote = None;
            for vote in &self.current_votes {
                if Some(vote) != self.previous_list_vote.as_ref()
                    && self.list_votes.contains(vote)
                    && !self.completed_list_votes.contains(vote)
                {
                    self.previous_vote = Some(vote.to_owned());
                }
            }
            if let Some(list_vote) = self.previous_list_vote.take() {
                self.completed_list_votes.insert(list_vote);
            }
            if let Some(vote) = self.previous_vote.as_ref() {
                self.previous_list_vote = Some(vote.to_owned());
                info!("list-voting for {:?}", PrettyBytes::new(vote));
                send_vote(&mut self.visited_votes, &vote, pid, net, socket);
            } else {
                info!("voting done");
                return true;
            }
        }
        self.state = PeerState::VoteSet(socket.time() + Duration::from_secs(5));
        false
    }
}

#[derive(Clone, Copy, Debug)]
enum PeerState {
    Connection,
    MapChange,
    // MapData(crc, chunk)
    MapData(i32, i32),
    ConReady,
    ReadyToEnter,
    // VoteSet(timeout)
    VoteSet(Duration),
    VoteEnd,
    // VoteResult(timeout)
    VoteResult(Duration),
}

struct Peers {
    peers: HashMap<PeerId, Peer>,
}

impl Peers {
    fn with_capacity(cap: usize) -> Peers {
        Peers {
            peers: HashMap::with_capacity(cap),
        }
    }
    fn insert(&mut self, pid: PeerId, peer: Peer) {
        self.peers.insert(pid, peer);
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

fn send(msg: System, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
    let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
    with_packer(&mut buf, |p| msg.encode(p).unwrap());
    net.send(socket, Chunk {
        data: &buf,
        addr: ChunkAddr::Peer(pid, ChunkType::Vital),
    }).unwrap();
    net.flush(socket, pid).unwrap();
}
fn sendg(msg: Game, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
    let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
    with_packer(&mut buf, |p| msg.encode(p).unwrap());
    net.send(socket, Chunk {
        data: &buf,
        addr: ChunkAddr::Peer(pid, ChunkType::Vital),
    }).unwrap();
    net.flush(socket, pid).unwrap();
}

struct Main {
    socket: Socket,
    peers: Peers,
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
            peers: Peers::with_capacity(addresses.len()),
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
    fn tick_peer(&mut self, pid: PeerId) -> bool {
        let peer = &mut self.peers[pid];
        match peer.state {
            PeerState::VoteSet(timeout) => {
                if self.socket.time() >= timeout {
                    if peer.vote(pid, &mut self.net, &mut self.socket) {
                        return true;
                    }
                }
                false
            },
            PeerState::VoteResult(timeout) => {
                if self.socket.time() >= timeout {
                    if peer.vote(pid, &mut self.net, &mut self.socket) {
                        return true;
                    }
                }
                false
            },
            _ => false,
        }
    }
    fn process_connected_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) -> bool {
        let _ = vital;
        let msg;
        if let Ok(m) = SystemOrGame::decode(&mut Warn(data), &mut Unpacker::new(data)) {
            msg = m;
        } else {
            warn!("decode error:");
            hexdump(LogLevel::Warn, data);
            return false;
        }
        debug!("{:?}", msg);
        let mut request_chunk = None;
        let mut processed = false;
        match msg {
            SystemOrGame::Game(Game::SvMotd(..))
                | SystemOrGame::Game(Game::SvKillMsg(..))
                | SystemOrGame::Game(Game::SvTuneParams(..))
                | SystemOrGame::Game(Game::SvWeaponPickup(..))
                | SystemOrGame::System(System::InputTiming(..))
            => {
                processed = true;
            },
            SystemOrGame::Game(Game::SvChat(chat)) => {
                if chat.team == 0 && chat.client_id == -1 {
                    processed = true;
                    info!("*** {:?}", PrettyBytes::new(chat.message));
                }
            }
            _ => {},
        }
        {
            let peer = &mut self.peers[pid];
            match msg {
                SystemOrGame::System(ref msg) => match *msg {
                    System::MapChange(MapChange { crc, size, name }) => {
                        if let Some(_) = size.to_usize() {
                            request_chunk = Some(0);
                            match peer.state {
                                PeerState::MapChange => {},
                                PeerState::VoteResult(..) => {},
                                _ => warn!("map change from state {:?}", peer.state),
                            }
                            peer.current_votes.clear();
                            peer.state = PeerState::MapData(crc, 0);
                            peer.num_snaps_since_reset = 0;
                            info!("map change: {:?}", String::from_utf8_lossy(name));
                            processed = true;
                        }
                    },
                    System::Snap(Snap { tick, .. })
                        | System::SnapEmpty(SnapEmpty { tick, .. })
                        | System::SnapSingle(SnapSingle { tick, ..})
                    => {
                        if peer.num_snaps_since_reset <= 3 {
                            peer.num_snaps_since_reset += 1;
                        }
                        if peer.num_snaps_since_reset == 3 {
                            send(System::Input(Input {
                                ack_snapshot: tick,
                                intended_tick: tick,
                                input: system::INPUT_DATA_EMPTY,
                            }), pid, &mut self.net, &mut self.socket);
                        }
                        processed = true;
                    },
                    _ => {},
                },
                SystemOrGame::Game(ref msg) => match *msg {
                    Game::SvVoteClearOptions(..) => {
                        processed = true;
                        peer.current_votes.clear();
                    },
                    Game::SvVoteOptionListAdd(l) => {
                        processed = true;
                        let current_votes = &mut peer.current_votes;
                        let mut ins = |v: &[u8]| {
                            current_votes.insert(v.to_owned());
                        };
                        let len = l.num_options;
                        if len >  0 { ins(l.description0); }
                        if len >  1 { ins(l.description1); }
                        if len >  2 { ins(l.description2); }
                        if len >  3 { ins(l.description3); }
                        if len >  4 { ins(l.description4); }
                        if len >  5 { ins(l.description5); }
                        if len >  6 { ins(l.description6); }
                        if len >  7 { ins(l.description7); }
                        if len >  8 { ins(l.description8); }
                        if len >  9 { ins(l.description9); }
                        if len > 10 { ins(l.description10); }
                        if len > 11 { ins(l.description11); }
                        if len > 12 { ins(l.description12); }
                        if len > 13 { ins(l.description13); }
                        if len > 14 { ins(l.description14); }
                    },
                    Game::SvVoteOptionAdd(SvVoteOptionAdd { description }) => {
                        processed = true;
                        peer.current_votes.insert(description.to_owned());
                    },
                    Game::SvVoteOptionRemove(SvVoteOptionRemove { description }) => {
                        processed = true;
                        // WARN: If vote doesn't exist.
                        peer.current_votes.remove(description);
                    }
                    _ => {},
                }
            }
            match peer.state {
                PeerState::Connection => unreachable!(),
                PeerState::MapChange => {}, // Handled above.
                PeerState::MapData(cur_crc, cur_chunk) => match msg {
                    SystemOrGame::System(System::MapData(MapData { last, crc, chunk, .. })) => {
                        if cur_crc == crc && cur_chunk == chunk {
                            if last != 0 {
                                peer.state = PeerState::ConReady;
                                let m = System::Ready(Ready);
                                send(m, pid, &mut self.net, &mut self.socket);
                                info!("finished");
                            } else {
                                let cur_chunk = cur_chunk.checked_add(1).unwrap();
                                peer.state = PeerState::MapData(cur_crc, cur_chunk);
                                request_chunk = Some(cur_chunk);
                                print!("{}\r", cur_chunk);
                                io::stdout().flush().unwrap();
                            }
                        }
                        processed = true;
                    }
                    _ => {},
                },
                PeerState::ConReady => match msg {
                    SystemOrGame::System(System::ConReady(..)) => {
                        processed = true;
                        sendg(Game::ClStartInfo(ClStartInfo {
                            name: b"downloader",
                            clan: b"",
                            country: -1,
                            skin: b"default",
                            use_custom_color: false,
                            color_body: 0,
                            color_feet: 0,
                        }), pid, &mut self.net, &mut self.socket);
                        peer.state = PeerState::ReadyToEnter;
                    }
                    _ => {},
                },
                PeerState::ReadyToEnter => match msg {
                    SystemOrGame::Game(Game::SvReadyToEnter(..)) => {
                        processed = true;
                        send(System::EnterGame(EnterGame), pid, &mut self.net, &mut self.socket);
                        if peer.vote(pid, &mut self.net, &mut self.socket) {
                            peer.state = PeerState::VoteResult(self.socket.time() + Duration::from_secs(3));
                        }
                    }
                    _ => {},
                },
                PeerState::VoteSet(_) => match msg {
                    SystemOrGame::Game(Game::SvChat(chat)) => {
                        if chat.client_id == -1 && chat.team == 0 {
                            // TODO: Remove the crash
                            let message = str::from_utf8(chat.message).unwrap();
                            if message.contains("Wait") || message.contains("wait") {
                                processed = true;
                                peer.visited_votes.remove(peer.previous_vote.as_ref().unwrap());
                                peer.state = PeerState::VoteResult(self.socket.time() + Duration::from_secs(5));
                            }
                        }
                    }
                    SystemOrGame::Game(Game::SvVoteSet(vote_set)) => {
                        if vote_set.timeout != 0 {
                            processed = true;
                            peer.state = PeerState::VoteEnd;
                        }
                    },
                    _ => {},
                },
                PeerState::VoteEnd => match msg {
                    SystemOrGame::Game(Game::SvVoteSet(vote_set)) => {
                        // TODO: Currently, we're assuming that we're the only
                        // people on the server.
                        if vote_set.timeout == 0 {
                            processed = true;
                            peer.state = PeerState::VoteResult(self.socket.time() + Duration::from_secs(3));
                        }
                    },
                    SystemOrGame::Game(Game::SvVoteClearOptions(..))
                        | SystemOrGame::Game(Game::SvVoteOptionAdd(..))
                        | SystemOrGame::Game(Game::SvVoteOptionListAdd(..))
                        | SystemOrGame::Game(Game::SvVoteOptionRemove(..))
                    => {
                        processed = true;
                        let prev = peer.previous_vote.as_ref().unwrap();
                        if peer.list_votes.insert(prev.to_owned()) {
                            info!("list vote {:?}", PrettyBytes::new(prev));
                        }
                    },
                    _ => {},
                },
                PeerState::VoteResult(..) => {},
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
                let p = &mut self.peers[pid];
                p.state = PeerState::MapChange;
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
        let mut temp_peer_ids = vec![];
        while self.net.needs_tick() {
            self.net.tick(&mut self.socket).foreach(|e| panic!("{:?}", e));
            self.socket.sleep(Duration::from_millis(50)).unwrap();
            self.socket.update_time_cached();

            temp_peer_ids.clear();
            temp_peer_ids.extend(self.peers.peers.keys().cloned());
            for &pid in &temp_peer_ids {
                if self.tick_peer(pid) {
                    self.net.disconnect(&mut self.socket, pid, b"downloader").unwrap();
                }
            }

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
                        self.net.disconnect(&mut self.socket, pid, b"downloader").unwrap();
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
