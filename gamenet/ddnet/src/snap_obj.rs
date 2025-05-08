use crate::enums;
use crate::error::Error;
use buffer::CapacityError;
use libtw2_common::slice;
use libtw2_packer::ExcessData;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Packer;
use libtw2_packer::Unpacker;
use libtw2_packer::Warning;
use libtw2_packer::in_range;
use libtw2_packer::positive;
use libtw2_packer::to_bool;
use std::fmt;
use std::slice::from_ref;
use uuid::Uuid;
use warn::Warn;
use warn::wrap;

pub use libtw2_gamenet_common::snap_obj::Tick;
pub use libtw2_gamenet_common::snap_obj::TypeId;

pub const PLAYERFLAG_PLAYING: i32 = 1 << 0;
pub const PLAYERFLAG_IN_MENU: i32 = 1 << 1;
pub const PLAYERFLAG_CHATTING: i32 = 1 << 2;
pub const PLAYERFLAG_SCOREBOARD: i32 = 1 << 3;
pub const PLAYERFLAG_AIM: i32 = 1 << 4;
pub const PLAYERFLAG_SPEC_CAM: i32 = 1 << 5;

pub const GAMEFLAG_TEAMS: i32 = 1 << 0;
pub const GAMEFLAG_FLAGS: i32 = 1 << 1;

pub const GAMESTATEFLAG_GAMEOVER: i32 = 1 << 0;
pub const GAMESTATEFLAG_SUDDENDEATH: i32 = 1 << 1;
pub const GAMESTATEFLAG_PAUSED: i32 = 1 << 2;
pub const GAMESTATEFLAG_RACETIME: i32 = 1 << 3;

pub const CHARACTERFLAG_SOLO: i32 = 1 << 0;
pub const CHARACTERFLAG_JETPACK: i32 = 1 << 1;
pub const CHARACTERFLAG_COLLISION_DISABLED: i32 = 1 << 2;
pub const CHARACTERFLAG_ENDLESS_HOOK: i32 = 1 << 3;
pub const CHARACTERFLAG_ENDLESS_JUMP: i32 = 1 << 4;
pub const CHARACTERFLAG_SUPER: i32 = 1 << 5;
pub const CHARACTERFLAG_HAMMER_HIT_DISABLED: i32 = 1 << 6;
pub const CHARACTERFLAG_SHOTGUN_HIT_DISABLED: i32 = 1 << 7;
pub const CHARACTERFLAG_GRENADE_HIT_DISABLED: i32 = 1 << 8;
pub const CHARACTERFLAG_LASER_HIT_DISABLED: i32 = 1 << 9;
pub const CHARACTERFLAG_HOOK_HIT_DISABLED: i32 = 1 << 10;
pub const CHARACTERFLAG_TELEGUN_GUN: i32 = 1 << 11;
pub const CHARACTERFLAG_TELEGUN_GRENADE: i32 = 1 << 12;
pub const CHARACTERFLAG_TELEGUN_LASER: i32 = 1 << 13;
pub const CHARACTERFLAG_WEAPON_HAMMER: i32 = 1 << 14;
pub const CHARACTERFLAG_WEAPON_GUN: i32 = 1 << 15;
pub const CHARACTERFLAG_WEAPON_SHOTGUN: i32 = 1 << 16;
pub const CHARACTERFLAG_WEAPON_GRENADE: i32 = 1 << 17;
pub const CHARACTERFLAG_WEAPON_LASER: i32 = 1 << 18;
pub const CHARACTERFLAG_WEAPON_NINJA: i32 = 1 << 19;
pub const CHARACTERFLAG_MOVEMENTS_DISABLED: i32 = 1 << 20;
pub const CHARACTERFLAG_IN_FREEZE: i32 = 1 << 21;
pub const CHARACTERFLAG_PRACTICE_MODE: i32 = 1 << 22;
pub const CHARACTERFLAG_LOCK_MODE: i32 = 1 << 23;
pub const CHARACTERFLAG_TEAM0_MODE: i32 = 1 << 24;
pub const CHARACTERFLAG_INVINCIBLE: i32 = 1 << 25;

pub const GAMEINFOFLAG_TIMESCORE: i32 = 1 << 0;
pub const GAMEINFOFLAG_GAMETYPE_RACE: i32 = 1 << 1;
pub const GAMEINFOFLAG_GAMETYPE_FASTCAP: i32 = 1 << 2;
pub const GAMEINFOFLAG_GAMETYPE_FNG: i32 = 1 << 3;
pub const GAMEINFOFLAG_GAMETYPE_DDRACE: i32 = 1 << 4;
pub const GAMEINFOFLAG_GAMETYPE_DDNET: i32 = 1 << 5;
pub const GAMEINFOFLAG_GAMETYPE_BLOCK_WORLDS: i32 = 1 << 6;
pub const GAMEINFOFLAG_GAMETYPE_VANILLA: i32 = 1 << 7;
pub const GAMEINFOFLAG_GAMETYPE_PLUS: i32 = 1 << 8;
pub const GAMEINFOFLAG_FLAG_STARTS_RACE: i32 = 1 << 9;
pub const GAMEINFOFLAG_RACE: i32 = 1 << 10;
pub const GAMEINFOFLAG_UNLIMITED_AMMO: i32 = 1 << 11;
pub const GAMEINFOFLAG_DDRACE_RECORD_MESSAGE: i32 = 1 << 12;
pub const GAMEINFOFLAG_RACE_RECORD_MESSAGE: i32 = 1 << 13;
pub const GAMEINFOFLAG_ALLOW_EYE_WHEEL: i32 = 1 << 14;
pub const GAMEINFOFLAG_ALLOW_HOOK_COLL: i32 = 1 << 15;
pub const GAMEINFOFLAG_ALLOW_ZOOM: i32 = 1 << 16;
pub const GAMEINFOFLAG_BUG_DDRACE_GHOST: i32 = 1 << 17;
pub const GAMEINFOFLAG_BUG_DDRACE_INPUT: i32 = 1 << 18;
pub const GAMEINFOFLAG_BUG_FNG_LASER_RANGE: i32 = 1 << 19;
pub const GAMEINFOFLAG_BUG_VANILLA_BOUNCE: i32 = 1 << 20;
pub const GAMEINFOFLAG_PREDICT_FNG: i32 = 1 << 21;
pub const GAMEINFOFLAG_PREDICT_DDRACE: i32 = 1 << 22;
pub const GAMEINFOFLAG_PREDICT_DDRACE_TILES: i32 = 1 << 23;
pub const GAMEINFOFLAG_PREDICT_VANILLA: i32 = 1 << 24;
pub const GAMEINFOFLAG_ENTITIES_DDNET: i32 = 1 << 25;
pub const GAMEINFOFLAG_ENTITIES_DDRACE: i32 = 1 << 26;
pub const GAMEINFOFLAG_ENTITIES_RACE: i32 = 1 << 27;
pub const GAMEINFOFLAG_ENTITIES_FNG: i32 = 1 << 28;
pub const GAMEINFOFLAG_ENTITIES_VANILLA: i32 = 1 << 29;
pub const GAMEINFOFLAG_DONT_MASK_ENTITIES: i32 = 1 << 30;
pub const GAMEINFOFLAG_ENTITIES_BW: i32 = 1 << 31;

pub const GAMEINFOFLAG2_ALLOW_X_SKINS: i32 = 1 << 0;
pub const GAMEINFOFLAG2_GAMETYPE_CITY: i32 = 1 << 1;
pub const GAMEINFOFLAG2_GAMETYPE_FDDRACE: i32 = 1 << 2;
pub const GAMEINFOFLAG2_ENTITIES_FDDRACE: i32 = 1 << 3;
pub const GAMEINFOFLAG2_HUD_HEALTH_ARMOR: i32 = 1 << 4;
pub const GAMEINFOFLAG2_HUD_AMMO: i32 = 1 << 5;
pub const GAMEINFOFLAG2_HUD_DDRACE: i32 = 1 << 6;
pub const GAMEINFOFLAG2_NO_WEAK_HOOK: i32 = 1 << 7;
pub const GAMEINFOFLAG2_NO_SKIN_CHANGE_FOR_FROZEN: i32 = 1 << 8;
pub const GAMEINFOFLAG2_DDRACE_TEAM: i32 = 1 << 9;

pub const EXPLAYERFLAG_AFK: i32 = 1 << 0;
pub const EXPLAYERFLAG_PAUSED: i32 = 1 << 1;
pub const EXPLAYERFLAG_SPEC: i32 = 1 << 2;

pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT0: i32 = 1 << 0;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT1: i32 = 1 << 1;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT2: i32 = 1 << 2;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT3: i32 = 1 << 3;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT4: i32 = 1 << 4;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT5: i32 = 1 << 5;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT6: i32 = 1 << 6;
pub const LEGACYPROJECTILEFLAG_CLIENTID_BIT7: i32 = 1 << 7;
pub const LEGACYPROJECTILEFLAG_NO_OWNER: i32 = 1 << 8;
pub const LEGACYPROJECTILEFLAG_IS_DDNET: i32 = 1 << 9;
pub const LEGACYPROJECTILEFLAG_BOUNCE_HORIZONTAL: i32 = 1 << 10;
pub const LEGACYPROJECTILEFLAG_BOUNCE_VERTICAL: i32 = 1 << 11;
pub const LEGACYPROJECTILEFLAG_EXPLOSIVE: i32 = 1 << 12;
pub const LEGACYPROJECTILEFLAG_FREEZE: i32 = 1 << 13;

pub const PROJECTILEFLAG_BOUNCE_HORIZONTAL: i32 = 1 << 0;
pub const PROJECTILEFLAG_BOUNCE_VERTICAL: i32 = 1 << 1;
pub const PROJECTILEFLAG_EXPLOSIVE: i32 = 1 << 2;
pub const PROJECTILEFLAG_FREEZE: i32 = 1 << 3;
pub const PROJECTILEFLAG_NORMALIZE_VEL: i32 = 1 << 4;

pub const LASERFLAG_NO_PREDICT: i32 = 1 << 0;

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
pub const MY_OWN_OBJECT: Uuid = Uuid::from_u128(0x0dc77a02_bfee_3a53_ac8e_0bb0241bd722);
pub const DDNET_CHARACTER: Uuid = Uuid::from_u128(0x76ce455b_f9eb_3a48_add7_e04b941d045c);
pub const DDNET_PLAYER: Uuid = Uuid::from_u128(0x22ca938d_1380_3e2b_9e7b_d2558ea6be11);
pub const GAME_INFO_EX: Uuid = Uuid::from_u128(0x933dea6a_da79_30ea_a98f_8af03689a945);
pub const DDRACE_PROJECTILE: Uuid = Uuid::from_u128(0x0e6db85c_2b61_386f_bbf2_d0d0471b9272);
pub const DDNET_LASER: Uuid = Uuid::from_u128(0x29de68a2_6928_31b8_8360_a2307e0d844f);
pub const DDNET_PROJECTILE: Uuid = Uuid::from_u128(0x6550fbce_f317_3b31_8ffe_d2b37f3ab40e);
pub const DDNET_PICKUP: Uuid = Uuid::from_u128(0xea5e4a51_58fb_3684_96e4_e0d267f4ca65);
pub const DDNET_SPECTATOR_INFO: Uuid = Uuid::from_u128(0xd13307b2_9a19_37cb_8f8c_07c718521883);
pub const COMMON: u16 = 13;
pub const EXPLOSION: u16 = 14;
pub const SPAWN: u16 = 15;
pub const HAMMER_HIT: u16 = 16;
pub const DEATH: u16 = 17;
pub const SOUND_GLOBAL: u16 = 18;
pub const SOUND_WORLD: u16 = 19;
pub const DAMAGE_IND: u16 = 20;
pub const BIRTHDAY: Uuid = Uuid::from_u128(0x1fd35746_6263_358c_b4d6_6ef60e0efaaa);
pub const FINISH: Uuid = Uuid::from_u128(0x68bf8939_ef55_3878_9082_13527eb0a597);
pub const MY_OWN_EVENT: Uuid = Uuid::from_u128(0x0c4fd27d_47e3_3871_a226_9f417486a311);
pub const SPEC_CHAR: Uuid = Uuid::from_u128(0x4b801c74_e24c_3ce0_b92c_b754d02cfc8a);
pub const SWITCH_STATE: Uuid = Uuid::from_u128(0xec15e669_ce11_3367_ae8e_b90e5b27b9d5);
pub const ENTITY_EX: Uuid = Uuid::from_u128(0x2de9aec3_32e4_3986_8f7e_e7459da7f535);
pub const MAP_SOUND_WORLD: Uuid = Uuid::from_u128(0x54ecad2e_bfad_3be5_8903_621ba052458e);

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
    MyOwnObject(MyOwnObject),
    DdnetCharacter(DdnetCharacter),
    DdnetPlayer(DdnetPlayer),
    GameInfoEx(GameInfoEx),
    DdraceProjectile(DdraceProjectile),
    DdnetLaser(DdnetLaser),
    DdnetProjectile(DdnetProjectile),
    DdnetPickup(DdnetPickup),
    DdnetSpectatorInfo(DdnetSpectatorInfo),
    Common(Common),
    Explosion(Explosion),
    Spawn(Spawn),
    HammerHit(HammerHit),
    Death(Death),
    SoundGlobal(SoundGlobal),
    SoundWorld(SoundWorld),
    DamageInd(DamageInd),
    Birthday(Birthday),
    Finish(Finish),
    MyOwnEvent(MyOwnEvent),
    SpecChar(SpecChar),
    SwitchState(SwitchState),
    EntityEx(EntityEx),
    MapSoundWorld(MapSoundWorld),
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
            Ordinal(GAME_INFO) => SnapObj::GameInfo(GameInfo::decode(warn, _p)?),
            Ordinal(GAME_DATA) => SnapObj::GameData(GameData::decode(warn, _p)?),
            Ordinal(CHARACTER_CORE) => SnapObj::CharacterCore(CharacterCore::decode(warn, _p)?),
            Ordinal(CHARACTER) => SnapObj::Character(Character::decode(warn, _p)?),
            Ordinal(PLAYER_INFO) => SnapObj::PlayerInfo(PlayerInfo::decode(warn, _p)?),
            Ordinal(CLIENT_INFO) => SnapObj::ClientInfo(ClientInfo::decode(warn, _p)?),
            Ordinal(SPECTATOR_INFO) => SnapObj::SpectatorInfo(SpectatorInfo::decode(warn, _p)?),
            Uuid(MY_OWN_OBJECT) => SnapObj::MyOwnObject(MyOwnObject::decode(warn, _p)?),
            Uuid(DDNET_CHARACTER) => SnapObj::DdnetCharacter(DdnetCharacter::decode(warn, _p)?),
            Uuid(DDNET_PLAYER) => SnapObj::DdnetPlayer(DdnetPlayer::decode(warn, _p)?),
            Uuid(GAME_INFO_EX) => SnapObj::GameInfoEx(GameInfoEx::decode(warn, _p)?),
            Uuid(DDRACE_PROJECTILE) => SnapObj::DdraceProjectile(DdraceProjectile::decode(warn, _p)?),
            Uuid(DDNET_LASER) => SnapObj::DdnetLaser(DdnetLaser::decode(warn, _p)?),
            Uuid(DDNET_PROJECTILE) => SnapObj::DdnetProjectile(DdnetProjectile::decode(warn, _p)?),
            Uuid(DDNET_PICKUP) => SnapObj::DdnetPickup(DdnetPickup::decode(warn, _p)?),
            Uuid(DDNET_SPECTATOR_INFO) => SnapObj::DdnetSpectatorInfo(DdnetSpectatorInfo::decode(warn, _p)?),
            Ordinal(COMMON) => SnapObj::Common(Common::decode(warn, _p)?),
            Ordinal(EXPLOSION) => SnapObj::Explosion(Explosion::decode(warn, _p)?),
            Ordinal(SPAWN) => SnapObj::Spawn(Spawn::decode(warn, _p)?),
            Ordinal(HAMMER_HIT) => SnapObj::HammerHit(HammerHit::decode(warn, _p)?),
            Ordinal(DEATH) => SnapObj::Death(Death::decode(warn, _p)?),
            Ordinal(SOUND_GLOBAL) => SnapObj::SoundGlobal(SoundGlobal::decode(warn, _p)?),
            Ordinal(SOUND_WORLD) => SnapObj::SoundWorld(SoundWorld::decode(warn, _p)?),
            Ordinal(DAMAGE_IND) => SnapObj::DamageInd(DamageInd::decode(warn, _p)?),
            Uuid(BIRTHDAY) => SnapObj::Birthday(Birthday::decode(warn, _p)?),
            Uuid(FINISH) => SnapObj::Finish(Finish::decode(warn, _p)?),
            Uuid(MY_OWN_EVENT) => SnapObj::MyOwnEvent(MyOwnEvent::decode(warn, _p)?),
            Uuid(SPEC_CHAR) => SnapObj::SpecChar(SpecChar::decode(warn, _p)?),
            Uuid(SWITCH_STATE) => SnapObj::SwitchState(SwitchState::decode(warn, _p)?),
            Uuid(ENTITY_EX) => SnapObj::EntityEx(EntityEx::decode(warn, _p)?),
            Uuid(MAP_SOUND_WORLD) => SnapObj::MapSoundWorld(MapSoundWorld::decode(warn, _p)?),
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
            SnapObj::GameInfo(_) => TypeId::from(GAME_INFO),
            SnapObj::GameData(_) => TypeId::from(GAME_DATA),
            SnapObj::CharacterCore(_) => TypeId::from(CHARACTER_CORE),
            SnapObj::Character(_) => TypeId::from(CHARACTER),
            SnapObj::PlayerInfo(_) => TypeId::from(PLAYER_INFO),
            SnapObj::ClientInfo(_) => TypeId::from(CLIENT_INFO),
            SnapObj::SpectatorInfo(_) => TypeId::from(SPECTATOR_INFO),
            SnapObj::MyOwnObject(_) => TypeId::from(MY_OWN_OBJECT),
            SnapObj::DdnetCharacter(_) => TypeId::from(DDNET_CHARACTER),
            SnapObj::DdnetPlayer(_) => TypeId::from(DDNET_PLAYER),
            SnapObj::GameInfoEx(_) => TypeId::from(GAME_INFO_EX),
            SnapObj::DdraceProjectile(_) => TypeId::from(DDRACE_PROJECTILE),
            SnapObj::DdnetLaser(_) => TypeId::from(DDNET_LASER),
            SnapObj::DdnetProjectile(_) => TypeId::from(DDNET_PROJECTILE),
            SnapObj::DdnetPickup(_) => TypeId::from(DDNET_PICKUP),
            SnapObj::DdnetSpectatorInfo(_) => TypeId::from(DDNET_SPECTATOR_INFO),
            SnapObj::Common(_) => TypeId::from(COMMON),
            SnapObj::Explosion(_) => TypeId::from(EXPLOSION),
            SnapObj::Spawn(_) => TypeId::from(SPAWN),
            SnapObj::HammerHit(_) => TypeId::from(HAMMER_HIT),
            SnapObj::Death(_) => TypeId::from(DEATH),
            SnapObj::SoundGlobal(_) => TypeId::from(SOUND_GLOBAL),
            SnapObj::SoundWorld(_) => TypeId::from(SOUND_WORLD),
            SnapObj::DamageInd(_) => TypeId::from(DAMAGE_IND),
            SnapObj::Birthday(_) => TypeId::from(BIRTHDAY),
            SnapObj::Finish(_) => TypeId::from(FINISH),
            SnapObj::MyOwnEvent(_) => TypeId::from(MY_OWN_EVENT),
            SnapObj::SpecChar(_) => TypeId::from(SPEC_CHAR),
            SnapObj::SwitchState(_) => TypeId::from(SWITCH_STATE),
            SnapObj::EntityEx(_) => TypeId::from(ENTITY_EX),
            SnapObj::MapSoundWorld(_) => TypeId::from(MAP_SOUND_WORLD),
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
            SnapObj::MyOwnObject(ref i) => i.encode(),
            SnapObj::DdnetCharacter(ref i) => i.encode(),
            SnapObj::DdnetPlayer(ref i) => i.encode(),
            SnapObj::GameInfoEx(ref i) => i.encode(),
            SnapObj::DdraceProjectile(ref i) => i.encode(),
            SnapObj::DdnetLaser(ref i) => i.encode(),
            SnapObj::DdnetProjectile(ref i) => i.encode(),
            SnapObj::DdnetPickup(ref i) => i.encode(),
            SnapObj::DdnetSpectatorInfo(ref i) => i.encode(),
            SnapObj::Common(ref i) => i.encode(),
            SnapObj::Explosion(ref i) => i.encode(),
            SnapObj::Spawn(ref i) => i.encode(),
            SnapObj::HammerHit(ref i) => i.encode(),
            SnapObj::Death(ref i) => i.encode(),
            SnapObj::SoundGlobal(ref i) => i.encode(),
            SnapObj::SoundWorld(ref i) => i.encode(),
            SnapObj::DamageInd(ref i) => i.encode(),
            SnapObj::Birthday(ref i) => i.encode(),
            SnapObj::Finish(ref i) => i.encode(),
            SnapObj::MyOwnEvent(ref i) => i.encode(),
            SnapObj::SpecChar(ref i) => i.encode(),
            SnapObj::SwitchState(ref i) => i.encode(),
            SnapObj::EntityEx(ref i) => i.encode(),
            SnapObj::MapSoundWorld(ref i) => i.encode(),
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
            SnapObj::MyOwnObject(ref i) => i.fmt(f),
            SnapObj::DdnetCharacter(ref i) => i.fmt(f),
            SnapObj::DdnetPlayer(ref i) => i.fmt(f),
            SnapObj::GameInfoEx(ref i) => i.fmt(f),
            SnapObj::DdraceProjectile(ref i) => i.fmt(f),
            SnapObj::DdnetLaser(ref i) => i.fmt(f),
            SnapObj::DdnetProjectile(ref i) => i.fmt(f),
            SnapObj::DdnetPickup(ref i) => i.fmt(f),
            SnapObj::DdnetSpectatorInfo(ref i) => i.fmt(f),
            SnapObj::Common(ref i) => i.fmt(f),
            SnapObj::Explosion(ref i) => i.fmt(f),
            SnapObj::Spawn(ref i) => i.fmt(f),
            SnapObj::HammerHit(ref i) => i.fmt(f),
            SnapObj::Death(ref i) => i.fmt(f),
            SnapObj::SoundGlobal(ref i) => i.fmt(f),
            SnapObj::SoundWorld(ref i) => i.fmt(f),
            SnapObj::DamageInd(ref i) => i.fmt(f),
            SnapObj::Birthday(ref i) => i.fmt(f),
            SnapObj::Finish(ref i) => i.fmt(f),
            SnapObj::MyOwnEvent(ref i) => i.fmt(f),
            SnapObj::SpecChar(ref i) => i.fmt(f),
            SnapObj::SwitchState(ref i) => i.fmt(f),
            SnapObj::EntityEx(ref i) => i.fmt(f),
            SnapObj::MapSoundWorld(ref i) => i.fmt(f),
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

impl From<MyOwnObject> for SnapObj {
    fn from(i: MyOwnObject) -> SnapObj {
        SnapObj::MyOwnObject(i)
    }
}

impl From<DdnetCharacter> for SnapObj {
    fn from(i: DdnetCharacter) -> SnapObj {
        SnapObj::DdnetCharacter(i)
    }
}

impl From<DdnetPlayer> for SnapObj {
    fn from(i: DdnetPlayer) -> SnapObj {
        SnapObj::DdnetPlayer(i)
    }
}

impl From<GameInfoEx> for SnapObj {
    fn from(i: GameInfoEx) -> SnapObj {
        SnapObj::GameInfoEx(i)
    }
}

impl From<DdraceProjectile> for SnapObj {
    fn from(i: DdraceProjectile) -> SnapObj {
        SnapObj::DdraceProjectile(i)
    }
}

impl From<DdnetLaser> for SnapObj {
    fn from(i: DdnetLaser) -> SnapObj {
        SnapObj::DdnetLaser(i)
    }
}

impl From<DdnetProjectile> for SnapObj {
    fn from(i: DdnetProjectile) -> SnapObj {
        SnapObj::DdnetProjectile(i)
    }
}

impl From<DdnetPickup> for SnapObj {
    fn from(i: DdnetPickup) -> SnapObj {
        SnapObj::DdnetPickup(i)
    }
}

impl From<DdnetSpectatorInfo> for SnapObj {
    fn from(i: DdnetSpectatorInfo) -> SnapObj {
        SnapObj::DdnetSpectatorInfo(i)
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

impl From<Birthday> for SnapObj {
    fn from(i: Birthday) -> SnapObj {
        SnapObj::Birthday(i)
    }
}

impl From<Finish> for SnapObj {
    fn from(i: Finish) -> SnapObj {
        SnapObj::Finish(i)
    }
}

impl From<MyOwnEvent> for SnapObj {
    fn from(i: MyOwnEvent) -> SnapObj {
        SnapObj::MyOwnEvent(i)
    }
}

impl From<SpecChar> for SnapObj {
    fn from(i: SpecChar) -> SnapObj {
        SnapObj::SpecChar(i)
    }
}

impl From<SwitchState> for SnapObj {
    fn from(i: SwitchState) -> SnapObj {
        SnapObj::SwitchState(i)
    }
}

impl From<EntityEx> for SnapObj {
    fn from(i: EntityEx) -> SnapObj {
        SnapObj::EntityEx(i)
    }
}

impl From<MapSoundWorld> for SnapObj {
    fn from(i: MapSoundWorld) -> SnapObj {
        SnapObj::MapSoundWorld(i)
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
    pub type_: enums::Weapon,
    pub start_tick: crate::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Laser {
    pub x: i32,
    pub y: i32,
    pub from_x: i32,
    pub from_y: i32,
    pub start_tick: crate::snap_obj::Tick,
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
    pub round_start_tick: crate::snap_obj::Tick,
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
    pub hook_tick: i32,
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
    pub weapon: i32,
    pub emote: enums::Emote,
    pub attack_tick: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PlayerInfo {
    pub local: i32,
    pub client_id: i32,
    pub team: enums::Team,
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
pub struct MyOwnObject {
    pub test: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdnetCharacter {
    pub flags: i32,
    pub freeze_end: crate::snap_obj::Tick,
    pub jumps: i32,
    pub tele_checkpoint: i32,
    pub strong_weak_id: i32,
    pub jumped_total: i32,
    pub ninja_activation_tick: crate::snap_obj::Tick,
    pub freeze_start: crate::snap_obj::Tick,
    pub target_x: i32,
    pub target_y: i32,
    pub tune_zone_override: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdnetPlayer {
    pub flags: i32,
    pub auth_level: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GameInfoEx {
    pub flags: i32,
    pub version: i32,
    pub flags2: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdraceProjectile {
    pub x: i32,
    pub y: i32,
    pub angle: i32,
    pub data: i32,
    pub type_: enums::Weapon,
    pub start_tick: crate::snap_obj::Tick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdnetLaser {
    pub to_x: i32,
    pub to_y: i32,
    pub from_x: i32,
    pub from_y: i32,
    pub start_tick: crate::snap_obj::Tick,
    pub owner: i32,
    pub type_: i32,
    pub switch_number: i32,
    pub subtype: i32,
    pub flags: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdnetProjectile {
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub type_: enums::Weapon,
    pub start_tick: crate::snap_obj::Tick,
    pub owner: i32,
    pub switch_number: i32,
    pub tune_zone: i32,
    pub flags: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdnetPickup {
    pub x: i32,
    pub y: i32,
    pub type_: i32,
    pub subtype: i32,
    pub switch_number: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DdnetSpectatorInfo {
    pub has_camera_info: bool,
    pub zoom: i32,
    pub deadzone: i32,
    pub follow_factor: i32,
    pub spectator_count: i32,
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
    pub sound_id: enums::Sound,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SoundWorld {
    pub common: Common,
    pub sound_id: enums::Sound,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DamageInd {
    pub common: Common,
    pub angle: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Birthday {
    pub common: Common,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Finish {
    pub common: Common,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MyOwnEvent {
    pub test: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SpecChar {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SwitchState {
    pub highest_switch_number: i32,
    pub status: [i32; 8],
    pub switch_numbers: [i32; 4],
    pub end_ticks: [i32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EntityEx {
    pub switch_number: i32,
    pub layer: i32,
    pub entity_class: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MapSoundWorld {
    pub common: Common,
    pub sound_id: i32,
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
        unsafe { slice::transmute(from_ref(self)) }
    }
}
impl PlayerInput {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PlayerInput, Error> {
        let result = Ok(PlayerInput {
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
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode_msg<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
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
            start_tick: crate::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
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
            start_tick: crate::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
            round_start_tick: crate::snap_obj::Tick(_p.read_int()?),
            warmup_timer: _p.read_int()?,
            score_limit: positive(_p.read_int()?)?,
            time_limit: positive(_p.read_int()?)?,
            round_num: positive(_p.read_int()?)?,
            round_current: positive(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.game_flags && self.game_flags <= 256);
        assert!(0 <= self.game_state_flags && self.game_state_flags <= 256);
        assert!(self.score_limit >= 0);
        assert!(self.time_limit >= 0);
        assert!(self.round_num >= 0);
        assert!(self.round_current >= 0);
        unsafe { slice::transmute(from_ref(self)) }
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
            flag_carrier_red: in_range(_p.read_int()?, -3, 127)?,
            flag_carrier_blue: in_range(_p.read_int()?, -3, 127)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-3 <= self.flag_carrier_red && self.flag_carrier_red <= 127);
        assert!(-3 <= self.flag_carrier_blue && self.flag_carrier_blue <= 127);
        unsafe { slice::transmute(from_ref(self)) }
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
            hooked_player: in_range(_p.read_int()?, -1, 127)?,
            hook_state: in_range(_p.read_int()?, -1, 5)?,
            hook_tick: _p.read_int()?,
            hook_x: _p.read_int()?,
            hook_y: _p.read_int()?,
            hook_dx: _p.read_int()?,
            hook_dy: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.direction && self.direction <= 1);
        assert!(0 <= self.jumped && self.jumped <= 3);
        assert!(-1 <= self.hooked_player && self.hooked_player <= 127);
        assert!(-1 <= self.hook_state && self.hook_state <= 5);
        unsafe { slice::transmute(from_ref(self)) }
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
            ammo_count: in_range(_p.read_int()?, -1, 10)?,
            weapon: in_range(_p.read_int()?, -1, 5)?,
            emote: enums::Emote::from_i32(_p.read_int()?)?,
            attack_tick: positive(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.character_core.encode();
        assert!(0 <= self.player_flags && self.player_flags <= 256);
        assert!(0 <= self.health && self.health <= 10);
        assert!(0 <= self.armor && self.armor <= 10);
        assert!(-1 <= self.ammo_count && self.ammo_count <= 10);
        assert!(-1 <= self.weapon && self.weapon <= 5);
        assert!(self.attack_tick >= 0);
        unsafe { slice::transmute(from_ref(self)) }
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
            client_id: in_range(_p.read_int()?, 0, 127)?,
            team: enums::Team::from_i32(_p.read_int()?)?,
            score: _p.read_int()?,
            latency: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.local && self.local <= 1);
        assert!(0 <= self.client_id && self.client_id <= 127);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for ClientInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClientInfo")
            .field("name", &self.name)
            .field("clan", &self.clan)
            .field("country", &self.country)
            .field("skin", &self.skin)
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
        unsafe { slice::transmute(from_ref(self)) }
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
            spectator_id: in_range(_p.read_int()?, -1, 127)?,
            x: _p.read_int()?,
            y: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.spectator_id && self.spectator_id <= 127);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for MyOwnObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MyOwnObject")
            .field("test", &self.test)
            .finish()
    }
}
impl MyOwnObject {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<MyOwnObject, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<MyOwnObject, Error> {
        Ok(MyOwnObject {
            test: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdnetCharacter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdnetCharacter")
            .field("flags", &self.flags)
            .field("freeze_end", &self.freeze_end)
            .field("jumps", &self.jumps)
            .field("tele_checkpoint", &self.tele_checkpoint)
            .field("strong_weak_id", &self.strong_weak_id)
            .field("jumped_total", &self.jumped_total)
            .field("ninja_activation_tick", &self.ninja_activation_tick)
            .field("freeze_start", &self.freeze_start)
            .field("target_x", &self.target_x)
            .field("target_y", &self.target_y)
            .field("tune_zone_override", &self.tune_zone_override)
            .finish()
    }
}
impl DdnetCharacter {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdnetCharacter, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdnetCharacter, Error> {
        Ok(DdnetCharacter {
            flags: _p.read_int()?,
            freeze_end: crate::snap_obj::Tick(_p.read_int()?),
            jumps: in_range(_p.read_int()?, -1, 255)?,
            tele_checkpoint: _p.read_int()?,
            strong_weak_id: in_range(_p.read_int()?, 0, 127)?,
            jumped_total: in_range(_p.read_int()?, -1, 255)?,
            ninja_activation_tick: crate::snap_obj::Tick(_p.read_int()?),
            freeze_start: crate::snap_obj::Tick(_p.read_int()?),
            target_x: _p.read_int()?,
            target_y: _p.read_int()?,
            tune_zone_override: in_range(_p.read_int()?, -1, 255)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.jumps && self.jumps <= 255);
        assert!(0 <= self.strong_weak_id && self.strong_weak_id <= 127);
        assert!(-1 <= self.jumped_total && self.jumped_total <= 255);
        assert!(-1 <= self.tune_zone_override && self.tune_zone_override <= 255);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdnetPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdnetPlayer")
            .field("flags", &self.flags)
            .field("auth_level", &self.auth_level)
            .finish()
    }
}
impl DdnetPlayer {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdnetPlayer, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdnetPlayer, Error> {
        Ok(DdnetPlayer {
            flags: _p.read_int()?,
            auth_level: in_range(_p.read_int()?, 0, 3)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(0 <= self.auth_level && self.auth_level <= 3);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for GameInfoEx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GameInfoEx")
            .field("flags", &self.flags)
            .field("version", &self.version)
            .field("flags2", &self.flags2)
            .finish()
    }
}
impl GameInfoEx {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<GameInfoEx, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<GameInfoEx, Error> {
        Ok(GameInfoEx {
            flags: _p.read_int()?,
            version: _p.read_int()?,
            flags2: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdraceProjectile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdraceProjectile")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("angle", &self.angle)
            .field("data", &self.data)
            .field("type_", &self.type_)
            .field("start_tick", &self.start_tick)
            .finish()
    }
}
impl DdraceProjectile {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdraceProjectile, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdraceProjectile, Error> {
        Ok(DdraceProjectile {
            x: _p.read_int()?,
            y: _p.read_int()?,
            angle: _p.read_int()?,
            data: _p.read_int()?,
            type_: enums::Weapon::from_i32(_p.read_int()?)?,
            start_tick: crate::snap_obj::Tick(_p.read_int()?),
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdnetLaser {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdnetLaser")
            .field("to_x", &self.to_x)
            .field("to_y", &self.to_y)
            .field("from_x", &self.from_x)
            .field("from_y", &self.from_y)
            .field("start_tick", &self.start_tick)
            .field("owner", &self.owner)
            .field("type_", &self.type_)
            .field("switch_number", &self.switch_number)
            .field("subtype", &self.subtype)
            .field("flags", &self.flags)
            .finish()
    }
}
impl DdnetLaser {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdnetLaser, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdnetLaser, Error> {
        Ok(DdnetLaser {
            to_x: _p.read_int()?,
            to_y: _p.read_int()?,
            from_x: _p.read_int()?,
            from_y: _p.read_int()?,
            start_tick: crate::snap_obj::Tick(_p.read_int()?),
            owner: in_range(_p.read_int()?, -1, 127)?,
            type_: _p.read_int()?,
            switch_number: _p.read_int()?,
            subtype: _p.read_int()?,
            flags: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.owner && self.owner <= 127);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdnetProjectile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdnetProjectile")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("vel_x", &self.vel_x)
            .field("vel_y", &self.vel_y)
            .field("type_", &self.type_)
            .field("start_tick", &self.start_tick)
            .field("owner", &self.owner)
            .field("switch_number", &self.switch_number)
            .field("tune_zone", &self.tune_zone)
            .field("flags", &self.flags)
            .finish()
    }
}
impl DdnetProjectile {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdnetProjectile, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdnetProjectile, Error> {
        Ok(DdnetProjectile {
            x: _p.read_int()?,
            y: _p.read_int()?,
            vel_x: _p.read_int()?,
            vel_y: _p.read_int()?,
            type_: enums::Weapon::from_i32(_p.read_int()?)?,
            start_tick: crate::snap_obj::Tick(_p.read_int()?),
            owner: in_range(_p.read_int()?, -1, 127)?,
            switch_number: _p.read_int()?,
            tune_zone: _p.read_int()?,
            flags: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(-1 <= self.owner && self.owner <= 127);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdnetPickup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdnetPickup")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("type_", &self.type_)
            .field("subtype", &self.subtype)
            .field("switch_number", &self.switch_number)
            .finish()
    }
}
impl DdnetPickup {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdnetPickup, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdnetPickup, Error> {
        Ok(DdnetPickup {
            x: _p.read_int()?,
            y: _p.read_int()?,
            type_: positive(_p.read_int()?)?,
            subtype: positive(_p.read_int()?)?,
            switch_number: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(self.type_ >= 0);
        assert!(self.subtype >= 0);
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for DdnetSpectatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DdnetSpectatorInfo")
            .field("has_camera_info", &self.has_camera_info)
            .field("zoom", &self.zoom)
            .field("deadzone", &self.deadzone)
            .field("follow_factor", &self.follow_factor)
            .field("spectator_count", &self.spectator_count)
            .finish()
    }
}
impl DdnetSpectatorInfo {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<DdnetSpectatorInfo, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<DdnetSpectatorInfo, Error> {
        Ok(DdnetSpectatorInfo {
            has_camera_info: to_bool(_p.read_int()?)?,
            zoom: positive(_p.read_int()?)?,
            deadzone: positive(_p.read_int()?)?,
            follow_factor: positive(_p.read_int()?)?,
            spectator_count: in_range(_p.read_int()?, 0, 127)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        assert!(self.zoom >= 0);
        assert!(self.deadzone >= 0);
        assert!(self.follow_factor >= 0);
        assert!(0 <= self.spectator_count && self.spectator_count <= 127);
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
            client_id: in_range(_p.read_int()?, 0, 127)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        assert!(0 <= self.client_id && self.client_id <= 127);
        unsafe { slice::transmute(from_ref(self)) }
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
            sound_id: enums::Sound::from_i32(_p.read_int()?)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
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
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for Birthday {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Birthday")
            .field("common", &self.common)
            .finish()
    }
}
impl Birthday {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Birthday, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Birthday, Error> {
        Ok(Birthday {
            common: Common::decode_inner(_p)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for Finish {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Finish")
            .field("common", &self.common)
            .finish()
    }
}
impl Finish {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<Finish, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<Finish, Error> {
        Ok(Finish {
            common: Common::decode_inner(_p)?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for MyOwnEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MyOwnEvent")
            .field("test", &self.test)
            .finish()
    }
}
impl MyOwnEvent {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<MyOwnEvent, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<MyOwnEvent, Error> {
        Ok(MyOwnEvent {
            test: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for SpecChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpecChar")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}
impl SpecChar {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<SpecChar, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SpecChar, Error> {
        Ok(SpecChar {
            x: _p.read_int()?,
            y: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for SwitchState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SwitchState")
            .field("highest_switch_number", &self.highest_switch_number)
            .field("status", &self.status)
            .field("switch_numbers", &self.switch_numbers)
            .field("end_ticks", &self.end_ticks)
            .finish()
    }
}
impl SwitchState {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<SwitchState, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<SwitchState, Error> {
        Ok(SwitchState {
            highest_switch_number: _p.read_int()?,
            status: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
            switch_numbers: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
            end_ticks: [
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
                _p.read_int()?,
            ],
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for EntityEx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EntityEx")
            .field("switch_number", &self.switch_number)
            .field("layer", &self.layer)
            .field("entity_class", &self.entity_class)
            .finish()
    }
}
impl EntityEx {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<EntityEx, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<EntityEx, Error> {
        Ok(EntityEx {
            switch_number: _p.read_int()?,
            layer: _p.read_int()?,
            entity_class: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        unsafe { slice::transmute(from_ref(self)) }
    }
}

impl fmt::Debug for MapSoundWorld {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapSoundWorld")
            .field("common", &self.common)
            .field("sound_id", &self.sound_id)
            .finish()
    }
}
impl MapSoundWorld {
    pub fn decode<W: Warn<ExcessData>>(warn: &mut W, p: &mut IntUnpacker) -> Result<MapSoundWorld, Error> {
        let result = Self::decode_inner(p)?;
        p.finish(warn);
        Ok(result)
    }
    pub fn decode_inner(_p: &mut IntUnpacker) -> Result<MapSoundWorld, Error> {
        Ok(MapSoundWorld {
            common: Common::decode_inner(_p)?,
            sound_id: _p.read_int()?,
        })
    }
    pub fn encode(&self) -> &[i32] {
        self.common.encode();
        unsafe { slice::transmute(from_ref(self)) }
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
