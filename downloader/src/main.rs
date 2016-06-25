extern crate arrayvec;
extern crate common;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
extern crate net;
extern crate num;
extern crate packer;
extern crate rand;
extern crate snapshot;
extern crate socket;
extern crate tempfile;
extern crate warn;

use arrayvec::ArrayVec;
use common::pretty;
use gamenet::SnapObj;
use gamenet::VERSION;
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
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use net::Net;
use net::net::Callback;
use net::net::Chunk;
use net::net::ChunkAddr;
use net::net::ChunkOrEvent;
use net::net::ChunkType;
use net::net::PeerId;
use num::ToPrimitive;
use packer::IntUnpacker;
use packer::Unpacker;
use packer::with_packer;
use snapshot::Snap;
use snapshot::format::Item as SnapItem;
use socket::Addr;
use socket::Socket;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt;
use std::fs;
use std::io::Write;
use std::io;
use std::ops;
use std::path::PathBuf;
use std::str::FromStr;
use std::str;
use std::time::Duration;
use std::u32;
use tempfile::NamedTempFile;
use tempfile::NamedTempFileOptions;
use warn::Log;

const NETWORK_LOSS_RATE: f32 = 0.0;

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

#[derive(Debug)]
struct WarnSnap<'a>(SnapItem<'a>);

impl<'a, W: fmt::Debug> warn::Warn<W> for WarnSnap<'a> {
    fn warn(&mut self, w: W) {
        warn!("{:?} for {:?}", w, self.0);
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
            sendg(ClCallVote {
                type_: game::CL_CALL_VOTE_TYPE_OPTION,
                value: vote,
                reason: b"downloader",
            }, pid, net, socket);
            visited_votes.insert(vote.to_owned());
        }
        // TODO: This probably has bad performance:
        self.previous_vote = self.current_votes.difference(&self.visited_votes).cloned().next();
        if let Some(ref vote) = self.previous_vote {
            send_vote(&mut self.visited_votes, vote, pid, net, socket);
            info!("voting for {}", pretty::AlmostString::new(vote));
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
                info!("list-voting for {}", pretty::AlmostString::new(vote));
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

fn send<'a, S: Into<System<'a>>>(msg: S, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
    fn inner(msg: System, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
        let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
        with_packer(&mut buf, |p| msg.encode(p).unwrap());
        net.send(socket, Chunk {
            data: &buf,
            addr: ChunkAddr::Peer(pid, ChunkType::Vital),
        }).unwrap();
    }
    inner(msg.into(), pid, net, socket)
}
fn sendg<'a, G: Into<Game<'a>>>(msg: G, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
    fn inner(msg: Game, pid: PeerId, net: &mut Net<Addr>, socket: &mut Socket) {
        let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
        with_packer(&mut buf, |p| msg.encode(p).unwrap());
        net.send(socket, Chunk {
            data: &buf,
            addr: ChunkAddr::Peer(pid, ChunkType::Vital),
        }).unwrap();
    }
    inner(msg.into(), pid, net, socket)
}

fn num_players(snap: &Snap) -> u32 {
    let mut num_players = 0;
    for item in snap.items() {
        match SnapObj::decode_obj(&mut WarnSnap(item), item.type_id, &mut IntUnpacker::new(item.data)) {
            Ok(SnapObj::PlayerInfo(..)) => num_players += 1,
            Ok(_) => {},
            Err(e) => warn!("item decode error {:?}: {:?}", e, item),
        }
    }
    num_players
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
            socket: Socket::with_loss_rate(NETWORK_LOSS_RATE).unwrap(),
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
            self.net.flush(&mut self.socket, pid).unwrap();
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
        match SystemOrGame::decode(&mut Warn(data), &mut Unpacker::new(data)) {
            Ok(m) => msg = m,
            Err(err) => {
                warn!("decode error {:?}:", err);
                hexdump(LogLevel::Warn, data);
                return false;
            }
        }
        debug!("{:?}", msg);
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
                    info!("*** {}", pretty::AlmostString::new(chat.message));
                }
            }
            SystemOrGame::Game(Game::SvBroadcast(broadcast)) => {
                info!("broadcast: {}", pretty::AlmostString::new(broadcast.message));
                ignored = true;
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
                            info!("map change: {}", pretty::AlmostString::new(name));
                            let name = String::from_utf8_lossy(name);
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
                                send(RequestMapData {
                                    chunk: 0,
                                }, pid, &mut self.net, &mut self.socket);
                                peer.state = PeerState::MapData(crc, 0);
                            } else {
                                peer.state = PeerState::ConReady;
                                send(Ready, pid, &mut self.net, &mut self.socket);
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
                            send(Input {
                                ack_snapshot: tick,
                                intended_tick: tick,
                                input: system::INPUT_DATA_EMPTY,
                            }, pid, &mut self.net, &mut self.socket);
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
                                send(Ready, pid, &mut self.net, &mut self.socket);
                                info!("download finished");
                            } else {
                                let cur_chunk = cur_chunk.checked_add(1).unwrap();
                                peer.state = PeerState::MapData(cur_crc, cur_chunk);
                                send(RequestMapData {
                                    chunk: cur_chunk,
                                }, pid, &mut self.net, &mut self.socket);
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
                        sendg(ClStartInfo {
                            name: b"downloader",
                            clan: b"",
                            country: -1,
                            skin: b"default",
                            use_custom_color: false,
                            color_body: 0,
                            color_feet: 0,
                        }, pid, &mut self.net, &mut self.socket);
                        peer.state = PeerState::ReadyToEnter;
                    }
                    _ => {},
                },
                PeerState::ReadyToEnter => match msg {
                    SystemOrGame::Game(Game::SvReadyToEnter(..)) => {
                        progress = true;
                        send(EnterGame, pid, &mut self.net, &mut self.socket);
                        sendg(ClSetTeam {
                            team: enums::TEAM_RED,
                        }, pid, &mut self.net, &mut self.socket);
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
                            info!("list vote {}", pretty::AlmostString::new(prev));
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
        self.net.flush(&mut self.socket, pid).unwrap();
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
                error!("disconnected pid={:?} error={}", pid, pretty::AlmostString::new(reason));
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
            self.socket.sleep(Some(Duration::from_millis(50))).unwrap();

            temp_peer_ids.clear();
            temp_peer_ids.extend(self.peers.peers.keys().cloned());
            for &pid in &temp_peer_ids {
                if self.tick_peer(pid) {
                    self.net.disconnect(&mut self.socket, pid, b"downloader").unwrap();
                }
            }

            while let Some(res) = { buf1.clear(); self.socket.receive(&mut buf1) } {
                let (addr, data) = res.unwrap();
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
    logger::init();
    let args = env::args().dropping(1);
    let addresses = parse_connections(args).expect("invalid addresses");
    Main::init(&addresses).run();
}
