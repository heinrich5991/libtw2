use buffer::CapacityError;
use common::pretty;
use enums;
use error::Error;
use gamenet_common::debug::DebugSlice;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::in_range;
use packer::sanitize;
use packer::to_bool;
use packer::with_packer;
use std::fmt;
use super::MessageId;
use super::SystemOrGame;
use uuid::Uuid;
use warn::Panic;
use warn::Warn;

pub use gamenet_common::msg::TuneParam;

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
pub const CL_IS_DDNET_LEGACY: i32 = 26;
pub const SV_DDRACE_TIME_LEGACY: i32 = 27;
pub const SV_RECORD_LEGACY: i32 = 28;
pub const UNUSED: i32 = 29;
pub const SV_TEAMS_STATE_LEGACY: i32 = 30;
pub const CL_SHOW_OTHERS_LEGACY: i32 = 31;
pub const SV_MY_OWN_MESSAGE: Uuid = Uuid::from_u128(0x1231e484_f607_3722_a89a_bd85db46f5d2);
pub const CL_SHOW_DISTANCE: Uuid = Uuid::from_u128(0x53bb28af_4252_3ac9_8fd3_6ccbc2a603e3);
pub const CL_SHOW_OTHERS: Uuid = Uuid::from_u128(0x7f264cdd_71a2_3962_bbce_0f94bbd81913);
pub const SV_TEAMS_STATE: Uuid = Uuid::from_u128(0xa091961a_95e8_3744_bb60_5eac9bd563c6);
pub const SV_DDRACE_TIME: Uuid = Uuid::from_u128(0x5dde8b3c_6f6f_37ac_a72a_bb341fe76de5);
pub const SV_RECORD: Uuid = Uuid::from_u128(0x804f149f_9b53_3b0a_897f_59663a1c4eb9);

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
    ClIsDdnetLegacy(ClIsDdnetLegacy),
    SvDdraceTimeLegacy(SvDdraceTimeLegacy),
    SvRecordLegacy(SvRecordLegacy),
    Unused(Unused),
    SvTeamsStateLegacy(SvTeamsStateLegacy),
    ClShowOthersLegacy(ClShowOthersLegacy),
    SvMyOwnMessage(SvMyOwnMessage),
    ClShowDistance(ClShowDistance),
    ClShowOthers(ClShowOthers),
    SvTeamsState(SvTeamsState),
    SvDdraceTime(SvDdraceTime),
    SvRecord(SvRecord),
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
            Ordinal(SV_VOTE_OPTION_LIST_ADD) => Game::SvVoteOptionListAdd(SvVoteOptionListAdd::decode(warn, _p)?),
            Ordinal(SV_VOTE_OPTION_ADD) => Game::SvVoteOptionAdd(SvVoteOptionAdd::decode(warn, _p)?),
            Ordinal(SV_VOTE_OPTION_REMOVE) => Game::SvVoteOptionRemove(SvVoteOptionRemove::decode(warn, _p)?),
            Ordinal(SV_VOTE_SET) => Game::SvVoteSet(SvVoteSet::decode(warn, _p)?),
            Ordinal(SV_VOTE_STATUS) => Game::SvVoteStatus(SvVoteStatus::decode(warn, _p)?),
            Ordinal(CL_SAY) => Game::ClSay(ClSay::decode(warn, _p)?),
            Ordinal(CL_SET_TEAM) => Game::ClSetTeam(ClSetTeam::decode(warn, _p)?),
            Ordinal(CL_SET_SPECTATOR_MODE) => Game::ClSetSpectatorMode(ClSetSpectatorMode::decode(warn, _p)?),
            Ordinal(CL_START_INFO) => Game::ClStartInfo(ClStartInfo::decode(warn, _p)?),
            Ordinal(CL_CHANGE_INFO) => Game::ClChangeInfo(ClChangeInfo::decode(warn, _p)?),
            Ordinal(CL_KILL) => Game::ClKill(ClKill::decode(warn, _p)?),
            Ordinal(CL_EMOTICON) => Game::ClEmoticon(ClEmoticon::decode(warn, _p)?),
            Ordinal(CL_VOTE) => Game::ClVote(ClVote::decode(warn, _p)?),
            Ordinal(CL_CALL_VOTE) => Game::ClCallVote(ClCallVote::decode(warn, _p)?),
            Ordinal(CL_IS_DDNET_LEGACY) => Game::ClIsDdnetLegacy(ClIsDdnetLegacy::decode(warn, _p)?),
            Ordinal(SV_DDRACE_TIME_LEGACY) => Game::SvDdraceTimeLegacy(SvDdraceTimeLegacy::decode(warn, _p)?),
            Ordinal(SV_RECORD_LEGACY) => Game::SvRecordLegacy(SvRecordLegacy::decode(warn, _p)?),
            Ordinal(UNUSED) => Game::Unused(Unused::decode(warn, _p)?),
            Ordinal(SV_TEAMS_STATE_LEGACY) => Game::SvTeamsStateLegacy(SvTeamsStateLegacy::decode(warn, _p)?),
            Ordinal(CL_SHOW_OTHERS_LEGACY) => Game::ClShowOthersLegacy(ClShowOthersLegacy::decode(warn, _p)?),
            Uuid(SV_MY_OWN_MESSAGE) => Game::SvMyOwnMessage(SvMyOwnMessage::decode(warn, _p)?),
            Uuid(CL_SHOW_DISTANCE) => Game::ClShowDistance(ClShowDistance::decode(warn, _p)?),
            Uuid(CL_SHOW_OTHERS) => Game::ClShowOthers(ClShowOthers::decode(warn, _p)?),
            Uuid(SV_TEAMS_STATE) => Game::SvTeamsState(SvTeamsState::decode(warn, _p)?),
            Uuid(SV_DDRACE_TIME) => Game::SvDdraceTime(SvDdraceTime::decode(warn, _p)?),
            Uuid(SV_RECORD) => Game::SvRecord(SvRecord::decode(warn, _p)?),
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
            Game::SvVoteOptionListAdd(_) => MessageId::from(SV_VOTE_OPTION_LIST_ADD),
            Game::SvVoteOptionAdd(_) => MessageId::from(SV_VOTE_OPTION_ADD),
            Game::SvVoteOptionRemove(_) => MessageId::from(SV_VOTE_OPTION_REMOVE),
            Game::SvVoteSet(_) => MessageId::from(SV_VOTE_SET),
            Game::SvVoteStatus(_) => MessageId::from(SV_VOTE_STATUS),
            Game::ClSay(_) => MessageId::from(CL_SAY),
            Game::ClSetTeam(_) => MessageId::from(CL_SET_TEAM),
            Game::ClSetSpectatorMode(_) => MessageId::from(CL_SET_SPECTATOR_MODE),
            Game::ClStartInfo(_) => MessageId::from(CL_START_INFO),
            Game::ClChangeInfo(_) => MessageId::from(CL_CHANGE_INFO),
            Game::ClKill(_) => MessageId::from(CL_KILL),
            Game::ClEmoticon(_) => MessageId::from(CL_EMOTICON),
            Game::ClVote(_) => MessageId::from(CL_VOTE),
            Game::ClCallVote(_) => MessageId::from(CL_CALL_VOTE),
            Game::ClIsDdnetLegacy(_) => MessageId::from(CL_IS_DDNET_LEGACY),
            Game::SvDdraceTimeLegacy(_) => MessageId::from(SV_DDRACE_TIME_LEGACY),
            Game::SvRecordLegacy(_) => MessageId::from(SV_RECORD_LEGACY),
            Game::Unused(_) => MessageId::from(UNUSED),
            Game::SvTeamsStateLegacy(_) => MessageId::from(SV_TEAMS_STATE_LEGACY),
            Game::ClShowOthersLegacy(_) => MessageId::from(CL_SHOW_OTHERS_LEGACY),
            Game::SvMyOwnMessage(_) => MessageId::from(SV_MY_OWN_MESSAGE),
            Game::ClShowDistance(_) => MessageId::from(CL_SHOW_DISTANCE),
            Game::ClShowOthers(_) => MessageId::from(CL_SHOW_OTHERS),
            Game::SvTeamsState(_) => MessageId::from(SV_TEAMS_STATE),
            Game::SvDdraceTime(_) => MessageId::from(SV_DDRACE_TIME),
            Game::SvRecord(_) => MessageId::from(SV_RECORD),
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
            Game::ClIsDdnetLegacy(ref i) => i.encode(p),
            Game::SvDdraceTimeLegacy(ref i) => i.encode(p),
            Game::SvRecordLegacy(ref i) => i.encode(p),
            Game::Unused(ref i) => i.encode(p),
            Game::SvTeamsStateLegacy(ref i) => i.encode(p),
            Game::ClShowOthersLegacy(ref i) => i.encode(p),
            Game::SvMyOwnMessage(ref i) => i.encode(p),
            Game::ClShowDistance(ref i) => i.encode(p),
            Game::ClShowOthers(ref i) => i.encode(p),
            Game::SvTeamsState(ref i) => i.encode(p),
            Game::SvDdraceTime(ref i) => i.encode(p),
            Game::SvRecord(ref i) => i.encode(p),
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
            Game::ClIsDdnetLegacy(ref i) => i.fmt(f),
            Game::SvDdraceTimeLegacy(ref i) => i.fmt(f),
            Game::SvRecordLegacy(ref i) => i.fmt(f),
            Game::Unused(ref i) => i.fmt(f),
            Game::SvTeamsStateLegacy(ref i) => i.fmt(f),
            Game::ClShowOthersLegacy(ref i) => i.fmt(f),
            Game::SvMyOwnMessage(ref i) => i.fmt(f),
            Game::ClShowDistance(ref i) => i.fmt(f),
            Game::ClShowOthers(ref i) => i.fmt(f),
            Game::SvTeamsState(ref i) => i.fmt(f),
            Game::SvDdraceTime(ref i) => i.fmt(f),
            Game::SvRecord(ref i) => i.fmt(f),
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

impl<'a> From<SvVoteOptionListAdd<'a>> for Game<'a> {
    fn from(i: SvVoteOptionListAdd<'a>) -> Game<'a> {
        Game::SvVoteOptionListAdd(i)
    }
}

impl<'a> From<SvVoteOptionAdd<'a>> for Game<'a> {
    fn from(i: SvVoteOptionAdd<'a>) -> Game<'a> {
        Game::SvVoteOptionAdd(i)
    }
}

impl<'a> From<SvVoteOptionRemove<'a>> for Game<'a> {
    fn from(i: SvVoteOptionRemove<'a>) -> Game<'a> {
        Game::SvVoteOptionRemove(i)
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

impl<'a> From<ClSetSpectatorMode> for Game<'a> {
    fn from(i: ClSetSpectatorMode) -> Game<'a> {
        Game::ClSetSpectatorMode(i)
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

impl<'a> From<ClIsDdnetLegacy> for Game<'a> {
    fn from(i: ClIsDdnetLegacy) -> Game<'a> {
        Game::ClIsDdnetLegacy(i)
    }
}

impl<'a> From<SvDdraceTimeLegacy> for Game<'a> {
    fn from(i: SvDdraceTimeLegacy) -> Game<'a> {
        Game::SvDdraceTimeLegacy(i)
    }
}

impl<'a> From<SvRecordLegacy> for Game<'a> {
    fn from(i: SvRecordLegacy) -> Game<'a> {
        Game::SvRecordLegacy(i)
    }
}

impl<'a> From<Unused> for Game<'a> {
    fn from(i: Unused) -> Game<'a> {
        Game::Unused(i)
    }
}

impl<'a> From<SvTeamsStateLegacy> for Game<'a> {
    fn from(i: SvTeamsStateLegacy) -> Game<'a> {
        Game::SvTeamsStateLegacy(i)
    }
}

impl<'a> From<ClShowOthersLegacy> for Game<'a> {
    fn from(i: ClShowOthersLegacy) -> Game<'a> {
        Game::ClShowOthersLegacy(i)
    }
}

impl<'a> From<SvMyOwnMessage> for Game<'a> {
    fn from(i: SvMyOwnMessage) -> Game<'a> {
        Game::SvMyOwnMessage(i)
    }
}

impl<'a> From<ClShowDistance> for Game<'a> {
    fn from(i: ClShowDistance) -> Game<'a> {
        Game::ClShowDistance(i)
    }
}

impl<'a> From<ClShowOthers> for Game<'a> {
    fn from(i: ClShowOthers) -> Game<'a> {
        Game::ClShowOthers(i)
    }
}

impl<'a> From<SvTeamsState> for Game<'a> {
    fn from(i: SvTeamsState) -> Game<'a> {
        Game::SvTeamsState(i)
    }
}

impl<'a> From<SvDdraceTime> for Game<'a> {
    fn from(i: SvDdraceTime) -> Game<'a> {
        Game::SvDdraceTime(i)
    }
}

impl<'a> From<SvRecord> for Game<'a> {
    fn from(i: SvRecord) -> Game<'a> {
        Game::SvRecord(i)
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
    pub jetpack_strength: TuneParam,
    pub shotgun_strength: TuneParam,
    pub explosion_strength: TuneParam,
    pub hammer_strength: TuneParam,
    pub hook_duration: TuneParam,
    pub hammer_fire_delay: TuneParam,
    pub gun_fire_delay: TuneParam,
    pub shotgun_fire_delay: TuneParam,
    pub grenade_fire_delay: TuneParam,
    pub laser_fire_delay: TuneParam,
    pub ninja_fire_delay: TuneParam,
    pub hammer_hit_fire_delay: TuneParam,
}

#[derive(Clone, Copy)]
pub struct SvExtraProjectile {
    pub projectile: ::snap_obj::Projectile,
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
    pub team: enums::Team,
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
    pub reason: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct ClIsDdnetLegacy;

#[derive(Clone, Copy)]
pub struct SvDdraceTimeLegacy {
    pub time: i32,
    pub check: i32,
    pub finish: i32,
}

#[derive(Clone, Copy)]
pub struct SvRecordLegacy {
    pub server_time_best: i32,
    pub player_time_best: i32,
}

#[derive(Clone, Copy)]
pub struct Unused;

#[derive(Clone, Copy)]
pub struct SvTeamsStateLegacy;

#[derive(Clone, Copy)]
pub struct ClShowOthersLegacy {
    pub show: bool,
}

#[derive(Clone, Copy)]
pub struct SvMyOwnMessage {
    pub test: i32,
}

#[derive(Clone, Copy)]
pub struct ClShowDistance {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy)]
pub struct ClShowOthers {
    pub show: i32,
}

#[derive(Clone, Copy)]
pub struct SvTeamsState;

#[derive(Clone, Copy)]
pub struct SvDdraceTime {
    pub time: i32,
    pub check: i32,
    pub finish: i32,
}

#[derive(Clone, Copy)]
pub struct SvRecord {
    pub server_time_best: i32,
    pub player_time_best: i32,
}

impl<'a> SvMotd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvMotd<'a>, Error> {
        let result = Ok(SvMotd {
            message: _p.read_string()?,
        });
        _p.finish(warn);
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
        _p.finish(warn);
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
            team: in_range(_p.read_int(warn)?, -2, 3)?,
            client_id: in_range(_p.read_int(warn)?, -1, 63)?,
            message: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-2 <= self.team && self.team <= 3);
        assert!(-1 <= self.client_id && self.client_id <= 63);
        sanitize(&mut Panic, self.message).unwrap();
        _p.write_int(self.team)?;
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
            killer: in_range(_p.read_int(warn)?, 0, 63)?,
            victim: in_range(_p.read_int(warn)?, 0, 63)?,
            weapon: in_range(_p.read_int(warn)?, -3, 5)?,
            mode_special: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.killer && self.killer <= 63);
        assert!(0 <= self.victim && self.victim <= 63);
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
        _p.finish(warn);
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
            jetpack_strength: TuneParam(_p.read_int(warn)?),
            shotgun_strength: TuneParam(_p.read_int(warn)?),
            explosion_strength: TuneParam(_p.read_int(warn)?),
            hammer_strength: TuneParam(_p.read_int(warn)?),
            hook_duration: TuneParam(_p.read_int(warn)?),
            hammer_fire_delay: TuneParam(_p.read_int(warn)?),
            gun_fire_delay: TuneParam(_p.read_int(warn)?),
            shotgun_fire_delay: TuneParam(_p.read_int(warn)?),
            grenade_fire_delay: TuneParam(_p.read_int(warn)?),
            laser_fire_delay: TuneParam(_p.read_int(warn)?),
            ninja_fire_delay: TuneParam(_p.read_int(warn)?),
            hammer_hit_fire_delay: TuneParam(_p.read_int(warn)?),
        });
        _p.finish(warn);
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
        _p.write_int(self.jetpack_strength.0)?;
        _p.write_int(self.shotgun_strength.0)?;
        _p.write_int(self.explosion_strength.0)?;
        _p.write_int(self.hammer_strength.0)?;
        _p.write_int(self.hook_duration.0)?;
        _p.write_int(self.hammer_fire_delay.0)?;
        _p.write_int(self.gun_fire_delay.0)?;
        _p.write_int(self.shotgun_fire_delay.0)?;
        _p.write_int(self.grenade_fire_delay.0)?;
        _p.write_int(self.laser_fire_delay.0)?;
        _p.write_int(self.ninja_fire_delay.0)?;
        _p.write_int(self.hammer_hit_fire_delay.0)?;
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
            .field("jetpack_strength", &self.jetpack_strength)
            .field("shotgun_strength", &self.shotgun_strength)
            .field("explosion_strength", &self.explosion_strength)
            .field("hammer_strength", &self.hammer_strength)
            .field("hook_duration", &self.hook_duration)
            .field("hammer_fire_delay", &self.hammer_fire_delay)
            .field("gun_fire_delay", &self.gun_fire_delay)
            .field("shotgun_fire_delay", &self.shotgun_fire_delay)
            .field("grenade_fire_delay", &self.grenade_fire_delay)
            .field("laser_fire_delay", &self.laser_fire_delay)
            .field("ninja_fire_delay", &self.ninja_fire_delay)
            .field("hammer_hit_fire_delay", &self.hammer_hit_fire_delay)
            .finish()
    }
}

impl SvExtraProjectile {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvExtraProjectile, Error> {
        let result = Ok(SvExtraProjectile {
            projectile: ::snap_obj::Projectile::decode_msg(warn, _p)?,
        });
        _p.finish(warn);
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
            weapon: enums::Weapon::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
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
            client_id: in_range(_p.read_int(warn)?, 0, 63)?,
            emoticon: enums::Emoticon::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= 63);
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
            num_options: in_range(_p.read_int(warn)?, 1, 15)?,
            description: [
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
            ],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(1 <= self.num_options && self.num_options <= 15);
        for &e in &self.description {
            sanitize(&mut Panic, e).unwrap();
        }
        _p.write_int(self.num_options)?;
        for &e in &self.description {
            _p.write_string(e)?;
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
            description: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.description).unwrap();
        _p.write_string(self.description)?;
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
            description: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.description).unwrap();
        _p.write_string(self.description)?;
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
            timeout: in_range(_p.read_int(warn)?, 0, 60)?,
            description: sanitize(warn, _p.read_string()?)?,
            reason: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.timeout && self.timeout <= 60);
        sanitize(&mut Panic, self.description).unwrap();
        sanitize(&mut Panic, self.reason).unwrap();
        _p.write_int(self.timeout)?;
        _p.write_string(self.description)?;
        _p.write_string(self.reason)?;
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
            yes: in_range(_p.read_int(warn)?, 0, 64)?,
            no: in_range(_p.read_int(warn)?, 0, 64)?,
            pass: in_range(_p.read_int(warn)?, 0, 64)?,
            total: in_range(_p.read_int(warn)?, 0, 64)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.yes && self.yes <= 64);
        assert!(0 <= self.no && self.no <= 64);
        assert!(0 <= self.pass && self.pass <= 64);
        assert!(0 <= self.total && self.total <= 64);
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
            message: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.message).unwrap();
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
        _p.finish(warn);
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

impl ClSetSpectatorMode {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClSetSpectatorMode, Error> {
        let result = Ok(ClSetSpectatorMode {
            spectator_id: in_range(_p.read_int(warn)?, -1, 63)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.spectator_id && self.spectator_id <= 63);
        _p.write_int(self.spectator_id)?;
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
            name: sanitize(warn, _p.read_string()?)?,
            clan: sanitize(warn, _p.read_string()?)?,
            country: _p.read_int(warn)?,
            skin: sanitize(warn, _p.read_string()?)?,
            use_custom_color: to_bool(_p.read_int(warn)?)?,
            color_body: _p.read_int(warn)?,
            color_feet: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.clan).unwrap();
        sanitize(&mut Panic, self.skin).unwrap();
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_int(self.country)?;
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
            name: sanitize(warn, _p.read_string()?)?,
            clan: sanitize(warn, _p.read_string()?)?,
            country: _p.read_int(warn)?,
            skin: sanitize(warn, _p.read_string()?)?,
            use_custom_color: to_bool(_p.read_int(warn)?)?,
            color_body: _p.read_int(warn)?,
            color_feet: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.clan).unwrap();
        sanitize(&mut Panic, self.skin).unwrap();
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_int(self.country)?;
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
            emoticon: enums::Emoticon::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
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
        _p.finish(warn);
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
            reason: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.type_).unwrap();
        sanitize(&mut Panic, self.value).unwrap();
        sanitize(&mut Panic, self.reason).unwrap();
        _p.write_string(self.type_)?;
        _p.write_string(self.value)?;
        _p.write_string(self.reason)?;
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

impl ClIsDdnetLegacy {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClIsDdnetLegacy, Error> {
        let result = Ok(ClIsDdnetLegacy);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ClIsDdnetLegacy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClIsDdnetLegacy")
            .finish()
    }
}

impl SvDdraceTimeLegacy {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvDdraceTimeLegacy, Error> {
        let result = Ok(SvDdraceTimeLegacy {
            time: _p.read_int(warn)?,
            check: _p.read_int(warn)?,
            finish: in_range(_p.read_int(warn)?, 0, 1)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.finish && self.finish <= 1);
        _p.write_int(self.time)?;
        _p.write_int(self.check)?;
        _p.write_int(self.finish)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvDdraceTimeLegacy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvDdraceTimeLegacy")
            .field("time", &self.time)
            .field("check", &self.check)
            .field("finish", &self.finish)
            .finish()
    }
}

impl SvRecordLegacy {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvRecordLegacy, Error> {
        let result = Ok(SvRecordLegacy {
            server_time_best: _p.read_int(warn)?,
            player_time_best: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.server_time_best)?;
        _p.write_int(self.player_time_best)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvRecordLegacy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvRecordLegacy")
            .field("server_time_best", &self.server_time_best)
            .field("player_time_best", &self.player_time_best)
            .finish()
    }
}

impl Unused {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Unused, Error> {
        let result = Ok(Unused);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for Unused {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Unused")
            .finish()
    }
}

impl SvTeamsStateLegacy {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvTeamsStateLegacy, Error> {
        let result = Ok(SvTeamsStateLegacy);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for SvTeamsStateLegacy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvTeamsStateLegacy")
            .finish()
    }
}

impl ClShowOthersLegacy {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClShowOthersLegacy, Error> {
        let result = Ok(ClShowOthersLegacy {
            show: to_bool(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.show as i32)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ClShowOthersLegacy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClShowOthersLegacy")
            .field("show", &self.show)
            .finish()
    }
}

impl SvMyOwnMessage {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvMyOwnMessage, Error> {
        let result = Ok(SvMyOwnMessage {
            test: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.test)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvMyOwnMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvMyOwnMessage")
            .field("test", &self.test)
            .finish()
    }
}

impl ClShowDistance {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClShowDistance, Error> {
        let result = Ok(ClShowDistance {
            x: _p.read_int(warn)?,
            y: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.x)?;
        _p.write_int(self.y)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ClShowDistance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClShowDistance")
            .field("x", &self.x)
            .field("y", &self.y)
            .finish()
    }
}

impl ClShowOthers {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClShowOthers, Error> {
        let result = Ok(ClShowOthers {
            show: in_range(_p.read_int(warn)?, 0, 2)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.show && self.show <= 2);
        _p.write_int(self.show)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ClShowOthers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClShowOthers")
            .field("show", &self.show)
            .finish()
    }
}

impl SvTeamsState {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvTeamsState, Error> {
        let result = Ok(SvTeamsState);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for SvTeamsState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvTeamsState")
            .finish()
    }
}

impl SvDdraceTime {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvDdraceTime, Error> {
        let result = Ok(SvDdraceTime {
            time: _p.read_int(warn)?,
            check: _p.read_int(warn)?,
            finish: in_range(_p.read_int(warn)?, 0, 1)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.finish && self.finish <= 1);
        _p.write_int(self.time)?;
        _p.write_int(self.check)?;
        _p.write_int(self.finish)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvDdraceTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvDdraceTime")
            .field("time", &self.time)
            .field("check", &self.check)
            .field("finish", &self.finish)
            .finish()
    }
}

impl SvRecord {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvRecord, Error> {
        let result = Ok(SvRecord {
            server_time_best: _p.read_int(warn)?,
            player_time_best: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.server_time_best)?;
        _p.write_int(self.player_time_best)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvRecord")
            .field("server_time_best", &self.server_time_best)
            .field("player_time_best", &self.player_time_best)
            .finish()
    }
}

