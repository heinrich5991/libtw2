use arrayvec::ArrayVec;
use common::num::Cast;
use common::pretty;
use packer::Unpacker;
use packer::positive;
use serde::Serialize;
use serde::ser::SerializeSeq;
use serde;
use std::fmt;
use uuid::Uuid;
use warn::Ignore;

use super::MaybeEnd;
use super::Version;

pub const FINISH: i32 = -1;
pub const TICK_SKIP: i32 = -2;
pub const PLAYER_NEW: i32 = -3;
pub const PLAYER_OLD: i32 = -4;
pub const INPUT_DIFF: i32 = -5;
pub const INPUT_NEW: i32 = -6;
pub const MESSAGE: i32 = -7;
pub const JOIN: i32 = -8;
pub const DROP: i32 = -9;
pub const CONSOLE_COMMAND: i32 = -10;
pub const EX: i32 = -11;

pub const INPUT_LEN: usize = 10;
pub const CONSOLE_COMMAND_MAX_ARGS: usize = 16;

pub const UUID_AUTH_INIT: [u8; 16] = [
    // "60daba5c-52c4-3aeb-b8ba-b2953fb55a17"
    0x60, 0xda, 0xba, 0x5c, 0x52, 0xc4, 0x3a, 0xeb,
    0xb8, 0xba, 0xb2, 0x95, 0x3f, 0xb5, 0x5a, 0x17,
];
pub const UUID_AUTH_LOGIN: [u8; 16] = [
    // "37ecd3b8-9218-3bb9-a71b-a935b86f6a81"
    0x37, 0xec, 0xd3, 0xb8, 0x92, 0x18, 0x3b, 0xb9,
    0xa7, 0x1b, 0xa9, 0x35, 0xb8, 0x6f, 0x6a, 0x81,
];
pub const UUID_AUTH_LOGOUT: [u8; 16] = [
    // "d4f5abe8-edd2-3fb9-abd8-1c8bb84f4a63"
    0xd4, 0xf5, 0xab, 0xe8, 0xed, 0xd2, 0x3f, 0xb9,
    0xab, 0xd8, 0x1c, 0x8b, 0xb8, 0x4f, 0x4a, 0x63,
];
pub const UUID_DDNETVER: [u8; 16] = [
    // "1397b63e-ee4e-3919-b86a-b058887fcaf5"
    0x13, 0x97, 0xb6, 0x3e, 0xee, 0x4e, 0x39, 0x19,
    0xb8, 0x6a, 0xb0, 0x58, 0x88, 0x7f, 0xca, 0xf5,
];
pub const UUID_DDNETVER_OLD: [u8; 16] = [
    // "41b49541-f26f-325d-8715-9baf4b544ef9"
    0x41, 0xb4, 0x95, 0x41, 0xf2, 0x6f, 0x32, 0x5d,
    0x87, 0x15, 0x9b, 0xaf, 0x4b, 0x54, 0x4e, 0xf9,
];
pub const UUID_JOINVER6: [u8; 16] = [
    // "1899a382-71e3-36da-937d-c9de6bb95b1d"
    0x18, 0x99, 0xa3, 0x82, 0x71, 0xe3, 0x36, 0xda,
    0x93, 0x7d, 0xc9, 0xde, 0x6b, 0xb9, 0x5b, 0x1d,
];
pub const UUID_JOINVER7: [u8; 16] = [
    // "59239b05-0540-318d-bea4-9aa1e80e7d2b"
    0x59, 0x23, 0x9b, 0x05, 0x05, 0x40, 0x31, 0x8d,
    0xbe, 0xa4, 0x9a, 0xa1, 0xe8, 0x0e, 0x7d, 0x2b,
];
pub const UUID_PLAYER_TEAM: [u8; 16] = [
    // "a111c04e-1ea8-38e0-90b1-d7f993ca0da9"
    0xa1, 0x11, 0xc0, 0x4e, 0x1e, 0xa8, 0x38, 0xe0,
    0x90, 0xb1, 0xd7, 0xf9, 0x93, 0xca, 0x0d, 0xa9,
];
pub const UUID_TEAM_LOAD_FAILURE: [u8; 16] = [
    // "ef8905a2-c695-3591-a1cd-53d2015992dd"
    0xef, 0x89, 0x05, 0xa2, 0xc6, 0x95, 0x35, 0x91,
    0xa1, 0xcd, 0x53, 0xd2, 0x01, 0x59, 0x92, 0xdd,
];
pub const UUID_TEAM_LOAD_SUCCESS: [u8; 16] = [
    // "e05408d3-a313-33df-9eb3-ddb990ab954a"
    0xe0, 0x54, 0x08, 0xd3, 0xa3, 0x13, 0x33, 0xdf,
    0x9e, 0xb3, 0xdd, 0xb9, 0x90, 0xab, 0x95, 0x4a,
];
pub const UUID_TEAM_PRACTICE: [u8; 16] = [
    // "5792834e-81d1-34c9-a29b-b5ff25dac3bc"
    0x57, 0x92, 0x83, 0x4e, 0x81, 0xd1, 0x34, 0xc9,
    0xa2, 0x9b, 0xb5, 0xff, 0x25, 0xda, 0xc3, 0xbc,
];
pub const UUID_TEAM_SAVE_FAILURE: [u8; 16] = [
    // "b29901d5-1244-3bd0-bbde-23d04b1f7ba9"
    0xb2, 0x99, 0x01, 0xd5, 0x12, 0x44, 0x3b, 0xd0,
    0xbb, 0xde, 0x23, 0xd0, 0x4b, 0x1f, 0x7b, 0xa9,
];
pub const UUID_TEAM_SAVE_SUCCESS: [u8; 16] = [
    // "4560c756-da29-3036-81d4-90a50f0182cd"
    0x45, 0x60, 0xc7, 0x56, 0xda, 0x29, 0x30, 0x36,
    0x81, 0xd4, 0x90, 0xa5, 0x0f, 0x01, 0x82, 0xcd,
];

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// PlayerDiff(cid)
    PlayerDiff(i32),
    Finish,
    TickSkip,
    /// PlayerNew(cid)
    PlayerNew(i32),
    /// PlayerOld(cid)
    PlayerOld(i32),
    InputDiff,
    InputNew,
    Message,
    Join,
    Drop,
    ConsoleCommand,
    Ex,
}

pub struct UnknownType(i32);

impl From<UnknownType> for Error {
    fn from(e: UnknownType) -> Error {
        Error::UnknownType(e.0)
    }
}

impl From<UnknownType> for MaybeEnd<UnknownType> {
    fn from(e: UnknownType) -> MaybeEnd<UnknownType> {
        MaybeEnd::Err(e)
    }
}

impl From<MaybeEnd<UnknownType>> for MaybeEnd<Error> {
    fn from(me: MaybeEnd<UnknownType>) -> MaybeEnd<Error> {
        match me {
            MaybeEnd::Err(e) => MaybeEnd::Err(e.into()),
            MaybeEnd::UnexpectedEnd => MaybeEnd::UnexpectedEnd,
        }
    }
}

impl Kind {
    pub fn decode(p: &mut Unpacker, version: Version)
        -> Result<Kind, MaybeEnd<UnknownType>>
    {
        Ok(match p.read_int(&mut Ignore)? {
            i if i >= 0 => Kind::PlayerDiff(i),
            FINISH => Kind::Finish,
            TICK_SKIP => Kind::TickSkip,
            PLAYER_NEW => Kind::PlayerNew(p.read_int(&mut Ignore)?),
            PLAYER_OLD => Kind::PlayerOld(p.read_int(&mut Ignore)?),
            INPUT_DIFF => Kind::InputDiff,
            INPUT_NEW => Kind::InputNew,
            MESSAGE => Kind::Message,
            JOIN => Kind::Join,
            DROP => Kind::Drop,
            CONSOLE_COMMAND => Kind::ConsoleCommand,
            EX if version.has_ex() => Kind::Ex,
            x => return Err(UnknownType(x).into()),
        })
    }
    pub fn decode_rest<'a>(&self, p: &mut Unpacker<'a>)
        -> Result<Item<'a>, MaybeEnd<Error>>
    {
        Ok(match *self {
            Kind::PlayerDiff(cid) => PlayerDiff::decode(cid, p)?.into(),
            Kind::Finish => Finish::decode(p)?.into(),
            Kind::TickSkip => TickSkip::decode(p)?.into(),
            Kind::PlayerNew(cid) => PlayerNew::decode(cid, p)?.into(),
            Kind::PlayerOld(cid) => PlayerOld::decode(cid, p)?.into(),
            Kind::InputDiff => InputDiff::decode(p)?.into(),
            Kind::InputNew => InputNew::decode(p)?.into(),
            Kind::Message => Message::decode(p)?.into(),
            Kind::Join => Join::decode(p)?.into(),
            Kind::Drop => Drop::decode(p)?.into(),
            Kind::ConsoleCommand => ConsoleCommand::decode(p)?.into(),
            Kind::Ex => Item::decode_ex(p)?,
        })
    }
    pub fn player_cid(&self) -> Option<i32> {
        Some(match *self {
            Kind::PlayerDiff(cid) => cid,
            Kind::PlayerNew(cid) => cid,
            Kind::PlayerOld(cid) => cid,
            _ => return None,
        })
    }
}

fn serialize_str_lossy<S>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer,
{
    String::from_utf8_lossy(bytes).serialize(s)
}

fn serialize_str_slice_lossy<S>(bytess: &ArrayVec<[&[u8]; 16]>, s: S)
    -> Result<S::Ok, S::Error>
    where S: serde::Serializer,
{
    let mut seq = s.serialize_seq(Some(bytess.len()))?;
    for &bytes in bytess {
        seq.serialize_element(&String::from_utf8_lossy(bytes))?;
    }
    seq.end()
}

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Item<'a> {
    PlayerDiff(PlayerDiff),
    Finish(Finish),
    TickSkip(TickSkip),
    PlayerNew(PlayerNew),
    PlayerOld(PlayerOld),
    InputDiff(InputDiff),
    InputNew(InputNew),
    Message(Message<'a>),
    Join(Join),
    Drop(Drop<'a>),
    ConsoleCommand(ConsoleCommand<'a>),

    AuthInit(AuthInit<'a>),
    AuthLogin(AuthLogin<'a>),
    AuthLogout(AuthLogout),
    Ddnetver(Ddnetver<'a>),
    DdnetverOld(DdnetverOld),
    Joinver6(Joinver6),
    Joinver7(Joinver7),
    PlayerTeam(PlayerTeam),
    TeamLoadFailure(TeamLoadFailure),
    TeamLoadSuccess(TeamLoadSuccess<'a>),
    TeamPractice(TeamPractice),
    TeamSaveFailure(TeamSaveFailure),
    TeamSaveSuccess(TeamSaveSuccess<'a>),

    UnknownEx(UnknownEx<'a>),
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerDiff {
    pub cid: i32,
    pub dx: i32,
    pub dy: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Finish;

#[derive(Clone, Debug, Serialize)]
pub struct TickSkip {
    pub dt: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerNew {
    pub cid: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerOld {
    pub cid: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct InputDiff {
    pub cid: i32,
    pub diff: [i32; INPUT_LEN],
}

#[derive(Clone, Debug, Serialize)]
pub struct InputNew {
    pub cid: i32,
    pub new: [i32; INPUT_LEN],
}

#[derive(Clone, Serialize)]
pub struct Message<'a> {
    pub cid: i32,
    pub msg: &'a [u8],
}

#[derive(Clone, Debug, Serialize)]
pub struct Join {
    pub cid: i32,
}

#[derive(Clone, Serialize)]
pub struct Drop<'a> {
    pub cid: i32,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub reason: &'a [u8],
}

#[derive(Clone, Serialize)]
pub struct ConsoleCommand<'a> {
    pub cid: i32,
    pub flag_mask: u32,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub cmd: &'a [u8],
    #[serde(serialize_with = "serialize_str_slice_lossy")]
    pub args: ArrayVec<[&'a [u8]; CONSOLE_COMMAND_MAX_ARGS]>,
}

#[derive(Clone, Serialize)]
pub struct AuthInit<'a> {
    pub cid: i32,
    pub level: i32,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub identity: &'a [u8],
}

#[derive(Clone, Serialize)]
pub struct AuthLogin<'a> {
    pub cid: i32,
    pub level: i32,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub identity: &'a [u8],
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthLogout {
    pub cid: i32,
}

#[derive(Clone, Serialize)]
pub struct Ddnetver<'a> {
    pub cid: i32,
    pub connection_id: Uuid,
    pub ddnet_version: i32,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub ddnet_version_str: &'a [u8],
}

#[derive(Clone, Debug, Serialize)]
pub struct DdnetverOld {
    pub cid: i32,
    pub ddnet_version: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Joinver6 {
    pub cid: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Joinver7 {
    pub cid: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlayerTeam {
    pub cid: i32,
    pub team: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TeamLoadFailure {
    pub team: i32,
}

#[derive(Clone, Serialize)]
pub struct TeamLoadSuccess<'a> {
    pub team: i32,
    pub save_uuid: Uuid,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub save: &'a [u8],
}

#[derive(Clone, Debug, Serialize)]
pub struct TeamPractice {
    pub team: i32,
    pub practice: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TeamSaveFailure {
    pub team: i32,
}

#[derive(Clone, Serialize)]
pub struct TeamSaveSuccess<'a> {
    pub team: i32,
    pub save_uuid: Uuid,
    #[serde(serialize_with = "serialize_str_lossy")]
    pub save: &'a [u8],
}

#[derive(Clone, Serialize)]
pub struct UnknownEx<'a> {
    pub uuid: Uuid,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub enum Error {
    UnknownType(i32),
    NegativeDt,
    NegativeNumArgs,
    NumArgsTooLarge,
}

impl From<Error> for MaybeEnd<Error> {
    fn from(e: Error) -> MaybeEnd<Error> {
        MaybeEnd::Err(e)
    }
}

impl<'a> Item<'a> {
    pub fn decode(p: &mut Unpacker<'a>, version: Version)
        -> Result<Item<'a>, MaybeEnd<Error>>
    {
        Kind::decode(p, version)?.decode_rest(p)
    }
    pub fn decode_ex(p: &mut Unpacker<'a>) -> Result<Item<'a>, MaybeEnd<Error>> {
        let uuid = p.read_uuid()?;
        let data = p.read_data(&mut Ignore)?;
        Ok(match *uuid.as_bytes() {
            UUID_AUTH_INIT => AuthInit::decode(&mut Unpacker::new(data))?.into(),
            UUID_AUTH_LOGIN => AuthLogin::decode(&mut Unpacker::new(data))?.into(),
            UUID_AUTH_LOGOUT => AuthLogout::decode(&mut Unpacker::new(data))?.into(),
            UUID_DDNETVER => Ddnetver::decode(&mut Unpacker::new(data))?.into(),
            UUID_DDNETVER_OLD => DdnetverOld::decode(&mut Unpacker::new(data))?.into(),
            UUID_JOINVER6 => Joinver6::decode(&mut Unpacker::new(data))?.into(),
            UUID_JOINVER7 => Joinver7::decode(&mut Unpacker::new(data))?.into(),
            UUID_PLAYER_TEAM => PlayerTeam::decode(&mut Unpacker::new(data))?.into(),
            UUID_TEAM_LOAD_FAILURE => TeamLoadFailure::decode(&mut Unpacker::new(data))?.into(),
            UUID_TEAM_LOAD_SUCCESS => TeamLoadSuccess::decode(&mut Unpacker::new(data))?.into(),
            UUID_TEAM_PRACTICE => TeamPractice::decode(&mut Unpacker::new(data))?.into(),
            UUID_TEAM_SAVE_FAILURE => TeamSaveFailure::decode(&mut Unpacker::new(data))?.into(),
            UUID_TEAM_SAVE_SUCCESS => TeamSaveSuccess::decode(&mut Unpacker::new(data))?.into(),
            _ => UnknownEx {
                uuid: uuid,
                data: data,
            }.into(),
        })
    }
    pub fn cid(&self) -> Option<i32> {
        Some(match *self {
            Item::PlayerDiff(ref i) => i.cid,
            Item::Finish(_) => return None,
            Item::TickSkip(_) => return None,
            Item::PlayerNew(ref i) => i.cid,
            Item::PlayerOld(ref i) => i.cid,
            Item::InputDiff(ref i) => i.cid,
            Item::InputNew(ref i) => i.cid,
            Item::Message(ref i) => i.cid,
            Item::Join(ref i) => i.cid,
            Item::Drop(ref i) => i.cid,
            Item::ConsoleCommand(ref i) => i.cid,
            Item::AuthInit(ref i) => i.cid,
            Item::AuthLogin(ref i) => i.cid,
            Item::AuthLogout(ref i) => i.cid,
            Item::Ddnetver(ref i) => i.cid,
            Item::DdnetverOld(ref i) => i.cid,
            Item::Joinver6(ref i) => i.cid,
            Item::Joinver7(ref i) => i.cid,
            Item::PlayerTeam(ref i) => i.cid,
            Item::TeamLoadFailure(_) => return None,
            Item::TeamLoadSuccess(_) => return None,
            Item::TeamPractice(_) => return None,
            Item::TeamSaveFailure(_) => return None,
            Item::TeamSaveSuccess(_) => return None,
            Item::UnknownEx(_) => return None,
        })
    }
}

impl PlayerDiff {
    fn decode(cid: i32, _p: &mut Unpacker) -> Result<PlayerDiff, MaybeEnd<Error>> {
        Ok(PlayerDiff {
            cid: cid,
            dx: _p.read_int(&mut Ignore)?,
            dy: _p.read_int(&mut Ignore)?,
        })
    }
}

impl Finish {
    fn decode(_p: &mut Unpacker) -> Result<Finish, MaybeEnd<Error>> {
        Ok(Finish)
    }
}

impl TickSkip {
    fn decode(_p: &mut Unpacker) -> Result<TickSkip, MaybeEnd<Error>> {
        Ok(TickSkip {
            dt: positive(_p.read_int(&mut Ignore)?)
                .map_err(|_| Error::NegativeDt)?
                .assert_u32()
        })
    }
}

impl PlayerNew {
    fn decode(cid: i32, _p: &mut Unpacker) -> Result<PlayerNew, MaybeEnd<Error>> {
        Ok(PlayerNew {
            cid: cid,
            x: _p.read_int(&mut Ignore)?,
            y: _p.read_int(&mut Ignore)?,
        })
    }
}

impl PlayerOld {
    fn decode(cid: i32, _p: &mut Unpacker) -> Result<PlayerOld, MaybeEnd<Error>> {
        Ok(PlayerOld {
            cid: cid,
        })
    }
}

impl InputDiff {
    fn decode(_p: &mut Unpacker) -> Result<InputDiff, MaybeEnd<Error>> {
        Ok(InputDiff {
            cid: _p.read_int(&mut Ignore)?,
            diff: [
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
            ],
        })
    }
}

impl InputNew {
    fn decode(_p: &mut Unpacker) -> Result<InputNew, MaybeEnd<Error>> {
        Ok(InputNew {
            cid: _p.read_int(&mut Ignore)?,
            new: [
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
                _p.read_int(&mut Ignore)?,
            ],
        })
    }
}

impl<'a> Message<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<Message<'a>, MaybeEnd<Error>> {
        Ok(Message {
            cid: _p.read_int(&mut Ignore)?,
            msg: _p.read_data(&mut Ignore)?
        })
    }
}

impl Join {
    fn decode(_p: &mut Unpacker) -> Result<Join, MaybeEnd<Error>> {
        Ok(Join {
            cid: _p.read_int(&mut Ignore)?,
        })
    }
}

impl<'a> Drop<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<Drop<'a>, MaybeEnd<Error>> {
        Ok(Drop {
            cid: _p.read_int(&mut Ignore)?,
            reason: _p.read_string()?
        })
    }
}

impl<'a> ConsoleCommand<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<ConsoleCommand<'a>, MaybeEnd<Error>> {
        let cid = _p.read_int(&mut Ignore)?;
        let flag_mask = _p.read_int(&mut Ignore)? as u32;
        let cmd = _p.read_string()?;
        let num_args = positive(_p.read_int(&mut Ignore)?)
            .map_err(|_| Error::NegativeNumArgs)?;
        let mut args = ArrayVec::new();
        for _ in 0..num_args {
            args.try_push(_p.read_string()?).map_err(|_| Error::NumArgsTooLarge)?;
        }
        Ok(ConsoleCommand {
            cid: cid,
            flag_mask: flag_mask,
            cmd: cmd,
            args: args,
        })
    }
}

impl<'a> AuthInit<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<AuthInit<'a>, MaybeEnd<Error>> {
        Ok(AuthInit {
            cid: _p.read_int(&mut Ignore)?,
            level: _p.read_int(&mut Ignore)?,
            identity: _p.read_string()?,
        })
    }
}

impl<'a> AuthLogin<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<AuthLogin<'a>, MaybeEnd<Error>> {
        Ok(AuthLogin {
            cid: _p.read_int(&mut Ignore)?,
            level: _p.read_int(&mut Ignore)?,
            identity: _p.read_string()?,
        })
    }
}

impl AuthLogout {
    fn decode(_p: &mut Unpacker) -> Result<AuthLogout, MaybeEnd<Error>> {
        Ok(AuthLogout {
            cid: _p.read_int(&mut Ignore)?,
        })
    }
}

impl<'a> Ddnetver<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<Ddnetver<'a>, MaybeEnd<Error>> {
        Ok(Ddnetver {
            cid: _p.read_int(&mut Ignore)?,
            connection_id: _p.read_uuid()?,
            ddnet_version: _p.read_int(&mut Ignore)?,
            ddnet_version_str: _p.read_string()?,
        })
    }
}

impl DdnetverOld {
    fn decode(_p: &mut Unpacker) -> Result<DdnetverOld, MaybeEnd<Error>> {
        Ok(DdnetverOld {
            cid: _p.read_int(&mut Ignore)?,
            ddnet_version: _p.read_int(&mut Ignore)?,
        })
    }
}

impl Joinver6 {
    fn decode(_p: &mut Unpacker) -> Result<Joinver6, MaybeEnd<Error>> {
        Ok(Joinver6 {
            cid: _p.read_int(&mut Ignore)?,
        })
    }
}

impl Joinver7 {
    fn decode(_p: &mut Unpacker) -> Result<Joinver7, MaybeEnd<Error>> {
        Ok(Joinver7 {
            cid: _p.read_int(&mut Ignore)?,
        })
    }
}

impl PlayerTeam {
    fn decode(_p: &mut Unpacker) -> Result<PlayerTeam, MaybeEnd<Error>> {
        Ok(PlayerTeam {
            cid: _p.read_int(&mut Ignore)?,
            team: _p.read_int(&mut Ignore)?,
        })
    }
}

impl TeamLoadFailure {
    fn decode(_p: &mut Unpacker) -> Result<TeamLoadFailure, MaybeEnd<Error>> {
        Ok(TeamLoadFailure {
            team: _p.read_int(&mut Ignore)?,
        })
    }
}

impl<'a> TeamLoadSuccess<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<TeamLoadSuccess<'a>, MaybeEnd<Error>> {
        Ok(TeamLoadSuccess {
            team: _p.read_int(&mut Ignore)?,
            save_uuid: _p.read_uuid()?,
            save: _p.read_string()?,
        })
    }
}

impl TeamPractice {
    fn decode(_p: &mut Unpacker) -> Result<TeamPractice, MaybeEnd<Error>> {
        Ok(TeamPractice {
            team: _p.read_int(&mut Ignore)?,
            practice: _p.read_int(&mut Ignore)?,
        })
    }
}

impl TeamSaveFailure {
    fn decode(_p: &mut Unpacker) -> Result<TeamSaveFailure, MaybeEnd<Error>> {
        Ok(TeamSaveFailure {
            team: _p.read_int(&mut Ignore)?,
        })
    }
}

impl<'a> TeamSaveSuccess<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<TeamSaveSuccess<'a>, MaybeEnd<Error>> {
        Ok(TeamSaveSuccess {
            team: _p.read_int(&mut Ignore)?,
            save_uuid: _p.read_uuid()?,
            save: _p.read_string()?,
        })
    }
}

impl<'a> fmt::Debug for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Item::PlayerDiff(ref i) => i.fmt(f),
            Item::Finish(ref i) => i.fmt(f),
            Item::TickSkip(ref i) => i.fmt(f),
            Item::PlayerNew(ref i) => i.fmt(f),
            Item::PlayerOld(ref i) => i.fmt(f),
            Item::InputDiff(ref i) => i.fmt(f),
            Item::InputNew(ref i) => i.fmt(f),
            Item::Message(ref i) => i.fmt(f),
            Item::Join(ref i) => i.fmt(f),
            Item::Drop(ref i) => i.fmt(f),
            Item::ConsoleCommand(ref i) => i.fmt(f),
            Item::AuthInit(ref i) => i.fmt(f),
            Item::AuthLogin(ref i) => i.fmt(f),
            Item::AuthLogout(ref i) => i.fmt(f),
            Item::Ddnetver(ref i) => i.fmt(f),
            Item::DdnetverOld(ref i) => i.fmt(f),
            Item::Joinver6(ref i) => i.fmt(f),
            Item::Joinver7(ref i) => i.fmt(f),
            Item::PlayerTeam(ref i) => i.fmt(f),
            Item::TeamLoadFailure(ref i) => i.fmt(f),
            Item::TeamLoadSuccess(ref i) => i.fmt(f),
            Item::TeamPractice(ref i) => i.fmt(f),
            Item::TeamSaveFailure(ref i) => i.fmt(f),
            Item::TeamSaveSuccess(ref i) => i.fmt(f),
            Item::UnknownEx(ref i) => i.fmt(f),
        }
    }
}

impl<'a> fmt::Debug for Message<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Message")
            .field("cid", &self.cid)
            .field("msg", &pretty::Bytes::new(&self.msg))
            .finish()
    }
}

impl<'a> fmt::Debug for Drop<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Drop")
            .field("cid", &self.cid)
            .field("reason", &pretty::AlmostString::new(&self.reason))
            .finish()
    }
}

impl<'a> fmt::Debug for ConsoleCommand<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConsoleCommand")
            .field("cid", &self.cid)
            .field("flag_mask", &self.flag_mask)
            .field("cmd", &pretty::AlmostString::new(&self.cmd))
            .field("args", &pretty::AlmostStringSlice::new(&self.args))
            .finish()
    }
}

impl<'a> fmt::Debug for AuthInit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthInit")
            .field("cid", &self.cid)
            .field("level", &self.level)
            .field("identity", &pretty::AlmostString::new(&self.identity))
            .finish()
    }
}

impl<'a> fmt::Debug for AuthLogin<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthLogin")
            .field("cid", &self.cid)
            .field("level", &self.level)
            .field("identity", &pretty::AlmostString::new(&self.identity))
            .finish()
    }
}

impl<'a> fmt::Debug for Ddnetver<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ddnetver")
            .field("cid", &self.cid)
            .field("connection_id", &self.connection_id)
            .field("ddnet_version", &self.ddnet_version)
            .field("ddnet_version_str", &pretty::AlmostString::new(&self.ddnet_version_str))
            .finish()
    }
}

impl<'a> fmt::Debug for TeamLoadSuccess<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TeamLoadSuccess")
            .field("team", &self.team)
            .field("save_uuid", &self.save_uuid)
            .field("save", &pretty::AlmostString::new(&self.save))
            .finish()
    }
}

impl<'a> fmt::Debug for TeamSaveSuccess<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TeamSaveSuccess")
            .field("team", &self.team)
            .field("save_uuid", &self.save_uuid)
            .field("save", &pretty::AlmostString::new(&self.save))
            .finish()
    }
}

impl<'a> fmt::Debug for UnknownEx<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnknownEx")
            .field("uuid", &self.uuid)
            .field("data", &pretty::Bytes::new(&self.data))
            .finish()
    }
}

impl<'a> From<PlayerDiff> for Item<'a> {
    fn from(i: PlayerDiff) -> Item<'a> {
        Item::PlayerDiff(i)
    }
}

impl<'a> From<Finish> for Item<'a> {
    fn from(i: Finish) -> Item<'a> {
        Item::Finish(i)
    }
}

impl<'a> From<TickSkip> for Item<'a> {
    fn from(i: TickSkip) -> Item<'a> {
        Item::TickSkip(i)
    }
}

impl<'a> From<PlayerNew> for Item<'a> {
    fn from(i: PlayerNew) -> Item<'a> {
        Item::PlayerNew(i)
    }
}

impl<'a> From<PlayerOld> for Item<'a> {
    fn from(i: PlayerOld) -> Item<'a> {
        Item::PlayerOld(i)
    }
}

impl<'a> From<InputDiff> for Item<'a> {
    fn from(i: InputDiff) -> Item<'a> {
        Item::InputDiff(i)
    }
}

impl<'a> From<InputNew> for Item<'a> {
    fn from(i: InputNew) -> Item<'a> {
        Item::InputNew(i)
    }
}

impl<'a> From<Message<'a>> for Item<'a> {
    fn from(i: Message<'a>) -> Item<'a> {
        Item::Message(i)
    }
}

impl<'a> From<Join> for Item<'a> {
    fn from(i: Join) -> Item<'a> {
        Item::Join(i)
    }
}

impl<'a> From<Drop<'a>> for Item<'a> {
    fn from(i: Drop<'a>) -> Item<'a> {
        Item::Drop(i)
    }
}

impl<'a> From<ConsoleCommand<'a>> for Item<'a> {
    fn from(i: ConsoleCommand<'a>) -> Item<'a> {
        Item::ConsoleCommand(i)
    }
}

impl<'a> From<AuthInit<'a>> for Item<'a> {
    fn from(i: AuthInit<'a>) -> Item<'a> {
        Item::AuthInit(i)
    }
}

impl<'a> From<AuthLogin<'a>> for Item<'a> {
    fn from(i: AuthLogin<'a>) -> Item<'a> {
        Item::AuthLogin(i)
    }
}

impl<'a> From<Ddnetver<'a>> for Item<'a> {
    fn from(i: Ddnetver<'a>) -> Item<'a> {
        Item::Ddnetver(i)
    }
}

impl<'a> From<DdnetverOld> for Item<'a> {
    fn from(i: DdnetverOld) -> Item<'a> {
        Item::DdnetverOld(i)
    }
}

impl<'a> From<AuthLogout> for Item<'a> {
    fn from(i: AuthLogout) -> Item<'a> {
        Item::AuthLogout(i)
    }
}

impl<'a> From<Joinver6> for Item<'a> {
    fn from(i: Joinver6) -> Item<'a> {
        Item::Joinver6(i)
    }
}

impl<'a> From<Joinver7> for Item<'a> {
    fn from(i: Joinver7) -> Item<'a> {
        Item::Joinver7(i)
    }
}

impl<'a> From<PlayerTeam> for Item<'a> {
    fn from(i: PlayerTeam) -> Item<'a> {
        Item::PlayerTeam(i)
    }
}

impl<'a> From<TeamLoadFailure> for Item<'a> {
    fn from(i: TeamLoadFailure) -> Item<'a> {
        Item::TeamLoadFailure(i)
    }
}

impl<'a> From<TeamLoadSuccess<'a>> for Item<'a> {
    fn from(i: TeamLoadSuccess<'a>) -> Item<'a> {
        Item::TeamLoadSuccess(i)
    }
}

impl<'a> From<TeamPractice> for Item<'a> {
    fn from(i: TeamPractice) -> Item<'a> {
        Item::TeamPractice(i)
    }
}

impl<'a> From<TeamSaveFailure> for Item<'a> {
    fn from(i: TeamSaveFailure) -> Item<'a> {
        Item::TeamSaveFailure(i)
    }
}

impl<'a> From<TeamSaveSuccess<'a>> for Item<'a> {
    fn from(i: TeamSaveSuccess<'a>) -> Item<'a> {
        Item::TeamSaveSuccess(i)
    }
}

impl<'a> From<UnknownEx<'a>> for Item<'a> {
    fn from(i: UnknownEx<'a>) -> Item<'a> {
        Item::UnknownEx(i)
    }
}
