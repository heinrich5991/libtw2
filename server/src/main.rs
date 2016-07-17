extern crate arrayvec;
extern crate common;
extern crate event_loop;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
#[macro_use] extern crate matches;
extern crate net;
extern crate num;
extern crate packer;
extern crate snapshot;
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
use gamenet::SnapObj;
use gamenet::enums::MAX_CLIENTS;
use gamenet::enums::Team;
use gamenet::msg::Connless;
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::connless;
use gamenet::msg::game;
use gamenet::msg::system;
use gamenet::msg;
use gamenet::snap_obj::ClientInfo;
use gamenet::snap_obj::GameInfo;
use gamenet::snap_obj::PlayerInfo;
use gamenet::snap_obj::Tick;
use gamenet::snap_obj::obj_size;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use net::net::Chunk;
use net::net::ChunkOrEvent;
use net::net::ConnlessChunk;
use net::net::PeerId;
use net::time::Timestamp;
use num::ToPrimitive;
use packer::Unpacker;
use packer::string_to_ints3;
use packer::string_to_ints4;
use packer::string_to_ints6;
use packer::with_packer;
use snapshot::snap;
use std::collections::HashMap;
use std::collections::hash_map;
use std::fmt::Write;
use std::fmt;
use std::ops;
use std::time::Duration;

const TICKS_PER_SECOND: u32 = 50;

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
    fn sendc<'a, C: Into<Connless<'a>>>(&mut self, addr: Addr, msg: C) {
        fn inner<L: Loop+?Sized>(msg: Connless, addr: Addr, loop_: &mut L) {
            let mut buf: ArrayVec<[u8; 2048]> = ArrayVec::new();
            with_packer(&mut buf, |p| msg.encode(p).unwrap());
            loop_.send_connless(addr, &buf)
        }
        inner(msg.into(), addr, self)
    }
}
impl<L: Loop> LoopExt for L { }

trait SnapBuilderExt {
    fn add<O: Into<SnapObj>>(&mut self, id: u16, obj: O);
}
impl SnapBuilderExt for snap::Builder {
    fn add<O: Into<SnapObj>>(&mut self, id: u16, obj: O) {
        fn inner(builder: &mut snap::Builder, id: u16, obj: SnapObj) {
            builder.add_item(obj.obj_type_id(), id, obj.encode()).unwrap();
        }
        inner(self, id, obj.into())
    }
}

#[derive(Default)]
struct Server {
    peers: Peers,
    game_start: Timestamp,
    game_tick: u32,
    delta_buffer: Vec<u8>,
}

impl Server {
    fn game_tick_time(&self, tick: u32) -> Timestamp {
        let millis = tick.to_u64().unwrap() * 1000 / TICKS_PER_SECOND.to_u64().unwrap();
        self.game_start + Duration::from_millis(millis)
    }
}

#[derive(Default)]
struct Peers {
    peers: HashMap<PeerId, Peer>,
}

impl Peers {
    fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
    fn len(&self) -> usize {
        self.peers.len()
    }
    fn insert(&mut self, pid: PeerId, peer: Peer) {
        self.peers.insert(pid, peer);
    }
    fn entry(&mut self, pid: PeerId) -> hash_map::OccupiedEntry<PeerId, Peer> {
        match self.peers.entry(pid) {
            hash_map::Entry::Occupied(o) => o,
            hash_map::Entry::Vacant(_) => panic!("invalid pid"),
        }
    }
    fn remove(&mut self, pid: PeerId) {
        self.peers.remove(&pid).expect("invalid pid");
    }
    fn iter_mut(&mut self) -> hash_map::IterMut<PeerId, Peer> {
        self.peers.iter_mut()
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

#[derive(Default)]
struct Peer {
    state: PeerState,
}

impl Default for PeerState {
    fn default() -> PeerState {
        PeerState::SystemInfo
    }
}

enum PeerState {
    SystemInfo,
    SystemReady,
    GameInfo,
    SystemEnterGame,
    Ingame(IngameState),
}

impl PeerState {
    fn assert_ingame(&mut self) -> &mut IngameState {
        if let PeerState::Ingame(ref mut ingame) = *self {
            ingame
        } else {
            panic!("not ingame");
        }
    }
}

#[derive(Default)]
struct IngameState {
    snaps: snapshot::Storage,
}

struct ServerLoop<'a, L: Loop+'a> {
    loop_: &'a mut L,
    server: &'a mut Server,
}

impl<L: Loop> Application<L> for Server {
    fn needs_tick(&mut self) -> Timeout {
        if !self.peers.is_empty() {
            Timeout::active(self.game_tick_time(self.game_tick + 1) + Duration::from_millis(1))
        } else {
            Timeout::inactive()
        }
    }
    fn on_tick(&mut self, loop_: &mut L) {
        if !self.peers.is_empty() {
            ServerLoop { server: self, loop_: loop_ }.tick();
        }
    }

    fn on_packet(&mut self, loop_: &mut L, event: ChunkOrEvent<Addr>) {
        ServerLoop { server: self, loop_: loop_ }.process_event(event);
    }
}

impl Server {
    fn run<L: Loop>() {
        L::accept_connections_on_port(8303).run(Server::default());
    }
}
impl<'a, L: Loop> ServerLoop<'a, L> {
    fn process_client_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) {
        use PeerState::*;

        let msg = match SystemOrGame::decode(&mut Warn(pid, data), &mut Unpacker::new(data)) {
            Ok(m) => m,
            Err(err) => {
                warn!("decode error {:?}:", err);
                hexdump(LogLevel::Warn, data);
                return;
            }
        };
        if !vital && !matches!(msg, SystemOrGame::System(System::Input(..))) {
            warn!("non-vital message {:?}", msg);
            return;
        }
        let mut processed = false;
        let mut peer = self.server.peers.entry(pid);
        match (&peer.get().state, msg) {
            (&SystemInfo, SystemOrGame::System(System::Info(info))) => {
                if info.version == VERSION {
                    if info.password == Some(b"foobar") {
                        self.loop_.sends(pid, system::MapChange {
                            name: b"dm1",
                            crc: 0xf2159e6e_u32 as i32,
                            size: 5805,
                        });
                        self.loop_.flush(pid);
                        peer.get_mut().state = SystemReady;
                    } else {
                        self.loop_.disconnect(pid, b"Wrong password");
                        peer.remove();
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
                    peer.remove();
                }
                processed = true;
            }
            (&SystemReady, SystemOrGame::System(System::Ready(system::Ready))) => {
                self.loop_.sendg(pid, game::SvMotd {
                    message: b"Hello World!",
                });
                self.loop_.sends(pid, system::ConReady);
                self.loop_.flush(pid);
                peer.get_mut().state = GameInfo;
                processed = true;
            }
            (&GameInfo, SystemOrGame::Game(Game::ClStartInfo(info))) => {
                info!("{:?}:{} enters the game", pid, AlmostString::new(info.name));
                self.loop_.sendg(pid, game::SvVoteClearOptions);
                self.loop_.sendg(pid, game::SV_TUNE_PARAMS_DEFAULT);
                self.loop_.sendg(pid, game::SvReadyToEnter);
                self.loop_.flush(pid);
                peer.get_mut().state = SystemEnterGame;
                processed = true;
            }
            (&SystemEnterGame, SystemOrGame::System(System::EnterGame(system::EnterGame))) => {
                peer.get_mut().state = Ingame(IngameState::default());
                processed = true;
            }
            (_, SystemOrGame::System(System::RconAuth(..))) => {
                self.loop_.sends(pid, system::RconLine {
                    line: b"Wrong password",
                });
                processed = true;
            }
            (&Ingame(..), SystemOrGame::System(System::Input(input))) => {
                let ingame = peer.get_mut().state.assert_ingame();
                if let Err(e) = ingame.snaps.set_delta_tick(&mut Warn(pid, data), input.ack_snapshot) {
                    warn!("invalid input tick: {:?} ({})", e, input.ack_snapshot);
                }
                processed = true;
            }
            (&Ingame(..), SystemOrGame::Game(Game::ClCallVote(call_vote))) => {
                let error: Option<&[u8]> = match call_vote.type_ {
                    b"kick" => Some(b"Server does not allow voting to kick players"),
                    b"spectate" => Some(b"Server does not allow voting to move players to spectators"),
                    _ => None,
                };
                if let Some(msg) = error {
                    self.loop_.sendg(pid, game::SvChat {
                        team: Team::Red,
                        client_id: -1,
                        message: msg,
                    });
                    processed = true;
                }
            }
            (&Ingame(..), SystemOrGame::Game(Game::ClSetTeam(set_team))) => {
                if set_team.team != Team::Spectators {
                    self.loop_.sendg(pid, game::SvBroadcast {
                        message: b"Teams are locked",
                    });
                    processed = true;
                }
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
                    flags: 1,
                    num_players: 0,
                    max_players: MAX_CLIENTS,
                    num_clients: 0,
                    max_clients: MAX_CLIENTS,
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
                if self.server.peers.is_empty() {
                    self.server.game_start = self.loop_.time();
                    self.server.game_tick = 0;
                }
                if self.server.peers.len() == MAX_CLIENTS.to_usize().unwrap() {
                    self.loop_.disconnect(pid, b"This server is full");
                    return;
                }
                self.loop_.accept(pid);
                self.server.peers.insert(pid, Peer::default());
            },
            ChunkOrEvent::Chunk(Chunk { pid, vital, data }) => {
                self.process_client_packet(pid, vital, data);
            },
            ChunkOrEvent::Connless(ConnlessChunk { addr, data, .. }) => {
                self.process_connless_packet(addr, data);
            },
            ChunkOrEvent::Ready(..) => unreachable!(),
            ChunkOrEvent::Disconnect(pid, reason) => {
                if !reason.is_empty() {
                    info!("{:?} leaves the game ({})", pid, AlmostString::new(reason));
                } else {
                    info!("{:?} leaves the game", pid);
                }
                self.server.peers.remove(pid);
            },
        }
    }
    fn game_tick(&mut self) {
        // TODO: Do tick. :)
    }
    fn send_snapshots(&mut self) {
        for (&pid, peer) in self.server.peers.iter_mut() {
            if let PeerState::Ingame(ref mut ingame) = peer.state {
                let mut builder = ingame.snaps.new_builder();
                builder.add(0, GameInfo {
                    game_flags: 0,
                    game_state_flags: 0,
                    round_start_tick: Tick(0),
                    warmup_timer: 0,
                    score_limit: 20,
                    time_limit: 0,
                    round_num: 1,
                    round_current: 1,
                });
                builder.add(0, ClientInfo {
                    name: string_to_ints4(b"nameless tee"),
                    clan: string_to_ints3(b""),
                    country: -1,
                    skin: string_to_ints6(b"default"),
                    use_custom_color: 0,
                    color_body: 0,
                    color_feet: 0,
                });
                builder.add(0, PlayerInfo {
                    local: 1,
                    client_id: 0,
                    team: Team::Spectators,
                    score: 0,
                    latency: 20,
                });
                let snap = builder.finish();
                let crc = snap.crc();
                let game_tick = self.server.game_tick.to_i32().unwrap();
                let delta_tick = ingame.snaps.delta_tick().unwrap_or(-1);
                let delta = ingame.snaps.add_snap(game_tick, snap);

                self.server.delta_buffer.clear();
                // TODO: Do this better:
                self.server.delta_buffer.reserve(64 * 1024);
                with_packer(&mut self.server.delta_buffer, |p| delta.write(obj_size, p)).unwrap();
                for m in snap::delta_chunks(game_tick, delta_tick, &self.server.delta_buffer, crc) {
                    self.loop_.sends(pid, m);
                    self.loop_.flush(pid);
                }

            }
        }
    }
    fn tick(&mut self) {
        while self.server.game_tick_time(self.server.game_tick + 1) <= self.loop_.time() {
            self.server.game_tick += 1;
            self.game_tick();
            if self.server.game_tick % 2 == 0 {
                self.send_snapshots();
            }
        }
    }
}

fn main() {
    logger::init();
    Server::run::<SocketLoop>();
}
