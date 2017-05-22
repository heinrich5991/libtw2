extern crate arrayvec;
#[macro_use] extern crate common;
extern crate datafile;
extern crate event_loop;
extern crate gamenet;
extern crate hexdump;
extern crate itertools;
#[macro_use] extern crate log;
extern crate logger;
extern crate map;
#[macro_use] extern crate matches;
extern crate ndarray;
extern crate packer;
extern crate snapshot;
extern crate socket;
extern crate warn;
extern crate world;

use arrayvec::ArrayString;
use arrayvec::ArrayVec;
use common::Takeable;
use common::num::Cast;
use common::num::CastFloat;
use common::pretty::AlmostString;
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
use event_loop::collections::PeerSet;
use gamenet::SnapObj;
use gamenet::VERSION;
use gamenet::enums::Emote;
use gamenet::enums::MAX_CLIENTS;
use gamenet::enums::Team;
use gamenet::enums::Weapon;
use gamenet::msg::Connless;
use gamenet::msg::Game;
use gamenet::msg::System;
use gamenet::msg::SystemOrGame;
use gamenet::msg::connless;
use gamenet::msg::game::SV_TUNE_PARAMS_DEFAULT;
use gamenet::msg::game;
use gamenet::msg::system;
use gamenet::msg;
use gamenet::snap_obj::Character;
use gamenet::snap_obj::ClientInfo;
use gamenet::snap_obj::GameInfo;
use gamenet::snap_obj::PlayerInfo;
use gamenet::snap_obj::Tick;
use gamenet::snap_obj::obj_size;
use gamenet::snap_obj;
use hexdump::hexdump_iter;
use itertools::Itertools;
use log::LogLevel;
use ndarray::Array2;
use packer::Unpacker;
use packer::string_to_ints3;
use packer::string_to_ints4;
use packer::string_to_ints6;
use packer::with_packer;
use snapshot::snap;
use std::cell::Cell;
use std::fmt::Write;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use world::vec2;

const TICKS_PER_SECOND: u32 = 50;
const PLAYER_NAME_LENGTH: usize = 16-1; // -1 for null termination
const MAPDOWNLOAD_CHUNK_SIZE: u64 = 1024-128;

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

struct MapContents {
    // TODO: Implement an actual memory map. Is that possible in a safe way?
    contents: Vec<u8>,
}

impl Default for MapContents {
    fn default() -> MapContents {
        let mut file = File::open("dm1.map").unwrap();
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();
        MapContents {
            contents: contents,
        }
    }
}

impl MapContents {
    fn serve_request(&self, rmd: system::RequestMapData) -> Option<system::MapData> {
        let chunk = unwrap_or_return!(rmd.chunk.try_u64());
        let offset = chunk * MAPDOWNLOAD_CHUNK_SIZE;
        if offset >= self.contents.len().u64() {
            return None;
        }
        let last = offset + MAPDOWNLOAD_CHUNK_SIZE >= self.contents.len().u64();
        let offset = offset.assert_usize();
        let data;
        if !last {
            data = &self.contents[offset..offset+MAPDOWNLOAD_CHUNK_SIZE.assert_usize()];
        } else {
            data = &self.contents[offset..];
        }
        Some(system::MapData {
            last: last as i32,
            crc: 0xf2159e6e_u32 as i32,
            chunk: rmd.chunk,
            data: data,
        })
    }
}

struct Map {
    spawn: vec2,
    collision: Array2<Option<world::CollisionType>>,
    data: MapContents,
}

impl Default for Map {
    fn default() -> Map {
        let map_contents = MapContents::default();
        let reader = datafile::Reader::open("dm1.map").unwrap();
        let mut map = map::Reader::from_datafile(reader);
        map.check_version().unwrap();
        let gamelayers = map.game_layers().unwrap();
        let tiles = map.layer_tiles(gamelayers.game).unwrap();
        let tiles = Array2::from_shape_vec((gamelayers.height.usize(), gamelayers.width.usize()), tiles).unwrap();
        let result = Map {
            spawn: vec2::new(160.0, 160.0),
            collision: tiles.mapv(|t| match t.index {
                1 => Some(world::CollisionType::Normal),
                3 => Some(world::CollisionType::Unhookable),
                _ => None,
            }),
            data: map_contents,
        };
        for y in 0..result.collision.dim().0 {
            for x in 0..result.collision.dim().1 {
                let c = match result.collision[(y, x)] {
                    Some(world::CollisionType::Normal) => '#',
                    Some(world::CollisionType::Unhookable) => '!',
                    None => ' ',
                };
                print!("{}", c);
            }
            println!("");
        }
        result
    }
}

impl world::Collision for Map {
    fn check_point(&mut self, pos: vec2) -> Option<world::CollisionType> {
        let (x, y) = (pos.x.round_to_i32(), pos.y.round_to_i32());
        let (mut tx, mut ty) = ((x as f32 / 32.0).trunc_to_i32(), (y as f32 / 32.0).trunc_to_i32());
        if tx < 0 {
            tx = 0;
        }
        if tx > self.collision.dim().1.assert_i32() {
            tx = self.collision.dim().1.assert_i32() - 1;
        }
        if ty < 0 {
            ty = 0;
        }
        if ty > self.collision.dim().0.assert_i32() {
            ty = self.collision.dim().0.assert_i32() - 1;
        }
        self.collision[(ty.assert_usize(), tx.assert_usize())]
    }
}

#[derive(Default)]
struct Server {
    peers: PeerMap<Peer>,
    players: Vec<Player>,
    game_start: Timestamp,
    game_tick: u32,
    delta_buffer: Vec<u8>,
    map: Map,

    send_snapshots_peer_set: Takeable<PeerSet>,
}

impl Server {
    fn game_tick_time(&self, tick: u32) -> Timestamp {
        let millis = tick.u64() * 1000 / TICKS_PER_SECOND.u64();
        self.game_start + Duration::from_millis(millis)
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
    SystemEnterGame(SystemEnterGameState),
    Ingame(IngameState),
}

impl PeerState {
    fn assert_system_enter_game(&mut self) -> &mut SystemEnterGameState {
        if let PeerState::SystemEnterGame(ref mut system_enter_game) = *self {
            system_enter_game
        } else {
            panic!("not in state system enter game");
        }
    }
    fn assert_ingame(&mut self) -> &mut IngameState {
        if let PeerState::Ingame(ref mut ingame) = *self {
            ingame
        } else {
            panic!("not ingame");
        }
    }
}

#[derive(Clone)]
struct SystemEnterGameState {
    name: ArrayVec<[u8; PLAYER_NAME_LENGTH]>,
}

impl SystemEnterGameState {
    fn new(name: &[u8]) -> SystemEnterGameState {
        // TODO: Warn for overlong name.
        SystemEnterGameState {
            name: name.iter().cloned().collect(),
        }
    }
}

struct IngameState {
    name: ArrayVec<[u8; PLAYER_NAME_LENGTH]>,
    snaps: snapshot::Storage,
    spectator: bool,
    input: snap_obj::PlayerInput,
}

impl From<SystemEnterGameState> for IngameState {
    fn from(system_enter_game: SystemEnterGameState) -> IngameState {
        IngameState {
            name: system_enter_game.name,
            snaps: Default::default(),
            spectator: true,
            input: Default::default(),
        }
    }
}

struct Player {
    character: Cell<world::Character>,
    pid: PeerId,
}

impl Player {
    fn new(pid: PeerId, spawn: vec2) -> Player {
        Player {
            character: Cell::new(world::Character::spawn(spawn)),
            pid: pid,
        }
    }
}

struct ServerLoop<'a, L: Loop+'a> {
    loop_: &'a mut L,
    server: &'a mut Server,
}

impl<L: Loop> Application<L> for Server {
    fn needs_tick(&mut self) -> Timeout {
        if !self.peers.is_empty() {
            Timeout::active(self.game_tick_time(self.game_tick + 1))
        } else {
            Timeout::inactive()
        }
    }
    fn on_tick(&mut self, loop_: &mut L) {
        if !self.peers.is_empty() {
            self.loop_(loop_).tick();
        }
    }
    fn on_packet(&mut self, loop_: &mut L, chunk: Chunk) {
        self.loop_(loop_).on_packet(chunk.pid, chunk.vital, chunk.data);
    }
    fn on_connless_packet(&mut self, loop_: &mut L, chunk: ConnlessChunk) {
        self.loop_(loop_).on_connless_packet(chunk.addr, chunk.data);
    }
    fn on_connect(&mut self, loop_: &mut L, pid: PeerId) {
        self.loop_(loop_).on_connect(pid);
    }
    fn on_ready(&mut self, _: &mut L, _: PeerId) {
        unreachable!();
    }
    fn on_disconnect(&mut self, loop_: &mut L, pid: PeerId, remote: bool, reason: &[u8]) {
        self.loop_(loop_).on_disconnect(pid, remote, reason);
    }
}

impl Server {
    fn run<L: Loop>() {
        L::accept_connections_on_port(8303).run(Server::default());
    }
    fn loop_<'a, L: Loop+'a>(&'a mut self, loop_: &'a mut L) -> ServerLoop<'a, L> {
        ServerLoop { server: self, loop_: loop_ }
    }
}
impl<'a, L: Loop> ServerLoop<'a, L> {
    fn on_packet(&mut self, pid: PeerId, vital: bool, data: &[u8]) {
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
        let peer = &mut self.server.peers[pid];
        match (&peer.state, msg) {
            (&SystemInfo, SystemOrGame::System(System::Info(info))) => {
                if info.version == VERSION {
                    if info.password == Some(b"foobar") {
                        self.loop_.sends(pid, system::MapChange {
                            name: b"dm1",
                            crc: 0xf2159e6e_u32 as i32,
                            size: 5805,
                        });
                        self.loop_.flush(pid);
                        peer.state = SystemReady;
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
            (&SystemReady, SystemOrGame::System(System::RequestMapData(rmd))) => {
                if let Some(md) = self.server.map.data.serve_request(rmd) {
                    self.loop_.sends(pid, md);
                }
            }
            (&SystemReady, SystemOrGame::System(System::Ready(system::Ready))) => {
                self.loop_.sendg(pid, game::SvMotd {
                    message: b"Hello World!",
                });
                self.loop_.sends(pid, system::ConReady);
                self.loop_.flush(pid);
                peer.state = GameInfo;
                processed = true;
            }
            (&GameInfo, SystemOrGame::Game(Game::ClStartInfo(info))) => {
                info!("{}:{} enters the game", pid, AlmostString::new(info.name));
                self.loop_.sendg(pid, game::SvVoteClearOptions);
                self.loop_.sendg(pid, SV_TUNE_PARAMS_DEFAULT);
                self.loop_.sendg(pid, game::SvReadyToEnter);
                self.loop_.flush(pid);
                peer.state = SystemEnterGame(SystemEnterGameState::new(info.name));
                processed = true;
            }
            (&SystemEnterGame(..), SystemOrGame::System(System::EnterGame(system::EnterGame))) => {
                let system_enter_game = peer.state.assert_system_enter_game().clone();
                peer.state = Ingame(system_enter_game.into());
                processed = true;
            }
            (_, SystemOrGame::System(System::RconAuth(..))) => {
                self.loop_.sends(pid, system::RconLine {
                    line: b"Wrong password",
                });
                processed = true;
            }
            (&Ingame(..), SystemOrGame::System(System::Input(input))) => {
                let ingame = peer.state.assert_ingame();
                if let Err(e) = ingame.snaps.set_delta_tick(&mut Warn(pid, data), input.ack_snapshot) {
                    warn!("invalid input tick: {:?} ({})", e, input.ack_snapshot);
                }
                // TODO: Teeworlds never ignores old inputs?
                ingame.input = input.input;
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
                        team: false,
                        client_id: -1,
                        message: msg,
                    });
                    processed = true;
                }
            }
            (&Ingame(..), SystemOrGame::Game(Game::ClSetTeam(set_team))) => {
                let ingame = peer.state.assert_ingame();
                // TODO: Spam filter
                let join_spectators = set_team.team == Team::Spectators;
                if ingame.spectator == join_spectators {
                    return;
                }
                ingame.spectator = join_spectators;

                let mut msg: ArrayString<[u8; 64]> = ArrayString::new();
                if ingame.spectator {
                    let idx = self.server.players.iter().position(|p| p.pid == pid).unwrap();
                    self.server.players.swap_remove(idx);
                    // Fix usage of AlmostString, sometimes it quotes.
                    write!(&mut msg, "'{}' joined the spectators", AlmostString::new(&ingame.name)).unwrap();
                } else {
                    self.server.players.push(Player::new(pid, self.server.map.spawn));
                    write!(&mut msg, "'{}' joined the game", AlmostString::new(&ingame.name)).unwrap();
                }
                self.loop_.sendg(pid, game::SvChat {
                    team: false,
                    client_id: -1,
                    message: msg.as_bytes(),
                });
            }
            _ => {},
        }
        if !processed {
            warn!("unprocessed message {:?}", msg);
        }
    }
    fn on_connless_packet(&mut self, addr: Addr, data: &[u8]) {
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
                // TODO: Send clients. :)
                self.loop_.sendc(addr, connless::Info {
                    token: request.token.i32(),
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
    fn on_connect(&mut self, pid: PeerId) {
        if self.server.peers.is_empty() {
            self.server.game_start = self.loop_.time();
            self.server.game_tick = 0;
        }
        if self.server.peers.len() == MAX_CLIENTS.assert_usize() {
            self.loop_.reject(pid, b"This server is full");
            return;
        }
        self.loop_.accept(pid);
        self.server.peers.insert(pid, Peer::default());
        info!("{} starting to connect", pid);
    }
    fn on_disconnect(&mut self, pid: PeerId, remote: bool, reason: &[u8]) {
        let _ = remote;
        if !reason.is_empty() {
            info!("{} leaves the game ({})", pid, AlmostString::new(reason));
        } else {
            info!("{} leaves the game", pid);
        }
        self.server.peers.remove(pid);
        for i in 0..self.server.players.len() {
            if self.server.players[i].pid == pid {
                self.server.players.swap_remove(i);
            }
        }
    }
    fn game_tick(&mut self) {
        use world::Character;
        use world::CharacterId;
        struct OtherCharacters<'a> {
            own_cid: CharacterId,
            players: &'a [Player],
        }
        impl<'a> world::OtherCharacters for OtherCharacters<'a> {
            type Iter = CharacterId;
            fn is_self(&self, cid: CharacterId) -> bool {
                cid == self.own_cid
            }
            fn get(&self, cid: CharacterId) -> Character {
                assert!(!self.is_self(cid));
                self.players[cid.0.usize()].character.get()
            }
            fn modify<F: FnOnce(&mut Character)>(&self, cid: CharacterId, f: F) {
                let mut character = self.get(cid);
                f(&mut character);
                self.players[cid.0.usize()].character.set(character);
            }
            fn iter(&self) -> CharacterId {
                CharacterId(0)
            }
            fn next(&self, iter: &mut CharacterId) -> Option<(CharacterId, Character)> {
                if *iter == self.own_cid {
                    *iter = CharacterId(iter.0 + 1);
                }
                let cur = *iter;
                if cur.0.usize() == self.players.len() {
                    return None;
                }
                *iter = CharacterId(iter.0 + 1);
                Some((cur, self.get(cur)))
            }
        }

        for (cid, p) in self.server.players.iter().enumerate() {
            let input = self.server.peers[p.pid].state.assert_ingame().input;
            let other = &mut OtherCharacters {
                own_cid: CharacterId(cid.assert_u32()),
                players: &self.server.players,
            };
            let mut character = p.character.get();
            character.tick(&mut self.server.map, other, input, &SV_TUNE_PARAMS_DEFAULT);
            character.move_(&mut self.server.map, other, &SV_TUNE_PARAMS_DEFAULT);
            character.quantize();
            p.character.set(character);
        }
    }
    fn send_snapshots(&mut self) {
        let mut peer_set = self.server.send_snapshots_peer_set.take();
        peer_set.clear();
        peer_set.extend(self.server.peers.keys());
        for snap_pid in &peer_set {
            let mut builder;
            let delta_tick;
            if let PeerState::Ingame(ref mut ingame) = self.server.peers[snap_pid].state {
                builder = ingame.snaps.new_builder();
                delta_tick = ingame.snaps.delta_tick().unwrap_or(-1);
            } else {
                continue;
            }
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
            for (pid, peer) in self.server.peers.iter() {
                if let PeerState::Ingame(ref ingame) = peer.state {
                    // TODO: Fix ID!
                    builder.add(pid.0.assert_u16(), ClientInfo {
                        name: string_to_ints4(&ingame.name),
                        clan: string_to_ints3(b""),
                        country: -1,
                        skin: string_to_ints6(b"default"),
                        use_custom_color: 0,
                        color_body: 0,
                        color_feet: 0,
                    });
                    builder.add(pid.0.assert_u16(), PlayerInfo {
                        local: (snap_pid == pid) as i32,
                        client_id: pid.0.assert_i32(),
                        team: if ingame.spectator { Team::Spectators } else { Team::Red },
                        score: 0,
                        latency: 20,
                    });
                }
            }
            for player in &self.server.players {
                builder.add(player.pid.0.assert_u16(), Character {
                    character_core: player.character.get().to_net(),
                    player_flags: snap_obj::PLAYERFLAG_PLAYING,
                    health: 10,
                    armor: 0,
                    ammo_count: 10,
                    weapon: Weapon::Pistol,
                    emote: Emote::Normal,
                    attack_tick: 0,
                });
            }
            let snap = builder.finish();
            let crc = snap.crc();
            let game_tick = self.server.game_tick.assert_i32();
            let delta = self.server.peers[snap_pid].state.assert_ingame().snaps.add_snap(game_tick, snap);

            self.server.delta_buffer.clear();
            // TODO: Do this better:
            self.server.delta_buffer.reserve(64 * 1024);
            with_packer(&mut self.server.delta_buffer, |p| delta.write(obj_size, p)).unwrap();
            for m in snap::delta_chunks(game_tick, delta_tick, &self.server.delta_buffer, crc) {
                self.loop_.sends(snap_pid, m);
                self.loop_.flush(snap_pid);
            }
        }
        self.server.send_snapshots_peer_set.restore(peer_set);
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
