use buffer::CapacityError;
use common::slice;
use debug::DebugSlice;
use enums::Emote;
use enums::FLAG_MISSING;
use enums::MAX_CLIENTS;
use enums::SPEC_FREEVIEW;
use enums::Sound;
use enums::TEAM_BLUE;
use enums::TEAM_RED;
use enums::Team;
use enums::Weapon;
use error::Error;
use packer::ExcessData;
use packer::IntUnpacker;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::in_range;
use packer::positive;
use std::fmt;
use warn::Warn;

#[derive(Clone, Copy, Debug)]
pub struct Tick(pub i32);

impl Projectile {
    pub fn decode_msg_inner<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Projectile, Error> {
        Ok(Projectile {
            x: _p.read_int(warn)?,
            y: _p.read_int(warn)?,
            vel_x: _p.read_int(warn)?,
            vel_y: _p.read_int(warn)?,
            type_: Weapon::from_i32(_p.read_int(warn)?)?,
            start_tick: Tick(_p.read_int(warn)?),
        })
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        // For the assert!()s.
        self.encode();

        _p.write_int(self.x)?;
        _p.write_int(self.y)?;
        _p.write_int(self.vel_x)?;
        _p.write_int(self.vel_y)?;
        _p.write_int(self.type_.to_i32())?;
        _p.write_int(self.start_tick.0)?;
        Ok(_p.written())
    }
}

impl PlayerInput {
    pub fn decode_msg_inner<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PlayerInput, Error> {
        Ok(PlayerInput {
            direction: _p.read_int(warn)?,
            target_x: _p.read_int(warn)?,
            target_y: _p.read_int(warn)?,
            jump: _p.read_int(warn)?,
            fire: _p.read_int(warn)?,
            hook: _p.read_int(warn)?,
            player_flags: _p.read_int(warn)?,
            wanted_weapon: _p.read_int(warn)?,
            next_weapon: _p.read_int(warn)?,
            prev_weapon: _p.read_int(warn)?,
        })
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        // For the assert!()s.
        self.encode();

        _p.write_int(self.direction)?;
        _p.write_int(self.target_x)?;
        _p.write_int(self.target_y)?;
        _p.write_int(self.jump)?;
        _p.write_int(self.fire)?;
        _p.write_int(self.hook)?;
        _p.write_int(self.player_flags)?;
        _p.write_int(self.wanted_weapon)?;
        _p.write_int(self.next_weapon)?;
        _p.write_int(self.prev_weapon)?;
        Ok(_p.written())
    }
}

pub const PLAYER_INPUT_EMPTY: PlayerInput = PlayerInput {
    direction: 0,
    target_x: 0,
    target_y: 0,
    jump: 0,
    fire: 0,
    hook: 0,
    player_flags: 0,
    wanted_weapon: 0,
    next_weapon: 0,
    prev_weapon: 0,
};

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
pub enum SnapObj {
    PlayerInput(PlayerInput),
    Projectile(Projectile),
    Laser(Laser),
    Pickup(Pickup),
    Flag(Flag),
    GameInfo(GameInfo),
    GameData(GameData),
    CharacterCore(CharacterCore),
    Character(Character),
    PlayerInfo(PlayerInfo),
    ClientInfo(ClientInfo),
    SpectatorInfo(SpectatorInfo),
    Common(Common),
    Explosion(Explosion),
    Spawn(Spawn),
    HammerHit(HammerHit),
    Death(Death),
    SoundGlobal(SoundGlobal),
    SoundWorld(SoundWorld),
    DamageInd(DamageInd),
}

impl SnapObj {
    pub fn decode_obj<W: Warn<ExcessData>>(warn: &mut W, obj_type_id: u16, _p: &mut IntUnpacker) -> Result<SnapObj, Error> {
        Ok(match obj_type_id {
            PLAYER_INPUT => SnapObj::PlayerInput(PlayerInput::decode(warn, _p)?),
            PROJECTILE => SnapObj::Projectile(Projectile::decode(warn, _p)?),
            LASER => SnapObj::Laser(Laser::decode(warn, _p)?),
            PICKUP => SnapObj::Pickup(Pickup::decode(warn, _p)?),
            FLAG => SnapObj::Flag(Flag::decode(warn, _p)?),
            GAME_INFO => SnapObj::GameInfo(GameInfo::decode(warn, _p)?),
            GAME_DATA => SnapObj::GameData(GameData::decode(warn, _p)?),
            CHARACTER_CORE => SnapObj::CharacterCore(CharacterCore::decode(warn, _p)?),
            CHARACTER => SnapObj::Character(Character::decode(warn, _p)?),
            PLAYER_INFO => SnapObj::PlayerInfo(PlayerInfo::decode(warn, _p)?),
            CLIENT_INFO => SnapObj::ClientInfo(ClientInfo::decode(warn, _p)?),
            SPECTATOR_INFO => SnapObj::SpectatorInfo(SpectatorInfo::decode(warn, _p)?),
            COMMON => SnapObj::Common(Common::decode(warn, _p)?),
            EXPLOSION => SnapObj::Explosion(Explosion::decode(warn, _p)?),
            SPAWN => SnapObj::Spawn(Spawn::decode(warn, _p)?),
            HAMMER_HIT => SnapObj::HammerHit(HammerHit::decode(warn, _p)?),
            DEATH => SnapObj::Death(Death::decode(warn, _p)?),
            SOUND_GLOBAL => SnapObj::SoundGlobal(SoundGlobal::decode(warn, _p)?),
            SOUND_WORLD => SnapObj::SoundWorld(SoundWorld::decode(warn, _p)?),
            DAMAGE_IND => SnapObj::DamageInd(DamageInd::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn obj_type_id(&self) -> u16 {
        match *self {
            SnapObj::PlayerInput(_) => PLAYER_INPUT,
            SnapObj::Projectile(_) => PROJECTILE,
            SnapObj::Laser(_) => LASER,
            SnapObj::Pickup(_) => PICKUP,
            SnapObj::Flag(_) => FLAG,
            SnapObj::GameInfo(_) => GAME_INFO,
            SnapObj::GameData(_) => GAME_DATA,
            SnapObj::CharacterCore(_) => CHARACTER_CORE,
            SnapObj::Character(_) => CHARACTER,
            SnapObj::PlayerInfo(_) => PLAYER_INFO,
            SnapObj::ClientInfo(_) => CLIENT_INFO,
            SnapObj::SpectatorInfo(_) => SPECTATOR_INFO,
            SnapObj::Common(_) => COMMON,
            SnapObj::Explosion(_) => EXPLOSION,
            SnapObj::Spawn(_) => SPAWN,
            SnapObj::HammerHit(_) => HAMMER_HIT,
            SnapObj::Death(_) => DEATH,
            SnapObj::SoundGlobal(_) => SOUND_GLOBAL,
            SnapObj::SoundWorld(_) => SOUND_WORLD,
            SnapObj::DamageInd(_) => DAMAGE_IND,
        }
    }
    pub fn encode(&self) -> &[i32] {
        match *self {
            SnapObj::PlayerInput(ref i) => i.encode(),
            SnapObj::Projectile(ref i) => i.encode(),
            SnapObj::Laser(ref i) => i.encode(),
            SnapObj::Pickup(ref i) => i.encode(),
            SnapObj::Flag(ref i) => i.encode(),
            SnapObj::GameInfo(ref i) => i.encode(),
            SnapObj::GameData(ref i) => i.encode(),
            SnapObj::CharacterCore(ref i) => i.encode(),
            SnapObj::Character(ref i) => i.encode(),
            SnapObj::PlayerInfo(ref i) => i.encode(),
            SnapObj::ClientInfo(ref i) => i.encode(),
            SnapObj::SpectatorInfo(ref i) => i.encode(),
            SnapObj::Common(ref i) => i.encode(),
            SnapObj::Explosion(ref i) => i.encode(),
            SnapObj::Spawn(ref i) => i.encode(),
            SnapObj::HammerHit(ref i) => i.encode(),
            SnapObj::Death(ref i) => i.encode(),
            SnapObj::SoundGlobal(ref i) => i.encode(),
            SnapObj::SoundWorld(ref i) => i.encode(),
            SnapObj::DamageInd(ref i) => i.encode(),
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
            SnapObj::GameInfo(ref i) => i.fmt(f),
            SnapObj::GameData(ref i) => i.fmt(f),
            SnapObj::CharacterCore(ref i) => i.fmt(f),
            SnapObj::Character(ref i) => i.fmt(f),
            SnapObj::PlayerInfo(ref i) => i.fmt(f),
            SnapObj::ClientInfo(ref i) => i.fmt(f),
            SnapObj::SpectatorInfo(ref i) => i.fmt(f),
            SnapObj::Common(ref i) => i.fmt(f),
            SnapObj::Explosion(ref i) => i.fmt(f),
            SnapObj::Spawn(ref i) => i.fmt(f),
            SnapObj::HammerHit(ref i) => i.fmt(f),
            SnapObj::Death(ref i) => i.fmt(f),
            SnapObj::SoundGlobal(ref i) => i.fmt(f),
            SnapObj::SoundWorld(ref i) => i.fmt(f),
            SnapObj::DamageInd(ref i) => i.fmt(f),
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

impl From<GameInfo> for SnapObj {
    fn from(i: GameInfo) -> SnapObj {
        SnapObj::GameInfo(i)
    }
}

impl From<GameData> for SnapObj {
    fn from(i: GameData) -> SnapObj {
        SnapObj::GameData(i)
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

impl From<ClientInfo> for SnapObj {
    fn from(i: ClientInfo) -> SnapObj {
        SnapObj::ClientInfo(i)
    }
}

impl From<SpectatorInfo> for SnapObj {
    fn from(i: SpectatorInfo) -> SnapObj {
        SnapObj::SpectatorInfo(i)
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

impl From<SoundGlobal> for SnapObj {
    fn from(i: SoundGlobal) -> SnapObj {
        SnapObj::SoundGlobal(i)
    }
}

impl From<SoundWorld> for SnapObj {
    fn from(i: SoundWorld) -> SnapObj {
        SnapObj::SoundWorld(i)
    }
}

impl From<DamageInd> for SnapObj {
    fn from(i: DamageInd) -> SnapObj {
        SnapObj::DamageInd(i)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Projectile {
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub type_: Weapon,
    pub start_tick: Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Laser {
    pub x: i32,
    pub y: i32,
    pub from_x: i32,
    pub from_y: i32,
    pub start_tick: Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Pickup {
    pub x: i32,
    pub y: i32,
    pub type_: i32,
    pub subtype: i32,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameData {
    pub teamscore_red: i32,
    pub teamscore_blue: i32,
    pub flag_carrier_red: i32,
    pub flag_carrier_blue: i32,
}

#[repr(C)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Character {
    pub character_core: CharacterCore,
    pub player_flags: i32,
    pub health: i32,
    pub armor: i32,
    pub ammo_count: i32,
    pub weapon: Weapon,
    pub emote: Emote,
    pub attack_tick: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerInfo {
    pub local: i32,
    pub client_id: i32,
    pub team: Team,
    pub score: i32,
    pub latency: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClientInfo {
    pub name: [i32; 4],
    pub clan: [i32; 3],
    pub country: i32,
    pub skin: [i32; 6],
    pub use_custom_color: i32,
    pub color_body: i32,
    pub color_feet: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SpectatorInfo {
    pub spectator_id: i32,
    pub x: i32,
    pub y: i32,
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
pub struct SoundGlobal {
    pub common: Common,
    pub sound_id: Sound,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SoundWorld {
    pub common: Common,
    pub sound_id: Sound,
}

#[repr(C)]
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInput, Error> {
        Ok(PlayerInput {
            direction: _p.read_int()?,
            target_x: _p.read_int()?,
            target_y: _p.read_int()?,
            jump: _p.read_int()?,
            fire: _p.read_int()?,
            hook: _p.read_int()?,
            player_flags: _p.read_int()?,
            wanted_weapon: _p.read_int()?,
            next_weapon: _p.read_int()?,
            prev_weapon: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
            type_: Weapon::from_i32(_p.read_int()?)?,
            start_tick: Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
            start_tick: Tick(_p.read_int()?),
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
            .field("subtype", &self.subtype)
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
            type_: positive(_p.read_int()?)?,
            subtype: positive(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(self.type_ >= 0);
        assert!(self.subtype >= 0);
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
            team: in_range(_p.read_int()?, TEAM_RED, TEAM_BLUE)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(TEAM_RED <= self.team && self.team <= TEAM_BLUE);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameInfo, Error> {
        Ok(GameInfo {
            game_flags: in_range(_p.read_int()?, 0, 256)?,
            game_state_flags: in_range(_p.read_int()?, 0, 256)?,
            round_start_tick: Tick(_p.read_int()?),
            warmup_timer: positive(_p.read_int()?)?,
            score_limit: positive(_p.read_int()?)?,
            time_limit: positive(_p.read_int()?)?,
            round_num: positive(_p.read_int()?)?,
            round_current: positive(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.game_flags && self.game_flags <= 256);
        assert!(0 <= self.game_state_flags && self.game_state_flags <= 256);
        assert!(self.warmup_timer >= 0);
        assert!(self.score_limit >= 0);
        assert!(self.time_limit >= 0);
        assert!(self.round_num >= 0);
        assert!(self.round_current >= 0);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameData, Error> {
        Ok(GameData {
            teamscore_red: _p.read_int()?,
            teamscore_blue: _p.read_int()?,
            flag_carrier_red: in_range(_p.read_int()?, FLAG_MISSING, MAX_CLIENTS-1)?,
            flag_carrier_blue: in_range(_p.read_int()?, FLAG_MISSING, MAX_CLIENTS-1)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(FLAG_MISSING <= self.flag_carrier_red && self.flag_carrier_red <= MAX_CLIENTS-1);
        assert!(FLAG_MISSING <= self.flag_carrier_blue && self.flag_carrier_blue <= MAX_CLIENTS-1);
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
            tick: _p.read_int()?,
            x: _p.read_int()?,
            y: _p.read_int()?,
            vel_x: _p.read_int()?,
            vel_y: _p.read_int()?,
            angle: _p.read_int()?,
            direction: in_range(_p.read_int()?, -1, 1)?,
            jumped: in_range(_p.read_int()?, 0, 3)?,
            hooked_player: in_range(_p.read_int()?, -1, MAX_CLIENTS-1)?,
            hook_state: in_range(_p.read_int()?, -1, 5)?,
            hook_tick: Tick(_p.read_int()?),
            hook_x: _p.read_int()?,
            hook_y: _p.read_int()?,
            hook_dx: _p.read_int()?,
            hook_dy: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.direction && self.direction <= 1);
        assert!(0 <= self.jumped && self.jumped <= 3);
        assert!(-1 <= self.hooked_player && self.hooked_player <= MAX_CLIENTS-1);
        assert!(-1 <= self.hook_state && self.hook_state <= 5);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Character, Error> {
        Ok(Character {
            character_core: CharacterCore::decode_inner(_p)?,
            player_flags: in_range(_p.read_int()?, 0, 256)?,
            health: in_range(_p.read_int()?, 0, 10)?,
            armor: in_range(_p.read_int()?, 0, 10)?,
            ammo_count: in_range(_p.read_int()?, 0, 10)?,
            weapon: Weapon::from_i32(_p.read_int()?)?,
            emote: Emote::from_i32(_p.read_int()?)?,
            attack_tick: positive(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.character_core.encode();
        assert!(0 <= self.player_flags && self.player_flags <= 256);
        assert!(0 <= self.health && self.health <= 10);
        assert!(0 <= self.armor && self.armor <= 10);
        assert!(0 <= self.ammo_count && self.ammo_count <= 10);
        assert!(self.attack_tick >= 0);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<PlayerInfo, Error> {
        Ok(PlayerInfo {
            local: in_range(_p.read_int()?, 0, 1)?,
            client_id: in_range(_p.read_int()?, 0, MAX_CLIENTS-1)?,
            team: Team::from_i32(_p.read_int()?)?,
            score: _p.read_int()?,
            latency: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.local && self.local <= 1);
        assert!(0 <= self.client_id && self.client_id <= MAX_CLIENTS-1);
        unsafe { slice::transmute(slice::ref_slice(self)) }
    }
}

impl fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClientInfo")
            .field("name", &DebugSlice::new(&self.name, |e| e))
            .field("clan", &DebugSlice::new(&self.clan, |e| e))
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<ClientInfo, Error> {
        Ok(ClientInfo {
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
            skin: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
            use_custom_color: in_range(_p.read_int()?, 0, 1)?,
            color_body: _p.read_int()?,
            color_feet: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.use_custom_color && self.use_custom_color <= 1);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SpectatorInfo, Error> {
        Ok(SpectatorInfo {
            spectator_id: in_range(_p.read_int()?, SPEC_FREEVIEW, MAX_CLIENTS-1)?,
            x: _p.read_int()?,
            y: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(SPEC_FREEVIEW <= self.spectator_id && self.spectator_id <= MAX_CLIENTS-1);
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
            client_id: in_range(_p.read_int()?, 0, MAX_CLIENTS-1)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        assert!(0 <= self.client_id && self.client_id <= MAX_CLIENTS-1);
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SoundGlobal, Error> {
        Ok(SoundGlobal {
            common: Common::decode_inner(_p)?,
            sound_id: Sound::from_i32(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
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
            sound_id: Sound::from_i32(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DamageInd, Error> {
        Ok(DamageInd {
            common: Common::decode_inner(_p)?,
            angle: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(slice::ref_slice(self)) }
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
