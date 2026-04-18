use libtw2_packer::IntOutOfRange;

pub const MAX_CLIENTS: i32 = 64;
pub const WEAPON_GAME: i32 = -3;
pub const WEAPON_SELF: i32 = -2;
pub const WEAPON_WORLD: i32 = -1;
pub const FLAG_MISSING: i32 = -3;
pub const FLAG_ATSTAND: i32 = -2;
pub const FLAG_TAKEN: i32 = -1;
pub const VERSION: &'static str = "0.7 802f1be60a05665f";
pub const CLIENT_VERSION: i32 = 1797;
pub const CL_CALL_VOTE_TYPE_OPTION: &'static str = "option";
pub const CL_CALL_VOTE_TYPE_KICK: &'static str = "kick";
pub const CL_CALL_VOTE_TYPE_SPEC: &'static str = "spec";
pub const VOTE_CHOICE_NO: i32 = -1;
pub const VOTE_CHOICE_PASS: i32 = 0;
pub const VOTE_CHOICE_YES: i32 = 1;

pub const PICKUP_HEALTH: i32 = 0;
pub const PICKUP_ARMOR: i32 = 1;
pub const PICKUP_GRENADE: i32 = 2;
pub const PICKUP_SHOTGUN: i32 = 3;
pub const PICKUP_LASER: i32 = 4;
pub const PICKUP_NINJA: i32 = 5;
pub const PICKUP_GUN: i32 = 6;
pub const PICKUP_HAMMER: i32 = 7;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Pickup {
    Health,
    Armor,
    Grenade,
    Shotgun,
    Laser,
    Ninja,
    Gun,
    Hammer,
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

#[repr(i32)]
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

pub const VOTE_UNKNOWN: i32 = 0;
pub const VOTE_START_OP: i32 = 1;
pub const VOTE_START_KICK: i32 = 2;
pub const VOTE_START_SPEC: i32 = 3;
pub const VOTE_END_ABORT: i32 = 4;
pub const VOTE_END_PASS: i32 = 5;
pub const VOTE_END_FAIL: i32 = 6;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Vote {
    Unknown,
    StartOp,
    StartKick,
    StartSpec,
    EndAbort,
    EndPass,
    EndFail,
}

pub const CHAT_NONE: i32 = 0;
pub const CHAT_ALL: i32 = 1;
pub const CHAT_TEAM: i32 = 2;
pub const CHAT_WHISPER: i32 = 3;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Chat {
    None,
    All,
    Team,
    Whisper,
}

pub const GAMEMSG_TEAM_SWAP: i32 = 0;
pub const GAMEMSG_SPEC_INVALID_ID: i32 = 1;
pub const GAMEMSG_TEAM_SHUFFLE: i32 = 2;
pub const GAMEMSG_TEAM_BALANCE: i32 = 3;
pub const GAMEMSG_CTF_DROP: i32 = 4;
pub const GAMEMSG_CTF_RETURN: i32 = 5;
pub const GAMEMSG_TEAM_ALL: i32 = 6;
pub const GAMEMSG_TEAM_BALANCE_VICTIM: i32 = 7;
pub const GAMEMSG_CTF_GRAB: i32 = 8;
pub const GAMEMSG_CTF_CAPTURE: i32 = 9;
pub const GAMEMSG_GAME_PAUSED: i32 = 10;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Gamemsg {
    TeamSwap,
    SpecInvalidId,
    TeamShuffle,
    TeamBalance,
    CtfDrop,
    CtfReturn,
    TeamAll,
    TeamBalanceVictim,
    CtfGrab,
    CtfCapture,
    GamePaused,
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
pub const SOUND_CHAT_HIGHLIGHT: i32 = 34;
pub const SOUND_CTF_DROP: i32 = 35;
pub const SOUND_CTF_RETURN: i32 = 36;
pub const SOUND_CTF_GRAB_PL: i32 = 37;
pub const SOUND_CTF_GRAB_EN: i32 = 38;
pub const SOUND_CTF_CAPTURE: i32 = 39;
pub const SOUND_MENU: i32 = 40;

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
    ChatHighlight,
    CtfDrop,
    CtfReturn,
    CtfGrabPl,
    CtfGrabEn,
    CtfCapture,
    Menu,
}

pub const SPEC_FREEVIEW: i32 = 0;
pub const SPEC_PLAYER: i32 = 1;
pub const SPEC_FLAGRED: i32 = 2;
pub const SPEC_FLAGBLUE: i32 = 3;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Spec {
    Freeview,
    Player,
    Flagred,
    Flagblue,
}

pub const SKINPART_BODY: i32 = 0;
pub const SKINPART_MARKING: i32 = 1;
pub const SKINPART_DECORATION: i32 = 2;
pub const SKINPART_HANDS: i32 = 3;
pub const SKINPART_FEET: i32 = 4;
pub const SKINPART_EYES: i32 = 5;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Skinpart {
    Body,
    Marking,
    Decoration,
    Hands,
    Feet,
    Eyes,
}

impl Pickup {
    pub fn from_i32(i: i32) -> Result<Pickup, IntOutOfRange> {
        use self::Pickup::*;
        Ok(match i {
            PICKUP_HEALTH => Health,
            PICKUP_ARMOR => Armor,
            PICKUP_GRENADE => Grenade,
            PICKUP_SHOTGUN => Shotgun,
            PICKUP_LASER => Laser,
            PICKUP_NINJA => Ninja,
            PICKUP_GUN => Gun,
            PICKUP_HAMMER => Hammer,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Pickup::*;
        match self {
            Health => PICKUP_HEALTH,
            Armor => PICKUP_ARMOR,
            Grenade => PICKUP_GRENADE,
            Shotgun => PICKUP_SHOTGUN,
            Laser => PICKUP_LASER,
            Ninja => PICKUP_NINJA,
            Gun => PICKUP_GUN,
            Hammer => PICKUP_HAMMER,
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

impl Vote {
    pub fn from_i32(i: i32) -> Result<Vote, IntOutOfRange> {
        use self::Vote::*;
        Ok(match i {
            VOTE_UNKNOWN => Unknown,
            VOTE_START_OP => StartOp,
            VOTE_START_KICK => StartKick,
            VOTE_START_SPEC => StartSpec,
            VOTE_END_ABORT => EndAbort,
            VOTE_END_PASS => EndPass,
            VOTE_END_FAIL => EndFail,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Vote::*;
        match self {
            Unknown => VOTE_UNKNOWN,
            StartOp => VOTE_START_OP,
            StartKick => VOTE_START_KICK,
            StartSpec => VOTE_START_SPEC,
            EndAbort => VOTE_END_ABORT,
            EndPass => VOTE_END_PASS,
            EndFail => VOTE_END_FAIL,
        }
    }
}

impl Chat {
    pub fn from_i32(i: i32) -> Result<Chat, IntOutOfRange> {
        use self::Chat::*;
        Ok(match i {
            CHAT_NONE => None,
            CHAT_ALL => All,
            CHAT_TEAM => Team,
            CHAT_WHISPER => Whisper,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Chat::*;
        match self {
            None => CHAT_NONE,
            All => CHAT_ALL,
            Team => CHAT_TEAM,
            Whisper => CHAT_WHISPER,
        }
    }
}

impl Gamemsg {
    pub fn from_i32(i: i32) -> Result<Gamemsg, IntOutOfRange> {
        use self::Gamemsg::*;
        Ok(match i {
            GAMEMSG_TEAM_SWAP => TeamSwap,
            GAMEMSG_SPEC_INVALID_ID => SpecInvalidId,
            GAMEMSG_TEAM_SHUFFLE => TeamShuffle,
            GAMEMSG_TEAM_BALANCE => TeamBalance,
            GAMEMSG_CTF_DROP => CtfDrop,
            GAMEMSG_CTF_RETURN => CtfReturn,
            GAMEMSG_TEAM_ALL => TeamAll,
            GAMEMSG_TEAM_BALANCE_VICTIM => TeamBalanceVictim,
            GAMEMSG_CTF_GRAB => CtfGrab,
            GAMEMSG_CTF_CAPTURE => CtfCapture,
            GAMEMSG_GAME_PAUSED => GamePaused,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Gamemsg::*;
        match self {
            TeamSwap => GAMEMSG_TEAM_SWAP,
            SpecInvalidId => GAMEMSG_SPEC_INVALID_ID,
            TeamShuffle => GAMEMSG_TEAM_SHUFFLE,
            TeamBalance => GAMEMSG_TEAM_BALANCE,
            CtfDrop => GAMEMSG_CTF_DROP,
            CtfReturn => GAMEMSG_CTF_RETURN,
            TeamAll => GAMEMSG_TEAM_ALL,
            TeamBalanceVictim => GAMEMSG_TEAM_BALANCE_VICTIM,
            CtfGrab => GAMEMSG_CTF_GRAB,
            CtfCapture => GAMEMSG_CTF_CAPTURE,
            GamePaused => GAMEMSG_GAME_PAUSED,
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

impl Spec {
    pub fn from_i32(i: i32) -> Result<Spec, IntOutOfRange> {
        use self::Spec::*;
        Ok(match i {
            SPEC_FREEVIEW => Freeview,
            SPEC_PLAYER => Player,
            SPEC_FLAGRED => Flagred,
            SPEC_FLAGBLUE => Flagblue,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Spec::*;
        match self {
            Freeview => SPEC_FREEVIEW,
            Player => SPEC_PLAYER,
            Flagred => SPEC_FLAGRED,
            Flagblue => SPEC_FLAGBLUE,
        }
    }
}

impl Skinpart {
    pub fn from_i32(i: i32) -> Result<Skinpart, IntOutOfRange> {
        use self::Skinpart::*;
        Ok(match i {
            SKINPART_BODY => Body,
            SKINPART_MARKING => Marking,
            SKINPART_DECORATION => Decoration,
            SKINPART_HANDS => Hands,
            SKINPART_FEET => Feet,
            SKINPART_EYES => Eyes,
            _ => return Err(IntOutOfRange),
        })
    }
    pub fn to_i32(self) -> i32 {
        use self::Skinpart::*;
        match self {
            Body => SKINPART_BODY,
            Marking => SKINPART_MARKING,
            Decoration => SKINPART_DECORATION,
            Hands => SKINPART_HANDS,
            Feet => SKINPART_FEET,
            Eyes => SKINPART_EYES,
        }
    }
}

