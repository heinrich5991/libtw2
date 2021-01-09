use buffer::CapacityError;
use common::pretty;
use enums;
use error::Error;
use gamenet_common::debug::DebugSlice;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::at_least;
use packer::in_range;
use packer::positive;
use packer::sanitize;
use packer::to_bool;
use packer::with_packer;
use std::fmt;
use super::MessageId;
use super::SystemOrGame;
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
pub const SV_TEAM: i32 = 4;
pub const SV_KILL_MSG: i32 = 5;
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
pub const SV_SERVER_SETTINGS: i32 = 17;
pub const SV_CLIENT_INFO: i32 = 18;
pub const SV_GAME_INFO: i32 = 19;
pub const SV_CLIENT_DROP: i32 = 20;
pub const SV_GAME_MSG: i32 = 21;
pub const DE_CLIENT_ENTER: i32 = 22;
pub const DE_CLIENT_LEAVE: i32 = 23;
pub const CL_SAY: i32 = 24;
pub const CL_SET_TEAM: i32 = 25;
pub const CL_SET_SPECTATOR_MODE: i32 = 26;
pub const CL_START_INFO: i32 = 27;
pub const CL_KILL: i32 = 28;
pub const CL_READY_CHANGE: i32 = 29;
pub const CL_EMOTICON: i32 = 30;
pub const CL_VOTE: i32 = 31;
pub const CL_CALL_VOTE: i32 = 32;
pub const SV_SKIN_CHANGE: i32 = 33;
pub const CL_SKIN_CHANGE: i32 = 34;
pub const SV_RACE_FINISH: i32 = 35;
pub const SV_CHECKPOINT: i32 = 36;
pub const SV_COMMAND_INFO: i32 = 37;
pub const SV_COMMAND_INFO_REMOVE: i32 = 38;
pub const CL_COMMAND: i32 = 39;

#[derive(Clone, Copy)]
pub enum Game<'a> {
    SvMotd(SvMotd<'a>),
    SvBroadcast(SvBroadcast<'a>),
    SvChat(SvChat<'a>),
    SvTeam(SvTeam),
    SvKillMsg(SvKillMsg),
    SvTuneParams(SvTuneParams),
    SvExtraProjectile(SvExtraProjectile),
    SvReadyToEnter(SvReadyToEnter),
    SvWeaponPickup(SvWeaponPickup),
    SvEmoticon(SvEmoticon),
    SvVoteClearOptions(SvVoteClearOptions),
    SvVoteOptionListAdd(SvVoteOptionListAdd),
    SvVoteOptionAdd(SvVoteOptionAdd<'a>),
    SvVoteOptionRemove(SvVoteOptionRemove<'a>),
    SvVoteSet(SvVoteSet<'a>),
    SvVoteStatus(SvVoteStatus),
    SvServerSettings(SvServerSettings),
    SvClientInfo(SvClientInfo<'a>),
    SvGameInfo(SvGameInfo),
    SvClientDrop(SvClientDrop<'a>),
    SvGameMsg(SvGameMsg),
    DeClientEnter(DeClientEnter<'a>),
    DeClientLeave(DeClientLeave<'a>),
    ClSay(ClSay<'a>),
    ClSetTeam(ClSetTeam),
    ClSetSpectatorMode(ClSetSpectatorMode),
    ClStartInfo(ClStartInfo<'a>),
    ClKill(ClKill),
    ClReadyChange(ClReadyChange),
    ClEmoticon(ClEmoticon),
    ClVote(ClVote),
    ClCallVote(ClCallVote<'a>),
    SvSkinChange(SvSkinChange<'a>),
    ClSkinChange(ClSkinChange<'a>),
    SvRaceFinish(SvRaceFinish),
    SvCheckpoint(SvCheckpoint),
    SvCommandInfo(SvCommandInfo<'a>),
    SvCommandInfoRemove(SvCommandInfoRemove<'a>),
    ClCommand(ClCommand<'a>),
}

impl<'a> Game<'a> {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: MessageId, _p: &mut Unpacker<'a>) -> Result<Game<'a>, Error> {
        use self::MessageId::*;
        Ok(match msg_id {
            Ordinal(SV_MOTD) => Game::SvMotd(SvMotd::decode(warn, _p)?),
            Ordinal(SV_BROADCAST) => Game::SvBroadcast(SvBroadcast::decode(warn, _p)?),
            Ordinal(SV_CHAT) => Game::SvChat(SvChat::decode(warn, _p)?),
            Ordinal(SV_TEAM) => Game::SvTeam(SvTeam::decode(warn, _p)?),
            Ordinal(SV_KILL_MSG) => Game::SvKillMsg(SvKillMsg::decode(warn, _p)?),
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
            Ordinal(SV_SERVER_SETTINGS) => Game::SvServerSettings(SvServerSettings::decode(warn, _p)?),
            Ordinal(SV_CLIENT_INFO) => Game::SvClientInfo(SvClientInfo::decode(warn, _p)?),
            Ordinal(SV_GAME_INFO) => Game::SvGameInfo(SvGameInfo::decode(warn, _p)?),
            Ordinal(SV_CLIENT_DROP) => Game::SvClientDrop(SvClientDrop::decode(warn, _p)?),
            Ordinal(SV_GAME_MSG) => Game::SvGameMsg(SvGameMsg::decode(warn, _p)?),
            Ordinal(DE_CLIENT_ENTER) => Game::DeClientEnter(DeClientEnter::decode(warn, _p)?),
            Ordinal(DE_CLIENT_LEAVE) => Game::DeClientLeave(DeClientLeave::decode(warn, _p)?),
            Ordinal(CL_SAY) => Game::ClSay(ClSay::decode(warn, _p)?),
            Ordinal(CL_SET_TEAM) => Game::ClSetTeam(ClSetTeam::decode(warn, _p)?),
            Ordinal(CL_SET_SPECTATOR_MODE) => Game::ClSetSpectatorMode(ClSetSpectatorMode::decode(warn, _p)?),
            Ordinal(CL_START_INFO) => Game::ClStartInfo(ClStartInfo::decode(warn, _p)?),
            Ordinal(CL_KILL) => Game::ClKill(ClKill::decode(warn, _p)?),
            Ordinal(CL_READY_CHANGE) => Game::ClReadyChange(ClReadyChange::decode(warn, _p)?),
            Ordinal(CL_EMOTICON) => Game::ClEmoticon(ClEmoticon::decode(warn, _p)?),
            Ordinal(CL_VOTE) => Game::ClVote(ClVote::decode(warn, _p)?),
            Ordinal(CL_CALL_VOTE) => Game::ClCallVote(ClCallVote::decode(warn, _p)?),
            Ordinal(SV_SKIN_CHANGE) => Game::SvSkinChange(SvSkinChange::decode(warn, _p)?),
            Ordinal(CL_SKIN_CHANGE) => Game::ClSkinChange(ClSkinChange::decode(warn, _p)?),
            Ordinal(SV_RACE_FINISH) => Game::SvRaceFinish(SvRaceFinish::decode(warn, _p)?),
            Ordinal(SV_CHECKPOINT) => Game::SvCheckpoint(SvCheckpoint::decode(warn, _p)?),
            Ordinal(SV_COMMAND_INFO) => Game::SvCommandInfo(SvCommandInfo::decode(warn, _p)?),
            Ordinal(SV_COMMAND_INFO_REMOVE) => Game::SvCommandInfoRemove(SvCommandInfoRemove::decode(warn, _p)?),
            Ordinal(CL_COMMAND) => Game::ClCommand(ClCommand::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn msg_id(&self) -> MessageId {
        match *self {
            Game::SvMotd(_) => MessageId::from(SV_MOTD),
            Game::SvBroadcast(_) => MessageId::from(SV_BROADCAST),
            Game::SvChat(_) => MessageId::from(SV_CHAT),
            Game::SvTeam(_) => MessageId::from(SV_TEAM),
            Game::SvKillMsg(_) => MessageId::from(SV_KILL_MSG),
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
            Game::SvServerSettings(_) => MessageId::from(SV_SERVER_SETTINGS),
            Game::SvClientInfo(_) => MessageId::from(SV_CLIENT_INFO),
            Game::SvGameInfo(_) => MessageId::from(SV_GAME_INFO),
            Game::SvClientDrop(_) => MessageId::from(SV_CLIENT_DROP),
            Game::SvGameMsg(_) => MessageId::from(SV_GAME_MSG),
            Game::DeClientEnter(_) => MessageId::from(DE_CLIENT_ENTER),
            Game::DeClientLeave(_) => MessageId::from(DE_CLIENT_LEAVE),
            Game::ClSay(_) => MessageId::from(CL_SAY),
            Game::ClSetTeam(_) => MessageId::from(CL_SET_TEAM),
            Game::ClSetSpectatorMode(_) => MessageId::from(CL_SET_SPECTATOR_MODE),
            Game::ClStartInfo(_) => MessageId::from(CL_START_INFO),
            Game::ClKill(_) => MessageId::from(CL_KILL),
            Game::ClReadyChange(_) => MessageId::from(CL_READY_CHANGE),
            Game::ClEmoticon(_) => MessageId::from(CL_EMOTICON),
            Game::ClVote(_) => MessageId::from(CL_VOTE),
            Game::ClCallVote(_) => MessageId::from(CL_CALL_VOTE),
            Game::SvSkinChange(_) => MessageId::from(SV_SKIN_CHANGE),
            Game::ClSkinChange(_) => MessageId::from(CL_SKIN_CHANGE),
            Game::SvRaceFinish(_) => MessageId::from(SV_RACE_FINISH),
            Game::SvCheckpoint(_) => MessageId::from(SV_CHECKPOINT),
            Game::SvCommandInfo(_) => MessageId::from(SV_COMMAND_INFO),
            Game::SvCommandInfoRemove(_) => MessageId::from(SV_COMMAND_INFO_REMOVE),
            Game::ClCommand(_) => MessageId::from(CL_COMMAND),
        }
    }
    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            Game::SvMotd(ref i) => i.encode(p),
            Game::SvBroadcast(ref i) => i.encode(p),
            Game::SvChat(ref i) => i.encode(p),
            Game::SvTeam(ref i) => i.encode(p),
            Game::SvKillMsg(ref i) => i.encode(p),
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
            Game::SvServerSettings(ref i) => i.encode(p),
            Game::SvClientInfo(ref i) => i.encode(p),
            Game::SvGameInfo(ref i) => i.encode(p),
            Game::SvClientDrop(ref i) => i.encode(p),
            Game::SvGameMsg(ref i) => i.encode(p),
            Game::DeClientEnter(ref i) => i.encode(p),
            Game::DeClientLeave(ref i) => i.encode(p),
            Game::ClSay(ref i) => i.encode(p),
            Game::ClSetTeam(ref i) => i.encode(p),
            Game::ClSetSpectatorMode(ref i) => i.encode(p),
            Game::ClStartInfo(ref i) => i.encode(p),
            Game::ClKill(ref i) => i.encode(p),
            Game::ClReadyChange(ref i) => i.encode(p),
            Game::ClEmoticon(ref i) => i.encode(p),
            Game::ClVote(ref i) => i.encode(p),
            Game::ClCallVote(ref i) => i.encode(p),
            Game::SvSkinChange(ref i) => i.encode(p),
            Game::ClSkinChange(ref i) => i.encode(p),
            Game::SvRaceFinish(ref i) => i.encode(p),
            Game::SvCheckpoint(ref i) => i.encode(p),
            Game::SvCommandInfo(ref i) => i.encode(p),
            Game::SvCommandInfoRemove(ref i) => i.encode(p),
            Game::ClCommand(ref i) => i.encode(p),
        }
    }
}

impl<'a> fmt::Debug for Game<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Game::SvMotd(ref i) => i.fmt(f),
            Game::SvBroadcast(ref i) => i.fmt(f),
            Game::SvChat(ref i) => i.fmt(f),
            Game::SvTeam(ref i) => i.fmt(f),
            Game::SvKillMsg(ref i) => i.fmt(f),
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
            Game::SvServerSettings(ref i) => i.fmt(f),
            Game::SvClientInfo(ref i) => i.fmt(f),
            Game::SvGameInfo(ref i) => i.fmt(f),
            Game::SvClientDrop(ref i) => i.fmt(f),
            Game::SvGameMsg(ref i) => i.fmt(f),
            Game::DeClientEnter(ref i) => i.fmt(f),
            Game::DeClientLeave(ref i) => i.fmt(f),
            Game::ClSay(ref i) => i.fmt(f),
            Game::ClSetTeam(ref i) => i.fmt(f),
            Game::ClSetSpectatorMode(ref i) => i.fmt(f),
            Game::ClStartInfo(ref i) => i.fmt(f),
            Game::ClKill(ref i) => i.fmt(f),
            Game::ClReadyChange(ref i) => i.fmt(f),
            Game::ClEmoticon(ref i) => i.fmt(f),
            Game::ClVote(ref i) => i.fmt(f),
            Game::ClCallVote(ref i) => i.fmt(f),
            Game::SvSkinChange(ref i) => i.fmt(f),
            Game::ClSkinChange(ref i) => i.fmt(f),
            Game::SvRaceFinish(ref i) => i.fmt(f),
            Game::SvCheckpoint(ref i) => i.fmt(f),
            Game::SvCommandInfo(ref i) => i.fmt(f),
            Game::SvCommandInfoRemove(ref i) => i.fmt(f),
            Game::ClCommand(ref i) => i.fmt(f),
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

impl<'a> From<SvTeam> for Game<'a> {
    fn from(i: SvTeam) -> Game<'a> {
        Game::SvTeam(i)
    }
}

impl<'a> From<SvKillMsg> for Game<'a> {
    fn from(i: SvKillMsg) -> Game<'a> {
        Game::SvKillMsg(i)
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

impl<'a> From<SvVoteOptionListAdd> for Game<'a> {
    fn from(i: SvVoteOptionListAdd) -> Game<'a> {
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

impl<'a> From<SvServerSettings> for Game<'a> {
    fn from(i: SvServerSettings) -> Game<'a> {
        Game::SvServerSettings(i)
    }
}

impl<'a> From<SvClientInfo<'a>> for Game<'a> {
    fn from(i: SvClientInfo<'a>) -> Game<'a> {
        Game::SvClientInfo(i)
    }
}

impl<'a> From<SvGameInfo> for Game<'a> {
    fn from(i: SvGameInfo) -> Game<'a> {
        Game::SvGameInfo(i)
    }
}

impl<'a> From<SvClientDrop<'a>> for Game<'a> {
    fn from(i: SvClientDrop<'a>) -> Game<'a> {
        Game::SvClientDrop(i)
    }
}

impl<'a> From<SvGameMsg> for Game<'a> {
    fn from(i: SvGameMsg) -> Game<'a> {
        Game::SvGameMsg(i)
    }
}

impl<'a> From<DeClientEnter<'a>> for Game<'a> {
    fn from(i: DeClientEnter<'a>) -> Game<'a> {
        Game::DeClientEnter(i)
    }
}

impl<'a> From<DeClientLeave<'a>> for Game<'a> {
    fn from(i: DeClientLeave<'a>) -> Game<'a> {
        Game::DeClientLeave(i)
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

impl<'a> From<ClKill> for Game<'a> {
    fn from(i: ClKill) -> Game<'a> {
        Game::ClKill(i)
    }
}

impl<'a> From<ClReadyChange> for Game<'a> {
    fn from(i: ClReadyChange) -> Game<'a> {
        Game::ClReadyChange(i)
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

impl<'a> From<SvSkinChange<'a>> for Game<'a> {
    fn from(i: SvSkinChange<'a>) -> Game<'a> {
        Game::SvSkinChange(i)
    }
}

impl<'a> From<ClSkinChange<'a>> for Game<'a> {
    fn from(i: ClSkinChange<'a>) -> Game<'a> {
        Game::ClSkinChange(i)
    }
}

impl<'a> From<SvRaceFinish> for Game<'a> {
    fn from(i: SvRaceFinish) -> Game<'a> {
        Game::SvRaceFinish(i)
    }
}

impl<'a> From<SvCheckpoint> for Game<'a> {
    fn from(i: SvCheckpoint) -> Game<'a> {
        Game::SvCheckpoint(i)
    }
}

impl<'a> From<SvCommandInfo<'a>> for Game<'a> {
    fn from(i: SvCommandInfo<'a>) -> Game<'a> {
        Game::SvCommandInfo(i)
    }
}

impl<'a> From<SvCommandInfoRemove<'a>> for Game<'a> {
    fn from(i: SvCommandInfoRemove<'a>) -> Game<'a> {
        Game::SvCommandInfoRemove(i)
    }
}

impl<'a> From<ClCommand<'a>> for Game<'a> {
    fn from(i: ClCommand<'a>) -> Game<'a> {
        Game::ClCommand(i)
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
    pub mode: enums::Chat,
    pub client_id: i32,
    pub target_id: i32,
    pub message: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvTeam {
    pub client_id: i32,
    pub team: enums::Team,
    pub silent: bool,
    pub cooldown_tick: ::snap_obj::Tick,
}

#[derive(Clone, Copy)]
pub struct SvKillMsg {
    pub killer: i32,
    pub victim: i32,
    pub weapon: i32,
    pub mode_special: i32,
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
    pub player_collision: TuneParam,
    pub player_hooking: TuneParam,
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
pub struct SvVoteOptionListAdd;

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
    pub client_id: i32,
    pub type_: enums::Vote,
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
pub struct SvServerSettings {
    pub kick_vote: bool,
    pub kick_min: i32,
    pub spec_vote: bool,
    pub team_lock: bool,
    pub team_balance: bool,
    pub player_slots: i32,
}

#[derive(Clone, Copy)]
pub struct SvClientInfo<'a> {
    pub client_id: i32,
    pub local: bool,
    pub team: enums::Team,
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub skin_part_names: [&'a [u8]; 6],
    pub use_custom_colors: [bool; 6],
    pub skin_part_colors: [i32; 6],
    pub silent: bool,
}

#[derive(Clone, Copy)]
pub struct SvGameInfo {
    pub game_flags: i32,
    pub score_limit: i32,
    pub time_limit: i32,
    pub match_num: i32,
    pub match_current: i32,
}

#[derive(Clone, Copy)]
pub struct SvClientDrop<'a> {
    pub client_id: i32,
    pub reason: &'a [u8],
    pub silent: bool,
}

#[derive(Clone, Copy)]
pub struct SvGameMsg;

#[derive(Clone, Copy)]
pub struct DeClientEnter<'a> {
    pub name: &'a [u8],
    pub client_id: i32,
    pub team: enums::Team,
}

#[derive(Clone, Copy)]
pub struct DeClientLeave<'a> {
    pub name: &'a [u8],
    pub client_id: i32,
    pub reason: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct ClSay<'a> {
    pub mode: enums::Chat,
    pub target: i32,
    pub message: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct ClSetTeam {
    pub team: enums::Team,
}

#[derive(Clone, Copy)]
pub struct ClSetSpectatorMode {
    pub spec_mode: enums::Spec,
    pub spectator_id: i32,
}

#[derive(Clone, Copy)]
pub struct ClStartInfo<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub skin_part_names: [&'a [u8]; 6],
    pub use_custom_colors: [bool; 6],
    pub skin_part_colors: [i32; 6],
}

#[derive(Clone, Copy)]
pub struct ClKill;

#[derive(Clone, Copy)]
pub struct ClReadyChange;

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
    pub force: bool,
}

#[derive(Clone, Copy)]
pub struct SvSkinChange<'a> {
    pub client_id: i32,
    pub skin_part_names: [&'a [u8]; 6],
    pub use_custom_colors: [bool; 6],
    pub skin_part_colors: [i32; 6],
}

#[derive(Clone, Copy)]
pub struct ClSkinChange<'a> {
    pub skin_part_names: [&'a [u8]; 6],
    pub use_custom_colors: [bool; 6],
    pub skin_part_colors: [i32; 6],
}

#[derive(Clone, Copy)]
pub struct SvRaceFinish {
    pub client_id: i32,
    pub time: i32,
    pub diff: i32,
    pub record_personal: bool,
    pub record_server: bool,
}

#[derive(Clone, Copy)]
pub struct SvCheckpoint {
    pub diff: i32,
}

#[derive(Clone, Copy)]
pub struct SvCommandInfo<'a> {
    pub name: &'a [u8],
    pub args_format: &'a [u8],
    pub help_text: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SvCommandInfoRemove<'a> {
    pub name: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct ClCommand<'a> {
    pub name: &'a [u8],
    pub arguments: &'a [u8],
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
            mode: enums::Chat::from_i32(_p.read_int(warn)?)?,
            client_id: in_range(_p.read_int(warn)?, -1, 63)?,
            target_id: in_range(_p.read_int(warn)?, -1, 63)?,
            message: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.client_id && self.client_id <= 63);
        assert!(-1 <= self.target_id && self.target_id <= 63);
        sanitize(&mut Panic, self.message).unwrap();
        _p.write_int(self.mode.to_i32())?;
        _p.write_int(self.client_id)?;
        _p.write_int(self.target_id)?;
        _p.write_string(self.message)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvChat<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvChat")
            .field("mode", &self.mode)
            .field("client_id", &self.client_id)
            .field("target_id", &self.target_id)
            .field("message", &pretty::Bytes::new(&self.message))
            .finish()
    }
}

impl SvTeam {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvTeam, Error> {
        let result = Ok(SvTeam {
            client_id: in_range(_p.read_int(warn)?, -1, 63)?,
            team: enums::Team::from_i32(_p.read_int(warn)?)?,
            silent: to_bool(_p.read_int(warn)?)?,
            cooldown_tick: ::snap_obj::Tick(_p.read_int(warn)?),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.client_id && self.client_id <= 63);
        _p.write_int(self.client_id)?;
        _p.write_int(self.team.to_i32())?;
        _p.write_int(self.silent as i32)?;
        _p.write_int(self.cooldown_tick.0)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvTeam")
            .field("client_id", &self.client_id)
            .field("team", &self.team)
            .field("silent", &self.silent)
            .field("cooldown_tick", &self.cooldown_tick)
            .finish()
    }
}

impl SvKillMsg {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvKillMsg, Error> {
        let result = Ok(SvKillMsg {
            killer: in_range(_p.read_int(warn)?, -2, 63)?,
            victim: in_range(_p.read_int(warn)?, 0, 63)?,
            weapon: in_range(_p.read_int(warn)?, -3, 5)?,
            mode_special: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-2 <= self.killer && self.killer <= 63);
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
            player_collision: TuneParam(_p.read_int(warn)?),
            player_hooking: TuneParam(_p.read_int(warn)?),
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
        _p.write_int(self.player_collision.0)?;
        _p.write_int(self.player_hooking.0)?;
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
            .field("player_collision", &self.player_collision)
            .field("player_hooking", &self.player_hooking)
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

impl SvVoteOptionListAdd {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvVoteOptionListAdd, Error> {
        let result = Ok(SvVoteOptionListAdd);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for SvVoteOptionListAdd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteOptionListAdd")
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
            client_id: in_range(_p.read_int(warn)?, -1, 63)?,
            type_: enums::Vote::from_i32(_p.read_int(warn)?)?,
            timeout: in_range(_p.read_int(warn)?, 0, 60)?,
            description: sanitize(warn, _p.read_string()?)?,
            reason: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.client_id && self.client_id <= 63);
        assert!(0 <= self.timeout && self.timeout <= 60);
        sanitize(&mut Panic, self.description).unwrap();
        sanitize(&mut Panic, self.reason).unwrap();
        _p.write_int(self.client_id)?;
        _p.write_int(self.type_.to_i32())?;
        _p.write_int(self.timeout)?;
        _p.write_string(self.description)?;
        _p.write_string(self.reason)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvVoteSet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvVoteSet")
            .field("client_id", &self.client_id)
            .field("type_", &self.type_)
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

impl SvServerSettings {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvServerSettings, Error> {
        let result = Ok(SvServerSettings {
            kick_vote: to_bool(_p.read_int(warn)?)?,
            kick_min: in_range(_p.read_int(warn)?, 0, 64)?,
            spec_vote: to_bool(_p.read_int(warn)?)?,
            team_lock: to_bool(_p.read_int(warn)?)?,
            team_balance: to_bool(_p.read_int(warn)?)?,
            player_slots: in_range(_p.read_int(warn)?, 0, 64)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.kick_min && self.kick_min <= 64);
        assert!(0 <= self.player_slots && self.player_slots <= 64);
        _p.write_int(self.kick_vote as i32)?;
        _p.write_int(self.kick_min)?;
        _p.write_int(self.spec_vote as i32)?;
        _p.write_int(self.team_lock as i32)?;
        _p.write_int(self.team_balance as i32)?;
        _p.write_int(self.player_slots)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvServerSettings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvServerSettings")
            .field("kick_vote", &self.kick_vote)
            .field("kick_min", &self.kick_min)
            .field("spec_vote", &self.spec_vote)
            .field("team_lock", &self.team_lock)
            .field("team_balance", &self.team_balance)
            .field("player_slots", &self.player_slots)
            .finish()
    }
}

impl<'a> SvClientInfo<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvClientInfo<'a>, Error> {
        let result = Ok(SvClientInfo {
            client_id: in_range(_p.read_int(warn)?, 0, 63)?,
            local: to_bool(_p.read_int(warn)?)?,
            team: enums::Team::from_i32(_p.read_int(warn)?)?,
            name: sanitize(warn, _p.read_string()?)?,
            clan: sanitize(warn, _p.read_string()?)?,
            country: _p.read_int(warn)?,
            skin_part_names: [
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
            ],
            use_custom_colors: [
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
            ],
            skin_part_colors: [
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
            ],
            silent: to_bool(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= 63);
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.clan).unwrap();
        for &e in &self.skin_part_names {
            sanitize(&mut Panic, e).unwrap();
        }
        _p.write_int(self.client_id)?;
        _p.write_int(self.local as i32)?;
        _p.write_int(self.team.to_i32())?;
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_int(self.country)?;
        for &e in &self.skin_part_names {
            _p.write_string(e)?;
        }
        for &e in &self.use_custom_colors {
            _p.write_int(e as i32)?;
        }
        for &e in &self.skin_part_colors {
            _p.write_int(e)?;
        }
        _p.write_int(self.silent as i32)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvClientInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvClientInfo")
            .field("client_id", &self.client_id)
            .field("local", &self.local)
            .field("team", &self.team)
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("skin_part_names", &DebugSlice::new(&self.skin_part_names, |e| pretty::Bytes::new(&e)))
            .field("use_custom_colors", &self.use_custom_colors)
            .field("skin_part_colors", &self.skin_part_colors)
            .field("silent", &self.silent)
            .finish()
    }
}

impl SvGameInfo {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvGameInfo, Error> {
        let result = Ok(SvGameInfo {
            game_flags: _p.read_int(warn)?,
            score_limit: positive(_p.read_int(warn)?)?,
            time_limit: positive(_p.read_int(warn)?)?,
            match_num: positive(_p.read_int(warn)?)?,
            match_current: positive(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(self.score_limit >= 0);
        assert!(self.time_limit >= 0);
        assert!(self.match_num >= 0);
        assert!(self.match_current >= 0);
        _p.write_int(self.game_flags)?;
        _p.write_int(self.score_limit)?;
        _p.write_int(self.time_limit)?;
        _p.write_int(self.match_num)?;
        _p.write_int(self.match_current)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvGameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvGameInfo")
            .field("game_flags", &self.game_flags)
            .field("score_limit", &self.score_limit)
            .field("time_limit", &self.time_limit)
            .field("match_num", &self.match_num)
            .field("match_current", &self.match_current)
            .finish()
    }
}

impl<'a> SvClientDrop<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvClientDrop<'a>, Error> {
        let result = Ok(SvClientDrop {
            client_id: in_range(_p.read_int(warn)?, 0, 63)?,
            reason: sanitize(warn, _p.read_string()?)?,
            silent: to_bool(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= 63);
        sanitize(&mut Panic, self.reason).unwrap();
        _p.write_int(self.client_id)?;
        _p.write_string(self.reason)?;
        _p.write_int(self.silent as i32)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvClientDrop<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvClientDrop")
            .field("client_id", &self.client_id)
            .field("reason", &pretty::Bytes::new(&self.reason))
            .field("silent", &self.silent)
            .finish()
    }
}

impl SvGameMsg {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvGameMsg, Error> {
        let result = Ok(SvGameMsg);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for SvGameMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvGameMsg")
            .finish()
    }
}

impl<'a> DeClientEnter<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<DeClientEnter<'a>, Error> {
        let result = Ok(DeClientEnter {
            name: sanitize(warn, _p.read_string()?)?,
            client_id: in_range(_p.read_int(warn)?, -1, 63)?,
            team: enums::Team::from_i32(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        assert!(-1 <= self.client_id && self.client_id <= 63);
        _p.write_string(self.name)?;
        _p.write_int(self.client_id)?;
        _p.write_int(self.team.to_i32())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for DeClientEnter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeClientEnter")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("client_id", &self.client_id)
            .field("team", &self.team)
            .finish()
    }
}

impl<'a> DeClientLeave<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<DeClientLeave<'a>, Error> {
        let result = Ok(DeClientLeave {
            name: sanitize(warn, _p.read_string()?)?,
            client_id: in_range(_p.read_int(warn)?, -1, 63)?,
            reason: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        assert!(-1 <= self.client_id && self.client_id <= 63);
        sanitize(&mut Panic, self.reason).unwrap();
        _p.write_string(self.name)?;
        _p.write_int(self.client_id)?;
        _p.write_string(self.reason)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for DeClientLeave<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DeClientLeave")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("client_id", &self.client_id)
            .field("reason", &pretty::Bytes::new(&self.reason))
            .finish()
    }
}

impl<'a> ClSay<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClSay<'a>, Error> {
        let result = Ok(ClSay {
            mode: enums::Chat::from_i32(_p.read_int(warn)?)?,
            target: in_range(_p.read_int(warn)?, -1, 63)?,
            message: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.target && self.target <= 63);
        sanitize(&mut Panic, self.message).unwrap();
        _p.write_int(self.mode.to_i32())?;
        _p.write_int(self.target)?;
        _p.write_string(self.message)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClSay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClSay")
            .field("mode", &self.mode)
            .field("target", &self.target)
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
            spec_mode: enums::Spec::from_i32(_p.read_int(warn)?)?,
            spectator_id: in_range(_p.read_int(warn)?, -1, 63)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(-1 <= self.spectator_id && self.spectator_id <= 63);
        _p.write_int(self.spec_mode.to_i32())?;
        _p.write_int(self.spectator_id)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ClSetSpectatorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClSetSpectatorMode")
            .field("spec_mode", &self.spec_mode)
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
            skin_part_names: [
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
            ],
            use_custom_colors: [
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
            ],
            skin_part_colors: [
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
            ],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.clan).unwrap();
        for &e in &self.skin_part_names {
            sanitize(&mut Panic, e).unwrap();
        }
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_int(self.country)?;
        for &e in &self.skin_part_names {
            _p.write_string(e)?;
        }
        for &e in &self.use_custom_colors {
            _p.write_int(e as i32)?;
        }
        for &e in &self.skin_part_colors {
            _p.write_int(e)?;
        }
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClStartInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClStartInfo")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("skin_part_names", &DebugSlice::new(&self.skin_part_names, |e| pretty::Bytes::new(&e)))
            .field("use_custom_colors", &self.use_custom_colors)
            .field("skin_part_colors", &self.skin_part_colors)
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

impl ClReadyChange {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ClReadyChange, Error> {
        let result = Ok(ClReadyChange);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ClReadyChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClReadyChange")
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
            force: to_bool(_p.read_int(warn)?)?,
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
        _p.write_int(self.force as i32)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClCallVote<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClCallVote")
            .field("type_", &pretty::Bytes::new(&self.type_))
            .field("value", &pretty::Bytes::new(&self.value))
            .field("reason", &pretty::Bytes::new(&self.reason))
            .field("force", &self.force)
            .finish()
    }
}

impl<'a> SvSkinChange<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvSkinChange<'a>, Error> {
        let result = Ok(SvSkinChange {
            client_id: in_range(_p.read_int(warn)?, 0, 63)?,
            skin_part_names: [
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
            ],
            use_custom_colors: [
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
            ],
            skin_part_colors: [
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
            ],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= 63);
        for &e in &self.skin_part_names {
            sanitize(&mut Panic, e).unwrap();
        }
        _p.write_int(self.client_id)?;
        for &e in &self.skin_part_names {
            _p.write_string(e)?;
        }
        for &e in &self.use_custom_colors {
            _p.write_int(e as i32)?;
        }
        for &e in &self.skin_part_colors {
            _p.write_int(e)?;
        }
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvSkinChange<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvSkinChange")
            .field("client_id", &self.client_id)
            .field("skin_part_names", &DebugSlice::new(&self.skin_part_names, |e| pretty::Bytes::new(&e)))
            .field("use_custom_colors", &self.use_custom_colors)
            .field("skin_part_colors", &self.skin_part_colors)
            .finish()
    }
}

impl<'a> ClSkinChange<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClSkinChange<'a>, Error> {
        let result = Ok(ClSkinChange {
            skin_part_names: [
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
                sanitize(warn, _p.read_string()?)?,
            ],
            use_custom_colors: [
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
                to_bool(_p.read_int(warn)?)?,
            ],
            skin_part_colors: [
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
                _p.read_int(warn)?,
            ],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        for &e in &self.skin_part_names {
            sanitize(&mut Panic, e).unwrap();
        }
        for &e in &self.skin_part_names {
            _p.write_string(e)?;
        }
        for &e in &self.use_custom_colors {
            _p.write_int(e as i32)?;
        }
        for &e in &self.skin_part_colors {
            _p.write_int(e)?;
        }
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClSkinChange<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClSkinChange")
            .field("skin_part_names", &DebugSlice::new(&self.skin_part_names, |e| pretty::Bytes::new(&e)))
            .field("use_custom_colors", &self.use_custom_colors)
            .field("skin_part_colors", &self.skin_part_colors)
            .finish()
    }
}

impl SvRaceFinish {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvRaceFinish, Error> {
        let result = Ok(SvRaceFinish {
            client_id: in_range(_p.read_int(warn)?, 0, 63)?,
            time: at_least(_p.read_int(warn)?, -1)?,
            diff: _p.read_int(warn)?,
            record_personal: to_bool(_p.read_int(warn)?)?,
            record_server: to_bool(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(0 <= self.client_id && self.client_id <= 63);
        assert!(self.time >= -1);
        _p.write_int(self.client_id)?;
        _p.write_int(self.time)?;
        _p.write_int(self.diff)?;
        _p.write_int(self.record_personal as i32)?;
        _p.write_int(self.record_server as i32)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvRaceFinish {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvRaceFinish")
            .field("client_id", &self.client_id)
            .field("time", &self.time)
            .field("diff", &self.diff)
            .field("record_personal", &self.record_personal)
            .field("record_server", &self.record_server)
            .finish()
    }
}

impl SvCheckpoint {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SvCheckpoint, Error> {
        let result = Ok(SvCheckpoint {
            diff: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.diff)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SvCheckpoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvCheckpoint")
            .field("diff", &self.diff)
            .finish()
    }
}

impl<'a> SvCommandInfo<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvCommandInfo<'a>, Error> {
        let result = Ok(SvCommandInfo {
            name: sanitize(warn, _p.read_string()?)?,
            args_format: sanitize(warn, _p.read_string()?)?,
            help_text: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.args_format).unwrap();
        sanitize(&mut Panic, self.help_text).unwrap();
        _p.write_string(self.name)?;
        _p.write_string(self.args_format)?;
        _p.write_string(self.help_text)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvCommandInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvCommandInfo")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("args_format", &pretty::Bytes::new(&self.args_format))
            .field("help_text", &pretty::Bytes::new(&self.help_text))
            .finish()
    }
}

impl<'a> SvCommandInfoRemove<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SvCommandInfoRemove<'a>, Error> {
        let result = Ok(SvCommandInfoRemove {
            name: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        _p.write_string(self.name)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SvCommandInfoRemove<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SvCommandInfoRemove")
            .field("name", &pretty::Bytes::new(&self.name))
            .finish()
    }
}

impl<'a> ClCommand<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClCommand<'a>, Error> {
        let result = Ok(ClCommand {
            name: sanitize(warn, _p.read_string()?)?,
            arguments: sanitize(warn, _p.read_string()?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.arguments).unwrap();
        _p.write_string(self.name)?;
        _p.write_string(self.arguments)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClCommand<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClCommand")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("arguments", &pretty::Bytes::new(&self.arguments))
            .finish()
    }
}

