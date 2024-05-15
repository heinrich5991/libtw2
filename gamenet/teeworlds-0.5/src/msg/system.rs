use crate::error::Error;
use buffer::CapacityError;
use libtw2_common::pretty;
use libtw2_packer::Packer;
use libtw2_packer::Unpacker;
use libtw2_packer::Warning;
use libtw2_packer::with_packer;
use std::fmt;
use super::MessageId;
use super::SystemOrGame;
use warn::Warn;

impl<'a> System<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<System<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::System(msg_id) = SystemOrGame::decode_id(warn, p)? {
            System::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        with_packer(&mut p, |p| SystemOrGame::System(self.msg_id()).encode_id(p))?;
        with_packer(&mut p, |p| self.encode_msg(p))?;
        Ok(p.written())
    }
}

pub const INFO: i32 = 1;
pub const MAP_CHANGE: i32 = 2;
pub const MAP_DATA: i32 = 3;
pub const SNAP: i32 = 4;
pub const SNAP_EMPTY: i32 = 5;
pub const SNAP_SINGLE: i32 = 6;
pub const INPUT_TIMING: i32 = 8;
pub const RCON_AUTH_STATUS: i32 = 9;
pub const RCON_LINE: i32 = 10;
pub const READY: i32 = 13;
pub const ENTER_GAME: i32 = 14;
pub const INPUT: i32 = 15;
pub const RCON_CMD: i32 = 16;
pub const RCON_AUTH: i32 = 17;
pub const REQUEST_MAP_DATA: i32 = 18;
pub const PING: i32 = 21;
pub const PING_REPLY: i32 = 22;

#[derive(Clone, Copy)]
pub enum System<'a> {
    Info(Info<'a>),
    MapChange(MapChange<'a>),
    MapData(MapData<'a>),
    Snap(Snap<'a>),
    SnapEmpty(SnapEmpty),
    SnapSingle(SnapSingle<'a>),
    InputTiming(InputTiming),
    RconAuthStatus(RconAuthStatus),
    RconLine(RconLine<'a>),
    Ready(Ready),
    EnterGame(EnterGame),
    Input(Input),
    RconCmd(RconCmd<'a>),
    RconAuth(RconAuth<'a>),
    RequestMapData(RequestMapData),
    Ping(Ping),
    PingReply(PingReply),
}

impl<'a> System<'a> {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: MessageId, _p: &mut Unpacker<'a>) -> Result<System<'a>, Error> {
        use self::MessageId::*;
        Ok(match msg_id {
            Ordinal(INFO) => System::Info(Info::decode(warn, _p)?),
            Ordinal(MAP_CHANGE) => System::MapChange(MapChange::decode(warn, _p)?),
            Ordinal(MAP_DATA) => System::MapData(MapData::decode(warn, _p)?),
            Ordinal(SNAP) => System::Snap(Snap::decode(warn, _p)?),
            Ordinal(SNAP_EMPTY) => System::SnapEmpty(SnapEmpty::decode(warn, _p)?),
            Ordinal(SNAP_SINGLE) => System::SnapSingle(SnapSingle::decode(warn, _p)?),
            Ordinal(INPUT_TIMING) => System::InputTiming(InputTiming::decode(warn, _p)?),
            Ordinal(RCON_AUTH_STATUS) => System::RconAuthStatus(RconAuthStatus::decode(warn, _p)?),
            Ordinal(RCON_LINE) => System::RconLine(RconLine::decode(warn, _p)?),
            Ordinal(READY) => System::Ready(Ready::decode(warn, _p)?),
            Ordinal(ENTER_GAME) => System::EnterGame(EnterGame::decode(warn, _p)?),
            Ordinal(INPUT) => System::Input(Input::decode(warn, _p)?),
            Ordinal(RCON_CMD) => System::RconCmd(RconCmd::decode(warn, _p)?),
            Ordinal(RCON_AUTH) => System::RconAuth(RconAuth::decode(warn, _p)?),
            Ordinal(REQUEST_MAP_DATA) => System::RequestMapData(RequestMapData::decode(warn, _p)?),
            Ordinal(PING) => System::Ping(Ping::decode(warn, _p)?),
            Ordinal(PING_REPLY) => System::PingReply(PingReply::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn msg_id(&self) -> MessageId {
        match *self {
            System::Info(_) => MessageId::from(INFO),
            System::MapChange(_) => MessageId::from(MAP_CHANGE),
            System::MapData(_) => MessageId::from(MAP_DATA),
            System::Snap(_) => MessageId::from(SNAP),
            System::SnapEmpty(_) => MessageId::from(SNAP_EMPTY),
            System::SnapSingle(_) => MessageId::from(SNAP_SINGLE),
            System::InputTiming(_) => MessageId::from(INPUT_TIMING),
            System::RconAuthStatus(_) => MessageId::from(RCON_AUTH_STATUS),
            System::RconLine(_) => MessageId::from(RCON_LINE),
            System::Ready(_) => MessageId::from(READY),
            System::EnterGame(_) => MessageId::from(ENTER_GAME),
            System::Input(_) => MessageId::from(INPUT),
            System::RconCmd(_) => MessageId::from(RCON_CMD),
            System::RconAuth(_) => MessageId::from(RCON_AUTH),
            System::RequestMapData(_) => MessageId::from(REQUEST_MAP_DATA),
            System::Ping(_) => MessageId::from(PING),
            System::PingReply(_) => MessageId::from(PING_REPLY),
        }
    }
    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            System::Info(ref i) => i.encode(p),
            System::MapChange(ref i) => i.encode(p),
            System::MapData(ref i) => i.encode(p),
            System::Snap(ref i) => i.encode(p),
            System::SnapEmpty(ref i) => i.encode(p),
            System::SnapSingle(ref i) => i.encode(p),
            System::InputTiming(ref i) => i.encode(p),
            System::RconAuthStatus(ref i) => i.encode(p),
            System::RconLine(ref i) => i.encode(p),
            System::Ready(ref i) => i.encode(p),
            System::EnterGame(ref i) => i.encode(p),
            System::Input(ref i) => i.encode(p),
            System::RconCmd(ref i) => i.encode(p),
            System::RconAuth(ref i) => i.encode(p),
            System::RequestMapData(ref i) => i.encode(p),
            System::Ping(ref i) => i.encode(p),
            System::PingReply(ref i) => i.encode(p),
        }
    }
}

impl<'a> fmt::Debug for System<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            System::Info(ref i) => i.fmt(f),
            System::MapChange(ref i) => i.fmt(f),
            System::MapData(ref i) => i.fmt(f),
            System::Snap(ref i) => i.fmt(f),
            System::SnapEmpty(ref i) => i.fmt(f),
            System::SnapSingle(ref i) => i.fmt(f),
            System::InputTiming(ref i) => i.fmt(f),
            System::RconAuthStatus(ref i) => i.fmt(f),
            System::RconLine(ref i) => i.fmt(f),
            System::Ready(ref i) => i.fmt(f),
            System::EnterGame(ref i) => i.fmt(f),
            System::Input(ref i) => i.fmt(f),
            System::RconCmd(ref i) => i.fmt(f),
            System::RconAuth(ref i) => i.fmt(f),
            System::RequestMapData(ref i) => i.fmt(f),
            System::Ping(ref i) => i.fmt(f),
            System::PingReply(ref i) => i.fmt(f),
        }
    }
}

impl<'a> From<Info<'a>> for System<'a> {
    fn from(i: Info<'a>) -> System<'a> {
        System::Info(i)
    }
}

impl<'a> From<MapChange<'a>> for System<'a> {
    fn from(i: MapChange<'a>) -> System<'a> {
        System::MapChange(i)
    }
}

impl<'a> From<MapData<'a>> for System<'a> {
    fn from(i: MapData<'a>) -> System<'a> {
        System::MapData(i)
    }
}

impl<'a> From<Snap<'a>> for System<'a> {
    fn from(i: Snap<'a>) -> System<'a> {
        System::Snap(i)
    }
}

impl<'a> From<SnapEmpty> for System<'a> {
    fn from(i: SnapEmpty) -> System<'a> {
        System::SnapEmpty(i)
    }
}

impl<'a> From<SnapSingle<'a>> for System<'a> {
    fn from(i: SnapSingle<'a>) -> System<'a> {
        System::SnapSingle(i)
    }
}

impl<'a> From<InputTiming> for System<'a> {
    fn from(i: InputTiming) -> System<'a> {
        System::InputTiming(i)
    }
}

impl<'a> From<RconAuthStatus> for System<'a> {
    fn from(i: RconAuthStatus) -> System<'a> {
        System::RconAuthStatus(i)
    }
}

impl<'a> From<RconLine<'a>> for System<'a> {
    fn from(i: RconLine<'a>) -> System<'a> {
        System::RconLine(i)
    }
}

impl<'a> From<Ready> for System<'a> {
    fn from(i: Ready) -> System<'a> {
        System::Ready(i)
    }
}

impl<'a> From<EnterGame> for System<'a> {
    fn from(i: EnterGame) -> System<'a> {
        System::EnterGame(i)
    }
}

impl<'a> From<Input> for System<'a> {
    fn from(i: Input) -> System<'a> {
        System::Input(i)
    }
}

impl<'a> From<RconCmd<'a>> for System<'a> {
    fn from(i: RconCmd<'a>) -> System<'a> {
        System::RconCmd(i)
    }
}

impl<'a> From<RconAuth<'a>> for System<'a> {
    fn from(i: RconAuth<'a>) -> System<'a> {
        System::RconAuth(i)
    }
}

impl<'a> From<RequestMapData> for System<'a> {
    fn from(i: RequestMapData) -> System<'a> {
        System::RequestMapData(i)
    }
}

impl<'a> From<Ping> for System<'a> {
    fn from(i: Ping) -> System<'a> {
        System::Ping(i)
    }
}

impl<'a> From<PingReply> for System<'a> {
    fn from(i: PingReply) -> System<'a> {
        System::PingReply(i)
    }
}
#[derive(Clone, Copy)]
pub struct Info<'a> {
    pub version: &'a [u8],
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub password: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct MapChange<'a> {
    pub name: &'a [u8],
    pub crc: i32,
}

#[derive(Clone, Copy)]
pub struct MapData<'a> {
    pub last: i32,
    pub total_size: i32,
    pub data: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct Snap<'a> {
    pub tick: i32,
    pub delta_tick: i32,
    pub num_parts: i32,
    pub part: i32,
    pub crc: i32,
    pub data: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct SnapEmpty {
    pub tick: i32,
    pub delta_tick: i32,
}

#[derive(Clone, Copy)]
pub struct SnapSingle<'a> {
    pub tick: i32,
    pub delta_tick: i32,
    pub crc: i32,
    pub data: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct InputTiming {
    pub input_pred_tick: i32,
    pub time_left: i32,
}

#[derive(Clone, Copy)]
pub struct RconAuthStatus {
    pub authed: i32,
}

#[derive(Clone, Copy)]
pub struct RconLine<'a> {
    pub line: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct Ready;

#[derive(Clone, Copy)]
pub struct EnterGame;

#[derive(Clone, Copy)]
pub struct Input {
    pub ack_snapshot: i32,
    pub intended_tick: i32,
    pub input_size: i32,
    pub input: crate::snap_obj::PlayerInput,
}

#[derive(Clone, Copy)]
pub struct RconCmd<'a> {
    pub cmd: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct RconAuth<'a> {
    pub _unused: &'a [u8],
    pub password: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct RequestMapData {
    pub chunk: i32,
}

#[derive(Clone, Copy)]
pub struct Ping;

#[derive(Clone, Copy)]
pub struct PingReply;

impl<'a> Info<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Info<'a>, Error> {
        let result = Ok(Info {
            version: _p.read_string()?,
            name: _p.read_string()?,
            clan: _p.read_string()?,
            password: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.version)?;
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_string(self.password)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for Info<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Info")
            .field("version", &pretty::Bytes::new(&self.version))
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("password", &pretty::Bytes::new(&self.password))
            .finish()
    }
}

impl<'a> MapChange<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapChange<'a>, Error> {
        let result = Ok(MapChange {
            name: _p.read_string()?,
            crc: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.name)?;
        _p.write_int(self.crc)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for MapChange<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapChange")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("crc", &self.crc)
            .finish()
    }
}

impl<'a> MapData<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapData<'a>, Error> {
        let result = Ok(MapData {
            last: _p.read_int(warn)?,
            total_size: _p.read_int(warn)?,
            data: _p.read_data(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.last)?;
        _p.write_int(self.total_size)?;
        _p.write_data(self.data)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for MapData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapData")
            .field("last", &self.last)
            .field("total_size", &self.total_size)
            .field("data", &pretty::Bytes::new(&self.data))
            .finish()
    }
}

impl<'a> Snap<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Snap<'a>, Error> {
        let result = Ok(Snap {
            tick: _p.read_int(warn)?,
            delta_tick: _p.read_int(warn)?,
            num_parts: _p.read_int(warn)?,
            part: _p.read_int(warn)?,
            crc: _p.read_int(warn)?,
            data: _p.read_data(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.tick)?;
        _p.write_int(self.delta_tick)?;
        _p.write_int(self.num_parts)?;
        _p.write_int(self.part)?;
        _p.write_int(self.crc)?;
        _p.write_data(self.data)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for Snap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Snap")
            .field("tick", &self.tick)
            .field("delta_tick", &self.delta_tick)
            .field("num_parts", &self.num_parts)
            .field("part", &self.part)
            .field("crc", &self.crc)
            .field("data", &pretty::Bytes::new(&self.data))
            .finish()
    }
}

impl SnapEmpty {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SnapEmpty, Error> {
        let result = Ok(SnapEmpty {
            tick: _p.read_int(warn)?,
            delta_tick: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.tick)?;
        _p.write_int(self.delta_tick)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for SnapEmpty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SnapEmpty")
            .field("tick", &self.tick)
            .field("delta_tick", &self.delta_tick)
            .finish()
    }
}

impl<'a> SnapSingle<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SnapSingle<'a>, Error> {
        let result = Ok(SnapSingle {
            tick: _p.read_int(warn)?,
            delta_tick: _p.read_int(warn)?,
            crc: _p.read_int(warn)?,
            data: _p.read_data(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.tick)?;
        _p.write_int(self.delta_tick)?;
        _p.write_int(self.crc)?;
        _p.write_data(self.data)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for SnapSingle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SnapSingle")
            .field("tick", &self.tick)
            .field("delta_tick", &self.delta_tick)
            .field("crc", &self.crc)
            .field("data", &pretty::Bytes::new(&self.data))
            .finish()
    }
}

impl InputTiming {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<InputTiming, Error> {
        let result = Ok(InputTiming {
            input_pred_tick: _p.read_int(warn)?,
            time_left: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.input_pred_tick)?;
        _p.write_int(self.time_left)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for InputTiming {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InputTiming")
            .field("input_pred_tick", &self.input_pred_tick)
            .field("time_left", &self.time_left)
            .finish()
    }
}

impl RconAuthStatus {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RconAuthStatus, Error> {
        let result = Ok(RconAuthStatus {
            authed: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.authed)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for RconAuthStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconAuthStatus")
            .field("authed", &self.authed)
            .finish()
    }
}

impl<'a> RconLine<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconLine<'a>, Error> {
        let result = Ok(RconLine {
            line: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.line)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for RconLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconLine")
            .field("line", &pretty::Bytes::new(&self.line))
            .finish()
    }
}

impl Ready {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Ready, Error> {
        let result = Ok(Ready);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for Ready {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ready")
            .finish()
    }
}

impl EnterGame {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<EnterGame, Error> {
        let result = Ok(EnterGame);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for EnterGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EnterGame")
            .finish()
    }
}

impl Input {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Input, Error> {
        let result = Ok(Input {
            ack_snapshot: _p.read_int(warn)?,
            intended_tick: _p.read_int(warn)?,
            input_size: _p.read_int(warn)?,
            input: crate::snap_obj::PlayerInput::decode_msg(warn, _p)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.ack_snapshot)?;
        _p.write_int(self.intended_tick)?;
        _p.write_int(self.input_size)?;
        with_packer(&mut _p, |p| self.input.encode_msg(p))?;
        Ok(_p.written())
    }
}
impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Input")
            .field("ack_snapshot", &self.ack_snapshot)
            .field("intended_tick", &self.intended_tick)
            .field("input_size", &self.input_size)
            .field("input", &self.input)
            .finish()
    }
}

impl<'a> RconCmd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconCmd<'a>, Error> {
        let result = Ok(RconCmd {
            cmd: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.cmd)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for RconCmd<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconCmd")
            .field("cmd", &pretty::Bytes::new(&self.cmd))
            .finish()
    }
}

impl<'a> RconAuth<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconAuth<'a>, Error> {
        let result = Ok(RconAuth {
            _unused: _p.read_string()?,
            password: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self._unused)?;
        _p.write_string(self.password)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for RconAuth<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconAuth")
            .field("_unused", &pretty::Bytes::new(&self._unused))
            .field("password", &pretty::Bytes::new(&self.password))
            .finish()
    }
}

impl RequestMapData {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RequestMapData, Error> {
        let result = Ok(RequestMapData {
            chunk: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.chunk)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for RequestMapData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RequestMapData")
            .field("chunk", &self.chunk)
            .finish()
    }
}

impl Ping {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Ping, Error> {
        let result = Ok(Ping);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for Ping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ping")
            .finish()
    }
}

impl PingReply {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PingReply, Error> {
        let result = Ok(PingReply);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for PingReply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PingReply")
            .finish()
    }
}

