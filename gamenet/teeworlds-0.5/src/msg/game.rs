use crate::enums;
use crate::error::Error;
use buffer::CapacityError;
use libtw2_common::pretty;
use libtw2_packer::Packer;
use libtw2_packer::Unpacker;
use libtw2_packer::Warning;
use libtw2_packer::in_range;
use libtw2_packer::sanitize;
use libtw2_packer::to_bool;
use libtw2_packer::with_packer;
use std::fmt;
use super::MessageId;
use super::SystemOrGame;
use warn::Panic;
use warn::Warn;
use warn::wrap;

pub use libtw2_gamenet_common::msg::TuneParam;

impl<'a> Game<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<Game<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::Game(msg_id) = SystemOrGame::decode_id(warn, p)? {
            Game::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        with_packer(&mut p, |p| SystemOrGame::Game(self.msg_id()).encode_id(p))?;
        with_packer(&mut p, |p| self.encode_msg(p))?;
        Ok(p.written())
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
pub const SV_VOTE_OPTION: i32 = 12;
pub const SV_VOTE_SET: i32 = 13;
pub const SV_VOTE_STATUS: i32 = 14;
pub const CL_SAY: i32 = 15;
pub const CL_SET_TEAM: i32 = 16;
pub const CL_START_INFO: i32 = 17;
pub const CL_CHANGE_INFO: i32 = 18;
pub const CL_KILL: i32 = 19;
pub const CL_EMOTICON: i32 = 20;
pub const CL_VOTE: i32 = 21;
pub const CL_CALL_VOTE: i32 = 22;

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
    SvVoteOption(SvVoteOption<'a>),
    SvVoteSet(SvVoteSet<'a>),
    SvVoteStatus(SvVoteStatus),
    ClSay(ClSay<'a>),
    ClSetTeam(ClSetTeam),
    ClStartInfo(ClStartInfo<'a>),
    ClChangeInfo(ClChangeInfo<'a>),
    ClKill(ClKill),
    ClEmoticon(ClEmoticon),
    ClVote(ClVote),
    ClCallVote(ClCallVote<'a>),
}

impl<'a> Game<'a> {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: MessageId, _p: &mut Unpacker<'a>) -> Result<Game<'a>, Error> {
        use self::MessageId::*;
        Ok(match msg_id {
            Ordinal(SV_MOTD) => Game::SvMotd(SvMotd::decode(warn, _p)?),
            Ordinal(SV_BROADCAST) => Game::SvBroadcast(SvBroadcast::decode(warn, _p)?),
            Ordinal(SV_CHAT) => Game::SvChat(SvChat::decode(warn, _p)?),
            Ordinal(SV_KILL_MSG) => Game::SvKillMsg(SvKillMsg::decode(warn, _p)?),
            Ordinal(SV_SOUND_GLOBAL) => Game::SvSoundGlobal(SvSoundGlobal::decode(warn, _p)?),
            Ordinal(SV_TUNE_PARAMS) => Game::SvTuneParams(SvTuneParams::decode(warn, _p)?),
            Ordinal(SV_EXTRA_PROJECTILE) => Game::SvExtraProjectile(SvExtraProjectile::decode(warn, _p)?),
            Ordinal(SV_READY_TO_ENTER) => Game::SvReadyToEnter(SvReadyToEnter::decode(warn, _p)?),
            Ordinal(SV_WEAPON_PICKUP) => Game::SvWeaponPickup(SvWeaponPickup::decode(warn, _p)?),
            Ordinal(SV_EMOTICON) => Game::SvEmoticon(SvEmoticon::decode(warn, _p)?),
            Ordinal(SV_VOTE_CLEAR_OPTIONS) => Game::SvVoteClearOptions(SvVoteClearOptions::decode(warn, _p)?),
            Ordinal(SV_VOTE_OPTION) => Game::SvVoteOption(SvVoteOption::decode(warn, _p)?),
            Ordinal(SV_VOTE_SET) => Game::SvVoteSet(SvVoteSet::decode(warn, _p)?),
            Ordinal(SV_VOTE_STATUS) => Game::SvVoteStatus(SvVoteStatus::decode(warn, _p)?),
            Ordinal(CL_SAY) => Game::ClSay(ClSay::decode(warn, _p)?),
            Ordinal(CL_SET_TEAM) => Game::ClSetTeam(ClSetTeam::decode(warn, _p)?),
            Ordinal(CL_START_INFO) => Game::ClStartInfo(ClStartInfo::decode(warn, _p)?),
            Ordinal(CL_CHANGE_INFO) => Game::ClChangeInfo(ClChangeInfo::decode(warn, _p)?),
            Ordinal(CL_KILL) => Game::ClKill(ClKill::decode(warn, _p)?),
            Ordinal(CL_EMOTICON) => Game::ClEmoticon(ClEmoticon::decode(warn, _p)?),
            Ordinal(CL_VOTE) => Game::ClVote(ClVote::decode(warn, _p)?),
            Ordinal(CL_CALL_VOTE) => Game::ClCallVote(ClCallVote::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn msg_id(&self) -> MessageId {
        match *self {
            Game::SvMotd(_) => MessageId::from(SV_MOTD),
            Game::SvBroadcast(_) => MessageId::from(SV_BROADCAST),
            Game::SvChat(_) => MessageId::from(SV_CHAT),
            Game::SvKillMsg(_) => MessageId::from(SV_KILL_MSG),
            Game::SvSoundGlobal(_) => MessageId::from(SV_SOUND_GLOBAL),
            Game::SvTuneParams(_) => MessageId::from(SV_TUNE_PARAMS),
            Game::SvExtraProjectile(_) => MessageId::from(SV_EXTRA_PROJECTILE),
            Game::SvReadyToEnter(_) => MessageId::from(SV_READY_TO_ENTER),
            Game::SvWeaponPickup(_) => MessageId::from(SV_WEAPON_PICKUP),
            Game::SvEmoticon(_) => MessageId::from(SV_EMOTICON),
            Game::SvVoteClearOptions(_) => MessageId::from(SV_VOTE_CLEAR_OPTIONS),
            Game::SvVoteOption(_) => MessageId::from(SV_VOTE_OPTION),
            Game::SvVoteSet(_) => MessageId::from(SV_VOTE_SET),
            Game::SvVoteStatus(_) => MessageId::from(SV_VOTE_STATUS),
            Game::ClSay(_) => MessageId::from(CL_SAY),
            Game::ClSetTeam(_) => MessageId::from(CL_SET_TEAM),
            Game::ClStartInfo(_) => MessageId::from(CL_START_INFO),
            Game::ClChangeInfo(_) => MessageId::from(CL_CHANGE_INFO),
            Game::ClKill(_) => MessageId::from(CL_KILL),
            Game::ClEmoticon(_) => MessageId::from(CL_EMOTICON),
            Game::ClVote(_) => MessageId::from(CL_VOTE),
            Game::ClCallVote(_) => MessageId::from(CL_CALL_VOTE),
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
            Game::SvVoteOption(ref i) => i.encode(p),
            Game::SvVoteSet(ref i) => i.encode(p),
            Game::SvVoteStatus(ref i) => i.encode(p),
            Game::ClSay(ref i) => i.encode(p),
            Game::ClSetTeam(ref i) => i.encode(p),
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
            Game::SvVoteOption(ref i) => i.fmt(f),
            Game::SvVoteSet(ref i) => i.fmt(f),
            Game::SvVoteStatus(ref i) => i.fmt(f),
            Game::ClSay(ref i) => i.fmt(f),
            Game::ClSetTeam(ref i) => i.fmt(f),
            Game::ClStartInfo(ref i) => i.fmt(f),
            Game::ClChangeInfo(ref i) => i.fmt(f),
            Game::ClKill(ref i) => i.fmt(f),
            Game::ClEmoticon(ref i) => i.fmt(f),
            Game::ClVote(ref i) => i.fmt(f),
            Game::ClCallVote(ref i) => i.fmt(f),
        }
    }
}

impl<'a> From<SvMotd<'a>> for Game<'a> {
    fn from(i: SvMotd<'a>) -> Game<'a> {
        Game::SvMotd(i)
    }
}

impl<'a> From<SvBroadcast<'a>> for Game<'a> {
    fn from(i: SvBroadcast<'a>) -> Game<'a> {
        Game::SvBroadcast(i)
    }
}

impl<'a> From<SvChat<'a>> for Game<'a> {
    fn from(i: SvChat<'a>) -> Game<'a> {
        Game::SvChat(i)
    }
}

impl<'a> From<SvKillMsg> for Game<'a> {
    fn from(i: SvKillMsg) -> Game<'a> {
        Game::SvKillMsg(i)
    }
}

impl<'a> From<SvSoundGlobal> for Game<'a> {
    fn from(i: SvSoundGlobal) -> Game<'a> {
        Game::SvSoundGlobal(i)
    }
}

impl<'a> From<SvTuneParams> for Game<'a> {
    fn from(i: SvTuneParams) -> Game<'a> {
        Game::SvTuneParams(i)
    }
}

impl<'a> From<SvExtraProjectile> for Game<'a> {
    fn from(i: SvExtraProjectile) -> Game<'a> {
        Game::SvExtraProjectile(i)
    }
}

impl<'a> From<SvReadyToEnter> for Game<'a> {
    fn from(i: SvReadyToEnter) -> Game<'a> {
        Game::SvReadyToEnter(i)
    }
}

impl<'a> From<SvWeaponPickup> for Game<'a> {
    fn from(i: SvWeaponPickup) -> Game<'a> {
        Game::SvWeaponPickup(i)
    }
}

impl<'a> From<SvEmoticon> for Game<'a> {
    fn from(i: SvEmoticon) -> Game<'a> {
        Game::SvEmoticon(i)
    }
}

impl<'a> From<SvVoteClearOptions> for Game<'a> {
    fn from(i: SvVoteClearOptions) -> Game<'a> {
        Game::SvVoteClearOptions(i)
    }
}

impl<'a> From<SvVoteOption<'a>> for Game<'a> {
    fn from(i: SvVoteOption<'a>) -> Game<'a> {
        Game::SvVoteOption(i)
    }
}

impl<'a> From<SvVoteSet<'a>> for Game<'a> {
    fn from(i: SvVoteSet<'a>) -> Game<'a> {
        Game::SvVoteSet(i)
    }
}

impl<'a> From<SvVoteStatus> for Game<'a> {
    fn from(i: SvVoteStatus) -> Game<'a> {
        Game::SvVoteStatus(i)
    }
}

impl<'a> From<ClSay<'a>> for Game<'a> {
    fn from(i: ClSay<'a>) -> Game<'a> {
        Game::ClSay(i)
    }
}

impl<'a> From<ClSetTeam> for Game<'a> {
    fn from(i: ClSetTeam) -> Game<'a> {
        Game::ClSetTeam(i)
    }
}

impl<'a> From<ClStartInfo<'a>> for Game<'a> {
    fn from(i: ClStartInfo<'a>) -> Game<'a> {
        Game::ClStartInfo(i)
    }
}

impl<'a> From<ClChangeInfo<'a>> for Game<'a> {
    fn from(i: ClChangeInfo<'a>) -> Game<'a> {
        Game::ClChangeInfo(i)
    }
}

impl<'a> From<ClKill> for Game<'a> {
    fn from(i: ClKill) -> Game<'a> {
        Game::ClKill(i)
    }
}

impl<'a> From<ClEmoticon> for Game<'a> {
    fn from(i: ClEmoticon) -> Game<'a> {
        Game::ClEmoticon(i)
    }
}

impl<'a> From<ClVote> for Game<'a> {
    fn from(i: ClVote) -> Game<'a> {
        Game::ClVote(i)
    }
}

impl<'a> From<ClCallVote<'a>> for Game<'a> {
    fn from(i: ClCallVote<'a>) -> Game<'a> {
        Game::ClCallVote(i)
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
    pub team: bool,
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
    pub sound_id: enums::Sound,
}

#[derive(Clone, Copy)]
pub struct SvTuneParams {
    pub ground_control_speed: TuneParam,
    pub ground_control_accel: TuneParam,
    pub ground_friction: TuneParam,
    pub ground_jump_impulse: TuneParam,
    pub air_jump_impulse: TuneParam,
    pub air_control_speed: TuneParam,
    pub air_control_accel: TuneParam,
    pub air_friction: TuneParam,
    pub hook_length: TuneParam,
    pub hook_fire_speed: TuneParam,
    pub hook_drag_accel: TuneParam,
    pub hook_drag_speed: TuneParam,
    pub gravity: TuneParam,
    pub velramp_start: TuneParam,
    pub velramp_range: TuneParam,
    pub velramp_curvature: TuneParam,
    pub gun_curvature: TuneParam,
    pub gun_speed: TuneParam,
    pub gun_lifetime: TuneParam,
    pub shotgun_curvature: TuneParam,
    pub shotgun_speed: TuneParam,
    pub shotgun_speeddiff: TuneParam,
    pub shotgun_lifetime: TuneParam,
    pub grenade_curvature: TuneParam,
    pub grenade_speed: TuneParam,
    pub grenade_lifetime: TuneParam,
    pub laser_reach: TuneParam,
    pub laser_bounce_delay: TuneParam,
    pub laser_bounce_num: TuneParam,
    pub laser_bounce_cost: TuneParam,
    pub laser_damage: TuneParam,
    pub player_collision: TuneParam,
    pub player_hooking: TuneParam,
}

#[derive(Clone, Copy)]
pub struct SvExtraProjectile {
    pub projectile: crate::snap_obj::Projectile,
}

#[derive(Clone, Copy)]
pub struct SvReadyToEnter;

#[derive(Clone, Copy)]
pub struct SvWeaponPickup {
    pub weapon: enums::Weapon,
}

#[derive(Clone, Copy)]
pub struct SvEmoticon {
    pub client_id: i32,
    pub emoticon: enums::Emoticon,
}

#[derive(Clone, Copy)]
pub struct SvVoteClearOptions;

#[derive(Clone, Copy)]
pub struct SvVoteOption<'a> {
    pub command: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvVoteSet<'a> {
    pub timeout: i32,
    pub description: &'a [u8],
    pub command: &'a [u8],
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
    pub team: enums::Team,
}

#[derive(Clone, Copy)]
pub struct ClStartInfo<'a> {
    pub name: &'a [u8],
    pub skin: &'a [u8],
    pub use_custom_color: bool,
    pub color_body: i32,
    pub color_feet: i32,
}

#[derive(Clone, Copy)]
pub struct ClChangeInfo<'a> {
    pub name: &'a [u8],
    pub skin: &'a [u8],
    pub use_custom_color: bool,
    pub color_body: i32,
    pub color_feet: i32,
}

#[derive(Clone, Copy)]
pub struct ClKill;

#[derive(Clone, Copy)]
pub struct ClEmoticon {
    pub emoticon: enums::Emoticon,
}

#[derive(Clone, Copy)]
pub struct ClVote {
    pub vote: i32,
}

#[derive(Clone, Copy)]
pub struct ClCallVote<'a> {
    pub type_: &'a [u8],
    pub value: &'a [u8],
}

impl<'a> SvMotd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvMotd<'a>, Error> {
        let result = Ok(SvMotd {
            message: _p.read_string()?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.message)?;
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
            message: _p.read_string()?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.message)?;
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
            team: to_bool(_p.read_int(warn)?)?,
            client_id: in_range(_p.read_int(warn)?, -1, 15)?,
            message: _p.read_string()?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.client_id && self.client_id <= 15);
        _p.write_int(self.team as i32)?;
        _p.write_int(self.client_id)?;
        _p.write_string(self.message)?;
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
            killer: in_range(_p.read_int(warn)?, 0, 15)?,
            victim: in_range(_p.read_int(warn)?, 0, 15)?,
            weapon: in_range(_p.read_int(warn)?, -3, 5)?,
            mode_special: _p.read_int(warn)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.killer && self.killer <= 15);
        assert!(0 <= self.victim && self.victim <= 15);
        assert!(-3 <= self.weapon && self.weapon <= 5);
        _p.write_int(self.killer)?;
        _p.write_int(self.victim)?;
        _p.write_int(self.weapon)?;
        _p.write_int(self.mode_special)?;
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
            sound_id: enums::Sound::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.sound_id.to_i32())?;
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
            ground_control_speed: TuneParam(_p.read_int(warn)?),
            ground_control_accel: TuneParam(_p.read_int(warn)?),
            ground_friction: TuneParam(_p.read_int(warn)?),
            ground_jump_impulse: TuneParam(_p.read_int(warn)?),
            air_jump_impulse: TuneParam(_p.read_int(warn)?),
            air_control_speed: TuneParam(_p.read_int(warn)?),
            air_control_accel: TuneParam(_p.read_int(warn)?),
            air_friction: TuneParam(_p.read_int(warn)?),
            hook_length: TuneParam(_p.read_int(warn)?),
            hook_fire_speed: TuneParam(_p.read_int(warn)?),
            hook_drag_accel: TuneParam(_p.read_int(warn)?),
            hook_drag_speed: TuneParam(_p.read_int(warn)?),
            gravity: TuneParam(_p.read_int(warn)?),
            velramp_start: TuneParam(_p.read_int(warn)?),
            velramp_range: TuneParam(_p.read_int(warn)?),
            velramp_curvature: TuneParam(_p.read_int(warn)?),
            gun_curvature: TuneParam(_p.read_int(warn)?),
            gun_speed: TuneParam(_p.read_int(warn)?),
            gun_lifetime: TuneParam(_p.read_int(warn)?),
            shotgun_curvature: TuneParam(_p.read_int(warn)?),
            shotgun_speed: TuneParam(_p.read_int(warn)?),
            shotgun_speeddiff: TuneParam(_p.read_int(warn)?),
            shotgun_lifetime: TuneParam(_p.read_int(warn)?),
            grenade_curvature: TuneParam(_p.read_int(warn)?),
            grenade_speed: TuneParam(_p.read_int(warn)?),
            grenade_lifetime: TuneParam(_p.read_int(warn)?),
            laser_reach: TuneParam(_p.read_int(warn)?),
            laser_bounce_delay: TuneParam(_p.read_int(warn)?),
            laser_bounce_num: TuneParam(_p.read_int(warn)?),
            laser_bounce_cost: TuneParam(_p.read_int(warn)?),
            laser_damage: TuneParam(_p.read_int(warn)?),
            player_collision: TuneParam(_p.read_int(warn)?),
            player_hooking: TuneParam(_p.read_int(warn)?),
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.ground_control_speed.0)?;
        _p.write_int(self.ground_control_accel.0)?;
        _p.write_int(self.ground_friction.0)?;
        _p.write_int(self.ground_jump_impulse.0)?;
        _p.write_int(self.air_jump_impulse.0)?;
        _p.write_int(self.air_control_speed.0)?;
        _p.write_int(self.air_control_accel.0)?;
        _p.write_int(self.air_friction.0)?;
        _p.write_int(self.hook_length.0)?;
        _p.write_int(self.hook_fire_speed.0)?;
        _p.write_int(self.hook_drag_accel.0)?;
        _p.write_int(self.hook_drag_speed.0)?;
        _p.write_int(self.gravity.0)?;
        _p.write_int(self.velramp_start.0)?;
        _p.write_int(self.velramp_range.0)?;
        _p.write_int(self.velramp_curvature.0)?;
        _p.write_int(self.gun_curvature.0)?;
        _p.write_int(self.gun_speed.0)?;
        _p.write_int(self.gun_lifetime.0)?;
        _p.write_int(self.shotgun_curvature.0)?;
        _p.write_int(self.shotgun_speed.0)?;
        _p.write_int(self.shotgun_speeddiff.0)?;
        _p.write_int(self.shotgun_lifetime.0)?;
        _p.write_int(self.grenade_curvature.0)?;
        _p.write_int(self.grenade_speed.0)?;
        _p.write_int(self.grenade_lifetime.0)?;
        _p.write_int(self.laser_reach.0)?;
        _p.write_int(self.laser_bounce_delay.0)?;
        _p.write_int(self.laser_bounce_num.0)?;
        _p.write_int(self.laser_bounce_cost.0)?;
        _p.write_int(self.laser_damage.0)?;
        _p.write_int(self.player_collision.0)?;
        _p.write_int(self.player_hooking.0)?;
        Ok(_p.written())
    }
}
pub const SV_TUNE_PARAMS_DEFAULT: SvTuneParams = SvTuneParams {
    ground_control_speed: TuneParam(1000),
    ground_control_accel: TuneParam(200),
    ground_friction: TuneParam(50),
    ground_jump_impulse: TuneParam(1320),
    air_jump_impulse: TuneParam(1200),
    air_control_speed: TuneParam(500),
    air_control_accel: TuneParam(150),
    air_friction: TuneParam(95),
    hook_length: TuneParam(38000),
    hook_fire_speed: TuneParam(8000),
    hook_drag_accel: TuneParam(300),
    hook_drag_speed: TuneParam(1500),
    gravity: TuneParam(50),
    velramp_start: TuneParam(55000),
    velramp_range: TuneParam(200000),
    velramp_curvature: TuneParam(140),
    gun_curvature: TuneParam(125),
    gun_speed: TuneParam(220000),
    gun_lifetime: TuneParam(200),
    shotgun_curvature: TuneParam(125),
    shotgun_speed: TuneParam(275000),
    shotgun_speeddiff: TuneParam(80),
    shotgun_lifetime: TuneParam(20),
    grenade_curvature: TuneParam(700),
    grenade_speed: TuneParam(100000),
    grenade_lifetime: TuneParam(200),
    laser_reach: TuneParam(80000),
    laser_bounce_delay: TuneParam(15000),
    laser_bounce_num: TuneParam(100),
    laser_bounce_cost: TuneParam(0),
    laser_damage: TuneParam(500),
    player_collision: TuneParam(100),
    player_hooking: TuneParam(100),
};

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
        let result = Ok(SvExtraProjectile {
            projectile: crate::snap_obj::Projectile::decode_msg(warn, _p)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        with_packer(&mut _p, |p| self.projectile.encode_msg(p))?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvExtraProjectile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvExtraProjectile")
            .field("projectile", &self.projectile)
            .finish()
    }
}

impl SvReadyToEnter {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvReadyToEnter, Error> {
        let result = Ok(SvReadyToEnter);
        _p.finish(wrap(warn));
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
            weapon: enums::Weapon::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.weapon.to_i32())?;
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
            client_id: in_range(_p.read_int(warn)?, 0, 15)?,
            emoticon: enums::Emoticon::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= 15);
        _p.write_int(self.client_id)?;
        _p.write_int(self.emoticon.to_i32())?;
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
        _p.finish(wrap(warn));
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

impl<'a> SvVoteOption<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteOption<'a>, Error> {
        let result = Ok(SvVoteOption {
            command: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.command).unwrap();
        _p.write_string(self.command)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvVoteOption<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteOption")
            .field("command", &pretty::Bytes::new(&self.command))
            .finish()
    }
}

impl<'a> SvVoteSet<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvVoteSet<'a>, Error> {
        let result = Ok(SvVoteSet {
            timeout: in_range(_p.read_int(warn)?, 0, 60)?,
            description: sanitize(warn, _p.read_string()?)?,
            command: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.timeout && self.timeout <= 60);
        sanitize(&mut Panic, self.description).unwrap();
        sanitize(&mut Panic, self.command).unwrap();
        _p.write_int(self.timeout)?;
        _p.write_string(self.description)?;
        _p.write_string(self.command)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvVoteSet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteSet")
            .field("timeout", &self.timeout)
            .field("description", &pretty::Bytes::new(&self.description))
            .field("command", &pretty::Bytes::new(&self.command))
            .finish()
    }
}

impl SvVoteStatus {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvVoteStatus, Error> {
        let result = Ok(SvVoteStatus {
            yes: in_range(_p.read_int(warn)?, 0, 16)?,
            no: in_range(_p.read_int(warn)?, 0, 16)?,
            pass: in_range(_p.read_int(warn)?, 0, 16)?,
            total: in_range(_p.read_int(warn)?, 0, 16)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.yes && self.yes <= 16);
        assert!(0 <= self.no && self.no <= 16);
        assert!(0 <= self.pass && self.pass <= 16);
        assert!(0 <= self.total && self.total <= 16);
        _p.write_int(self.yes)?;
        _p.write_int(self.no)?;
        _p.write_int(self.pass)?;
        _p.write_int(self.total)?;
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
            team: to_bool(_p.read_int(warn)?)?,
            message: _p.read_string()?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.team as i32)?;
        _p.write_string(self.message)?;
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
            team: enums::Team::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.team.to_i32())?;
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

impl<'a> ClStartInfo<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClStartInfo<'a>, Error> {
        let result = Ok(ClStartInfo {
            name: sanitize(warn, _p.read_string()?)?,
            skin: sanitize(warn, _p.read_string()?)?,
            use_custom_color: to_bool(_p.read_int(warn)?)?,
            color_body: _p.read_int(warn)?,
            color_feet: _p.read_int(warn)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.skin).unwrap();
        _p.write_string(self.name)?;
        _p.write_string(self.skin)?;
        _p.write_int(self.use_custom_color as i32)?;
        _p.write_int(self.color_body)?;
        _p.write_int(self.color_feet)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClStartInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClStartInfo")
            .field("name", &pretty::Bytes::new(&self.name))
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
            name: sanitize(warn, _p.read_string()?)?,
            skin: sanitize(warn, _p.read_string()?)?,
            use_custom_color: to_bool(_p.read_int(warn)?)?,
            color_body: _p.read_int(warn)?,
            color_feet: _p.read_int(warn)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.skin).unwrap();
        _p.write_string(self.name)?;
        _p.write_string(self.skin)?;
        _p.write_int(self.use_custom_color as i32)?;
        _p.write_int(self.color_body)?;
        _p.write_int(self.color_feet)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClChangeInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClChangeInfo")
            .field("name", &pretty::Bytes::new(&self.name))
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
        _p.finish(wrap(warn));
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
            emoticon: enums::Emoticon::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.emoticon.to_i32())?;
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
            vote: in_range(_p.read_int(warn)?, -1, 1)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.vote && self.vote <= 1);
        _p.write_int(self.vote)?;
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
            type_: sanitize(warn, _p.read_string()?)?,
            value: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(wrap(warn));
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.type_).unwrap();
        sanitize(&mut Panic, self.value).unwrap();
        _p.write_string(self.type_)?;
        _p.write_string(self.value)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClCallVote<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClCallVote")
            .field("type_", &pretty::Bytes::new(&self.type_))
            .field("value", &pretty::Bytes::new(&self.value))
            .finish()
    }
}

