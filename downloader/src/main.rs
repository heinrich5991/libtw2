extern crate arrayvec;
#[macro_use] extern crate clap;
extern crate common;
extern crate event_loop;
extern crate gamenet_teeworlds_0_6 as gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
extern crate packer;
extern crate rand;
extern crate snapshot;
extern crate tempfile;
extern crate warn;

use arrayvec::ArrayVec;
use clap::App;
use clap::Arg;
use clap::Error;
use clap::ErrorKind;
use common::num::Cast;
use common::pretty;
use event_loop::Addr;
use event_loop::Application;
use event_loop::Chunk;
use event_loop::ConnlessChunk;
use event_loop::Loop;
use event_loop::PeerId;
use event_loop::SocketLoop;
use event_loop::Timeout;
use event_loop::Timestamp;
use event_loop::collections::PeerMap;
use gamenet::SnapObj;
use gamenet::enums::Team;
use gamenet::enums::VERSION;
use gamenet::enums;
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::game::ClCallVote;
use gamenet::msg::game::ClSetTeam;
use gamenet::msg::game::ClStartInfo;
use gamenet::msg::game::SvVoteOptionAdd;
use gamenet::msg::game::SvVoteOptionRemove;
use gamenet::msg::system::EnterGame;
use gamenet::msg::system::Info;
use gamenet::msg::system::Input;
use gamenet::msg::system::MapChange;
use gamenet::msg::system::MapData;
use gamenet::msg::system::Ready;
use gamenet::msg::system::RequestMapData;
use gamenet::msg;
use gamenet::snap_obj::PlayerInput;
use gamenet::snap_obj::obj_size;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use packer::IntUnpacker;
use packer::Unpacker;
use packer::with_packer;
use snapshot::Snap;
use snapshot::format::Item as SnapItem;
use std::borrow::Cow;
use std::cmp;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::io::Write;
use std::io;
use std::mem;
use std::path::PathBuf;
use std::str;
use std::time::Duration;
use std::u32;
use tempfile::NamedTempFile;
use warn::Log;

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

fn check_dummy_map(name: &[u8], crc: u32, size: i32) -> bool {
    if name != b"dummy" {
        return false;
    }
    match (crc, size) {
        (0xbeae0b9f, 549) => {},
        (0x6c760ac4, 306) => {},
        _ => warn!("unknown dummy map, crc={}, size={}", crc, size),
    }
    true
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
    progress_timeout: Timestamp,
}

fn need_file(crc: i32, name: &str) -> bool {
    let mut path = PathBuf::new();
    path.push("maps");
    path.push(format!("{}_{:08x}.map", name, crc));
    !path.exists()
}

impl Peer {
    fn new<L: Loop>(loop_: &mut L) -> Peer {
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
            progress_timeout: Timestamp::sentinel(),
        };
        result.progress(loop_);
        result
    }
    fn needs_tick(&self) -> Timeout {
        cmp::min(Timeout::active(self.progress_timeout), self.state.needs_tick())
    }
    fn tick<L: Loop>(&mut self, pid: PeerId, config: &Config, loop_: &mut L) {
        // TODO: What happens with peers that are already disconnected?
        let vote;
        match self.state {
            PeerState::VoteSet(timeout) => vote = loop_.time() >= timeout,
            PeerState::VoteResult(timeout) => vote = loop_.time() >= timeout,
            _ => vote = false,
        }
        if vote {
            if self.vote(pid, config, loop_) {
                info!("voting done");
                loop_.disconnect(pid, config.nick.as_bytes());
                return;
            }
            loop_.flush(pid);
        }
        if self.has_timed_out(loop_) {
            error!("timed out due to lack of progress");
            loop_.disconnect(pid, config.timeout.as_bytes());
            return;
        }
    }
    fn vote<L: Loop>(&mut self, pid: PeerId, config: &Config, loop_: &mut L) -> bool {
        fn send_vote<L: Loop>(visited_votes: &mut HashSet<Vec<u8>>, vote: &[u8], pid: PeerId, reason: &[u8], loop_: &mut L) {
            loop_.sendg(pid, ClCallVote {
                type_: enums::CL_CALL_VOTE_TYPE_OPTION.as_bytes(),
                value: vote,
                reason: reason,
            });
            visited_votes.insert(vote.to_owned());
        }
        // TODO: This probably has bad performance:
        self.previous_vote = self.current_votes.difference(&self.visited_votes).cloned().next();
        if let Some(ref vote) = self.previous_vote {
            send_vote(&mut self.visited_votes, vote, pid, config.nick.as_bytes(), loop_);
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
                send_vote(&mut self.visited_votes, &vote, pid, config.nick.as_bytes(), loop_);
            } else {
                return true;
            }
        }
        self.state = PeerState::VoteSet(loop_.time() + Duration::from_secs(5));
        self.progress(loop_);
        false
    }
    fn has_timed_out<L: Loop>(&self, loop_: &mut L) -> bool {
        loop_.time() >= self.progress_timeout
    }
    fn progress<L: Loop>(&mut self, loop_: &mut L) {
        self.progress_timeout = loop_.time() + Duration::from_secs(120);
    }
    fn open_file(&mut self, crc: i32, name: String) -> Result<(), io::Error> {
        self.download = Some(Download {
            file: tempfile::Builder::new()
                .prefix(&format!("{}_{:08x}_", name, crc))
                .suffix(".map")
                .tempfile_in("downloading")?,
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
    VoteSet(Timestamp),
    VoteEnd,
    // VoteResult(timeout)
    VoteResult(Timestamp),
}

impl Default for PeerState {
    fn default() -> PeerState {
        PeerState::Connection
    }
}

impl PeerState {
    fn needs_tick(&self) -> Timeout {
        match *self {
            PeerState::VoteSet(to) | PeerState::VoteResult(to) => Timeout::active(to),
            _ => Timeout::inactive(),
        }
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
}
impl<L: Loop> LoopExt for L { }

fn num_players(snap: &Snap) -> u32 {
    let mut num_players = 0;
    for item in snap.items() {
        match SnapObj::decode_obj(&mut WarnSnap(item), item.type_id.into(), &mut IntUnpacker::new(item.data)) {
            Ok(SnapObj::PlayerInfo(..)) => num_players += 1,
            Ok(_) => {},
            Err(e) => warn!("item decode error {:?}: {:?}", e, item),
        }
    }
    num_players
}

struct Config {
    nick: String,
    clan: String,
    timeout: String,
    error: String,
}

struct Main {
    peers: PeerMap<Peer>,
    config: Config,
}

struct MainLoop<'a, L: Loop+'a> {
    peers: &'a mut PeerMap<Peer>,
    config: &'a Config,
    loop_: &'a mut L,
}

impl<'a, L: Loop> Application<L> for Main {
    fn needs_tick(&mut self) -> Timeout {
        self.peers.values().map(|p| p.needs_tick()).min().unwrap_or_default()
    }
    fn on_tick(&mut self, loop_: &mut L) {
        for (pid, peer) in self.peers.iter_mut() {
            peer.tick(pid, &self.config, loop_);
        }
    }
    fn on_packet(&mut self, loop_: &mut L, chunk: Chunk) {
        self.loop_(loop_).on_packet(chunk.pid, chunk.vital, chunk.data);
    }
    fn on_connless_packet(&mut self, _: &mut L, chunk: ConnlessChunk) {
        warn!("connless packet {} {:?}", chunk.addr, pretty::Bytes::new(chunk.data));
    }
    fn on_connect(&mut self, _: &mut L, _: PeerId) {
        unreachable!();
    }
    fn on_ready(&mut self, loop_: &mut L, pid: PeerId) {
        self.loop_(loop_).on_ready(pid);
    }
    fn on_disconnect(&mut self, _: &mut L, pid: PeerId, remote: bool, reason: &[u8]) {
        if remote {
            error!("disconnected pid={:?} error={}", pid, pretty::AlmostString::new(reason));
        }
        self.peers.remove(pid);
    }
}

impl Main {
    fn run<L: Loop>(addresses: &[Addr], config: Config) {
        let mut main = Main {
            peers: PeerMap::with_capacity(addresses.len()),
            config: config,
        };
        let mut loop_ = L::client();
        for &addr in addresses {
            let pid = loop_.connect(addr);
            main.peers.insert(pid, Peer::new(&mut loop_));
        }
        fs::create_dir_all("maps").unwrap();
        fs::create_dir_all("downloading").unwrap();
        loop_.run(main);
    }
    fn loop_<'a, L: Loop+'a>(&'a mut self, loop_: &'a mut L) -> MainLoop<'a, L> {
        MainLoop {
            peers: &mut self.peers,
            config: &self.config,
            loop_: loop_,
        }
    }
}
impl<'a, L: Loop> MainLoop<'a, L> {
    fn on_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) {
        let _ = vital;
        let msg;
        match msg::decode(&mut Warn(data), &mut Unpacker::new(data)) {
            Ok(m) => msg = m,
            Err(err) => {
                warn!("decode error {:?}:", err);
                hexdump(LogLevel::Warn, data);
                return;
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
                if !chat.team && chat.client_id == -1 {
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
                        if let Some(_) = size.try_usize() {
                            if name.iter().any(|&b| b == b'/' || b == b'\\') {
                                error!("invalid map name");
                                self.loop_.disconnect(pid, self.config.error.as_bytes());
                                return;
                            }
                            match peer.state {
                                PeerState::MapChange => {},
                                PeerState::VoteResult(..) => {},
                                PeerState::ReadyToEnter if peer.dummy_map => {},
                                _ => warn!("map change from state {:?}", peer.state),
                            }
                            peer.dummy_map = check_dummy_map(name, crc as u32, size);
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
                                self.loop_.sends(pid, RequestMapData { chunk: 0, });
                                peer.state = PeerState::MapData(crc, 0);
                            } else {
                                peer.state = PeerState::ConReady;
                                self.loop_.sends(pid, Ready);
                            }
                            progress = true;
                        } else {
                            error!("invalid map size");
                            self.loop_.disconnect(pid, self.config.error.as_bytes());
                            return;
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
                                        self.loop_.disconnect(pid, self.config.nick.as_bytes());
                                        return;
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
                            if peer.dummy_map && peer.num_snaps_since_reset == 3 {
                                // DDNet needs the INPUT message as the first
                                // chunk of the packet.
                                self.loop_.force_flush(pid);
                            }
                            let tick = peer.snaps.ack_tick().unwrap_or(-1);
                            self.loop_.sends(pid, Input {
                                ack_snapshot: tick,
                                intended_tick: tick,
                                input_size: mem::size_of::<PlayerInput>().assert_i32(),
                                input: PlayerInput::default(),
                            });
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
                        let len = l.num_options.assert_usize();
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
                                self.loop_.sends(pid, Ready);
                                info!("download finished");
                            } else {
                                let cur_chunk = cur_chunk.checked_add(1).unwrap();
                                peer.state = PeerState::MapData(cur_crc, cur_chunk);
                                self.loop_.sends(pid, RequestMapData { chunk: cur_chunk });
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
                        self.loop_.sendg(pid, ClStartInfo {
                            name: self.config.nick.as_bytes(),
                            clan: self.config.clan.as_bytes(),
                            country: -1,
                            skin: b"default",
                            use_custom_color: false,
                            color_body: 0,
                            color_feet: 0,
                        });
                        peer.state = PeerState::ReadyToEnter;
                    }
                    _ => {},
                },
                PeerState::ReadyToEnter => match msg {
                    SystemOrGame::Game(Game::SvReadyToEnter(..)) => {
                        progress = true;
                        self.loop_.sends(pid, EnterGame);
                        self.loop_.sendg(pid, ClSetTeam { team: Team::Red });
                        if peer.vote(pid, &self.config, self.loop_) {
                            peer.state = PeerState::VoteResult(self.loop_.time() + Duration::from_secs(3));
                        }
                    }
                    _ => {},
                },
                PeerState::VoteSet(_) => match msg {
                    SystemOrGame::Game(Game::SvChat(chat)) => {
                        if !chat.team && chat.client_id == -1 {
                            if let Ok(message) = str::from_utf8(chat.message) {
                                if message.contains("Wait") || message.contains("wait") {
                                    progress = true;
                                    peer.visited_votes.remove(peer.previous_vote.as_ref().unwrap());
                                    peer.state = PeerState::VoteResult(self.loop_.time() + Duration::from_secs(5));
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
                            peer.state = PeerState::VoteResult(self.loop_.time() + Duration::from_secs(3));
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
                peer.progress(self.loop_);
            }
        }
        if !progress && !ignored {
            warn!("unprocessed message {:?}", msg);
        }
        self.loop_.flush(pid);
    }
    fn on_ready(&mut self, pid: PeerId) {
        self.peers[pid].state = PeerState::MapChange;
        self.loop_.sends(pid, Info {
            version: VERSION.as_bytes(),
            password: Some(b""),
        });
        self.loop_.flush(pid);
    }
}

fn main() {
    logger::init();

    let matches = App::new("Teeworlds server map scraper")
        .about("Tries to download every map from an otherwise empty Teeworlds server.")
        .arg(Arg::with_name("nick")
            .help("Sets the nickname sent to servers")
            .long("nick")
            .takes_value(true)
            .value_name("NICK")
            .default_value("downloader")
        )
        .arg(Arg::with_name("clan")
            .help("Sets the clan name sent to servers")
            .long("clan")
            .takes_value(true)
            .value_name("CLAN")
            .default_value("")
        )
        .arg(Arg::with_name("server")
            .help("Server to scrape")
            .multiple(true)
            .required(true)
            .value_name("SERVER")
        )
        .get_matches();

    let addresses = values_t!(matches, "server", Addr).unwrap_or_else(|e| e.exit());
    let nick = matches.value_of("nick").unwrap();
    let clan = matches.value_of("clan").unwrap();

    if nick.len() >= 15 {
        Error::with_description("Nick can have at most 15 bytes", ErrorKind::ValueValidation).exit();
    }
    if clan.len() >= 11 {
        Error::with_description("Clan can have at most 11 bytes", ErrorKind::ValueValidation).exit();
    }

    let config = Config {
        nick: nick.to_owned(),
        clan: clan.to_owned(),
        timeout: format!("{} (timeout)", nick),
        error: format!("{} (error", nick),
    };

    Main::run::<SocketLoop>(&addresses, config);
}
