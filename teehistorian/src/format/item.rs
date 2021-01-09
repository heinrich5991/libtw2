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
    Joinver6(Joinver6),
    Joinver7(Joinver7),

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

#[derive(Clone, Debug, Serialize)]
pub struct Joinver6 {
    pub cid: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct Joinver7 {
    pub cid: i32,
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
            UUID_JOINVER6 => Joinver6::decode(&mut Unpacker::new(data))?.into(),
            UUID_JOINVER7 => Joinver7::decode(&mut Unpacker::new(data))?.into(),
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
            Item::Joinver6(ref i) => i.cid,
            Item::Joinver7(ref i) => i.cid,
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
            Item::Joinver6(ref i) => i.fmt(f),
            Item::Joinver7(ref i) => i.fmt(f),
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
            .field("reason", &pretty::Bytes::new(&self.reason))
            .finish()
    }
}

impl<'a> fmt::Debug for ConsoleCommand<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConsoleCommand")
            .field("cid", &self.cid)
            .field("flag_mask", &self.flag_mask)
            .field("cmd", &pretty::Bytes::new(&self.cmd))
            .field("args", &pretty::BytesSlice::new(&self.args))
            .finish()
    }
}

impl<'a> fmt::Debug for AuthInit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthInit")
            .field("cid", &self.cid)
            .field("level", &self.level)
            .field("identity", &pretty::Bytes::new(&self.identity))
            .finish()
    }
}

impl<'a> fmt::Debug for AuthLogin<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("AuthLogin")
            .field("cid", &self.cid)
            .field("level", &self.level)
            .field("identity", &pretty::Bytes::new(&self.identity))
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

impl<'a> From<UnknownEx<'a>> for Item<'a> {
    fn from(i: UnknownEx<'a>) -> Item<'a> {
        Item::UnknownEx(i)
    }
}
