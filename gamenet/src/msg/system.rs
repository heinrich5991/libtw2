use buffer::CapacityError;
use common::pretty;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::with_packer;
use snap_obj::PlayerInput;
use std::fmt;
use super::SystemOrGame;
use warn::Warn;

impl<'a> System<'a> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>) -> Result<System<'a>, Error>
        where W: Warn<Warning>
    {
        if let SystemOrGame::System(msg_id) =
            SystemOrGame::decode_id(try!(p.read_int(warn)))
        {
            System::decode_msg(warn, msg_id, p)
        } else {
            Err(Error::UnknownId)
        }
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(p.write_int(SystemOrGame::System(self.msg_id()).encode_id()));
        try!(with_packer(&mut p, |p| self.encode_msg(p)));
        Ok(p.written())
    }
}

pub const INFO: i32 = 1;
pub const MAP_CHANGE: i32 = 2;
pub const MAP_DATA: i32 = 3;
pub const CON_READY: i32 = 4;
pub const SNAP: i32 = 5;
pub const SNAP_EMPTY: i32 = 6;
pub const SNAP_SINGLE: i32 = 7;
pub const INPUT_TIMING: i32 = 9;
pub const RCON_AUTH_STATUS: i32 = 10;
pub const RCON_LINE: i32 = 11;
pub const READY: i32 = 14;
pub const ENTER_GAME: i32 = 15;
pub const INPUT: i32 = 16;
pub const RCON_CMD: i32 = 17;
pub const RCON_AUTH: i32 = 18;
pub const REQUEST_MAP_DATA: i32 = 19;
pub const PING: i32 = 20;
pub const PING_REPLY: i32 = 21;
pub const RCON_CMD_ADD: i32 = 25;
pub const RCON_CMD_REMOVE: i32 = 26;

#[derive(Clone, Copy)]
pub struct Info<'a> {
    pub version: &'a [u8],
    pub password: Option<&'a [u8]>,
}

#[derive(Clone, Copy)]
pub struct MapChange<'a> {
    pub name: &'a [u8],
    pub crc: i32,
    pub size: i32,
}

#[derive(Clone, Copy)]
pub struct MapData<'a> {
    pub last: i32,
    pub crc: i32,
    pub chunk: i32,
    pub data: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct ConReady;

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
    pub auth_level: Option<i32>,
    pub receive_commands: Option<i32>,
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
    pub input: PlayerInput,
}

#[derive(Clone, Copy)]
pub struct RconCmd<'a> {
    pub cmd: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct RconAuth<'a> {
    pub _unused: &'a [u8],
    pub password: &'a [u8],
    pub request_commands: Option<i32>,
}

#[derive(Clone, Copy)]
pub struct RequestMapData {
    pub chunk: i32,
}

#[derive(Clone, Copy)]
pub struct Ping;

#[derive(Clone, Copy)]
pub struct PingReply;

#[derive(Clone, Copy)]
pub struct RconCmdAdd<'a> {
    pub name: &'a [u8],
    pub help: &'a [u8],
    pub params: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct RconCmdRemove<'a> {
    pub name: &'a [u8],
}

impl<'a> fmt::Debug for Info<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Info")
            .field("version", &pretty::Bytes::new(&self.version))
            .field("password", &self.password.as_ref().map(|password| pretty::Bytes::new(password)))
            .finish()
    }
}
impl<'a> Info<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Info<'a>, Error> {
        let result = Ok(Info {
            version: try!(_p.read_string()),
            password: _p.read_string().ok(),
        });
        _p.finish(warn);
        result
    }
}

impl<'a> fmt::Debug for MapChange<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapChange")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("crc", &self.crc)
            .field("size", &self.size)
            .finish()
    }
}
impl<'a> MapChange<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapChange<'a>, Error> {
        let result = Ok(MapChange {
            name: try!(_p.read_string()),
            crc: try!(_p.read_int(warn)),
            size: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.name));
        try!(_p.write_int(self.crc));
        try!(_p.write_int(self.size));
        Ok(_p.written())
    }
}

impl<'a> fmt::Debug for MapData<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapData")
            .field("last", &self.last)
            .field("crc", &self.crc)
            .field("chunk", &self.chunk)
            .field("data", &pretty::Bytes::new(&self.data))
            .finish()
    }
}
impl<'a> MapData<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapData<'a>, Error> {
        let result = Ok(MapData {
            last: try!(_p.read_int(warn)),
            crc: try!(_p.read_int(warn)),
            chunk: try!(_p.read_int(warn)),
            data: try!(_p.read_data(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.last));
        try!(_p.write_int(self.crc));
        try!(_p.write_int(self.chunk));
        try!(_p.write_data(self.data));
        Ok(_p.written())
    }
}

impl fmt::Debug for ConReady {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConReady")
            .finish()
    }
}
impl ConReady {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ConReady, Error> {
        let result = Ok(ConReady);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
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
impl<'a> Snap<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Snap<'a>, Error> {
        let result = Ok(Snap {
            tick: try!(_p.read_int(warn)),
            delta_tick: try!(_p.read_int(warn)),
            num_parts: try!(_p.read_int(warn)),
            part: try!(_p.read_int(warn)),
            crc: try!(_p.read_int(warn)),
            data: try!(_p.read_data(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.tick));
        try!(_p.write_int(self.delta_tick));
        try!(_p.write_int(self.num_parts));
        try!(_p.write_int(self.part));
        try!(_p.write_int(self.crc));
        try!(_p.write_data(self.data));
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
impl SnapEmpty {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<SnapEmpty, Error> {
        let result = Ok(SnapEmpty {
            tick: try!(_p.read_int(warn)),
            delta_tick: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.tick));
        try!(_p.write_int(self.delta_tick));
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
impl<'a> SnapSingle<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<SnapSingle<'a>, Error> {
        let result = Ok(SnapSingle {
            tick: try!(_p.read_int(warn)),
            delta_tick: try!(_p.read_int(warn)),
            crc: try!(_p.read_int(warn)),
            data: try!(_p.read_data(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.tick));
        try!(_p.write_int(self.delta_tick));
        try!(_p.write_int(self.crc));
        try!(_p.write_data(self.data));
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
impl InputTiming {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<InputTiming, Error> {
        let result = Ok(InputTiming {
            input_pred_tick: try!(_p.read_int(warn)),
            time_left: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.input_pred_tick));
        try!(_p.write_int(self.time_left));
        Ok(_p.written())
    }
}

impl fmt::Debug for RconAuthStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconAuthStatus")
            .field("auth_level", &self.auth_level)
            .field("receive_commands", &self.receive_commands)
            .finish()
    }
}
impl RconAuthStatus {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RconAuthStatus, Error> {
        let result = Ok(RconAuthStatus {
            auth_level: _p.read_int(warn).ok(),
            receive_commands: _p.read_int(warn).ok(),
        });
        _p.finish(warn);
        result
    }
}

impl<'a> fmt::Debug for RconLine<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconLine")
            .field("line", &pretty::Bytes::new(&self.line))
            .finish()
    }
}
impl<'a> RconLine<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconLine<'a>, Error> {
        let result = Ok(RconLine {
            line: try!(_p.read_string()),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.line));
        Ok(_p.written())
    }
}

impl fmt::Debug for Ready {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ready")
            .finish()
    }
}
impl Ready {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Ready, Error> {
        let result = Ok(Ready);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        Ok(_p.written())
    }
}

impl fmt::Debug for EnterGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EnterGame")
            .finish()
    }
}
impl EnterGame {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<EnterGame, Error> {
        let result = Ok(EnterGame);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
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
impl Input {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Input, Error> {
        let result = Ok(Input {
            ack_snapshot: try!(_p.read_int(warn)),
            intended_tick: try!(_p.read_int(warn)),
            input_size: try!(_p.read_int(warn)),
            input: try!(PlayerInput::decode_msg_inner(warn, _p)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.ack_snapshot));
        try!(_p.write_int(self.intended_tick));
        try!(_p.write_int(self.input_size));
        try!(with_packer(&mut _p, |p| self.input.encode_msg(p)));
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
impl<'a> RconCmd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconCmd<'a>, Error> {
        let result = Ok(RconCmd {
            cmd: try!(_p.read_string()),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.cmd));
        Ok(_p.written())
    }
}

impl<'a> fmt::Debug for RconAuth<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconAuth")
            .field("_unused", &pretty::Bytes::new(&self._unused))
            .field("password", &pretty::Bytes::new(&self.password))
            .field("request_commands", &self.request_commands)
            .finish()
    }
}
impl<'a> RconAuth<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconAuth<'a>, Error> {
        let result = Ok(RconAuth {
            _unused: try!(_p.read_string()),
            password: try!(_p.read_string()),
            request_commands: _p.read_int(warn).ok(),
        });
        _p.finish(warn);
        result
    }
}

impl fmt::Debug for RequestMapData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RequestMapData")
            .field("chunk", &self.chunk)
            .finish()
    }
}
impl RequestMapData {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RequestMapData, Error> {
        let result = Ok(RequestMapData {
            chunk: try!(_p.read_int(warn)),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_int(self.chunk));
        Ok(_p.written())
    }
}

impl fmt::Debug for Ping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ping")
            .finish()
    }
}
impl Ping {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Ping, Error> {
        let result = Ok(Ping);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        Ok(_p.written())
    }
}

impl fmt::Debug for PingReply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PingReply")
            .finish()
    }
}
impl PingReply {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PingReply, Error> {
        let result = Ok(PingReply);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        Ok(_p.written())
    }
}

impl<'a> fmt::Debug for RconCmdAdd<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconCmdAdd")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("help", &pretty::Bytes::new(&self.help))
            .field("params", &pretty::Bytes::new(&self.params))
            .finish()
    }
}
impl<'a> RconCmdAdd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconCmdAdd<'a>, Error> {
        let result = Ok(RconCmdAdd {
            name: try!(_p.read_string()),
            help: try!(_p.read_string()),
            params: try!(_p.read_string()),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.name));
        try!(_p.write_string(self.help));
        try!(_p.write_string(self.params));
        Ok(_p.written())
    }
}

impl<'a> fmt::Debug for RconCmdRemove<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconCmdRemove")
            .field("name", &pretty::Bytes::new(&self.name))
            .finish()
    }
}
impl<'a> RconCmdRemove<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconCmdRemove<'a>, Error> {
        let result = Ok(RconCmdRemove {
            name: try!(_p.read_string()),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.name));
        Ok(_p.written())
    }
}

impl<'a> Info<'a> {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self.version));
        try!(_p.write_string(self.password.expect("Info needs a password")));
        Ok(_p.written())
    }
}
impl RconAuthStatus {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        assert!(self.auth_level.is_some() || self.receive_commands.is_none());
        try!(self.auth_level.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
        try!(self.receive_commands.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
        Ok(_p.written())
    }
}
impl<'a> RconAuth<'a> {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        try!(_p.write_string(self._unused));
        try!(_p.write_string(self.password));
        try!(self.request_commands.map(|v| _p.write_int(v)).unwrap_or(Ok(())));
        Ok(_p.written())
    }
}

#[derive(Clone, Copy)]
pub enum System<'a> {
    Info(Info<'a>),
    MapChange(MapChange<'a>),
    MapData(MapData<'a>),
    ConReady(ConReady),
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
    RconCmdAdd(RconCmdAdd<'a>),
    RconCmdRemove(RconCmdRemove<'a>),
}

impl<'a> System<'a> {
    pub fn decode_msg<W>(warn: &mut W, msg_id: i32, p: &mut Unpacker<'a>)
        -> Result<System<'a>, Error>
        where W: Warn<Warning>
    {
        Ok(match msg_id {
            INFO => System::Info(try!(Info::decode(warn, p))),
            MAP_CHANGE => System::MapChange(try!(MapChange::decode(warn, p))),
            MAP_DATA => System::MapData(try!(MapData::decode(warn, p))),
            CON_READY => System::ConReady(try!(ConReady::decode(warn, p))),
            SNAP => System::Snap(try!(Snap::decode(warn, p))),
            SNAP_EMPTY => System::SnapEmpty(try!(SnapEmpty::decode(warn, p))),
            SNAP_SINGLE => System::SnapSingle(try!(SnapSingle::decode(warn, p))),
            INPUT_TIMING => System::InputTiming(try!(InputTiming::decode(warn, p))),
            RCON_AUTH_STATUS => System::RconAuthStatus(try!(RconAuthStatus::decode(warn, p))),
            RCON_LINE => System::RconLine(try!(RconLine::decode(warn, p))),
            READY => System::Ready(try!(Ready::decode(warn, p))),
            ENTER_GAME => System::EnterGame(try!(EnterGame::decode(warn, p))),
            INPUT => System::Input(try!(Input::decode(warn, p))),
            RCON_CMD => System::RconCmd(try!(RconCmd::decode(warn, p))),
            RCON_AUTH => System::RconAuth(try!(RconAuth::decode(warn, p))),
            REQUEST_MAP_DATA => System::RequestMapData(try!(RequestMapData::decode(warn, p))),
            PING => System::Ping(try!(Ping::decode(warn, p))),
            PING_REPLY => System::PingReply(try!(PingReply::decode(warn, p))),
            RCON_CMD_ADD => System::RconCmdAdd(try!(RconCmdAdd::decode(warn, p))),
            RCON_CMD_REMOVE => System::RconCmdRemove(try!(RconCmdRemove::decode(warn, p))),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn msg_id(&self) -> i32 {
        match *self {
            System::Info(_) => INFO,
            System::MapChange(_) => MAP_CHANGE,
            System::MapData(_) => MAP_DATA,
            System::ConReady(_) => CON_READY,
            System::Snap(_) => SNAP,
            System::SnapEmpty(_) => SNAP_EMPTY,
            System::SnapSingle(_) => SNAP_SINGLE,
            System::InputTiming(_) => INPUT_TIMING,
            System::RconAuthStatus(_) => RCON_AUTH_STATUS,
            System::RconLine(_) => RCON_LINE,
            System::Ready(_) => READY,
            System::EnterGame(_) => ENTER_GAME,
            System::Input(_) => INPUT,
            System::RconCmd(_) => RCON_CMD,
            System::RconAuth(_) => RCON_AUTH,
            System::RequestMapData(_) => REQUEST_MAP_DATA,
            System::Ping(_) => PING,
            System::PingReply(_) => PING_REPLY,
            System::RconCmdAdd(_) => RCON_CMD_ADD,
            System::RconCmdRemove(_) => RCON_CMD_REMOVE,
        }
    }
    pub fn encode_msg<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            System::Info(ref i) => i.encode(p),
            System::MapChange(ref i) => i.encode(p),
            System::MapData(ref i) => i.encode(p),
            System::ConReady(ref i) => i.encode(p),
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
            System::RconCmdAdd(ref i) => i.encode(p),
            System::RconCmdRemove(ref i) => i.encode(p),
        }
    }
}
impl<'a> fmt::Debug for System<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            System::Info(m) => m.fmt(f),
            System::MapChange(m) => m.fmt(f),
            System::MapData(m) => m.fmt(f),
            System::ConReady(m) => m.fmt(f),
            System::Snap(m) => m.fmt(f),
            System::SnapEmpty(m) => m.fmt(f),
            System::SnapSingle(m) => m.fmt(f),
            System::InputTiming(m) => m.fmt(f),
            System::RconAuthStatus(m) => m.fmt(f),
            System::RconLine(m) => m.fmt(f),
            System::Ready(m) => m.fmt(f),
            System::EnterGame(m) => m.fmt(f),
            System::Input(m) => m.fmt(f),
            System::RconCmd(m) => m.fmt(f),
            System::RconAuth(m) => m.fmt(f),
            System::RequestMapData(m) => m.fmt(f),
            System::Ping(m) => m.fmt(f),
            System::PingReply(m) => m.fmt(f),
            System::RconCmdAdd(m) => m.fmt(f),
            System::RconCmdRemove(m) => m.fmt(f),
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
impl<'a> From<ConReady> for System<'a> {
    fn from(i: ConReady) -> System<'a> {
        System::ConReady(i)
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
impl<'a> From<RconCmdAdd<'a>> for System<'a> {
    fn from(i: RconCmdAdd<'a>) -> System<'a> {
        System::RconCmdAdd(i)
    }
}
impl<'a> From<RconCmdRemove<'a>> for System<'a> {
    fn from(i: RconCmdRemove<'a>) -> System<'a> {
        System::RconCmdRemove(i)
    }
}

