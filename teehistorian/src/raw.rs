use common::num::Cast;
use itertools::zip_eq;
use packer::Unpacker;
use std::cmp;
use std::fmt;
use std::ops;
use vec_map::VecMap;

use bitmagic::CallbackExt;
use format::MaybeEnd;
use format::item::INPUT_LEN;
use format::item;
use format;

pub use format::Header;

macro_rules! unexp_end {
    ($e:expr) => {
        unexp_end!($e, Ok(None))
    };
    ($e:expr, $end:expr) => {
        match $e {
            Ok(x) => Ok(x),
            Err(MaybeEnd::UnexpectedEnd) => return $end,
            Err(MaybeEnd::Err(e)) => Err(e),
        }
    };
}

const BUFFER_SIZE: usize = 8192;

pub fn read_header(data: &[u8])
    -> Result<Option<(usize, Header)>, format::HeaderError>
{
    let mut p = Unpacker::new(data);
    unexp_end!(format::read_magic(&mut p))?;
    let header = unexp_end!(format::read_header(&mut p))?;
    Ok(Some((p.num_bytes_read(), header)))
}

pub trait Callback {
    type Error;
    /// Return `Ok(None)` on EOF, `Ok(Some(n))` if `n` bytes were successfully
    /// read (might be lower than `buffer.len()`.
    fn read_at_most(&mut self, buffer: &mut [u8])
        -> Result<Option<usize>, Self::Error>;
}

#[derive(Debug)]
pub enum Error<CE> {
    Teehistorian(format::Error),
    Cb(CE),
}

impl<CE> From<format::HeaderError> for Error<CE> {
    fn from(e: format::HeaderError) -> Error<CE> {
        let e: format::Error = e.into();
        e.into()
    }
}

impl<CE> From<item::UnknownType> for Error<CE> {
    fn from(e: item::UnknownType) -> Error<CE> {
        item::Error::from(e).into()
    }
}

impl<CE> From<MaybeEnd<item::UnknownType>> for MaybeEnd<Error<CE>> {
    fn from(me: MaybeEnd<item::UnknownType>) -> MaybeEnd<Error<CE>> {
        match me {
            MaybeEnd::Err(e) => MaybeEnd::Err(item::Error::from(e).into()),
            MaybeEnd::UnexpectedEnd => MaybeEnd::UnexpectedEnd,
        }
    }
}

impl<CE> From<item::Error> for Error<CE> {
    fn from(e: item::Error) -> Error<CE> {
        let e: format::Error = e.into();
        e.into()
    }
}

impl<CE> From<format::Error> for Error<CE> {
    fn from(e: format::Error) -> Error<CE> {
        Error::Teehistorian(e)
    }
}

pub struct WrapCallbackError<CE>(pub CE);
impl<CE> From<WrapCallbackError<CE>> for Error<CE> {
    fn from(err: WrapCallbackError<CE>) -> Error<CE> {
        Error::Cb(err.0)
    }
}
pub trait ResultExt {
    type ResultWrapped;
    fn wrap(self) -> Self::ResultWrapped;
}
impl<T, CE> ResultExt for Result<T, CE> {
    type ResultWrapped = Result<T, WrapCallbackError<CE>>;
    fn wrap(self) -> Result<T, WrapCallbackError<CE>> {
        self.map_err(WrapCallbackError)
    }
}

pub struct Buffer {
    offset: usize,
    buffer: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            offset: 0,
            buffer: Vec::new(),
        }
    }
    pub fn clear(&mut self) {
        self.offset = 0;
        self.buffer.clear();
    }
}

#[derive(Clone, Copy, Serialize)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    fn wrapping_add(self, other: Pos) -> Pos {
        Pos {
            x: self.x.wrapping_add(other.x),
            y: self.y.wrapping_add(other.y),
        }
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.x)
            .field(&self.y)
            .finish()
    }
}

pub struct Reader {
    version: format::Version,
    tick: i32,
    players: VecMap<Pos>,
    inputs: VecMap<[i32; INPUT_LEN]>,
    max_cid: i32,
    prev_player_cid: Option<i32>,
    next_item_kind: Option<item::Kind>,
    in_tick: bool,
}

impl Reader {
    fn empty(version: format::Version) -> Reader {
        Reader {
            version: version,
            tick: 0,
            players: VecMap::new(),
            inputs: VecMap::new(),
            max_cid: -1,
            prev_player_cid: None,
            next_item_kind: None,
            in_tick: false,
        }
    }
    pub fn new<'a, CB>(cb: &mut CB, buffer: &'a mut Buffer)
        -> Result<(Header<'a>, Reader), Error<CB::Error>>
        where CB: Callback,
    {
        let header = Reader::new_impl(cb, buffer)?;
        let reader = Reader::from_header(&header)?;
        Ok((header, reader))
    }
    fn new_impl<'a, CB>(cb: &mut CB, buffer: &'a mut Buffer)
        -> Result<Header<'a>, Error<CB::Error>>
        where CB: Callback,
    {
        loop {
            unsafe {
                // FIXME(rust-lang/rfcs#811): Work around missing non-lexical borrows.
                let raw_buffer: *mut Buffer = buffer;
                if let Some((read, header)) = read_header(&(*raw_buffer).buffer)? {
                    buffer.offset += read;
                    return Ok(header);
                }
                buffer.read_more(cb)?;
            }
        }
    }
    pub fn from_header(header: &Header) -> Result<Reader, format::Error> {
        Ok(match header.version {
            1 => Reader::empty(format::Version::V1),
            2 => Reader::empty(format::Version::V2),
            _ => return Err(format::Error::UnknownVersion),
        })
    }
    pub fn read<'a, CB>(&mut self, cb: &mut CB, buffer: &'a mut Buffer)
        -> Result<Option<Item<'a>>, Error<CB::Error>>
        where CB: Callback,
    {
        let item_kind = if let Some(ik) = self.next_item_kind.take() {
            ik
        } else {
            buffer.read_kind(cb, self.version)?
        };

        // WARN: Detect two consecutive `TickSkip`s.
        if item_kind != item::Kind::TickSkip &&
            item_kind != item::Kind::Finish &&
            !self.in_tick
        {
            self.next_item_kind = Some(item_kind);
            self.in_tick = true;
            return Ok(Some(Item::TickStart(self.tick)));
        }

        if let Some(cid) = item_kind.player_cid() {
            if self.prev_player_cid.map(|p| p >= cid).unwrap_or(false) {
                let old_tick = self.tick;
                self.tick = old_tick
                    .checked_add(1).ok_or(format::Error::TickOverflow)?;
                self.prev_player_cid = None;
                self.next_item_kind = Some(item_kind);
                self.in_tick = false;
                return Ok(Some(Item::TickEnd(old_tick)));
            }
        } else if item_kind == item::Kind::Finish && self.in_tick {
            self.next_item_kind = Some(item_kind);
            self.in_tick = false;
            return Ok(Some(Item::TickEnd(self.tick)));
        }

        let item = buffer.read_item(cb, item_kind)?;

        if let Some(cid) = item.cid() {
            self.max_cid = cmp::max(self.max_cid, cid);
        }

        Ok(Some(match item {
            format::Item::TickSkip(i) => {
                let old_tick = self.tick;
                let dt = i.dt.try_i32()
                    .ok_or(format::Error::TickOverflow)?;
                self.tick = self.tick
                    .checked_add(1).ok_or(format::Error::TickOverflow)?
                    .checked_add(dt).ok_or(format::Error::TickOverflow)?;
                if self.in_tick {
                    self.in_tick = false;
                    Item::TickEnd(old_tick)
                } else {
                    self.in_tick = true;
                    Item::TickStart(self.tick)
                }
            },
            format::Item::Message(i) => Item::Message(i),
            format::Item::Join(i) => Item::Join(i),
            format::Item::Drop(i) => Item::Drop(i),
            format::Item::ConsoleCommand(i) => Item::ConsoleCommand(i),
            format::Item::AuthInit(i) => Item::AuthInit(i),
            format::Item::AuthLogin(i) => Item::AuthLogin(i),
            format::Item::AuthLogout(i) => Item::AuthLogout(i),
            format::Item::Ddnetver(i) => Item::Ddnetver(i),
            format::Item::DdnetverOld(i) => Item::DdnetverOld(i),
            format::Item::Joinver6(i) => Item::Joinver6(i),
            format::Item::Joinver7(i) => Item::Joinver7(i),
            format::Item::PlayerTeam(i) => Item::PlayerTeam(i),
            format::Item::TeamLoadFailure(i) => Item::TeamLoadFailure(i),
            format::Item::TeamLoadSuccess(i) => Item::TeamLoadSuccess(i),
            format::Item::TeamPractice(i) => Item::TeamPractice(i),
            format::Item::TeamSaveFailure(i) => Item::TeamSaveFailure(i),
            format::Item::TeamSaveSuccess(i) => Item::TeamSaveSuccess(i),
            format::Item::UnknownEx(i) => Item::UnknownEx(i),

            format::Item::PlayerDiff(i) => {
                self.prev_player_cid = Some(i.cid);
                let cid = i.cid.try_usize().ok_or(format::Error::InvalidClientId)?;
                let player = self.players.get_mut(cid)
                    .ok_or(format::Error::PlayerDiffWithoutNew)?;
                let old_pos = *player;
                *player = player.wrapping_add(Pos { x: i.dx, y: i.dy });
                Item::PlayerChange(PlayerChange {
                    cid: i.cid,
                    pos: *player,
                    old_pos: old_pos,
                })
            }
            format::Item::PlayerNew(i) => {
                self.prev_player_cid = Some(i.cid);
                let cid = i.cid.try_usize().ok_or(format::Error::InvalidClientId)?;
                let pos = Pos { x: i.x, y: i.y };
                if self.players.insert(cid, pos).is_some() {
                    return Err(format::Error::PlayerNewDuplicate.into());
                }
                Item::PlayerNew(Player { cid: i.cid, pos: pos })
            },
            format::Item::PlayerOld(i) => {
                self.prev_player_cid = Some(i.cid);
                let cid = i.cid.try_usize().ok_or(format::Error::InvalidClientId)?;
                let pos = self.players.remove(cid)
                    .ok_or(format::Error::PlayerOldWithoutNew)?;
                Item::PlayerOld(Player { cid: i.cid, pos: pos })
            },
            format::Item::InputDiff(i) => {
                let cid = i.cid.try_usize().ok_or(format::Error::InvalidClientId)?;
                let input = self.inputs.get_mut(cid)
                    .ok_or(format::Error::InputDiffWithoutNew)?;
                for (i, d) in zip_eq(input.iter_mut(), i.diff.iter()) {
                    *i = i.wrapping_add(*d);
                }
                Item::Input(Input { cid: i.cid, input: *input })
            },
            format::Item::InputNew(i) => {
                let cid = i.cid.try_usize().ok_or(format::Error::InvalidClientId)?;
                if self.inputs.insert(cid, i.new).is_some() {
                    return Err(format::Error::InputDiffWithoutNew.into());
                }
                Item::Input(Input { cid: i.cid, input: i.new })
            },
            // WARN: Detect overlong teehistorian files.
            format::Item::Finish(format::item::Finish) => return Ok(None),
        }))
    }
    pub fn player_pos(&self, cid: i32) -> Option<Pos> {
        self.players.get(cid.assert_usize()).cloned()
    }
    pub fn input(&self, cid: i32) -> Option<[i32; INPUT_LEN]> {
        self.inputs.get(cid.assert_usize()).cloned()
    }
    pub fn cids(&self) -> ops::Range<i32> {
        0..self.max_cid+1
    }
}

impl Buffer {
    fn read_more<CB: Callback>(&mut self, cb: &mut CB)
        -> Result<(), Error<CB::Error>>
    {
        if self.buffer.len() != self.buffer.capacity() {
            if cb.read_buffer(&mut self.buffer).wrap()?.is_some() {
                Ok(())
            } else {
                Err(format::Error::UnexpectedEnd.into())
            }
        } else {
            if self.offset != 0 {
                self.buffer.drain(0..self.offset);
                self.offset = 0;
            } else {
                let len = self.buffer.len();
                self.buffer.reserve(if len < BUFFER_SIZE {
                    BUFFER_SIZE
                } else {
                    len
                });
            }
            self.read_more(cb)
        }
    }
    fn read_kind<CB>(&mut self, cb: &mut CB, version: format::Version)
        -> Result<item::Kind, Error<CB::Error>>
        where CB: Callback,
    {
        loop {
            let maybe_item_kind;
            let num_bytes_read;
            {
                let mut p = Unpacker::new(&self.buffer[self.offset..]);
                maybe_item_kind = item::Kind::decode(&mut p, version);
                num_bytes_read = p.num_bytes_read();
            }
            match maybe_item_kind {
                Ok(x) => {
                    self.offset += num_bytes_read;
                    return Ok(x);
                },
                Err(MaybeEnd::Err(x)) => return Err(x.into()),
                Err(MaybeEnd::UnexpectedEnd) => self.read_more(cb)?,
            }
        }
    }
    fn read_item<'a, CB>(&'a mut self, cb: &mut CB, kind: item::Kind)
        -> Result<format::Item<'a>, Error<CB::Error>>
        where CB: Callback,
    {
        // FIXME(rust-lang/rfcs#811): Work around missing non-lexical borrows.
        let raw_self: *mut Buffer = self;
        unsafe {
            loop {
                let mut p = Unpacker::new(&(*raw_self).buffer[self.offset..]);
                match kind.decode_rest(&mut p) {
                    Ok(x) => {
                        self.offset += p.num_bytes_read();
                        return Ok(x);
                    },
                    Err(MaybeEnd::Err(x)) => return Err(x.into()),
                    Err(MaybeEnd::UnexpectedEnd) => self.read_more(cb)?,
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Player {
    pub cid: i32,
    pub pos: Pos,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PlayerChange {
    pub cid: i32,
    pub pos: Pos,
    pub old_pos: Pos,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Input {
    pub cid: i32,
    pub input: [i32; INPUT_LEN],
}

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Item<'a> {
    TickStart(i32),
    TickEnd(i32),
    PlayerNew(Player),
    PlayerChange(PlayerChange),
    PlayerOld(Player),
    Input(Input),
    Message(item::Message<'a>),
    Join(item::Join),
    Drop(item::Drop<'a>),
    ConsoleCommand(item::ConsoleCommand<'a>),
    AuthInit(item::AuthInit<'a>),
    AuthLogin(item::AuthLogin<'a>),
    AuthLogout(item::AuthLogout),
    Ddnetver(item::Ddnetver<'a>),
    DdnetverOld(item::DdnetverOld),
    Joinver6(item::Joinver6),
    Joinver7(item::Joinver7),
    PlayerTeam(item::PlayerTeam),
    TeamLoadFailure(item::TeamLoadFailure),
    TeamLoadSuccess(item::TeamLoadSuccess<'a>),
    TeamPractice(item::TeamPractice),
    TeamSaveFailure(item::TeamSaveFailure),
    TeamSaveSuccess(item::TeamSaveSuccess<'a>),
    UnknownEx(item::UnknownEx<'a>),
}

impl<'a> fmt::Debug for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Item::TickStart(ref i) => f.debug_tuple("TickStart").field(&i).finish(),
            Item::TickEnd(ref i) => f.debug_tuple("TickEnd").field(&i).finish(),
            Item::PlayerNew(ref i) => {
                f.debug_struct("PlayerNew")
                    .field("cid", &i.cid)
                    .field("pos", &i.pos)
                    .finish()
            },
            Item::PlayerChange(ref i) => {
                f.debug_struct("PlayerChange")
                    .field("cid", &i.cid)
                    .field("pos", &i.pos)
                    .field("old_pos", &i.old_pos)
                    .finish()
            },
            Item::PlayerOld(ref i) => {
                f.debug_struct("PlayerOld")
                    .field("cid", &i.cid)
                    .field("pos", &i.pos)
                    .finish()
            },
            Item::Input(ref i) => i.fmt(f),
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
