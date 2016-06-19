use buffer::CapacityError;
use common::pretty;
use debug::DebugSlice;
use enums::*;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::in_range;
use packer::sanitize;
use packer::to_bool;
use packer::with_packer;
use std::fmt;
use super::SystemOrGame;
use warn::Panic;
use warn::Warn;

impl<'a> Game<'a> {
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::Game(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode_msg(p)));
        Ok(p.written())
    }
}

pub const CL_CALL_VOTE_TYPE_OPTION: &'static [u8] = b"option";
pub const CL_CALL_VOTE_TYPE_KICK: &'static [u8] = b"kick";
pub const CL_CALL_VOTE_TYPE_SPEC: &'static [u8] = b"spectate";

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

#[derive(Clone, Copy)]
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

impl<'a> fmt::Debug for Game<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Game::SvMotd(ref i) => i.fmt(f),
            Game::SvBroadcast(ref i) => i.fmt(f),
            Game::SvChat(ref i) => i.fmt(f),
            Game::SvKillMsg(ref i) => i.fmt(f),
            Game::SvSoundGlobal(ref i) => i.fmt(f),
            Game::SvTuneParams(ref i) => i.fmt(f),
            Game::SvExtraProjectile(ref i) => i.fmt(f),
            Game::SvReadyToEnter(ref i) => i.fmt(f),
            Game::SvWeaponPickup(ref i) => i.fmt(f),
            Game::SvEmoticon(ref i) => i.fmt(f),
            Game::SvVoteClearOptions(ref i) => i.fmt(f),
            Game::SvVoteOptionListAdd(ref i) => i.fmt(f),
            Game::SvVoteOptionAdd(ref i) => i.fmt(f),
            Game::SvVoteOptionRemove(ref i) => i.fmt(f),
            Game::SvVoteSet(ref i) => i.fmt(f),
            Game::SvVoteStatus(ref i) => i.fmt(f),
            Game::ClSay(ref i) => i.fmt(f),
            Game::ClSetTeam(ref i) => i.fmt(f),
            Game::ClSetSpectatorMode(ref i) => i.fmt(f),
            Game::ClStartInfo(ref i) => i.fmt(f),
            Game::ClChangeInfo(ref i) => i.fmt(f),
            Game::ClKill(ref i) => i.fmt(f),
            Game::ClEmoticon(ref i) => i.fmt(f),
            Game::ClVote(ref i) => i.fmt(f),
            Game::ClCallVote(ref i) => i.fmt(f),
        }
    }
}
#[derive(Clone, Copy)]
pub struct SvMotd<'a> {
    pub message: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvBroadcast<'a> {
    pub message: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvChat<'a> {
    pub team: i32,
    pub client_id: i32,
    pub message: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvKillMsg {
    pub killer: i32,
    pub victim: i32,
    pub weapon: i32,
    pub mode_special: i32,
}

#[derive(Clone, Copy)]
pub struct SvSoundGlobal {
    pub sound_id: Sound,
}

#[derive(Clone, Copy)]
pub struct SvTuneParams {
    pub ground_control_speed: i32,
    pub ground_control_accel: i32,
    pub ground_friction: i32,
    pub ground_jump_impulse: i32,
    pub air_jump_impulse: i32,
    pub air_control_speed: i32,
    pub air_control_accel: i32,
    pub air_friction: i32,
    pub hook_length: i32,
    pub hook_fire_speed: i32,
    pub hook_drag_accel: i32,
    pub hook_drag_speed: i32,
    pub gravity: i32,
    pub velramp_start: i32,
    pub velramp_range: i32,
    pub velramp_curvature: i32,
    pub gun_curvature: i32,
    pub gun_speed: i32,
    pub gun_lifetime: i32,
    pub shotgun_curvature: i32,
    pub shotgun_speed: i32,
    pub shotgun_speeddiff: i32,
    pub shotgun_lifetime: i32,
    pub grenade_curvature: i32,
    pub grenade_speed: i32,
    pub grenade_lifetime: i32,
    pub laser_reach: i32,
    pub laser_bounce_delay: i32,
    pub laser_bounce_num: i32,
    pub laser_bounce_cost: i32,
    pub laser_damage: i32,
    pub player_collision: i32,
    pub player_hooking: i32,
}

#[derive(Clone, Copy)]
pub struct SvExtraProjectile;

#[derive(Clone, Copy)]
pub struct SvReadyToEnter;

#[derive(Clone, Copy)]
pub struct SvWeaponPickup {
    pub weapon: Weapon,
}

#[derive(Clone, Copy)]
pub struct SvEmoticon {
    pub client_id: i32,
    pub emoticon: Emoticon,
}

#[derive(Clone, Copy)]
pub struct SvVoteClearOptions;

#[derive(Clone, Copy)]
pub struct SvVoteOptionListAdd<'a> {
    pub num_options: i32,
    pub description: [&'a [u8]; 15],
}

#[derive(Clone, Copy)]
pub struct SvVoteOptionAdd<'a> {
    pub description: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvVoteOptionRemove<'a> {
    pub description: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvVoteSet<'a> {
    pub timeout: i32,
    pub description: &'a [u8],
    pub reason: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvVoteStatus {
    pub yes: i32,
    pub no: i32,
    pub pass: i32,
    pub total: i32,
}

#[derive(Clone, Copy)]
pub struct ClSay<'a> {
    pub team: bool,
    pub message: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct ClSetTeam {
    pub team: i32,
}

#[derive(Clone, Copy)]
pub struct ClSetSpectatorMode {
    pub spectator_id: i32,
}

#[derive(Clone, Copy)]
pub struct ClStartInfo<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub skin: &'a [u8],
    pub use_custom_color: bool,
    pub color_body: i32,
    pub color_feet: i32,
}

#[derive(Clone, Copy)]
pub struct ClChangeInfo<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub skin: &'a [u8],
    pub use_custom_color: bool,
    pub color_body: i32,
    pub color_feet: i32,
}

#[derive(Clone, Copy)]
pub struct ClKill;

#[derive(Clone, Copy)]
pub struct ClEmoticon {
    pub emoticon: Emoticon,
}

#[derive(Clone, Copy)]
pub struct ClVote {
    pub vote: i32,
}

#[derive(Clone, Copy)]
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
impl<'a> fmt::Debug for SvMotd<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvMotd")
            .field("message", &pretty::Bytes::new(&self.message))
            .finish()
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
impl<'a> fmt::Debug for SvBroadcast<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvBroadcast")
            .field("message", &pretty::Bytes::new(&self.message))
            .finish()
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
impl<'a> fmt::Debug for SvChat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvChat")
            .field("team", &self.team)
            .field("client_id", &self.client_id)
            .field("message", &pretty::Bytes::new(&self.message))
            .finish()
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
impl fmt::Debug for SvKillMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvKillMsg")
            .field("killer", &self.killer)
            .field("victim", &self.victim)
            .field("weapon", &self.weapon)
            .field("mode_special", &self.mode_special)
            .finish()
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
impl fmt::Debug for SvSoundGlobal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvSoundGlobal")
            .field("sound_id", &self.sound_id)
            .finish()
    }
}

impl SvTuneParams {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvTuneParams, Error> {
        let result = Ok(SvTuneParams {
            ground_control_speed: try!(_p.read_int(warn)),
            ground_control_accel: try!(_p.read_int(warn)),
            ground_friction: try!(_p.read_int(warn)),
            ground_jump_impulse: try!(_p.read_int(warn)),
            air_jump_impulse: try!(_p.read_int(warn)),
            air_control_speed: try!(_p.read_int(warn)),
            air_control_accel: try!(_p.read_int(warn)),
            air_friction: try!(_p.read_int(warn)),
            hook_length: try!(_p.read_int(warn)),
            hook_fire_speed: try!(_p.read_int(warn)),
            hook_drag_accel: try!(_p.read_int(warn)),
            hook_drag_speed: try!(_p.read_int(warn)),
            gravity: try!(_p.read_int(warn)),
            velramp_start: try!(_p.read_int(warn)),
            velramp_range: try!(_p.read_int(warn)),
            velramp_curvature: try!(_p.read_int(warn)),
            gun_curvature: try!(_p.read_int(warn)),
            gun_speed: try!(_p.read_int(warn)),
            gun_lifetime: try!(_p.read_int(warn)),
            shotgun_curvature: try!(_p.read_int(warn)),
            shotgun_speed: try!(_p.read_int(warn)),
            shotgun_speeddiff: try!(_p.read_int(warn)),
            shotgun_lifetime: try!(_p.read_int(warn)),
            grenade_curvature: try!(_p.read_int(warn)),
            grenade_speed: try!(_p.read_int(warn)),
            grenade_lifetime: try!(_p.read_int(warn)),
            laser_reach: try!(_p.read_int(warn)),
            laser_bounce_delay: try!(_p.read_int(warn)),
            laser_bounce_num: try!(_p.read_int(warn)),
            laser_bounce_cost: try!(_p.read_int(warn)),
            laser_damage: try!(_p.read_int(warn)),
            player_collision: try!(_p.read_int(warn)),
            player_hooking: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_int(self.ground_control_speed));
        try!(_p.write_int(self.ground_control_accel));
        try!(_p.write_int(self.ground_friction));
        try!(_p.write_int(self.ground_jump_impulse));
        try!(_p.write_int(self.air_jump_impulse));
        try!(_p.write_int(self.air_control_speed));
        try!(_p.write_int(self.air_control_accel));
        try!(_p.write_int(self.air_friction));
        try!(_p.write_int(self.hook_length));
        try!(_p.write_int(self.hook_fire_speed));
        try!(_p.write_int(self.hook_drag_accel));
        try!(_p.write_int(self.hook_drag_speed));
        try!(_p.write_int(self.gravity));
        try!(_p.write_int(self.velramp_start));
        try!(_p.write_int(self.velramp_range));
        try!(_p.write_int(self.velramp_curvature));
        try!(_p.write_int(self.gun_curvature));
        try!(_p.write_int(self.gun_speed));
        try!(_p.write_int(self.gun_lifetime));
        try!(_p.write_int(self.shotgun_curvature));
        try!(_p.write_int(self.shotgun_speed));
        try!(_p.write_int(self.shotgun_speeddiff));
        try!(_p.write_int(self.shotgun_lifetime));
        try!(_p.write_int(self.grenade_curvature));
        try!(_p.write_int(self.grenade_speed));
        try!(_p.write_int(self.grenade_lifetime));
        try!(_p.write_int(self.laser_reach));
        try!(_p.write_int(self.laser_bounce_delay));
        try!(_p.write_int(self.laser_bounce_num));
        try!(_p.write_int(self.laser_bounce_cost));
        try!(_p.write_int(self.laser_damage));
        try!(_p.write_int(self.player_collision));
        try!(_p.write_int(self.player_hooking));
        Ok(_p.written())
    }
}
impl fmt::Debug for SvTuneParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvTuneParams")
            .field("ground_control_speed", &self.ground_control_speed)
            .field("ground_control_accel", &self.ground_control_accel)
            .field("ground_friction", &self.ground_friction)
            .field("ground_jump_impulse", &self.ground_jump_impulse)
            .field("air_jump_impulse", &self.air_jump_impulse)
            .field("air_control_speed", &self.air_control_speed)
            .field("air_control_accel", &self.air_control_accel)
            .field("air_friction", &self.air_friction)
            .field("hook_length", &self.hook_length)
            .field("hook_fire_speed", &self.hook_fire_speed)
            .field("hook_drag_accel", &self.hook_drag_accel)
            .field("hook_drag_speed", &self.hook_drag_speed)
            .field("gravity", &self.gravity)
            .field("velramp_start", &self.velramp_start)
            .field("velramp_range", &self.velramp_range)
            .field("velramp_curvature", &self.velramp_curvature)
            .field("gun_curvature", &self.gun_curvature)
            .field("gun_speed", &self.gun_speed)
            .field("gun_lifetime", &self.gun_lifetime)
            .field("shotgun_curvature", &self.shotgun_curvature)
            .field("shotgun_speed", &self.shotgun_speed)
            .field("shotgun_speeddiff", &self.shotgun_speeddiff)
            .field("shotgun_lifetime", &self.shotgun_lifetime)
            .field("grenade_curvature", &self.grenade_curvature)
            .field("grenade_speed", &self.grenade_speed)
            .field("grenade_lifetime", &self.grenade_lifetime)
            .field("laser_reach", &self.laser_reach)
            .field("laser_bounce_delay", &self.laser_bounce_delay)
            .field("laser_bounce_num", &self.laser_bounce_num)
            .field("laser_bounce_cost", &self.laser_bounce_cost)
            .field("laser_damage", &self.laser_damage)
            .field("player_collision", &self.player_collision)
            .field("player_hooking", &self.player_hooking)
            .finish()
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
impl fmt::Debug for SvExtraProjectile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvExtraProjectile")
            .finish()
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
impl fmt::Debug for SvReadyToEnter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvReadyToEnter")
            .finish()
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
impl fmt::Debug for SvWeaponPickup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvWeaponPickup")
            .field("weapon", &self.weapon)
            .finish()
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
impl fmt::Debug for SvEmoticon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvEmoticon")
            .field("client_id", &self.client_id)
            .field("emoticon", &self.emoticon)
            .finish()
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
impl fmt::Debug for SvVoteClearOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteClearOptions")
            .finish()
    }
}

impl<'a> SvVoteOptionListAdd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteOptionListAdd<'a>, Error> {
        let result = Ok(SvVoteOptionListAdd {
            num_options: try!(in_range(try!(_p.read_int(warn)), 1, 15)),
            description: [
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
                try!(sanitize(warn, try!(_p.read_string()))),
            ],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(1 <= self.num_options && self.num_options <= 15);
        for e in &self.description {
            sanitize(&mut Panic, e).unwrap();
        }
        try!(_p.write_int(self.num_options));
        for e in &self.description {
            try!(_p.write_string(e));
        }
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvVoteOptionListAdd<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteOptionListAdd")
            .field("num_options", &self.num_options)
            .field("description", &DebugSlice::new(&self.description, |e| pretty::Bytes::new(&e)))
            .finish()
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
impl<'a> fmt::Debug for SvVoteOptionAdd<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteOptionAdd")
            .field("description", &pretty::Bytes::new(&self.description))
            .finish()
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
impl<'a> fmt::Debug for SvVoteOptionRemove<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteOptionRemove")
            .field("description", &pretty::Bytes::new(&self.description))
            .finish()
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
impl<'a> fmt::Debug for SvVoteSet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteSet")
            .field("timeout", &self.timeout)
            .field("description", &pretty::Bytes::new(&self.description))
            .field("reason", &pretty::Bytes::new(&self.reason))
            .finish()
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
impl fmt::Debug for SvVoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteStatus")
            .field("yes", &self.yes)
            .field("no", &self.no)
            .field("pass", &self.pass)
            .field("total", &self.total)
            .finish()
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
impl<'a> fmt::Debug for ClSay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClSay")
            .field("team", &self.team)
            .field("message", &pretty::Bytes::new(&self.message))
            .finish()
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
impl fmt::Debug for ClSetTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClSetTeam")
            .field("team", &self.team)
            .finish()
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
impl fmt::Debug for ClSetSpectatorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClSetSpectatorMode")
            .field("spectator_id", &self.spectator_id)
            .finish()
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
impl<'a> fmt::Debug for ClStartInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClStartInfo")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("skin", &pretty::Bytes::new(&self.skin))
            .field("use_custom_color", &self.use_custom_color)
            .field("color_body", &self.color_body)
            .field("color_feet", &self.color_feet)
            .finish()
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
impl<'a> fmt::Debug for ClChangeInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClChangeInfo")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("skin", &pretty::Bytes::new(&self.skin))
            .field("use_custom_color", &self.use_custom_color)
            .field("color_body", &self.color_body)
            .field("color_feet", &self.color_feet)
            .finish()
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
impl fmt::Debug for ClKill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClKill")
            .finish()
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
impl fmt::Debug for ClEmoticon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClEmoticon")
            .field("emoticon", &self.emoticon)
            .finish()
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
impl fmt::Debug for ClVote {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClVote")
            .field("vote", &self.vote)
            .finish()
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
impl<'a> fmt::Debug for ClCallVote<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClCallVote")
            .field("type_", &pretty::Bytes::new(&self.type_))
            .field("value", &pretty::Bytes::new(&self.value))
            .field("reason", &pretty::Bytes::new(&self.reason))
            .finish()
    }
}

