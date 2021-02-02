use buffer::CapacityError;
use common::slice;
use enums;
use error::Error;
use packer::ExcessData;
use packer::IntUnpacker;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::at_least;
use packer::in_range;
use packer::positive;
use packer::to_bool;
use std::fmt;
use warn::Warn;

pub use gamenet_common::snap_obj::Tick;
pub use gamenet_common::snap_obj::TypeId;

pub const PLAYERFLAG_ADMIN: i32 = 1 << 0;
pub const PLAYERFLAG_CHATTING: i32 = 1 << 1;
pub const PLAYERFLAG_SCOREBOARD: i32 = 1 << 2;
pub const PLAYERFLAG_READY: i32 = 1 << 3;
pub const PLAYERFLAG_DEAD: i32 = 1 << 4;
pub const PLAYERFLAG_WATCHING: i32 = 1 << 5;
pub const PLAYERFLAG_BOT: i32 = 1 << 6;

pub const GAMEFLAG_TEAMS: i32 = 1 << 0;
pub const GAMEFLAG_FLAGS: i32 = 1 << 1;
pub const GAMEFLAG_SURVIVAL: i32 = 1 << 2;
pub const GAMEFLAG_RACE: i32 = 1 << 3;

pub const GAMESTATEFLAG_WARMUP: i32 = 1 << 0;
pub const GAMESTATEFLAG_SUDDENDEATH: i32 = 1 << 1;
pub const GAMESTATEFLAG_ROUNDOVER: i32 = 1 << 2;
pub const GAMESTATEFLAG_GAMEOVER: i32 = 1 << 3;
pub const GAMESTATEFLAG_PAUSED: i32 = 1 << 4;
pub const GAMESTATEFLAG_STARTCOUNTDOWN: i32 = 1 << 5;

pub const COREEVENTFLAG_GROUND_JUMP: i32 = 1 << 0;
pub const COREEVENTFLAG_AIR_JUMP: i32 = 1 << 1;
pub const COREEVENTFLAG_HOOK_ATTACH_PLAYER: i32 = 1 << 2;
pub const COREEVENTFLAG_HOOK_ATTACH_GROUND: i32 = 1 << 3;
pub const COREEVENTFLAG_HOOK_HIT_NOHOOK: i32 = 1 << 4;

pub const RACEFLAG_HIDE_KILLMSG: i32 = 1 << 0;
pub const RACEFLAG_FINISHMSG_AS_CHAT: i32 = 1 << 1;
pub const RACEFLAG_KEEP_WANTED_WEAPON: i32 = 1 << 2;

pub const PLAYER_INPUT: u16 = 1;
pub const PROJECTILE: u16 = 2;
pub const LASER: u16 = 3;
pub const PICKUP: u16 = 4;
pub const FLAG: u16 = 5;
pub const GAME_DATA: u16 = 6;
pub const GAME_DATA_TEAM: u16 = 7;
pub const GAME_DATA_FLAG: u16 = 8;
pub const CHARACTER_CORE: u16 = 9;
pub const CHARACTER: u16 = 10;
pub const PLAYER_INFO: u16 = 11;
pub const SPECTATOR_INFO: u16 = 12;
pub const DE_CLIENT_INFO: u16 = 13;
pub const DE_GAME_INFO: u16 = 14;
pub const DE_TUNE_PARAMS: u16 = 15;
pub const COMMON: u16 = 16;
pub const EXPLOSION: u16 = 17;
pub const SPAWN: u16 = 18;
pub const HAMMER_HIT: u16 = 19;
pub const DEATH: u16 = 20;
pub const SOUND_WORLD: u16 = 21;
pub const DAMAGE: u16 = 22;
pub const PLAYER_INFO_RACE: u16 = 23;
pub const GAME_DATA_RACE: u16 = 24;

#[derive(Clone, Copy)]
pub enum SnapObj {
    PlayerInput(PlayerInput),
    Projectile(Projectile),
    Laser(Laser),
    Pickup(Pickup),
    Flag(Flag),
    GameData(GameData),
    GameDataTeam(GameDataTeam),
    GameDataFlag(GameDataFlag),
    CharacterCore(CharacterCore),
    Character(Character),
    PlayerInfo(PlayerInfo),
    SpectatorInfo(SpectatorInfo),
    DeClientInfo(DeClientInfo),
    DeGameInfo(DeGameInfo),
    DeTuneParams(DeTuneParams),
    Common(Common),
    Explosion(Explosion),
    Spawn(Spawn),
    HammerHit(HammerHit),
    Death(Death),
    SoundWorld(SoundWorld),
    Damage(Damage),
    PlayerInfoRace(PlayerInfoRace),
    GameDataRace(GameDataRace),
}

impl SnapObj {
    pub fn decode_obj<W: Warn<ExcessData>>(warn: &mut W, obj_type_id: TypeId, _p: &mut IntUnpacker) -> Result<SnapObj, Error> {
        use self::TypeId::*;
        Ok(match obj_type_id {
            Ordinal(PLAYER_INPUT) => SnapObj::PlayerInput(PlayerInput::decode(warn, _p)?),
            Ordinal(PROJECTILE) => SnapObj::Projectile(Projectile::decode(warn, _p)?),
            Ordinal(LASER) => SnapObj::Laser(Laser::decode(warn, _p)?),
            Ordinal(PICKUP) => SnapObj::Pickup(Pickup::decode(warn, _p)?),
            Ordinal(FLAG) => SnapObj::Flag(Flag::decode(warn, _p)?),
            Ordinal(GAME_DATA) => SnapObj::GameData(GameData::decode(warn, _p)?),
            Ordinal(GAME_DATA_TEAM) => SnapObj::GameDataTeam(GameDataTeam::decode(warn, _p)?),
            Ordinal(GAME_DATA_FLAG) => SnapObj::GameDataFlag(GameDataFlag::decode(warn, _p)?),
            Ordinal(CHARACTER_CORE) => SnapObj::CharacterCore(CharacterCore::decode(warn, _p)?),
            Ordinal(CHARACTER) => SnapObj::Character(Character::decode(warn, _p)?),
            Ordinal(PLAYER_INFO) => SnapObj::PlayerInfo(PlayerInfo::decode(warn, _p)?),
            Ordinal(SPECTATOR_INFO) => SnapObj::SpectatorInfo(SpectatorInfo::decode(warn, _p)?),
            Ordinal(DE_CLIENT_INFO) => SnapObj::DeClientInfo(DeClientInfo::decode(warn, _p)?),
            Ordinal(DE_GAME_INFO) => SnapObj::DeGameInfo(DeGameInfo::decode(warn, _p)?),
            Ordinal(DE_TUNE_PARAMS) => SnapObj::DeTuneParams(DeTuneParams::decode(warn, _p)?),
            Ordinal(COMMON) => SnapObj::Common(Common::decode(warn, _p)?),
            Ordinal(EXPLOSION) => SnapObj::Explosion(Explosion::decode(warn, _p)?),
            Ordinal(SPAWN) => SnapObj::Spawn(Spawn::decode(warn, _p)?),
            Ordinal(HAMMER_HIT) => SnapObj::HammerHit(HammerHit::decode(warn, _p)?),
            Ordinal(DEATH) => SnapObj::Death(Death::decode(warn, _p)?),
            Ordinal(SOUND_WORLD) => SnapObj::SoundWorld(SoundWorld::decode(warn, _p)?),
            Ordinal(DAMAGE) => SnapObj::Damage(Damage::decode(warn, _p)?),
            Ordinal(PLAYER_INFO_RACE) => SnapObj::PlayerInfoRace(PlayerInfoRace::decode(warn, _p)?),
            Ordinal(GAME_DATA_RACE) => SnapObj::GameDataRace(GameDataRace::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn obj_type_id(&self) -> TypeId {
        match *self {
            SnapObj::PlayerInput(_) => TypeId::from(PLAYER_INPUT),
            SnapObj::Projectile(_) => TypeId::from(PROJECTILE),
            SnapObj::Laser(_) => TypeId::from(LASER),
            SnapObj::Pickup(_) => TypeId::from(PICKUP),
            SnapObj::Flag(_) => TypeId::from(FLAG),
            SnapObj::GameData(_) => TypeId::from(GAME_DATA),
            SnapObj::GameDataTeam(_) => TypeId::from(GAME_DATA_TEAM),
            SnapObj::GameDataFlag(_) => TypeId::from(GAME_DATA_FLAG),
            SnapObj::CharacterCore(_) => TypeId::from(CHARACTER_CORE),
            SnapObj::Character(_) => TypeId::from(CHARACTER),
            SnapObj::PlayerInfo(_) => TypeId::from(PLAYER_INFO),
            SnapObj::SpectatorInfo(_) => TypeId::from(SPECTATOR_INFO),
            SnapObj::DeClientInfo(_) => TypeId::from(DE_CLIENT_INFO),
            SnapObj::DeGameInfo(_) => TypeId::from(DE_GAME_INFO),
            SnapObj::DeTuneParams(_) => TypeId::from(DE_TUNE_PARAMS),
            SnapObj::Common(_) => TypeId::from(COMMON),
            SnapObj::Explosion(_) => TypeId::from(EXPLOSION),
            SnapObj::Spawn(_) => TypeId::from(SPAWN),
            SnapObj::HammerHit(_) => TypeId::from(HAMMER_HIT),
            SnapObj::Death(_) => TypeId::from(DEATH),
            SnapObj::SoundWorld(_) => TypeId::from(SOUND_WORLD),
            SnapObj::Damage(_) => TypeId::from(DAMAGE),
            SnapObj::PlayerInfoRace(_) => TypeId::from(PLAYER_INFO_RACE),
            SnapObj::GameDataRace(_) => TypeId::from(GAME_DATA_RACE),
        }
    }
    pub fn encode(&self) -> &[i32] {
        match *self {
            SnapObj::PlayerInput(ref i) => i.encode(),
            SnapObj::Projectile(ref i) => i.encode(),
            SnapObj::Laser(ref i) => i.encode(),
            SnapObj::Pickup(ref i) => i.encode(),
            SnapObj::Flag(ref i) => i.encode(),
            SnapObj::GameData(ref i) => i.encode(),
            SnapObj::GameDataTeam(ref i) => i.encode(),
            SnapObj::GameDataFlag(ref i) => i.encode(),
            SnapObj::CharacterCore(ref i) => i.encode(),
            SnapObj::Character(ref i) => i.encode(),
            SnapObj::PlayerInfo(ref i) => i.encode(),
            SnapObj::SpectatorInfo(ref i) => i.encode(),
            SnapObj::DeClientInfo(ref i) => i.encode(),
            SnapObj::DeGameInfo(ref i) => i.encode(),
            SnapObj::DeTuneParams(ref i) => i.encode(),
            SnapObj::Common(ref i) => i.encode(),
            SnapObj::Explosion(ref i) => i.encode(),
            SnapObj::Spawn(ref i) => i.encode(),
            SnapObj::HammerHit(ref i) => i.encode(),
            SnapObj::Death(ref i) => i.encode(),
            SnapObj::SoundWorld(ref i) => i.encode(),
            SnapObj::Damage(ref i) => i.encode(),
            SnapObj::PlayerInfoRace(ref i) => i.encode(),
            SnapObj::GameDataRace(ref i) => i.encode(),
        }
    }
}

impl fmt::Debug for SnapObj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SnapObj::PlayerInput(ref i) => i.fmt(f),
            SnapObj::Projectile(ref i) => i.fmt(f),
            SnapObj::Laser(ref i) => i.fmt(f),
            SnapObj::Pickup(ref i) => i.fmt(f),
            SnapObj::Flag(ref i) => i.fmt(f),
            SnapObj::GameData(ref i) => i.fmt(f),
            SnapObj::GameDataTeam(ref i) => i.fmt(f),
            SnapObj::GameDataFlag(ref i) => i.fmt(f),
            SnapObj::CharacterCore(ref i) => i.fmt(f),
            SnapObj::Character(ref i) => i.fmt(f),
            SnapObj::PlayerInfo(ref i) => i.fmt(f),
            SnapObj::SpectatorInfo(ref i) => i.fmt(f),
            SnapObj::DeClientInfo(ref i) => i.fmt(f),
            SnapObj::DeGameInfo(ref i) => i.fmt(f),
            SnapObj::DeTuneParams(ref i) => i.fmt(f),
            SnapObj::Common(ref i) => i.fmt(f),
            SnapObj::Explosion(ref i) => i.fmt(f),
            SnapObj::Spawn(ref i) => i.fmt(f),
            SnapObj::HammerHit(ref i) => i.fmt(f),
            SnapObj::Death(ref i) => i.fmt(f),
            SnapObj::SoundWorld(ref i) => i.fmt(f),
            SnapObj::Damage(ref i) => i.fmt(f),
            SnapObj::PlayerInfoRace(ref i) => i.fmt(f),
            SnapObj::GameDataRace(ref i) => i.fmt(f),
        }
    }
}

impl From<PlayerInput> for SnapObj {
    fn from(i: PlayerInput) -> SnapObj {
        SnapObj::PlayerInput(i)
    }
}

impl From<Projectile> for SnapObj {
    fn from(i: Projectile) -> SnapObj {
        SnapObj::Projectile(i)
    }
}

impl From<Laser> for SnapObj {
    fn from(i: Laser) -> SnapObj {
        SnapObj::Laser(i)
    }
}

impl From<Pickup> for SnapObj {
    fn from(i: Pickup) -> SnapObj {
        SnapObj::Pickup(i)
    }
}

impl From<Flag> for SnapObj {
    fn from(i: Flag) -> SnapObj {
        SnapObj::Flag(i)
    }
}

impl From<GameData> for SnapObj {
    fn from(i: GameData) -> SnapObj {
        SnapObj::GameData(i)
    }
}

impl From<GameDataTeam> for SnapObj {
    fn from(i: GameDataTeam) -> SnapObj {
        SnapObj::GameDataTeam(i)
    }
}

impl From<GameDataFlag> for SnapObj {
    fn from(i: GameDataFlag) -> SnapObj {
        SnapObj::GameDataFlag(i)
    }
}

impl From<CharacterCore> for SnapObj {
    fn from(i: CharacterCore) -> SnapObj {
        SnapObj::CharacterCore(i)
    }
}

impl From<Character> for SnapObj {
    fn from(i: Character) -> SnapObj {
        SnapObj::Character(i)
    }
}

impl From<PlayerInfo> for SnapObj {
    fn from(i: PlayerInfo) -> SnapObj {
        SnapObj::PlayerInfo(i)
    }
}

impl From<SpectatorInfo> for SnapObj {
    fn from(i: SpectatorInfo) -> SnapObj {
        SnapObj::SpectatorInfo(i)
    }
}

impl From<DeClientInfo> for SnapObj {
    fn from(i: DeClientInfo) -> SnapObj {
        SnapObj::DeClientInfo(i)
    }
}

impl From<DeGameInfo> for SnapObj {
    fn from(i: DeGameInfo) -> SnapObj {
        SnapObj::DeGameInfo(i)
    }
}

impl From<DeTuneParams> for SnapObj {
    fn from(i: DeTuneParams) -> SnapObj {
        SnapObj::DeTuneParams(i)
    }
}

impl From<Common> for SnapObj {
    fn from(i: Common) -> SnapObj {
        SnapObj::Common(i)
    }
}

impl From<Explosion> for SnapObj {
    fn from(i: Explosion) -> SnapObj {
        SnapObj::Explosion(i)
    }
}

impl From<Spawn> for SnapObj {
    fn from(i: Spawn) -> SnapObj {
        SnapObj::Spawn(i)
    }
}

impl From<HammerHit> for SnapObj {
    fn from(i: HammerHit) -> SnapObj {
        SnapObj::HammerHit(i)
    }
}

impl From<Death> for SnapObj {
    fn from(i: Death) -> SnapObj {
        SnapObj::Death(i)
    }
}

impl From<SoundWorld> for SnapObj {
    fn from(i: SoundWorld) -> SnapObj {
        SnapObj::SoundWorld(i)
    }
}

impl From<Damage> for SnapObj {
    fn from(i: Damage) -> SnapObj {
        SnapObj::Damage(i)
    }
}

impl From<PlayerInfoRace> for SnapObj {
    fn from(i: PlayerInfoRace) -> SnapObj {
        SnapObj::PlayerInfoRace(i)
    }
}

impl From<GameDataRace> for SnapObj {
    fn from(i: GameDataRace) -> SnapObj {
        SnapObj::GameDataRace(i)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PlayerInput {
    pub direction: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub jump: bool,
    pub fire: i32,
    pub hook: bool,
    pub player_flags: i32,
    pub wanted_weapon: i32,
    pub next_weapon: i32,
    pub prev_weapon: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Projectile {
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub type_: enums::Weapon,
    pub start_tick: ::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Laser {
    pub x: i32,
    pub y: i32,
    pub from_x: i32,
    pub from_y: i32,
    pub start_tick: ::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Pickup {
    pub x: i32,
    pub y: i32,
    pub type_: enums::Pickup,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Flag {
    pub x: i32,
    pub y: i32,
    pub team: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameData {
    pub game_start_tick: ::snap_obj::Tick,
    pub game_state_flags: i32,
    pub game_state_end_tick: ::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameDataTeam {
    pub teamscore_red: i32,
    pub teamscore_blue: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameDataFlag {
    pub flag_carrier_red: i32,
    pub flag_carrier_blue: i32,
    pub flag_drop_tick_red: ::snap_obj::Tick,
    pub flag_drop_tick_blue: ::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharacterCore {
    pub tick: ::snap_obj::Tick,
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub angle: i32,
    pub direction: i32,
    pub jumped: i32,
    pub hooked_player: i32,
    pub hook_state: i32,
    pub hook_tick: ::snap_obj::Tick,
    pub hook_x: i32,
    pub hook_y: i32,
    pub hook_dx: i32,
    pub hook_dy: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Character {
    pub character_core: CharacterCore,
    pub health: i32,
    pub armor: i32,
    pub ammo_count: i32,
    pub weapon: enums::Weapon,
    pub emote: enums::Emote,
    pub attack_tick: ::snap_obj::Tick,
    pub triggered_events: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerInfo {
    pub player_flags: i32,
    pub score: i32,
    pub latency: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SpectatorInfo {
    pub spec_mode: enums::Spec,
    pub spectator_id: i32,
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DeClientInfo {
    pub local: bool,
    pub team: enums::Team,
    pub name: [i32; 4],
    pub clan: [i32; 3],
    pub country: i32,
    pub skin_part_names: [[i32; 6]; 6],
    pub use_custom_colors: [bool; 6],
    pub skin_part_colors: [i32; 6],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DeGameInfo {
    pub game_flags: i32,
    pub score_limit: i32,
    pub time_limit: i32,
    pub match_num: i32,
    pub match_current: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DeTuneParams {
    pub tune_params: [i32; 32],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Common {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Explosion {
    pub common: Common,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Spawn {
    pub common: Common,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HammerHit {
    pub common: Common,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Death {
    pub common: Common,
    pub client_id: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SoundWorld {
    pub common: Common,
    pub sound_id: enums::Sound,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Damage {
    pub common: Common,
    pub client_id: i32,
    pub angle: i32,
    pub health_amount: i32,
    pub armor_amount: i32,
    pub self_: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerInfoRace {
    pub race_start_tick: ::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameDataRace {
    pub best_time: i32,
    pub precision: i32,
    pub race_flags: i32,
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInput, Error> {
        Ok(PlayerInput {
            direction: in_range(_p.read_int()?, -1, 1)?,
            target_x: _p.read_int()?,
            target_y: _p.read_int()?,
            jump: to_bool(_p.read_int()?)?,
            fire: _p.read_int()?,
            hook: to_bool(_p.read_int()?)?,
            player_flags: _p.read_int()?,
            wanted_weapon: in_range(_p.read_int()?, 0, 6)?,
            next_weapon: _p.read_int()?,
            prev_weapon: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.direction && self.direction <= 1);
        assert!(0 <= self.wanted_weapon && self.wanted_weapon <= 6);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}
impl PlayerInput {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PlayerInput, Error> {
        let result = Ok(PlayerInput {
            direction: in_range(_p.read_int(warn)?, -1, 1)?,
            target_x: _p.read_int(warn)?,
            target_y: _p.read_int(warn)?,
            jump: to_bool(_p.read_int(warn)?)?,
            fire: _p.read_int(warn)?,
            hook: to_bool(_p.read_int(warn)?)?,
            player_flags: _p.read_int(warn)?,
            wanted_weapon: in_range(_p.read_int(warn)?, 0, 6)?,
            next_weapon: _p.read_int(warn)?,
            prev_weapon: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.direction && self.direction <= 1);
        assert!(0 <= self.wanted_weapon && self.wanted_weapon <= 6);
        _p.write_int(self.direction)?;
        _p.write_int(self.target_x)?;
        _p.write_int(self.target_y)?;
        _p.write_int(self.jump as i32)?;
        _p.write_int(self.fire)?;
        _p.write_int(self.hook as i32)?;
        _p.write_int(self.player_flags)?;
        _p.write_int(self.wanted_weapon)?;
        _p.write_int(self.next_weapon)?;
        _p.write_int(self.prev_weapon)?;
        Ok(_p.written())
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Projectile, Error> {
        Ok(Projectile {
            x: _p.read_int()?,
            y: _p.read_int()?,
            vel_x: _p.read_int()?,
            vel_y: _p.read_int()?,
            type_: enums::Weapon::from_i32(_p.read_int()?)?,
            start_tick: ::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}
impl Projectile {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Projectile, Error> {
        let result = Ok(Projectile {
            x: _p.read_int(warn)?,
            y: _p.read_int(warn)?,
            vel_x: _p.read_int(warn)?,
            vel_y: _p.read_int(warn)?,
            type_: enums::Weapon::from_i32(_p.read_int(warn)?)?,
            start_tick: ::snap_obj::Tick(_p.read_int(warn)?),
        });
        _p.finish(warn);
        result
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.x)?;
        _p.write_int(self.y)?;
        _p.write_int(self.vel_x)?;
        _p.write_int(self.vel_y)?;
        _p.write_int(self.type_.to_i32())?;
        _p.write_int(self.start_tick.0)?;
        Ok(_p.written())
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Laser, Error> {
        Ok(Laser {
            x: _p.read_int()?,
            y: _p.read_int()?,
            from_x: _p.read_int()?,
            from_y: _p.read_int()?,
            start_tick: ::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for Pickup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Pickup")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("type_", &self.type_)
            .finish()
    }
}
impl Pickup {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Pickup, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Pickup, Error> {
        Ok(Pickup {
            x: _p.read_int()?,
            y: _p.read_int()?,
            type_: enums::Pickup::from_i32(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Flag, Error> {
        Ok(Flag {
            x: _p.read_int()?,
            y: _p.read_int()?,
            team: in_range(_p.read_int()?, 0, 1)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.team && self.team <= 1);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for GameData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameData")
            .field("game_start_tick", &self.game_start_tick)
            .field("game_state_flags", &self.game_state_flags)
            .field("game_state_end_tick", &self.game_state_end_tick)
            .finish()
    }
}
impl GameData {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameData, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameData, Error> {
        Ok(GameData {
            game_start_tick: ::snap_obj::Tick(_p.read_int()?),
            game_state_flags: _p.read_int()?,
            game_state_end_tick: ::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for GameDataTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameDataTeam")
            .field("teamscore_red", &self.teamscore_red)
            .field("teamscore_blue", &self.teamscore_blue)
            .finish()
    }
}
impl GameDataTeam {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameDataTeam, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameDataTeam, Error> {
        Ok(GameDataTeam {
            teamscore_red: _p.read_int()?,
            teamscore_blue: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for GameDataFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameDataFlag")
            .field("flag_carrier_red", &self.flag_carrier_red)
            .field("flag_carrier_blue", &self.flag_carrier_blue)
            .field("flag_drop_tick_red", &self.flag_drop_tick_red)
            .field("flag_drop_tick_blue", &self.flag_drop_tick_blue)
            .finish()
    }
}
impl GameDataFlag {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameDataFlag, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameDataFlag, Error> {
        Ok(GameDataFlag {
            flag_carrier_red: in_range(_p.read_int()?, -3, 63)?,
            flag_carrier_blue: in_range(_p.read_int()?, -3, 63)?,
            flag_drop_tick_red: ::snap_obj::Tick(_p.read_int()?),
            flag_drop_tick_blue: ::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-3 <= self.flag_carrier_red && self.flag_carrier_red <= 63);
        assert!(-3 <= self.flag_carrier_blue && self.flag_carrier_blue <= 63);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<CharacterCore, Error> {
        Ok(CharacterCore {
            tick: ::snap_obj::Tick(_p.read_int()?),
            x: _p.read_int()?,
            y: _p.read_int()?,
            vel_x: _p.read_int()?,
            vel_y: _p.read_int()?,
            angle: _p.read_int()?,
            direction: in_range(_p.read_int()?, -1, 1)?,
            jumped: in_range(_p.read_int()?, 0, 3)?,
            hooked_player: in_range(_p.read_int()?, -1, 63)?,
            hook_state: in_range(_p.read_int()?, -1, 5)?,
            hook_tick: ::snap_obj::Tick(_p.read_int()?),
            hook_x: _p.read_int()?,
            hook_y: _p.read_int()?,
            hook_dx: _p.read_int()?,
            hook_dy: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.direction && self.direction <= 1);
        assert!(0 <= self.jumped && self.jumped <= 3);
        assert!(-1 <= self.hooked_player && self.hooked_player <= 63);
        assert!(-1 <= self.hook_state && self.hook_state <= 5);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Character")
            .field("character_core", &self.character_core)
            .field("health", &self.health)
            .field("armor", &self.armor)
            .field("ammo_count", &self.ammo_count)
            .field("weapon", &self.weapon)
            .field("emote", &self.emote)
            .field("attack_tick", &self.attack_tick)
            .field("triggered_events", &self.triggered_events)
            .finish()
    }
}
impl Character {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Character, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Character, Error> {
        Ok(Character {
            character_core: CharacterCore::decode_inner(_p)?,
            health: in_range(_p.read_int()?, 0, 10)?,
            armor: in_range(_p.read_int()?, 0, 10)?,
            ammo_count: _p.read_int()?,
            weapon: enums::Weapon::from_i32(_p.read_int()?)?,
            emote: enums::Emote::from_i32(_p.read_int()?)?,
            attack_tick: ::snap_obj::Tick(_p.read_int()?),
            triggered_events: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.character_core.encode();
        assert!(0 <= self.health && self.health <= 10);
        assert!(0 <= self.armor && self.armor <= 10);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for PlayerInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PlayerInfo")
            .field("player_flags", &self.player_flags)
            .field("score", &self.score)
            .field("latency", &self.latency)
            .finish()
    }
}
impl PlayerInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<PlayerInfo, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInfo, Error> {
        Ok(PlayerInfo {
            player_flags: _p.read_int()?,
            score: _p.read_int()?,
            latency: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for SpectatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpectatorInfo")
            .field("spec_mode", &self.spec_mode)
            .field("spectator_id", &self.spectator_id)
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
impl SpectatorInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<SpectatorInfo, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SpectatorInfo, Error> {
        Ok(SpectatorInfo {
            spec_mode: enums::Spec::from_i32(_p.read_int()?)?,
            spectator_id: in_range(_p.read_int()?, -1, 63)?,
            x: _p.read_int()?,
            y: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.spectator_id && self.spectator_id <= 63);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for DeClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeClientInfo")
            .field("local", &self.local)
            .field("team", &self.team)
            .field("name", &self.name)
            .field("clan", &self.clan)
            .field("country", &self.country)
            .field("skin_part_names", &self.skin_part_names)
            .field("use_custom_colors", &self.use_custom_colors)
            .field("skin_part_colors", &self.skin_part_colors)
            .finish()
    }
}
impl DeClientInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DeClientInfo, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DeClientInfo, Error> {
        Ok(DeClientInfo {
            local: to_bool(_p.read_int()?)?,
            team: enums::Team::from_i32(_p.read_int()?)?,
            name: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
            clan: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
            country: _p.read_int()?,
            skin_part_names: [
                [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
                [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
                [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
                [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
                [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
                [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
            ],
            use_custom_colors: [
                to_bool(_p.read_int()?)?,
                to_bool(_p.read_int()?)?,
                to_bool(_p.read_int()?)?,
                to_bool(_p.read_int()?)?,
                to_bool(_p.read_int()?)?,
                to_bool(_p.read_int()?)?,
            ],
            skin_part_colors: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for DeGameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeGameInfo")
            .field("game_flags", &self.game_flags)
            .field("score_limit", &self.score_limit)
            .field("time_limit", &self.time_limit)
            .field("match_num", &self.match_num)
            .field("match_current", &self.match_current)
            .finish()
    }
}
impl DeGameInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DeGameInfo, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DeGameInfo, Error> {
        Ok(DeGameInfo {
            game_flags: _p.read_int()?,
            score_limit: positive(_p.read_int()?)?,
            time_limit: positive(_p.read_int()?)?,
            match_num: positive(_p.read_int()?)?,
            match_current: positive(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(self.score_limit >= 0);
        assert!(self.time_limit >= 0);
        assert!(self.match_num >= 0);
        assert!(self.match_current >= 0);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for DeTuneParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeTuneParams")
            .field("tune_params", &self.tune_params)
            .finish()
    }
}
impl DeTuneParams {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DeTuneParams, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DeTuneParams, Error> {
        Ok(DeTuneParams {
            tune_params: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Common, Error> {
        Ok(Common {
            x: _p.read_int()?,
            y: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Explosion, Error> {
        Ok(Explosion {
            common: Common::decode_inner(_p)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Spawn, Error> {
        Ok(Spawn {
            common: Common::decode_inner(_p)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<HammerHit, Error> {
        Ok(HammerHit {
            common: Common::decode_inner(_p)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Death, Error> {
        Ok(Death {
            common: Common::decode_inner(_p)?,
            client_id: in_range(_p.read_int()?, 0, 63)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        assert!(0 <= self.client_id && self.client_id <= 63);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SoundWorld, Error> {
        Ok(SoundWorld {
            common: Common::decode_inner(_p)?,
            sound_id: enums::Sound::from_i32(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for Damage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Damage")
            .field("common", &self.common)
            .field("client_id", &self.client_id)
            .field("angle", &self.angle)
            .field("health_amount", &self.health_amount)
            .field("armor_amount", &self.armor_amount)
            .field("self_", &self.self_)
            .finish()
    }
}
impl Damage {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Damage, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Damage, Error> {
        Ok(Damage {
            common: Common::decode_inner(_p)?,
            client_id: in_range(_p.read_int()?, 0, 63)?,
            angle: _p.read_int()?,
            health_amount: in_range(_p.read_int()?, 0, 9)?,
            armor_amount: in_range(_p.read_int()?, 0, 9)?,
            self_: to_bool(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        assert!(0 <= self.client_id && self.client_id <= 63);
        assert!(0 <= self.health_amount && self.health_amount <= 9);
        assert!(0 <= self.armor_amount && self.armor_amount <= 9);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for PlayerInfoRace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PlayerInfoRace")
            .field("race_start_tick", &self.race_start_tick)
            .finish()
    }
}
impl PlayerInfoRace {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<PlayerInfoRace, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInfoRace, Error> {
        Ok(PlayerInfoRace {
            race_start_tick: ::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for GameDataRace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameDataRace")
            .field("best_time", &self.best_time)
            .field("precision", &self.precision)
            .field("race_flags", &self.race_flags)
            .finish()
    }
}
impl GameDataRace {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameDataRace, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameDataRace, Error> {
        Ok(GameDataRace {
            best_time: at_least(_p.read_int()?, -1)?,
            precision: in_range(_p.read_int()?, 0, 3)?,
            race_flags: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(self.best_time >= -1);
        assert!(0 <= self.precision && self.precision <= 3);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

pub fn obj_size(type_: u16) -> Option<u32> {
    Some(match type_ {
        PLAYER_INPUT => 10,
        PROJECTILE => 6,
        LASER => 5,
        PICKUP => 3,
        FLAG => 3,
        GAME_DATA => 3,
        GAME_DATA_TEAM => 2,
        GAME_DATA_FLAG => 4,
        CHARACTER_CORE => 15,
        CHARACTER => 22,
        PLAYER_INFO => 3,
        SPECTATOR_INFO => 4,
        DE_CLIENT_INFO => 58,
        DE_GAME_INFO => 5,
        DE_TUNE_PARAMS => 32,
        COMMON => 2,
        EXPLOSION => 2,
        SPAWN => 2,
        HAMMER_HIT => 2,
        DEATH => 3,
        SOUND_WORLD => 3,
        DAMAGE => 7,
        PLAYER_INFO_RACE => 1,
        GAME_DATA_RACE => 3,
        _ => return None,
    })
}
