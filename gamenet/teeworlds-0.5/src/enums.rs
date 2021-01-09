use packer::IntOutOfRange;

pub const MAX_CLIENTS: i32 = 16;
pub const MAX_SNAPSHOT_PACKSIZE: i32 = 900;
pub const VERSION: &'static str = "0.5 b67d1f1a1eea234e";
pub const CL_CALL_VOTE_TYPE_OPTION: &'static str = "option";
pub const CL_CALL_VOTE_TYPE_KICK: &'static str = "kick";

pub const PLAYERSTATE_UNKNOWN: i32 = 0;
pub const PLAYERSTATE_PLAYING: i32 = 1;
pub const PLAYERSTATE_IN_MENU: i32 = 2;
pub const PLAYERSTATE_CHATTING: i32 = 3;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Playerstate {
    Unknown,
    Playing,
    InMenu,
    Chatting,
}

pub const EMOTE_NORMAL: i32 = 0;
pub const EMOTE_PAIN: i32 = 1;
pub const EMOTE_HAPPY: i32 = 2;
pub const EMOTE_SURPRISE: i32 = 3;
pub const EMOTE_ANGRY: i32 = 4;
pub const EMOTE_BLINK: i32 = 5;

#[repr(i32)]
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

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Powerup {
    Health,
    Armor,
    Weapon,
    Ninja,
}

pub const EMOTICON_V1: i32 = 0;
pub const EMOTICON_V2: i32 = 1;
pub const EMOTICON_V3: i32 = 2;
pub const EMOTICON_V4: i32 = 3;
pub const EMOTICON_V5: i32 = 4;
pub const EMOTICON_V6: i32 = 5;
pub const EMOTICON_V7: i32 = 6;
pub const EMOTICON_V8: i32 = 7;
pub const EMOTICON_V9: i32 = 8;
pub const EMOTICON_V10: i32 = 9;
pub const EMOTICON_V11: i32 = 10;
pub const EMOTICON_V12: i32 = 11;
pub const EMOTICON_V13: i32 = 12;
pub const EMOTICON_V14: i32 = 13;
pub const EMOTICON_V15: i32 = 14;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Emoticon {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
}

pub const WEAPON_HAMMER: i32 = 0;
pub const WEAPON_PISTOL: i32 = 1;
pub const WEAPON_SHOTGUN: i32 = 2;
pub const WEAPON_GRENADE: i32 = 3;
pub const WEAPON_RIFLE: i32 = 4;
pub const WEAPON_NINJA: i32 = 5;

#[repr(i32)]
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

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Team {
    Spectators = -1,
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
pub const SOUND_CTF_DROP: i32 = 34;
pub const SOUND_CTF_RETURN: i32 = 35;
pub const SOUND_CTF_GRAB_PL: i32 = 36;
pub const SOUND_CTF_GRAB_EN: i32 = 37;
pub const SOUND_CTF_CAPTURE: i32 = 38;

#[repr(i32)]
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
    CtfDrop,
    CtfReturn,
    CtfGrabPl,
    CtfGrabEn,
    CtfCapture,
}

impl Playerstate {
    pub fn from_i32(i: i32) -> Result<Playerstate, IntOutOfRange> {
        use self::Playerstate::*;
        Ok(match i {
            PLAYERSTATE_UNKNOWN => Unknown,
            PLAYERSTATE_PLAYING => Playing,
            PLAYERSTATE_IN_MENU => InMenu,
            PLAYERSTATE_CHATTING => Chatting,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Playerstate::*;
        match self {
            Unknown => PLAYERSTATE_UNKNOWN,
            Playing => PLAYERSTATE_PLAYING,
            InMenu => PLAYERSTATE_IN_MENU,
            Chatting => PLAYERSTATE_CHATTING,
        }
    }
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
            EMOTICON_V1 => V1,
            EMOTICON_V2 => V2,
            EMOTICON_V3 => V3,
            EMOTICON_V4 => V4,
            EMOTICON_V5 => V5,
            EMOTICON_V6 => V6,
            EMOTICON_V7 => V7,
            EMOTICON_V8 => V8,
            EMOTICON_V9 => V9,
            EMOTICON_V10 => V10,
            EMOTICON_V11 => V11,
            EMOTICON_V12 => V12,
            EMOTICON_V13 => V13,
            EMOTICON_V14 => V14,
            EMOTICON_V15 => V15,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Emoticon::*;
        match self {
            V1 => EMOTICON_V1,
            V2 => EMOTICON_V2,
            V3 => EMOTICON_V3,
            V4 => EMOTICON_V4,
            V5 => EMOTICON_V5,
            V6 => EMOTICON_V6,
            V7 => EMOTICON_V7,
            V8 => EMOTICON_V8,
            V9 => EMOTICON_V9,
            V10 => EMOTICON_V10,
            V11 => EMOTICON_V11,
            V12 => EMOTICON_V12,
            V13 => EMOTICON_V13,
            V14 => EMOTICON_V14,
            V15 => EMOTICON_V15,
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
            SOUND_CTF_DROP => CtfDrop,
            SOUND_CTF_RETURN => CtfReturn,
            SOUND_CTF_GRAB_PL => CtfGrabPl,
            SOUND_CTF_GRAB_EN => CtfGrabEn,
            SOUND_CTF_CAPTURE => CtfCapture,
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
            CtfDrop => SOUND_CTF_DROP,
            CtfReturn => SOUND_CTF_RETURN,
            CtfGrabPl => SOUND_CTF_GRAB_PL,
            CtfGrabEn => SOUND_CTF_GRAB_EN,
            CtfCapture => SOUND_CTF_CAPTURE,
        }
    }
}

