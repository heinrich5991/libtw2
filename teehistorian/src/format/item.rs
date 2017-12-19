use arrayvec::ArrayVec;
use common::num::Cast;
use common::pretty;
use packer::Unpacker;
use packer::positive;
use std::fmt;
use warn::Ignore;

use super::MaybeEnd;

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

pub const INPUT_LEN: usize = 10;
pub const CONSOLE_COMMAND_MAX_ARGS: usize = 16;

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
    pub fn decode(p: &mut Unpacker) -> Result<Kind, MaybeEnd<UnknownType>> {
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

#[derive(Clone)]
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
}

#[derive(Clone, Debug)]
pub struct PlayerDiff {
    pub cid: i32,
    pub dx: i32,
    pub dy: i32,
}

#[derive(Clone, Debug)]
pub struct Finish;

#[derive(Clone, Debug)]
pub struct TickSkip {
    pub dt: u32,
}

#[derive(Clone, Debug)]
pub struct PlayerNew {
    pub cid: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug)]
pub struct PlayerOld {
    pub cid: i32,
}

#[derive(Clone, Debug)]
pub struct InputDiff {
    pub cid: i32,
    pub diff: [i32; INPUT_LEN],
}

#[derive(Clone, Debug)]
pub struct InputNew {
    pub cid: i32,
    pub new: [i32; INPUT_LEN],
}

#[derive(Clone)]
pub struct Message<'a> {
    pub cid: i32,
    pub msg: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct Join {
    pub cid: i32,
}

#[derive(Clone)]
pub struct Drop<'a> {
    pub cid: i32,
    pub reason: &'a [u8],
}

#[derive(Clone)]
pub struct ConsoleCommand<'a> {
    pub cid: i32,
    pub flag_mask: u32,
    pub cmd: &'a [u8],
    pub args: ArrayVec<[&'a [u8]; CONSOLE_COMMAND_MAX_ARGS]>,
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
    pub fn decode(p: &mut Unpacker<'a>) -> Result<Item<'a>, MaybeEnd<Error>> {
        Kind::decode(p)?.decode_rest(p)
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
