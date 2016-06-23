extern crate arrayvec;
extern crate buffer;
extern crate common;
extern crate env_logger;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate mio;
extern crate net;
extern crate num;
extern crate packer;
extern crate rand;
extern crate snapshot;
extern crate tempfile;
extern crate warn;

use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use common::pretty;
use gamenet::enums;
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::game::ClCallVote;
use gamenet::msg::game::ClSetTeam;
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
use gamenet::msg::system;
use gamenet::snap_obj::obj_size;
use gamenet::snap_obj;
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
use packer::Unpacker;
use packer::with_packer;
use snapshot::Snap;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::io;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::ops;
use std::path::PathBuf;
use std::str::FromStr;
use std::str;
use std::time::Duration;
use std::time::Instant;
use std::u32;
use tempfile::NamedTempFile;
use tempfile::NamedTempFileOptions;
use warn::Log;

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

struct Download {
    file: NamedTempFile,
    crc: i32,
    name: String,
}

struct Peer {
    visited_votes: HashSet<Vec<u8>>,
    current_votes: HashSet<Vec<u8>>,
    list_votes: HashSet<Vec<u8>>,
    completed_list_votes: HashSet<Vec<u8>>,
    previous_list_vote: Option<Vec<u8>>,
    previous_vote: Option<Vec<u8>>,
    snaps: snapshot::Manager,
    num_snaps_since_reset: u64,
    dummy_map: bool,
    state: PeerState,
    download: Option<Download>,
    progress_timeout: Duration,
}

fn need_file(crc: i32, name: &str) -> bool {
    let mut path = PathBuf::new();
    path.push("maps");
    path.push(format!("{}_{:08x}.map", name, crc));
    !path.exists()
}

impl Peer {
    fn new(socket: &mut Socket) -> Peer {
        let mut result = Peer {
            visited_votes: HashSet::new(),
            current_votes: HashSet::new(),
            list_votes: HashSet::new(),
            completed_list_votes: HashSet::new(),
            previous_list_vote: None,
            previous_vote: None,
            snaps: snapshot::Manager::new(),
            num_snaps_since_reset: 0,
            dummy_map: false,
            state: PeerState::Connection,
            download: None,
            progress_timeout: Duration::new(0, 0),
        };
        result.progress(socket);
        result
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
        // TODO: This probably has bad performance:
        self.previous_vote = self.current_votes.difference(&self.visited_votes).cloned().next();
        if let Some(ref vote) = self.previous_vote {
            send_vote(&mut self.visited_votes, vote, pid, net, socket);
            info!("voting for {:?}", pretty::Bytes::new(vote));
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
                info!("list-voting for {:?}", pretty::Bytes::new(vote));
                send_vote(&mut self.visited_votes, &vote, pid, net, socket);
            } else {
                return true;
            }
        }
        self.state = PeerState::VoteSet(socket.time() + Duration::from_secs(5));
        self.progress(socket);
        false
    }
    fn has_timed_out(&self, socket: &mut Socket) -> bool {
        socket.time() >= self.progress_timeout
    }
    fn progress(&mut self, socket: &mut Socket) {
        self.progress_timeout = socket.time() + Duration::from_secs(120);
    }
    fn open_file(&mut self, crc: i32, name: String) -> Result<(), io::Error> {
        self.download = Some(Download {
            file: try!(NamedTempFileOptions::new()
                .prefix(&format!("{}_{:08x}_", name, crc))
                .suffix(".map")
                .create_in("downloading")
            ),
            crc: crc,
            name: name,
        });
        Ok(())
    }
    fn write_file(&mut self, data: &[u8]) -> Result<(), io::Error> {
        self.download.as_mut().unwrap().file.write_all(data)
    }
    fn finish_file(&mut self) -> Result<(), io::Error> {
        let download = self.download.take().unwrap();
        let mut path = PathBuf::new();
        path.push("maps");
        path.push(format!("{}_{:08x}.map", &download.name, download.crc));
        download.file.persist(&path).map(|_| ()).map_err(|e| e.error)
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
fn num_players(snap: &Snap) -> u32 {
    snap.items().filter(|i| i.type_id == snap_obj::PLAYER_INFO).count().to_u32().unwrap()
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
            main.peers.insert(pid, Peer::new(&mut main.socket));
        }
        fs::create_dir_all("maps").unwrap();
        fs::create_dir_all("downloading").unwrap();
        main
    }
    fn tick_peer(&mut self, pid: PeerId) -> bool {
        let peer = &mut self.peers[pid];
        let vote;
        match peer.state {
            PeerState::VoteSet(timeout) => vote = self.socket.time() >= timeout,
            PeerState::VoteResult(timeout) => vote = self.socket.time() >= timeout,
            _ => vote = false,
        }
        if vote {
            if peer.vote(pid, &mut self.net, &mut self.socket) {
                info!("voting done");
                return true;
            }
        }
        if peer.has_timed_out(&mut self.socket) {
            error!("timed out due to lack of progress");
            return true;
        }
        false
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
        let mut ignored = false;
        let mut progress = false;
        match msg {
            SystemOrGame::Game(Game::SvMotd(..))
                | SystemOrGame::Game(Game::SvKillMsg(..))
                | SystemOrGame::Game(Game::SvTuneParams(..))
                | SystemOrGame::Game(Game::SvWeaponPickup(..))
                | SystemOrGame::System(System::InputTiming(..))
                | SystemOrGame::Game(Game::SvExtraProjectile(..))
            => {
                ignored = true;
            },
            SystemOrGame::Game(Game::SvChat(chat)) => {
                if chat.team == 0 && chat.client_id == -1 {
                    ignored = true;
                    info!("*** {:?}", pretty::Bytes::new(chat.message));
                }
            }
            SystemOrGame::Game(Game::SvBroadcast(broadcast)) => {
                info!("broadcast: {:?}", pretty::Bytes::new(broadcast.message));
            }
            _ => {},
        }
        {
            let peer = &mut self.peers[pid];
            match msg {
                SystemOrGame::System(ref msg) => match *msg {
                    System::MapChange(MapChange { crc, size, name }) => {
                        if let Some(_) = size.to_usize() {
                            if name.iter().any(|&b| b == b'/' || b == b'\\') {
                                error!("invalid map name");
                                return true;
                            }
                            match peer.state {
                                PeerState::MapChange => {},
                                PeerState::VoteResult(..) => {},
                                PeerState::ReadyToEnter if peer.dummy_map => {},
                                _ => warn!("map change from state {:?}", peer.state),
                            }
                            peer.dummy_map =
                                crc as u32 == 0xbeae0b9f
                                && size == 549
                                && name == b"dummy";
                            peer.current_votes.clear();
                            peer.num_snaps_since_reset = 0;
                            peer.snaps.reset();
                            let name = String::from_utf8_lossy(name);
                            info!("map change: {:?}", name);
                            if let Cow::Owned(..) = name {
                                warn!("weird characters in map name");
                            }
                            let mut start_download = false;
                            if need_file(crc, &name) {
                                if let Err(e) = peer.open_file(crc, name.into_owned()) {
                                    error!("error opening file {:?}", e);
                                } else {
                                    start_download = true;
                                }
                            }
                            if start_download {
                                info!("download starting");
                                request_chunk = Some(0);
                                peer.state = PeerState::MapData(crc, 0);
                            } else {
                                peer.state = PeerState::ConReady;
                                let m = System::Ready(Ready);
                                send(m, pid, &mut self.net, &mut self.socket);
                            }
                            progress = true;
                        } else {
                            error!("invalid map size");
                            return true;
                        }
                    },
                    System::Snap(_) | System::SnapEmpty(_) | System::SnapSingle(_)
                    => {
                        let mut check_num_snaps = true;
                        peer.num_snaps_since_reset += 1;
                        {
                            let res = match *msg {
                                System::Snap(s) => peer.snaps.snap(&mut Log, obj_size, s),
                                System::SnapEmpty(s) => peer.snaps.snap_empty(&mut Log, obj_size, s),
                                System::SnapSingle(s) => peer.snaps.snap_single(&mut Log, obj_size, s),
                                _ => unreachable!(),
                            };
                            match res {
                                Ok(Some(snap)) => {
                                    let num_players = num_players(snap);
                                    if num_players > 1 {
                                        error!("more than one player ({}) detected, quitting", num_players);
                                        return true;
                                    }
                                },
                                Ok(None) => {
                                    peer.num_snaps_since_reset -= 1;
                                    check_num_snaps = false;
                                },
                                Err(err) => warn!("snapshot error {:?}", err),
                            }
                        }
                        if check_num_snaps && peer.num_snaps_since_reset % 25 == 3 {
                            let tick = peer.snaps.ack_tick().unwrap_or(-1);
                            send(System::Input(Input {
                                ack_snapshot: tick,
                                intended_tick: tick,
                                input: system::INPUT_DATA_EMPTY,
                            }), pid, &mut self.net, &mut self.socket);
                        }
                        ignored = true;
                    },
                    _ => {},
                },
                SystemOrGame::Game(ref msg) => match *msg {
                    Game::SvVoteClearOptions(..) => {
                        ignored = true;
                        peer.current_votes.clear();
                    },
                    Game::SvVoteOptionListAdd(l) => {
                        ignored = true;
                        // `len` is bounded by the unpacking.
                        let len = l.num_options.to_usize().unwrap();
                        for &desc in l.description.iter().take(len) {
                            peer.current_votes.insert(desc.to_owned());
                        }
                    },
                    Game::SvVoteOptionAdd(SvVoteOptionAdd { description }) => {
                        ignored = true;
                        peer.current_votes.insert(description.to_owned());
                    },
                    Game::SvVoteOptionRemove(SvVoteOptionRemove { description }) => {
                        ignored = true;
                        if !peer.current_votes.remove(description) {
                            warn!("vote option removed even though it didn't exist");
                        }
                    }
                    _ => {},
                }
            }
            match peer.state {
                PeerState::Connection => unreachable!(),
                PeerState::MapChange => {}, // Handled above.
                PeerState::MapData(cur_crc, cur_chunk) => match msg {
                    SystemOrGame::System(System::MapData(MapData { last, crc, chunk, data })) => {
                        if cur_crc == crc && cur_chunk == chunk {
                            let res = peer.write_file(data);
                            if let Err(ref err) = res {
                                error!("error writing file {:?}", err);
                            }
                            if last != 0 || res.is_err() {
                                if !res.is_err() {
                                    if let Err(err) = peer.finish_file() {
                                        error!("error finishing file {:?}", err);
                                    }
                                    if last != 1 {
                                        warn!("weird map data packet");
                                    }
                                }
                                peer.state = PeerState::ConReady;
                                let m = System::Ready(Ready);
                                send(m, pid, &mut self.net, &mut self.socket);
                                info!("download finished");
                            } else {
                                let cur_chunk = cur_chunk.checked_add(1).unwrap();
                                peer.state = PeerState::MapData(cur_crc, cur_chunk);
                                request_chunk = Some(cur_chunk);
                            }
                        } else {
                            if cur_crc != crc || cur_chunk < chunk {
                                warn!("unsolicited map data crc={:08x} chunk={}", crc, chunk);
                                warn!("want crc={:08x} chunk={}", cur_crc, cur_chunk);
                            }
                        }
                        progress = true;
                    }
                    _ => {},
                },
                PeerState::ConReady => match msg {
                    SystemOrGame::System(System::ConReady(..)) => {
                        progress = true;
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
                        progress = true;
                        send(System::EnterGame(EnterGame), pid, &mut self.net, &mut self.socket);
                        sendg(Game::ClSetTeam(ClSetTeam {
                            team: enums::TEAM_RED,
                        }), pid, &mut self.net, &mut self.socket);
                        if peer.vote(pid, &mut self.net, &mut self.socket) {
                            peer.state = PeerState::VoteResult(self.socket.time() + Duration::from_secs(3));
                        }
                    }
                    _ => {},
                },
                PeerState::VoteSet(_) => match msg {
                    SystemOrGame::Game(Game::SvChat(chat)) => {
                        if chat.client_id == -1 && chat.team == 0 {
                            if let Ok(message) = str::from_utf8(chat.message) {
                                if message.contains("Wait") || message.contains("wait") {
                                    progress = true;
                                    peer.visited_votes.remove(peer.previous_vote.as_ref().unwrap());
                                    peer.state = PeerState::VoteResult(self.socket.time() + Duration::from_secs(5));
                                }
                            }
                        }
                    }
                    SystemOrGame::Game(Game::SvVoteSet(vote_set)) => {
                        if vote_set.timeout != 0 {
                            progress = true;
                            peer.state = PeerState::VoteEnd;
                        }
                    },
                    _ => {},
                },
                PeerState::VoteEnd => match msg {
                    SystemOrGame::Game(Game::SvVoteSet(vote_set)) => {
                        if vote_set.timeout == 0 {
                            progress = true;
                            peer.state = PeerState::VoteResult(self.socket.time() + Duration::from_secs(3));
                        }
                    },
                    SystemOrGame::Game(Game::SvVoteClearOptions(..))
                        | SystemOrGame::Game(Game::SvVoteOptionAdd(..))
                        | SystemOrGame::Game(Game::SvVoteOptionListAdd(..))
                        | SystemOrGame::Game(Game::SvVoteOptionRemove(..))
                    => {
                        ignored = true;
                        let prev = peer.previous_vote.as_ref().unwrap();
                        if peer.list_votes.insert(prev.to_owned()) {
                            info!("list vote {:?}", pretty::Bytes::new(prev));
                        }
                    },
                    _ => {},
                },
                PeerState::VoteResult(..) => {},
            }
            if progress {
                peer.progress(&mut self.socket);
            }
        }
        if !progress && !ignored {
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
