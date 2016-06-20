use debug::DebugSlice;
use enums::*;
use error::Error;
use packer::ExcessData;
use packer::IntUnpacker;
use packer::in_range;
use packer::positive;
use std::fmt;
use warn::Warn;

#[derive(Clone, Copy, Debug)]
pub struct Tick(i32);

pub const PLAYERFLAG_PLAYING: i32 = 1 << 0;
pub const PLAYERFLAG_IN_MENU: i32 = 1 << 1;
pub const PLAYERFLAG_CHATTING: i32 = 1 << 2;
pub const PLAYERFLAG_SCOREBOARD: i32 = 1 << 3;

pub const GAMEFLAG_TEAMS: i32 = 1 << 0;
pub const GAMEFLAG_FLAGS: i32 = 1 << 1;

pub const GAMESTATEFLAG_GAMEOVER: i32 = 1 << 0;
pub const GAMESTATEFLAG_SUDDENDEATH: i32 = 1 << 1;
pub const GAMESTATEFLAG_PAUSED: i32 = 1 << 2;

pub const PLAYER_INPUT: u16 = 1;
pub const PROJECTILE: u16 = 2;
pub const LASER: u16 = 3;
pub const PICKUP: u16 = 4;
pub const FLAG: u16 = 5;
pub const GAME_INFO: u16 = 6;
pub const GAME_DATA: u16 = 7;
pub const CHARACTER_CORE: u16 = 8;
pub const CHARACTER: u16 = 9;
pub const PLAYER_INFO: u16 = 10;
pub const CLIENT_INFO: u16 = 11;
pub const SPECTATOR_INFO: u16 = 12;
pub const COMMON: u16 = 13;
pub const EXPLOSION: u16 = 14;
pub const SPAWN: u16 = 15;
pub const HAMMER_HIT: u16 = 16;
pub const DEATH: u16 = 17;
pub const SOUND_GLOBAL: u16 = 18;
pub const SOUND_WORLD: u16 = 19;
pub const DAMAGE_IND: u16 = 20;

#[derive(Clone, Copy)]
pub struct PlayerInput {
    pub direction: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub jump: i32,
    pub fire: i32,
    pub hook: i32,
    pub player_flags: i32,
    pub wanted_weapon: i32,
    pub next_weapon: i32,
    pub prev_weapon: i32,
}

#[derive(Clone, Copy)]
pub struct Projectile {
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub type_: Weapon,
    pub start_tick: Tick,
}

#[derive(Clone, Copy)]
pub struct Laser {
    pub x: i32,
    pub y: i32,
    pub from_x: i32,
    pub from_y: i32,
    pub start_tick: Tick,
}

#[derive(Clone, Copy)]
pub struct Pickup {
    pub x: i32,
    pub y: i32,
    pub type_: i32,
    pub subtype: i32,
}

#[derive(Clone, Copy)]
pub struct Flag {
    pub x: i32,
    pub y: i32,
    pub team: i32,
}

#[derive(Clone, Copy)]
pub struct GameInfo {
    pub game_flags: i32,
    pub game_state_flags: i32,
    pub round_start_tick: Tick,
    pub warmup_timer: i32,
    pub score_limit: i32,
    pub time_limit: i32,
    pub round_num: i32,
    pub round_current: i32,
}

#[derive(Clone, Copy)]
pub struct GameData {
    pub teamscore_red: i32,
    pub teamscore_blue: i32,
    pub flag_carrier_red: i32,
    pub flag_carrier_blue: i32,
}

#[derive(Clone, Copy)]
pub struct CharacterCore {
    pub tick: i32,
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub angle: i32,
    pub direction: i32,
    pub jumped: i32,
    pub hooked_player: i32,
    pub hook_state: i32,
    pub hook_tick: Tick,
    pub hook_x: i32,
    pub hook_y: i32,
    pub hook_dx: i32,
    pub hook_dy: i32,
}

#[derive(Clone, Copy)]
pub struct Character {
    pub character_core: CharacterCore,
    pub player_flags: i32,
    pub health: i32,
    pub armor: i32,
    pub ammo_count: i32,
    pub weapon: Weapon,
    pub emote: i32,
    pub attack_tick: i32,
}

#[derive(Clone, Copy)]
pub struct PlayerInfo {
    pub local: i32,
    pub client_id: i32,
    pub team: i32,
    pub score: i32,
    pub latency: i32,
}

#[derive(Clone, Copy)]
pub struct ClientInfo {
    pub name: [i32; 4],
    pub clan0: i32,
    pub clan1: i32,
    pub clan2: i32,
    pub country: i32,
    pub skin: [i32; 6],
    pub use_custom_color: i32,
    pub color_body: i32,
    pub color_feet: i32,
}

#[derive(Clone, Copy)]
pub struct SpectatorInfo {
    pub spectator_id: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub struct Common {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub struct Explosion {
    pub common: Common,
}

#[derive(Clone, Copy)]
pub struct Spawn {
    pub common: Common,
}

#[derive(Clone, Copy)]
pub struct HammerHit {
    pub common: Common,
}

#[derive(Clone, Copy)]
pub struct Death {
    pub common: Common,
    pub client_id: i32,
}

#[derive(Clone, Copy)]
pub struct SoundGlobal {
    pub common: Common,
    pub sound_id: Sound,
}

#[derive(Clone, Copy)]
pub struct SoundWorld {
    pub common: Common,
    pub sound_id: Sound,
}

#[derive(Clone, Copy)]
pub struct DamageInd {
    pub common: Common,
    pub angle: i32,
}

impl fmt::Debug for PlayerInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PlayerInput")
            .field("direction", &self.direction)
            .field("target_x", &self.target_x)
            .field("target_y", &self.target_y)
            .field("jump", &self.jump)
            .field("fire", &self.fire)
            .field("hook", &self.hook)
            .field("player_flags", &self.player_flags)
            .field("wanted_weapon", &self.wanted_weapon)
            .field("next_weapon", &self.next_weapon)
            .field("prev_weapon", &self.prev_weapon)
            .finish()
    }
}
impl PlayerInput {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<PlayerInput, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInput, Error> {
        Ok(PlayerInput {
            direction: try!(_p.read_int()),
            target_x: try!(_p.read_int()),
            target_y: try!(_p.read_int()),
            jump: try!(_p.read_int()),
            fire: try!(_p.read_int()),
            hook: try!(_p.read_int()),
            player_flags: try!(in_range(try!(_p.read_int()), 0, 256)),
            wanted_weapon: try!(_p.read_int()),
            next_weapon: try!(_p.read_int()),
            prev_weapon: try!(_p.read_int()),
        })
    }
}

impl fmt::Debug for Projectile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Projectile")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("vel_x", &self.vel_x)
            .field("vel_y", &self.vel_y)
            .field("type_", &self.type_)
            .field("start_tick", &self.start_tick)
            .finish()
    }
}
impl Projectile {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Projectile, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Projectile, Error> {
        Ok(Projectile {
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
            vel_x: try!(_p.read_int()),
            vel_y: try!(_p.read_int()),
            type_: try!(Weapon::from_i32(try!(_p.read_int()))),
            start_tick: Tick(try!(_p.read_int())),
        })
    }
}

impl fmt::Debug for Laser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Laser")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("from_x", &self.from_x)
            .field("from_y", &self.from_y)
            .field("start_tick", &self.start_tick)
            .finish()
    }
}
impl Laser {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Laser, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Laser, Error> {
        Ok(Laser {
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
            from_x: try!(_p.read_int()),
            from_y: try!(_p.read_int()),
            start_tick: Tick(try!(_p.read_int())),
        })
    }
}

impl fmt::Debug for Pickup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Pickup")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("type_", &self.type_)
            .field("subtype", &self.subtype)
            .finish()
    }
}
impl Pickup {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Pickup, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Pickup, Error> {
        Ok(Pickup {
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
            type_: try!(positive(try!(_p.read_int()))),
            subtype: try!(positive(try!(_p.read_int()))),
        })
    }
}

impl fmt::Debug for Flag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Flag")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("team", &self.team)
            .finish()
    }
}
impl Flag {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Flag, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Flag, Error> {
        Ok(Flag {
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
            team: try!(in_range(try!(_p.read_int()), TEAM_RED, TEAM_BLUE)),
        })
    }
}

impl fmt::Debug for GameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameInfo")
            .field("game_flags", &self.game_flags)
            .field("game_state_flags", &self.game_state_flags)
            .field("round_start_tick", &self.round_start_tick)
            .field("warmup_timer", &self.warmup_timer)
            .field("score_limit", &self.score_limit)
            .field("time_limit", &self.time_limit)
            .field("round_num", &self.round_num)
            .field("round_current", &self.round_current)
            .finish()
    }
}
impl GameInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameInfo, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameInfo, Error> {
        Ok(GameInfo {
            game_flags: try!(in_range(try!(_p.read_int()), 0, 256)),
            game_state_flags: try!(in_range(try!(_p.read_int()), 0, 256)),
            round_start_tick: Tick(try!(_p.read_int())),
            warmup_timer: try!(positive(try!(_p.read_int()))),
            score_limit: try!(positive(try!(_p.read_int()))),
            time_limit: try!(positive(try!(_p.read_int()))),
            round_num: try!(positive(try!(_p.read_int()))),
            round_current: try!(positive(try!(_p.read_int()))),
        })
    }
}

impl fmt::Debug for GameData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameData")
            .field("teamscore_red", &self.teamscore_red)
            .field("teamscore_blue", &self.teamscore_blue)
            .field("flag_carrier_red", &self.flag_carrier_red)
            .field("flag_carrier_blue", &self.flag_carrier_blue)
            .finish()
    }
}
impl GameData {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameData, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameData, Error> {
        Ok(GameData {
            teamscore_red: try!(_p.read_int()),
            teamscore_blue: try!(_p.read_int()),
            flag_carrier_red: try!(in_range(try!(_p.read_int()), FLAG_MISSING, MAX_CLIENTS-1)),
            flag_carrier_blue: try!(in_range(try!(_p.read_int()), FLAG_MISSING, MAX_CLIENTS-1)),
        })
    }
}

impl fmt::Debug for CharacterCore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CharacterCore")
            .field("tick", &self.tick)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("vel_x", &self.vel_x)
            .field("vel_y", &self.vel_y)
            .field("angle", &self.angle)
            .field("direction", &self.direction)
            .field("jumped", &self.jumped)
            .field("hooked_player", &self.hooked_player)
            .field("hook_state", &self.hook_state)
            .field("hook_tick", &self.hook_tick)
            .field("hook_x", &self.hook_x)
            .field("hook_y", &self.hook_y)
            .field("hook_dx", &self.hook_dx)
            .field("hook_dy", &self.hook_dy)
            .finish()
    }
}
impl CharacterCore {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<CharacterCore, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<CharacterCore, Error> {
        Ok(CharacterCore {
            tick: try!(_p.read_int()),
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
            vel_x: try!(_p.read_int()),
            vel_y: try!(_p.read_int()),
            angle: try!(_p.read_int()),
            direction: try!(in_range(try!(_p.read_int()), -1, 1)),
            jumped: try!(in_range(try!(_p.read_int()), 0, 3)),
            hooked_player: try!(in_range(try!(_p.read_int()), 0, MAX_CLIENTS-1)),
            hook_state: try!(in_range(try!(_p.read_int()), -1, 5)),
            hook_tick: Tick(try!(_p.read_int())),
            hook_x: try!(_p.read_int()),
            hook_y: try!(_p.read_int()),
            hook_dx: try!(_p.read_int()),
            hook_dy: try!(_p.read_int()),
        })
    }
}

impl fmt::Debug for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Character")
            .field("character_core", &self.character_core)
            .field("player_flags", &self.player_flags)
            .field("health", &self.health)
            .field("armor", &self.armor)
            .field("ammo_count", &self.ammo_count)
            .field("weapon", &self.weapon)
            .field("emote", &self.emote)
            .field("attack_tick", &self.attack_tick)
            .finish()
    }
}
impl Character {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Character, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Character, Error> {
        Ok(Character {
            character_core: try!(CharacterCore::decode_inner(_p)),
            player_flags: try!(in_range(try!(_p.read_int()), 0, 256)),
            health: try!(in_range(try!(_p.read_int()), 0, 10)),
            armor: try!(in_range(try!(_p.read_int()), 0, 10)),
            ammo_count: try!(in_range(try!(_p.read_int()), 0, 10)),
            weapon: try!(Weapon::from_i32(try!(_p.read_int()))),
            emote: try!(in_range(try!(_p.read_int()), 0, 6)),
            attack_tick: try!(positive(try!(_p.read_int()))),
        })
    }
}

impl fmt::Debug for PlayerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PlayerInfo")
            .field("local", &self.local)
            .field("client_id", &self.client_id)
            .field("team", &self.team)
            .field("score", &self.score)
            .field("latency", &self.latency)
            .finish()
    }
}
impl PlayerInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<PlayerInfo, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInfo, Error> {
        Ok(PlayerInfo {
            local: try!(in_range(try!(_p.read_int()), 0, 1)),
            client_id: try!(in_range(try!(_p.read_int()), 0, MAX_CLIENTS-1)),
            team: try!(in_range(try!(_p.read_int()), TEAM_SPECTATORS, TEAM_BLUE)),
            score: try!(_p.read_int()),
            latency: try!(_p.read_int()),
        })
    }
}

impl fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClientInfo")
            .field("name", &DebugSlice::new(&self.name, |e| e))
            .field("clan0", &self.clan0)
            .field("clan1", &self.clan1)
            .field("clan2", &self.clan2)
            .field("country", &self.country)
            .field("skin", &DebugSlice::new(&self.skin, |e| e))
            .field("use_custom_color", &self.use_custom_color)
            .field("color_body", &self.color_body)
            .field("color_feet", &self.color_feet)
            .finish()
    }
}
impl ClientInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<ClientInfo, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<ClientInfo, Error> {
        Ok(ClientInfo {
            name: [
                try!(_p.read_int()),
                try!(_p.read_int()),
                try!(_p.read_int()),
                try!(_p.read_int()),
            ],
            clan0: try!(_p.read_int()),
            clan1: try!(_p.read_int()),
            clan2: try!(_p.read_int()),
            country: try!(_p.read_int()),
            skin: [
                try!(_p.read_int()),
                try!(_p.read_int()),
                try!(_p.read_int()),
                try!(_p.read_int()),
                try!(_p.read_int()),
                try!(_p.read_int()),
            ],
            use_custom_color: try!(in_range(try!(_p.read_int()), 0, 1)),
            color_body: try!(_p.read_int()),
            color_feet: try!(_p.read_int()),
        })
    }
}

impl fmt::Debug for SpectatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpectatorInfo")
            .field("spectator_id", &self.spectator_id)
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
impl SpectatorInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<SpectatorInfo, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SpectatorInfo, Error> {
        Ok(SpectatorInfo {
            spectator_id: try!(in_range(try!(_p.read_int()), SPEC_FREEVIEW, MAX_CLIENTS-1)),
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
        })
    }
}

impl fmt::Debug for Common {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Common")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
impl Common {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Common, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Common, Error> {
        Ok(Common {
            x: try!(_p.read_int()),
            y: try!(_p.read_int()),
        })
    }
}

impl fmt::Debug for Explosion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Explosion")
            .field("common", &self.common)
            .finish()
    }
}
impl Explosion {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Explosion, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Explosion, Error> {
        Ok(Explosion {
            common: try!(Common::decode_inner(_p)),
        })
    }
}

impl fmt::Debug for Spawn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Spawn")
            .field("common", &self.common)
            .finish()
    }
}
impl Spawn {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Spawn, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Spawn, Error> {
        Ok(Spawn {
            common: try!(Common::decode_inner(_p)),
        })
    }
}

impl fmt::Debug for HammerHit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HammerHit")
            .field("common", &self.common)
            .finish()
    }
}
impl HammerHit {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<HammerHit, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<HammerHit, Error> {
        Ok(HammerHit {
            common: try!(Common::decode_inner(_p)),
        })
    }
}

impl fmt::Debug for Death {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Death")
            .field("common", &self.common)
            .field("client_id", &self.client_id)
            .finish()
    }
}
impl Death {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Death, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Death, Error> {
        Ok(Death {
            common: try!(Common::decode_inner(_p)),
            client_id: try!(in_range(try!(_p.read_int()), 0, MAX_CLIENTS-1)),
        })
    }
}

impl fmt::Debug for SoundGlobal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SoundGlobal")
            .field("common", &self.common)
            .field("sound_id", &self.sound_id)
            .finish()
    }
}
impl SoundGlobal {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<SoundGlobal, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SoundGlobal, Error> {
        Ok(SoundGlobal {
            common: try!(Common::decode_inner(_p)),
            sound_id: try!(Sound::from_i32(try!(_p.read_int()))),
        })
    }
}

impl fmt::Debug for SoundWorld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SoundWorld")
            .field("common", &self.common)
            .field("sound_id", &self.sound_id)
            .finish()
    }
}
impl SoundWorld {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<SoundWorld, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SoundWorld, Error> {
        Ok(SoundWorld {
            common: try!(Common::decode_inner(_p)),
            sound_id: try!(Sound::from_i32(try!(_p.read_int()))),
        })
    }
}

impl fmt::Debug for DamageInd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DamageInd")
            .field("common", &self.common)
            .field("angle", &self.angle)
            .finish()
    }
}
impl DamageInd {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DamageInd, Error> {
        let result = try!(Self::decode_inner(p));
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DamageInd, Error> {
        Ok(DamageInd {
            common: try!(Common::decode_inner(_p)),
            angle: try!(_p.read_int()),
        })
    }
}

pub fn obj_size(type_: u16) -> Option<u32> {
    Some(match type_ {
        PLAYER_INPUT => 10,
        PROJECTILE => 6,
        LASER => 5,
        PICKUP => 4,
        FLAG => 3,
        GAME_INFO => 8,
        GAME_DATA => 4,
        CHARACTER_CORE => 15,
        CHARACTER => 22,
        PLAYER_INFO => 5,
        CLIENT_INFO => 17,
        SPECTATOR_INFO => 3,
        COMMON => 2,
        EXPLOSION => 2,
        SPAWN => 2,
        HAMMER_HIT => 2,
        DEATH => 3,
        SOUND_GLOBAL => 3,
        SOUND_WORLD => 3,
        DAMAGE_IND => 3,
        _ => return None,
    })
}
