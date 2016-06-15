use buffer::CapacityError;
use error::ControlCharacters;
use error::Error;
use error::IntOutOfRange;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::with_packer;
use super::SystemOrGame;
use warn::Panic;
use warn::Warn;

fn in_range(v: i32, min: i32, max: i32) -> Result<i32, IntOutOfRange> {
    if min <= v && v <= max {
        Ok(v)
    } else {
        Err(IntOutOfRange)
    }
}

fn to_bool(v: i32) -> Result<bool, IntOutOfRange> {
    Ok(try!(in_range(v, 0, 1)) != 0)
}

fn sanitize<'a, W: Warn<Warning>>(warn: &mut W, v: &'a [u8])
    -> Result<&'a [u8], ControlCharacters>
{
    if v.iter().any(|&b| b < b' ') {
        return Err(ControlCharacters);
    }
    let _ = warn;
    // TODO: Implement whitespace skipping.
    Ok(v)
}

impl<'a> Game<'a> {
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::Game(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode_msg(p)));
        Ok(p.written())
    }
}

pub const MAX_CLIENTS: i32 = 16;
pub const SPEC_FREEVIEW: i32 = -1;

pub const EMOTE_NORMAL: i32 = 0;
pub const EMOTE_PAIN: i32 = 1;
pub const EMOTE_HAPPY: i32 = 2;
pub const EMOTE_SURPRISE: i32 = 3;
pub const EMOTE_ANGRY: i32 = 4;
pub const EMOTE_BLINK: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Emote {
    Normal,
    Pain,
    Happy,
    Surprise,
    Angry,
    Blink,
}

pub const POWERUP_HEALTH: i32 = 0;
pub const POWERUP_ARMOR: i32 = 1;
pub const POWERUP_WEAPON: i32 = 2;
pub const POWERUP_NINJA: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Powerup {
    Health,
    Armor,
    Weapon,
    Ninja,
}

pub const EMOTICON_OOP: i32 = 0;
pub const EMOTICON_EXCLAMATION: i32 = 1;
pub const EMOTICON_HEARTS: i32 = 2;
pub const EMOTICON_DROP: i32 = 3;
pub const EMOTICON_DOTDOT: i32 = 4;
pub const EMOTICON_MUSIC: i32 = 5;
pub const EMOTICON_SORRY: i32 = 6;
pub const EMOTICON_GHOST: i32 = 7;
pub const EMOTICON_SUSHI: i32 = 8;
pub const EMOTICON_SPLATTEE: i32 = 9;
pub const EMOTICON_DEVILTEE: i32 = 10;
pub const EMOTICON_ZOMG: i32 = 11;
pub const EMOTICON_ZZZ: i32 = 12;
pub const EMOTICON_WTF: i32 = 13;
pub const EMOTICON_EYES: i32 = 14;
pub const EMOTICON_QUESTION: i32 = 15;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Emoticon {
    Oop,
    Exclamation,
    Hearts,
    Drop,
    Dotdot,
    Music,
    Sorry,
    Ghost,
    Sushi,
    Splattee,
    Deviltee,
    Zomg,
    Zzz,
    Wtf,
    Eyes,
    Question,
}

pub const WEAPON_HAMMER: i32 = 0;
pub const WEAPON_PISTOL: i32 = 1;
pub const WEAPON_SHOTGUN: i32 = 2;
pub const WEAPON_GRENADE: i32 = 3;
pub const WEAPON_RIFLE: i32 = 4;
pub const WEAPON_NINJA: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Weapon {
    Hammer,
    Pistol,
    Shotgun,
    Grenade,
    Rifle,
    Ninja,
}

pub const TEAM_SPECTATORS: i32 = -1;
pub const TEAM_RED: i32 = 0;
pub const TEAM_BLUE: i32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Team {
    Spectators,
    Red,
    Blue,
}

pub const SOUND_GUN_FIRE: i32 = 0;
pub const SOUND_SHOTGUN_FIRE: i32 = 1;
pub const SOUND_GRENADE_FIRE: i32 = 2;
pub const SOUND_HAMMER_FIRE: i32 = 3;
pub const SOUND_HAMMER_HIT: i32 = 4;
pub const SOUND_NINJA_FIRE: i32 = 5;
pub const SOUND_GRENADE_EXPLODE: i32 = 6;
pub const SOUND_NINJA_HIT: i32 = 7;
pub const SOUND_RIFLE_FIRE: i32 = 8;
pub const SOUND_RIFLE_BOUNCE: i32 = 9;
pub const SOUND_WEAPON_SWITCH: i32 = 10;
pub const SOUND_PLAYER_PAIN_SHORT: i32 = 11;
pub const SOUND_PLAYER_PAIN_LONG: i32 = 12;
pub const SOUND_BODY_LAND: i32 = 13;
pub const SOUND_PLAYER_AIRJUMP: i32 = 14;
pub const SOUND_PLAYER_JUMP: i32 = 15;
pub const SOUND_PLAYER_DIE: i32 = 16;
pub const SOUND_PLAYER_SPAWN: i32 = 17;
pub const SOUND_PLAYER_SKID: i32 = 18;
pub const SOUND_TEE_CRY: i32 = 19;
pub const SOUND_HOOK_LOOP: i32 = 20;
pub const SOUND_HOOK_ATTACH_GROUND: i32 = 21;
pub const SOUND_HOOK_ATTACH_PLAYER: i32 = 22;
pub const SOUND_HOOK_NOATTACH: i32 = 23;
pub const SOUND_PICKUP_HEALTH: i32 = 24;
pub const SOUND_PICKUP_ARMOR: i32 = 25;
pub const SOUND_PICKUP_GRENADE: i32 = 26;
pub const SOUND_PICKUP_SHOTGUN: i32 = 27;
pub const SOUND_PICKUP_NINJA: i32 = 28;
pub const SOUND_WEAPON_SPAWN: i32 = 29;
pub const SOUND_WEAPON_NOAMMO: i32 = 30;
pub const SOUND_HIT: i32 = 31;
pub const SOUND_CHAT_SERVER: i32 = 32;
pub const SOUND_CHAT_CLIENT: i32 = 33;
pub const SOUND_CHAT_HIGHLIGHT: i32 = 34;
pub const SOUND_CTF_DROP: i32 = 35;
pub const SOUND_CTF_RETURN: i32 = 36;
pub const SOUND_CTF_GRAB_PL: i32 = 37;
pub const SOUND_CTF_GRAB_EN: i32 = 38;
pub const SOUND_CTF_CAPTURE: i32 = 39;
pub const SOUND_MENU: i32 = 40;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Sound {
    GunFire,
    ShotgunFire,
    GrenadeFire,
    HammerFire,
    HammerHit,
    NinjaFire,
    GrenadeExplode,
    NinjaHit,
    RifleFire,
    RifleBounce,
    WeaponSwitch,
    PlayerPainShort,
    PlayerPainLong,
    BodyLand,
    PlayerAirjump,
    PlayerJump,
    PlayerDie,
    PlayerSpawn,
    PlayerSkid,
    TeeCry,
    HookLoop,
    HookAttachGround,
    HookAttachPlayer,
    HookNoattach,
    PickupHealth,
    PickupArmor,
    PickupGrenade,
    PickupShotgun,
    PickupNinja,
    WeaponSpawn,
    WeaponNoammo,
    Hit,
    ChatServer,
    ChatClient,
    ChatHighlight,
    CtfDrop,
    CtfReturn,
    CtfGrabPl,
    CtfGrabEn,
    CtfCapture,
    Menu,
}

impl Emote {
    pub fn from_i32(i: i32) -> Result<Emote, IntOutOfRange> {
        use self::Emote::*;
        Ok(match i {
            EMOTE_NORMAL => Normal,
            EMOTE_PAIN => Pain,
            EMOTE_HAPPY => Happy,
            EMOTE_SURPRISE => Surprise,
            EMOTE_ANGRY => Angry,
            EMOTE_BLINK => Blink,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Emote::*;
        match self {
            Normal => EMOTE_NORMAL,
            Pain => EMOTE_PAIN,
            Happy => EMOTE_HAPPY,
            Surprise => EMOTE_SURPRISE,
            Angry => EMOTE_ANGRY,
            Blink => EMOTE_BLINK,
        }
    }
}

impl Powerup {
    pub fn from_i32(i: i32) -> Result<Powerup, IntOutOfRange> {
        use self::Powerup::*;
        Ok(match i {
            POWERUP_HEALTH => Health,
            POWERUP_ARMOR => Armor,
            POWERUP_WEAPON => Weapon,
            POWERUP_NINJA => Ninja,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Powerup::*;
        match self {
            Health => POWERUP_HEALTH,
            Armor => POWERUP_ARMOR,
            Weapon => POWERUP_WEAPON,
            Ninja => POWERUP_NINJA,
        }
    }
}

impl Emoticon {
    pub fn from_i32(i: i32) -> Result<Emoticon, IntOutOfRange> {
        use self::Emoticon::*;
        Ok(match i {
            EMOTICON_OOP => Oop,
            EMOTICON_EXCLAMATION => Exclamation,
            EMOTICON_HEARTS => Hearts,
            EMOTICON_DROP => Drop,
            EMOTICON_DOTDOT => Dotdot,
            EMOTICON_MUSIC => Music,
            EMOTICON_SORRY => Sorry,
            EMOTICON_GHOST => Ghost,
            EMOTICON_SUSHI => Sushi,
            EMOTICON_SPLATTEE => Splattee,
            EMOTICON_DEVILTEE => Deviltee,
            EMOTICON_ZOMG => Zomg,
            EMOTICON_ZZZ => Zzz,
            EMOTICON_WTF => Wtf,
            EMOTICON_EYES => Eyes,
            EMOTICON_QUESTION => Question,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Emoticon::*;
        match self {
            Oop => EMOTICON_OOP,
            Exclamation => EMOTICON_EXCLAMATION,
            Hearts => EMOTICON_HEARTS,
            Drop => EMOTICON_DROP,
            Dotdot => EMOTICON_DOTDOT,
            Music => EMOTICON_MUSIC,
            Sorry => EMOTICON_SORRY,
            Ghost => EMOTICON_GHOST,
            Sushi => EMOTICON_SUSHI,
            Splattee => EMOTICON_SPLATTEE,
            Deviltee => EMOTICON_DEVILTEE,
            Zomg => EMOTICON_ZOMG,
            Zzz => EMOTICON_ZZZ,
            Wtf => EMOTICON_WTF,
            Eyes => EMOTICON_EYES,
            Question => EMOTICON_QUESTION,
        }
    }
}

impl Weapon {
    pub fn from_i32(i: i32) -> Result<Weapon, IntOutOfRange> {
        use self::Weapon::*;
        Ok(match i {
            WEAPON_HAMMER => Hammer,
            WEAPON_PISTOL => Pistol,
            WEAPON_SHOTGUN => Shotgun,
            WEAPON_GRENADE => Grenade,
            WEAPON_RIFLE => Rifle,
            WEAPON_NINJA => Ninja,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Weapon::*;
        match self {
            Hammer => WEAPON_HAMMER,
            Pistol => WEAPON_PISTOL,
            Shotgun => WEAPON_SHOTGUN,
            Grenade => WEAPON_GRENADE,
            Rifle => WEAPON_RIFLE,
            Ninja => WEAPON_NINJA,
        }
    }
}

impl Team {
    pub fn from_i32(i: i32) -> Result<Team, IntOutOfRange> {
        use self::Team::*;
        Ok(match i {
            TEAM_SPECTATORS => Spectators,
            TEAM_RED => Red,
            TEAM_BLUE => Blue,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Team::*;
        match self {
            Spectators => TEAM_SPECTATORS,
            Red => TEAM_RED,
            Blue => TEAM_BLUE,
        }
    }
}

impl Sound {
    pub fn from_i32(i: i32) -> Result<Sound, IntOutOfRange> {
        use self::Sound::*;
        Ok(match i {
            SOUND_GUN_FIRE => GunFire,
            SOUND_SHOTGUN_FIRE => ShotgunFire,
            SOUND_GRENADE_FIRE => GrenadeFire,
            SOUND_HAMMER_FIRE => HammerFire,
            SOUND_HAMMER_HIT => HammerHit,
            SOUND_NINJA_FIRE => NinjaFire,
            SOUND_GRENADE_EXPLODE => GrenadeExplode,
            SOUND_NINJA_HIT => NinjaHit,
            SOUND_RIFLE_FIRE => RifleFire,
            SOUND_RIFLE_BOUNCE => RifleBounce,
            SOUND_WEAPON_SWITCH => WeaponSwitch,
            SOUND_PLAYER_PAIN_SHORT => PlayerPainShort,
            SOUND_PLAYER_PAIN_LONG => PlayerPainLong,
            SOUND_BODY_LAND => BodyLand,
            SOUND_PLAYER_AIRJUMP => PlayerAirjump,
            SOUND_PLAYER_JUMP => PlayerJump,
            SOUND_PLAYER_DIE => PlayerDie,
            SOUND_PLAYER_SPAWN => PlayerSpawn,
            SOUND_PLAYER_SKID => PlayerSkid,
            SOUND_TEE_CRY => TeeCry,
            SOUND_HOOK_LOOP => HookLoop,
            SOUND_HOOK_ATTACH_GROUND => HookAttachGround,
            SOUND_HOOK_ATTACH_PLAYER => HookAttachPlayer,
            SOUND_HOOK_NOATTACH => HookNoattach,
            SOUND_PICKUP_HEALTH => PickupHealth,
            SOUND_PICKUP_ARMOR => PickupArmor,
            SOUND_PICKUP_GRENADE => PickupGrenade,
            SOUND_PICKUP_SHOTGUN => PickupShotgun,
            SOUND_PICKUP_NINJA => PickupNinja,
            SOUND_WEAPON_SPAWN => WeaponSpawn,
            SOUND_WEAPON_NOAMMO => WeaponNoammo,
            SOUND_HIT => Hit,
            SOUND_CHAT_SERVER => ChatServer,
            SOUND_CHAT_CLIENT => ChatClient,
            SOUND_CHAT_HIGHLIGHT => ChatHighlight,
            SOUND_CTF_DROP => CtfDrop,
            SOUND_CTF_RETURN => CtfReturn,
            SOUND_CTF_GRAB_PL => CtfGrabPl,
            SOUND_CTF_GRAB_EN => CtfGrabEn,
            SOUND_CTF_CAPTURE => CtfCapture,
            SOUND_MENU => Menu,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Sound::*;
        match self {
            GunFire => SOUND_GUN_FIRE,
            ShotgunFire => SOUND_SHOTGUN_FIRE,
            GrenadeFire => SOUND_GRENADE_FIRE,
            HammerFire => SOUND_HAMMER_FIRE,
            HammerHit => SOUND_HAMMER_HIT,
            NinjaFire => SOUND_NINJA_FIRE,
            GrenadeExplode => SOUND_GRENADE_EXPLODE,
            NinjaHit => SOUND_NINJA_HIT,
            RifleFire => SOUND_RIFLE_FIRE,
            RifleBounce => SOUND_RIFLE_BOUNCE,
            WeaponSwitch => SOUND_WEAPON_SWITCH,
            PlayerPainShort => SOUND_PLAYER_PAIN_SHORT,
            PlayerPainLong => SOUND_PLAYER_PAIN_LONG,
            BodyLand => SOUND_BODY_LAND,
            PlayerAirjump => SOUND_PLAYER_AIRJUMP,
            PlayerJump => SOUND_PLAYER_JUMP,
            PlayerDie => SOUND_PLAYER_DIE,
            PlayerSpawn => SOUND_PLAYER_SPAWN,
            PlayerSkid => SOUND_PLAYER_SKID,
            TeeCry => SOUND_TEE_CRY,
            HookLoop => SOUND_HOOK_LOOP,
            HookAttachGround => SOUND_HOOK_ATTACH_GROUND,
            HookAttachPlayer => SOUND_HOOK_ATTACH_PLAYER,
            HookNoattach => SOUND_HOOK_NOATTACH,
            PickupHealth => SOUND_PICKUP_HEALTH,
            PickupArmor => SOUND_PICKUP_ARMOR,
            PickupGrenade => SOUND_PICKUP_GRENADE,
            PickupShotgun => SOUND_PICKUP_SHOTGUN,
            PickupNinja => SOUND_PICKUP_NINJA,
            WeaponSpawn => SOUND_WEAPON_SPAWN,
            WeaponNoammo => SOUND_WEAPON_NOAMMO,
            Hit => SOUND_HIT,
            ChatServer => SOUND_CHAT_SERVER,
            ChatClient => SOUND_CHAT_CLIENT,
            ChatHighlight => SOUND_CHAT_HIGHLIGHT,
            CtfDrop => SOUND_CTF_DROP,
            CtfReturn => SOUND_CTF_RETURN,
            CtfGrabPl => SOUND_CTF_GRAB_PL,
            CtfGrabEn => SOUND_CTF_GRAB_EN,
            CtfCapture => SOUND_CTF_CAPTURE,
            Menu => SOUND_MENU,
        }
    }
}

pub const SV_MOTD: i32 = 1;
pub const SV_BROADCAST: i32 = 2;
pub const SV_CHAT: i32 = 3;
pub const SV_KILL_MSG: i32 = 4;
pub const SV_SOUND_GLOBAL: i32 = 5;
pub const SV_TUNE_PARAMS: i32 = 6;
pub const SV_EXTRA_PROJECTILE: i32 = 7;
pub const SV_READY_TO_ENTER: i32 = 8;
pub const SV_WEAPON_PICKUP: i32 = 9;
pub const SV_EMOTICON: i32 = 10;
pub const SV_VOTE_CLEAR_OPTIONS: i32 = 11;
pub const SV_VOTE_OPTION_LIST_ADD: i32 = 12;
pub const SV_VOTE_OPTION_ADD: i32 = 13;
pub const SV_VOTE_OPTION_REMOVE: i32 = 14;
pub const SV_VOTE_SET: i32 = 15;
pub const SV_VOTE_STATUS: i32 = 16;
pub const CL_SAY: i32 = 17;
pub const CL_SET_TEAM: i32 = 18;
pub const CL_SET_SPECTATOR_MODE: i32 = 19;
pub const CL_START_INFO: i32 = 20;
pub const CL_CHANGE_INFO: i32 = 21;
pub const CL_KILL: i32 = 22;
pub const CL_EMOTICON: i32 = 23;
pub const CL_VOTE: i32 = 24;
pub const CL_CALL_VOTE: i32 = 25;

pub enum Game<'a> {
    SvMotd(SvMotd<'a>),
    SvBroadcast(SvBroadcast<'a>),
    SvChat(SvChat<'a>),
    SvKillMsg(SvKillMsg),
    SvSoundGlobal(SvSoundGlobal),
    SvTuneParams(SvTuneParams),
    SvExtraProjectile(SvExtraProjectile),
    SvReadyToEnter(SvReadyToEnter),
    SvWeaponPickup(SvWeaponPickup),
    SvEmoticon(SvEmoticon),
    SvVoteClearOptions(SvVoteClearOptions),
    SvVoteOptionListAdd(SvVoteOptionListAdd<'a>),
    SvVoteOptionAdd(SvVoteOptionAdd<'a>),
    SvVoteOptionRemove(SvVoteOptionRemove<'a>),
    SvVoteSet(SvVoteSet<'a>),
    SvVoteStatus(SvVoteStatus),
    ClSay(ClSay<'a>),
    ClSetTeam(ClSetTeam),
    ClSetSpectatorMode(ClSetSpectatorMode),
    ClStartInfo(ClStartInfo<'a>),
    ClChangeInfo(ClChangeInfo<'a>),
    ClKill(ClKill),
    ClEmoticon(ClEmoticon),
    ClVote(ClVote),
    ClCallVote(ClCallVote<'a>),
}

impl<'a> Game<'a> {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: i32, _p: &mut Unpacker<'a>) -> Result<Game<'a>, Error> {
        Ok(match msg_id {
            SV_MOTD => Game::SvMotd(try!(SvMotd::decode(warn, _p))),
            SV_BROADCAST => Game::SvBroadcast(try!(SvBroadcast::decode(warn, _p))),
            SV_CHAT => Game::SvChat(try!(SvChat::decode(warn, _p))),
            SV_KILL_MSG => Game::SvKillMsg(try!(SvKillMsg::decode(warn, _p))),
            SV_SOUND_GLOBAL => Game::SvSoundGlobal(try!(SvSoundGlobal::decode(warn, _p))),
            SV_TUNE_PARAMS => Game::SvTuneParams(try!(SvTuneParams::decode(warn, _p))),
            SV_EXTRA_PROJECTILE => Game::SvExtraProjectile(try!(SvExtraProjectile::decode(warn, _p))),
            SV_READY_TO_ENTER => Game::SvReadyToEnter(try!(SvReadyToEnter::decode(warn, _p))),
            SV_WEAPON_PICKUP => Game::SvWeaponPickup(try!(SvWeaponPickup::decode(warn, _p))),
            SV_EMOTICON => Game::SvEmoticon(try!(SvEmoticon::decode(warn, _p))),
            SV_VOTE_CLEAR_OPTIONS => Game::SvVoteClearOptions(try!(SvVoteClearOptions::decode(warn, _p))),
            SV_VOTE_OPTION_LIST_ADD => Game::SvVoteOptionListAdd(try!(SvVoteOptionListAdd::decode(warn, _p))),
            SV_VOTE_OPTION_ADD => Game::SvVoteOptionAdd(try!(SvVoteOptionAdd::decode(warn, _p))),
            SV_VOTE_OPTION_REMOVE => Game::SvVoteOptionRemove(try!(SvVoteOptionRemove::decode(warn, _p))),
            SV_VOTE_SET => Game::SvVoteSet(try!(SvVoteSet::decode(warn, _p))),
            SV_VOTE_STATUS => Game::SvVoteStatus(try!(SvVoteStatus::decode(warn, _p))),
            CL_SAY => Game::ClSay(try!(ClSay::decode(warn, _p))),
            CL_SET_TEAM => Game::ClSetTeam(try!(ClSetTeam::decode(warn, _p))),
            CL_SET_SPECTATOR_MODE => Game::ClSetSpectatorMode(try!(ClSetSpectatorMode::decode(warn, _p))),
            CL_START_INFO => Game::ClStartInfo(try!(ClStartInfo::decode(warn, _p))),
            CL_CHANGE_INFO => Game::ClChangeInfo(try!(ClChangeInfo::decode(warn, _p))),
            CL_KILL => Game::ClKill(try!(ClKill::decode(warn, _p))),
            CL_EMOTICON => Game::ClEmoticon(try!(ClEmoticon::decode(warn, _p))),
            CL_VOTE => Game::ClVote(try!(ClVote::decode(warn, _p))),
            CL_CALL_VOTE => Game::ClCallVote(try!(ClCallVote::decode(warn, _p))),
            _ => return Err(Error::UnknownMessage),
        })
    }
    pub fn msg_id(&self) -> i32 {
        match *self {
            Game::SvMotd(_) => SV_MOTD,
            Game::SvBroadcast(_) => SV_BROADCAST,
            Game::SvChat(_) => SV_CHAT,
            Game::SvKillMsg(_) => SV_KILL_MSG,
            Game::SvSoundGlobal(_) => SV_SOUND_GLOBAL,
            Game::SvTuneParams(_) => SV_TUNE_PARAMS,
            Game::SvExtraProjectile(_) => SV_EXTRA_PROJECTILE,
            Game::SvReadyToEnter(_) => SV_READY_TO_ENTER,
            Game::SvWeaponPickup(_) => SV_WEAPON_PICKUP,
            Game::SvEmoticon(_) => SV_EMOTICON,
            Game::SvVoteClearOptions(_) => SV_VOTE_CLEAR_OPTIONS,
            Game::SvVoteOptionListAdd(_) => SV_VOTE_OPTION_LIST_ADD,
            Game::SvVoteOptionAdd(_) => SV_VOTE_OPTION_ADD,
            Game::SvVoteOptionRemove(_) => SV_VOTE_OPTION_REMOVE,
            Game::SvVoteSet(_) => SV_VOTE_SET,
            Game::SvVoteStatus(_) => SV_VOTE_STATUS,
            Game::ClSay(_) => CL_SAY,
            Game::ClSetTeam(_) => CL_SET_TEAM,
            Game::ClSetSpectatorMode(_) => CL_SET_SPECTATOR_MODE,
            Game::ClStartInfo(_) => CL_START_INFO,
            Game::ClChangeInfo(_) => CL_CHANGE_INFO,
            Game::ClKill(_) => CL_KILL,
            Game::ClEmoticon(_) => CL_EMOTICON,
            Game::ClVote(_) => CL_VOTE,
            Game::ClCallVote(_) => CL_CALL_VOTE,
        }
    }
    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            Game::SvMotd(ref i) => i.encode(p),
            Game::SvBroadcast(ref i) => i.encode(p),
            Game::SvChat(ref i) => i.encode(p),
            Game::SvKillMsg(ref i) => i.encode(p),
            Game::SvSoundGlobal(ref i) => i.encode(p),
            Game::SvTuneParams(ref i) => i.encode(p),
            Game::SvExtraProjectile(ref i) => i.encode(p),
            Game::SvReadyToEnter(ref i) => i.encode(p),
            Game::SvWeaponPickup(ref i) => i.encode(p),
            Game::SvEmoticon(ref i) => i.encode(p),
            Game::SvVoteClearOptions(ref i) => i.encode(p),
            Game::SvVoteOptionListAdd(ref i) => i.encode(p),
            Game::SvVoteOptionAdd(ref i) => i.encode(p),
            Game::SvVoteOptionRemove(ref i) => i.encode(p),
            Game::SvVoteSet(ref i) => i.encode(p),
            Game::SvVoteStatus(ref i) => i.encode(p),
            Game::ClSay(ref i) => i.encode(p),
            Game::ClSetTeam(ref i) => i.encode(p),
            Game::ClSetSpectatorMode(ref i) => i.encode(p),
            Game::ClStartInfo(ref i) => i.encode(p),
            Game::ClChangeInfo(ref i) => i.encode(p),
            Game::ClKill(ref i) => i.encode(p),
            Game::ClEmoticon(ref i) => i.encode(p),
            Game::ClVote(ref i) => i.encode(p),
            Game::ClCallVote(ref i) => i.encode(p),
        }
    }
}
pub struct SvMotd<'a> {
    pub message: &'a [u8],
}

pub struct SvBroadcast<'a> {
    pub message: &'a [u8],
}

pub struct SvChat<'a> {
    pub team: i32,
    pub client_id: i32,
    pub message: &'a [u8],
}

pub struct SvKillMsg {
    pub killer: i32,
    pub victim: i32,
    pub weapon: i32,
    pub mode_special: i32,
}

pub struct SvSoundGlobal {
    pub sound_id: Sound,
}

pub struct SvTuneParams;

pub struct SvExtraProjectile;

pub struct SvReadyToEnter;

pub struct SvWeaponPickup {
    pub weapon: Weapon,
}

pub struct SvEmoticon {
    pub client_id: i32,
    pub emoticon: Emoticon,
}

pub struct SvVoteClearOptions;

pub struct SvVoteOptionListAdd<'a> {
    pub num_options: i32,
    pub description0: &'a [u8],
    pub description1: &'a [u8],
    pub description2: &'a [u8],
    pub description3: &'a [u8],
    pub description4: &'a [u8],
    pub description5: &'a [u8],
    pub description6: &'a [u8],
    pub description7: &'a [u8],
    pub description8: &'a [u8],
    pub description9: &'a [u8],
    pub description10: &'a [u8],
    pub description11: &'a [u8],
    pub description12: &'a [u8],
    pub description13: &'a [u8],
    pub description14: &'a [u8],
}

pub struct SvVoteOptionAdd<'a> {
    pub description: &'a [u8],
}

pub struct SvVoteOptionRemove<'a> {
    pub description: &'a [u8],
}

pub struct SvVoteSet<'a> {
    pub timeout: i32,
    pub description: &'a [u8],
    pub reason: &'a [u8],
}

pub struct SvVoteStatus {
    pub yes: i32,
    pub no: i32,
    pub pass: i32,
    pub total: i32,
}

pub struct ClSay<'a> {
    pub team: bool,
    pub message: &'a [u8],
}

pub struct ClSetTeam {
    pub team: i32,
}

pub struct ClSetSpectatorMode {
    pub spectator_id: i32,
}

pub struct ClStartInfo<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub skin: &'a [u8],
    pub use_custom_color: bool,
    pub color_body: i32,
    pub color_feet: i32,
}

pub struct ClChangeInfo<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub skin: &'a [u8],
    pub use_custom_color: bool,
    pub color_body: i32,
    pub color_feet: i32,
}

pub struct ClKill;

pub struct ClEmoticon {
    pub emoticon: Emoticon,
}

pub struct ClVote {
    pub vote: i32,
}

pub struct ClCallVote<'a> {
    pub type_: &'a [u8],
    pub value: &'a [u8],
    pub reason: &'a [u8],
}

impl<'a> SvMotd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvMotd<'a>, Error> {
        let result = Ok(SvMotd {
            message: try!(_p.read_string()),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_string(self.message));
        Ok(_p.written())
    }
}

impl<'a> SvBroadcast<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvBroadcast<'a>, Error> {
        let result = Ok(SvBroadcast {
            message: try!(_p.read_string()),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_string(self.message));
        Ok(_p.written())
    }
}

impl<'a> SvChat<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvChat<'a>, Error> {
        let result = Ok(SvChat {
            team: try!(in_range(try!(_p.read_int(warn)), TEAM_SPECTATORS, TEAM_BLUE)),
            client_id: try!(in_range(try!(_p.read_int(warn)), -1, MAX_CLIENTS-1)),
            message: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(TEAM_SPECTATORS <= self.team && self.team <= TEAM_BLUE);
        assert!(-1 <= self.client_id && self.client_id <= MAX_CLIENTS-1);
        sanitize(&mut Panic, self.message).unwrap();
        try!(_p.write_int(self.team));
        try!(_p.write_int(self.client_id));
        try!(_p.write_string(self.message));
        Ok(_p.written())
    }
}

impl SvKillMsg {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvKillMsg, Error> {
        let result = Ok(SvKillMsg {
            killer: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS-1)),
            victim: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS-1)),
            weapon: try!(in_range(try!(_p.read_int(warn)), -3, 5)),
            mode_special: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.killer && self.killer <= MAX_CLIENTS-1);
        assert!(0 <= self.victim && self.victim <= MAX_CLIENTS-1);
        assert!(-3 <= self.weapon && self.weapon <= 5);
        try!(_p.write_int(self.killer));
        try!(_p.write_int(self.victim));
        try!(_p.write_int(self.weapon));
        try!(_p.write_int(self.mode_special));
        Ok(_p.written())
    }
}

impl SvSoundGlobal {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvSoundGlobal, Error> {
        let result = Ok(SvSoundGlobal {
            sound_id: try!(Sound::from_i32(try!(_p.read_int(warn)))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_int(self.sound_id.to_i32()));
        Ok(_p.written())
    }
}

impl SvTuneParams {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvTuneParams, Error> {
        let result = Ok(SvTuneParams);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}

impl SvExtraProjectile {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvExtraProjectile, Error> {
        let result = Ok(SvExtraProjectile);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}

impl SvReadyToEnter {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvReadyToEnter, Error> {
        let result = Ok(SvReadyToEnter);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}

impl SvWeaponPickup {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvWeaponPickup, Error> {
        let result = Ok(SvWeaponPickup {
            weapon: try!(Weapon::from_i32(try!(_p.read_int(warn)))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_int(self.weapon.to_i32()));
        Ok(_p.written())
    }
}

impl SvEmoticon {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvEmoticon, Error> {
        let result = Ok(SvEmoticon {
            client_id: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS-1)),
            emoticon: try!(Emoticon::from_i32(try!(_p.read_int(warn)))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= MAX_CLIENTS-1);
        try!(_p.write_int(self.client_id));
        try!(_p.write_int(self.emoticon.to_i32()));
        Ok(_p.written())
    }
}

impl SvVoteClearOptions {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvVoteClearOptions, Error> {
        let result = Ok(SvVoteClearOptions);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}

impl<'a> SvVoteOptionListAdd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteOptionListAdd<'a>, Error> {
        let result = Ok(SvVoteOptionListAdd {
            num_options: try!(in_range(try!(_p.read_int(warn)), 1, 15)),
            description0: try!(sanitize(warn, try!(_p.read_string()))),
            description1: try!(sanitize(warn, try!(_p.read_string()))),
            description2: try!(sanitize(warn, try!(_p.read_string()))),
            description3: try!(sanitize(warn, try!(_p.read_string()))),
            description4: try!(sanitize(warn, try!(_p.read_string()))),
            description5: try!(sanitize(warn, try!(_p.read_string()))),
            description6: try!(sanitize(warn, try!(_p.read_string()))),
            description7: try!(sanitize(warn, try!(_p.read_string()))),
            description8: try!(sanitize(warn, try!(_p.read_string()))),
            description9: try!(sanitize(warn, try!(_p.read_string()))),
            description10: try!(sanitize(warn, try!(_p.read_string()))),
            description11: try!(sanitize(warn, try!(_p.read_string()))),
            description12: try!(sanitize(warn, try!(_p.read_string()))),
            description13: try!(sanitize(warn, try!(_p.read_string()))),
            description14: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(1 <= self.num_options && self.num_options <= 15);
        sanitize(&mut Panic, self.description0).unwrap();
        sanitize(&mut Panic, self.description1).unwrap();
        sanitize(&mut Panic, self.description2).unwrap();
        sanitize(&mut Panic, self.description3).unwrap();
        sanitize(&mut Panic, self.description4).unwrap();
        sanitize(&mut Panic, self.description5).unwrap();
        sanitize(&mut Panic, self.description6).unwrap();
        sanitize(&mut Panic, self.description7).unwrap();
        sanitize(&mut Panic, self.description8).unwrap();
        sanitize(&mut Panic, self.description9).unwrap();
        sanitize(&mut Panic, self.description10).unwrap();
        sanitize(&mut Panic, self.description11).unwrap();
        sanitize(&mut Panic, self.description12).unwrap();
        sanitize(&mut Panic, self.description13).unwrap();
        sanitize(&mut Panic, self.description14).unwrap();
        try!(_p.write_int(self.num_options));
        try!(_p.write_string(self.description0));
        try!(_p.write_string(self.description1));
        try!(_p.write_string(self.description2));
        try!(_p.write_string(self.description3));
        try!(_p.write_string(self.description4));
        try!(_p.write_string(self.description5));
        try!(_p.write_string(self.description6));
        try!(_p.write_string(self.description7));
        try!(_p.write_string(self.description8));
        try!(_p.write_string(self.description9));
        try!(_p.write_string(self.description10));
        try!(_p.write_string(self.description11));
        try!(_p.write_string(self.description12));
        try!(_p.write_string(self.description13));
        try!(_p.write_string(self.description14));
        Ok(_p.written())
    }
}

impl<'a> SvVoteOptionAdd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteOptionAdd<'a>, Error> {
        let result = Ok(SvVoteOptionAdd {
            description: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.description).unwrap();
        try!(_p.write_string(self.description));
        Ok(_p.written())
    }
}

impl<'a> SvVoteOptionRemove<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteOptionRemove<'a>, Error> {
        let result = Ok(SvVoteOptionRemove {
            description: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.description).unwrap();
        try!(_p.write_string(self.description));
        Ok(_p.written())
    }
}

impl<'a> SvVoteSet<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteSet<'a>, Error> {
        let result = Ok(SvVoteSet {
            timeout: try!(in_range(try!(_p.read_int(warn)), 0, 60)),
            description: try!(sanitize(warn, try!(_p.read_string()))),
            reason: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.timeout && self.timeout <= 60);
        sanitize(&mut Panic, self.description).unwrap();
        sanitize(&mut Panic, self.reason).unwrap();
        try!(_p.write_int(self.timeout));
        try!(_p.write_string(self.description));
        try!(_p.write_string(self.reason));
        Ok(_p.written())
    }
}

impl SvVoteStatus {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvVoteStatus, Error> {
        let result = Ok(SvVoteStatus {
            yes: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS)),
            no: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS)),
            pass: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS)),
            total: try!(in_range(try!(_p.read_int(warn)), 0, MAX_CLIENTS)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.yes && self.yes <= MAX_CLIENTS);
        assert!(0 <= self.no && self.no <= MAX_CLIENTS);
        assert!(0 <= self.pass && self.pass <= MAX_CLIENTS);
        assert!(0 <= self.total && self.total <= MAX_CLIENTS);
        try!(_p.write_int(self.yes));
        try!(_p.write_int(self.no));
        try!(_p.write_int(self.pass));
        try!(_p.write_int(self.total));
        Ok(_p.written())
    }
}

impl<'a> ClSay<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClSay<'a>, Error> {
        let result = Ok(ClSay {
            team: try!(to_bool(try!(_p.read_int(warn)))),
            message: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.message).unwrap();
        try!(_p.write_int(self.team as i32));
        try!(_p.write_string(self.message));
        Ok(_p.written())
    }
}

impl ClSetTeam {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClSetTeam, Error> {
        let result = Ok(ClSetTeam {
            team: try!(in_range(try!(_p.read_int(warn)), TEAM_SPECTATORS, TEAM_BLUE)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(TEAM_SPECTATORS <= self.team && self.team <= TEAM_BLUE);
        try!(_p.write_int(self.team));
        Ok(_p.written())
    }
}

impl ClSetSpectatorMode {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClSetSpectatorMode, Error> {
        let result = Ok(ClSetSpectatorMode {
            spectator_id: try!(in_range(try!(_p.read_int(warn)), SPEC_FREEVIEW, MAX_CLIENTS-1)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(SPEC_FREEVIEW <= self.spectator_id && self.spectator_id <= MAX_CLIENTS-1);
        try!(_p.write_int(self.spectator_id));
        Ok(_p.written())
    }
}

impl<'a> ClStartInfo<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClStartInfo<'a>, Error> {
        let result = Ok(ClStartInfo {
            name: try!(sanitize(warn, try!(_p.read_string()))),
            clan: try!(sanitize(warn, try!(_p.read_string()))),
            country: try!(_p.read_int(warn)),
            skin: try!(sanitize(warn, try!(_p.read_string()))),
            use_custom_color: try!(to_bool(try!(_p.read_int(warn)))),
            color_body: try!(_p.read_int(warn)),
            color_feet: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.clan).unwrap();
        sanitize(&mut Panic, self.skin).unwrap();
        try!(_p.write_string(self.name));
        try!(_p.write_string(self.clan));
        try!(_p.write_int(self.country));
        try!(_p.write_string(self.skin));
        try!(_p.write_int(self.use_custom_color as i32));
        try!(_p.write_int(self.color_body));
        try!(_p.write_int(self.color_feet));
        Ok(_p.written())
    }
}

impl<'a> ClChangeInfo<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClChangeInfo<'a>, Error> {
        let result = Ok(ClChangeInfo {
            name: try!(sanitize(warn, try!(_p.read_string()))),
            clan: try!(sanitize(warn, try!(_p.read_string()))),
            country: try!(_p.read_int(warn)),
            skin: try!(sanitize(warn, try!(_p.read_string()))),
            use_custom_color: try!(to_bool(try!(_p.read_int(warn)))),
            color_body: try!(_p.read_int(warn)),
            color_feet: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.clan).unwrap();
        sanitize(&mut Panic, self.skin).unwrap();
        try!(_p.write_string(self.name));
        try!(_p.write_string(self.clan));
        try!(_p.write_int(self.country));
        try!(_p.write_string(self.skin));
        try!(_p.write_int(self.use_custom_color as i32));
        try!(_p.write_int(self.color_body));
        try!(_p.write_int(self.color_feet));
        Ok(_p.written())
    }
}

impl ClKill {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClKill, Error> {
        let result = Ok(ClKill);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}

impl ClEmoticon {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClEmoticon, Error> {
        let result = Ok(ClEmoticon {
            emoticon: try!(Emoticon::from_i32(try!(_p.read_int(warn)))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_int(self.emoticon.to_i32()));
        Ok(_p.written())
    }
}

impl ClVote {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClVote, Error> {
        let result = Ok(ClVote {
            vote: try!(in_range(try!(_p.read_int(warn)), -1, 1)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.vote && self.vote <= 1);
        try!(_p.write_int(self.vote));
        Ok(_p.written())
    }
}

impl<'a> ClCallVote<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClCallVote<'a>, Error> {
        let result = Ok(ClCallVote {
            type_: try!(sanitize(warn, try!(_p.read_string()))),
            value: try!(sanitize(warn, try!(_p.read_string()))),
            reason: try!(sanitize(warn, try!(_p.read_string()))),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.type_).unwrap();
        sanitize(&mut Panic, self.value).unwrap();
        sanitize(&mut Panic, self.reason).unwrap();
        try!(_p.write_string(self.type_));
        try!(_p.write_string(self.value));
        try!(_p.write_string(self.reason));
        Ok(_p.written())
    }
}

