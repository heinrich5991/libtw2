use arrayvec::ArrayVec;
use common::num::Cast;
use common::pretty;
use packer::UnexpectedEnd;
use packer::Unpacker;
use packer::positive;
use std::fmt;
use warn::Ignore;

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
    UnexpectedEnd,
    UnknownType(i32),
    NegativeDt,
    NegativeNumArgs,
    NumArgsTooLarge,
}

impl From<UnexpectedEnd> for Error {
    fn from(_: UnexpectedEnd) -> Error {
        Error::UnexpectedEnd
    }
}

impl<'a> Item<'a> {
    pub fn decode(p: &mut Unpacker<'a>) -> Result<Item<'a>, Error> {
        match p.read_int(&mut Ignore)? {
            x if x >= 0 => Ok(PlayerDiff::decode(x, p)?.into()),
            FINISH => Ok(Finish::decode(p)?.into()),
            TICK_SKIP => Ok(TickSkip::decode(p)?.into()),
            PLAYER_NEW => Ok(PlayerNew::decode(p)?.into()),
            PLAYER_OLD => Ok(PlayerOld::decode(p)?.into()),
            INPUT_DIFF => Ok(InputDiff::decode(p)?.into()),
            INPUT_NEW => Ok(InputNew::decode(p)?.into()),
            MESSAGE => Ok(Message::decode(p)?.into()),
            JOIN => Ok(Join::decode(p)?.into()),
            DROP => Ok(Drop::decode(p)?.into()),
            CONSOLE_COMMAND => Ok(ConsoleCommand::decode(p)?.into()),
            x => Err(Error::UnknownType(x)),
        }
    }
}

impl PlayerDiff {
    fn decode(cid: i32, _p: &mut Unpacker) -> Result<PlayerDiff, Error> {
        Ok(PlayerDiff {
            cid: cid,
            dx: _p.read_int(&mut Ignore)?,
            dy: _p.read_int(&mut Ignore)?,
        })
    }
}

impl Finish {
    fn decode(_p: &mut Unpacker) -> Result<Finish, Error> {
        Ok(Finish)
    }
}

impl TickSkip {
    fn decode(_p: &mut Unpacker) -> Result<TickSkip, Error> {
        Ok(TickSkip {
            dt: positive(_p.read_int(&mut Ignore)?)
                .map_err(|_| Error::NegativeDt)?
                .assert_u32()
        })
    }
}

impl PlayerNew {
    fn decode(_p: &mut Unpacker) -> Result<PlayerNew, Error> {
        Ok(PlayerNew {
            cid: _p.read_int(&mut Ignore)?,
            x: _p.read_int(&mut Ignore)?,
            y: _p.read_int(&mut Ignore)?,
        })
    }
}

impl PlayerOld {
    fn decode(_p: &mut Unpacker) -> Result<PlayerOld, Error> {
        Ok(PlayerOld {
            cid: _p.read_int(&mut Ignore)?,
        })
    }
}

impl InputDiff {
    fn decode(_p: &mut Unpacker) -> Result<InputDiff, Error> {
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
    fn decode(_p: &mut Unpacker) -> Result<InputNew, Error> {
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
    fn decode(_p: &mut Unpacker<'a>) -> Result<Message<'a>, Error> {
        Ok(Message {
            cid: _p.read_int(&mut Ignore)?,
            msg: _p.read_data(&mut Ignore)?
        })
    }
}

impl Join {
    fn decode(_p: &mut Unpacker) -> Result<Join, Error> {
        Ok(Join {
            cid: _p.read_int(&mut Ignore)?,
        })
    }
}

impl<'a> Drop<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<Drop<'a>, Error> {
        Ok(Drop {
            cid: _p.read_int(&mut Ignore)?,
            reason: _p.read_string()?
        })
    }
}

impl<'a> ConsoleCommand<'a> {
    fn decode(_p: &mut Unpacker<'a>) -> Result<ConsoleCommand<'a>, Error> {
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
