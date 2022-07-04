use buffer::CapacityError;
use common::digest::Sha256;
use common::pretty;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::to_bool;
use packer::with_packer;
use std::fmt;
use super::MessageId;
use super::SystemOrGame;
use uuid::Uuid;
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
pub const WHAT_IS: Uuid = Uuid::from_u128(0x245e5097_9fe0_39d6_bf7d_9a29e1691e4c);
pub const IT_IS: Uuid = Uuid::from_u128(0x6954847e_2e87_3603_b562_36da29ed1aca);
pub const I_DONT_KNOW: Uuid = Uuid::from_u128(0x416911b5_7973_33bf_8d52_7bf01e519cf0);
pub const RCON_TYPE: Uuid = Uuid::from_u128(0x12810e1f_a1db_3378_b4fb_164ed6505926);
pub const MAP_DETAILS: Uuid = Uuid::from_u128(0xf9117b3c_8039_3416_9fc0_aef2bcb75c03);
pub const CAPABILITIES: Uuid = Uuid::from_u128(0xf621a5a1_f585_3775_8e73_41beee79f2b2);
pub const CLIENT_VERSION: Uuid = Uuid::from_u128(0x8c001304_8461_3e47_8787_f672b3835bd4);
pub const PING_EX: Uuid = Uuid::from_u128(0xbcb43bf5_427c_36d8_b5b8_7975c8c06aa1);
pub const PONG_EX: Uuid = Uuid::from_u128(0xd8295530_14a7_3a0a_b02e_b2cee08d2033);
pub const CHECKSUM_REQUEST: Uuid = Uuid::from_u128(0x60a7cef1_2ecc_3ed4_b138_00fd0c8f5994);
pub const CHECKSUM_RESPONSE: Uuid = Uuid::from_u128(0x88fc61ec_5a3c_3fc3_8dfa_fd3b715db9e0);
pub const CHECKSUM_ERROR: Uuid = Uuid::from_u128(0x090960d1_4000_3fd5_9670_4976ae702a6a);

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
    WhatIs(WhatIs),
    ItIs(ItIs<'a>),
    IDontKnow(IDontKnow),
    RconType(RconType),
    MapDetails(MapDetails<'a>),
    Capabilities(Capabilities),
    ClientVersion(ClientVersion<'a>),
    PingEx(PingEx),
    PongEx(PongEx),
    ChecksumRequest(ChecksumRequest),
    ChecksumResponse(ChecksumResponse),
    ChecksumError(ChecksumError),
}

impl<'a> System<'a> {
    pub fn decode_msg<W: Warn<Warning>>(warn: &mut W, msg_id: MessageId, _p: &mut Unpacker<'a>) -> Result<System<'a>, Error> {
        use self::MessageId::*;
        Ok(match msg_id {
            Ordinal(INFO) => System::Info(Info::decode(warn, _p)?),
            Ordinal(MAP_CHANGE) => System::MapChange(MapChange::decode(warn, _p)?),
            Ordinal(MAP_DATA) => System::MapData(MapData::decode(warn, _p)?),
            Ordinal(CON_READY) => System::ConReady(ConReady::decode(warn, _p)?),
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
            Ordinal(RCON_CMD_ADD) => System::RconCmdAdd(RconCmdAdd::decode(warn, _p)?),
            Ordinal(RCON_CMD_REMOVE) => System::RconCmdRemove(RconCmdRemove::decode(warn, _p)?),
            Uuid(WHAT_IS) => System::WhatIs(WhatIs::decode(warn, _p)?),
            Uuid(IT_IS) => System::ItIs(ItIs::decode(warn, _p)?),
            Uuid(I_DONT_KNOW) => System::IDontKnow(IDontKnow::decode(warn, _p)?),
            Uuid(RCON_TYPE) => System::RconType(RconType::decode(warn, _p)?),
            Uuid(MAP_DETAILS) => System::MapDetails(MapDetails::decode(warn, _p)?),
            Uuid(CAPABILITIES) => System::Capabilities(Capabilities::decode(warn, _p)?),
            Uuid(CLIENT_VERSION) => System::ClientVersion(ClientVersion::decode(warn, _p)?),
            Uuid(PING_EX) => System::PingEx(PingEx::decode(warn, _p)?),
            Uuid(PONG_EX) => System::PongEx(PongEx::decode(warn, _p)?),
            Uuid(CHECKSUM_REQUEST) => System::ChecksumRequest(ChecksumRequest::decode(warn, _p)?),
            Uuid(CHECKSUM_RESPONSE) => System::ChecksumResponse(ChecksumResponse::decode(warn, _p)?),
            Uuid(CHECKSUM_ERROR) => System::ChecksumError(ChecksumError::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn msg_id(&self) -> MessageId {
        match *self {
            System::Info(_) => MessageId::from(INFO),
            System::MapChange(_) => MessageId::from(MAP_CHANGE),
            System::MapData(_) => MessageId::from(MAP_DATA),
            System::ConReady(_) => MessageId::from(CON_READY),
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
            System::RconCmdAdd(_) => MessageId::from(RCON_CMD_ADD),
            System::RconCmdRemove(_) => MessageId::from(RCON_CMD_REMOVE),
            System::WhatIs(_) => MessageId::from(WHAT_IS),
            System::ItIs(_) => MessageId::from(IT_IS),
            System::IDontKnow(_) => MessageId::from(I_DONT_KNOW),
            System::RconType(_) => MessageId::from(RCON_TYPE),
            System::MapDetails(_) => MessageId::from(MAP_DETAILS),
            System::Capabilities(_) => MessageId::from(CAPABILITIES),
            System::ClientVersion(_) => MessageId::from(CLIENT_VERSION),
            System::PingEx(_) => MessageId::from(PING_EX),
            System::PongEx(_) => MessageId::from(PONG_EX),
            System::ChecksumRequest(_) => MessageId::from(CHECKSUM_REQUEST),
            System::ChecksumResponse(_) => MessageId::from(CHECKSUM_RESPONSE),
            System::ChecksumError(_) => MessageId::from(CHECKSUM_ERROR),
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
            System::WhatIs(ref i) => i.encode(p),
            System::ItIs(ref i) => i.encode(p),
            System::IDontKnow(ref i) => i.encode(p),
            System::RconType(ref i) => i.encode(p),
            System::MapDetails(ref i) => i.encode(p),
            System::Capabilities(ref i) => i.encode(p),
            System::ClientVersion(ref i) => i.encode(p),
            System::PingEx(ref i) => i.encode(p),
            System::PongEx(ref i) => i.encode(p),
            System::ChecksumRequest(ref i) => i.encode(p),
            System::ChecksumResponse(ref i) => i.encode(p),
            System::ChecksumError(ref i) => i.encode(p),
        }
    }
}

impl<'a> fmt::Debug for System<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            System::Info(ref i) => i.fmt(f),
            System::MapChange(ref i) => i.fmt(f),
            System::MapData(ref i) => i.fmt(f),
            System::ConReady(ref i) => i.fmt(f),
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
            System::RconCmdAdd(ref i) => i.fmt(f),
            System::RconCmdRemove(ref i) => i.fmt(f),
            System::WhatIs(ref i) => i.fmt(f),
            System::ItIs(ref i) => i.fmt(f),
            System::IDontKnow(ref i) => i.fmt(f),
            System::RconType(ref i) => i.fmt(f),
            System::MapDetails(ref i) => i.fmt(f),
            System::Capabilities(ref i) => i.fmt(f),
            System::ClientVersion(ref i) => i.fmt(f),
            System::PingEx(ref i) => i.fmt(f),
            System::PongEx(ref i) => i.fmt(f),
            System::ChecksumRequest(ref i) => i.fmt(f),
            System::ChecksumResponse(ref i) => i.fmt(f),
            System::ChecksumError(ref i) => i.fmt(f),
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

impl<'a> From<WhatIs> for System<'a> {
    fn from(i: WhatIs) -> System<'a> {
        System::WhatIs(i)
    }
}

impl<'a> From<ItIs<'a>> for System<'a> {
    fn from(i: ItIs<'a>) -> System<'a> {
        System::ItIs(i)
    }
}

impl<'a> From<IDontKnow> for System<'a> {
    fn from(i: IDontKnow) -> System<'a> {
        System::IDontKnow(i)
    }
}

impl<'a> From<RconType> for System<'a> {
    fn from(i: RconType) -> System<'a> {
        System::RconType(i)
    }
}

impl<'a> From<MapDetails<'a>> for System<'a> {
    fn from(i: MapDetails<'a>) -> System<'a> {
        System::MapDetails(i)
    }
}

impl<'a> From<Capabilities> for System<'a> {
    fn from(i: Capabilities) -> System<'a> {
        System::Capabilities(i)
    }
}

impl<'a> From<ClientVersion<'a>> for System<'a> {
    fn from(i: ClientVersion<'a>) -> System<'a> {
        System::ClientVersion(i)
    }
}

impl<'a> From<PingEx> for System<'a> {
    fn from(i: PingEx) -> System<'a> {
        System::PingEx(i)
    }
}

impl<'a> From<PongEx> for System<'a> {
    fn from(i: PongEx) -> System<'a> {
        System::PongEx(i)
    }
}

impl<'a> From<ChecksumRequest> for System<'a> {
    fn from(i: ChecksumRequest) -> System<'a> {
        System::ChecksumRequest(i)
    }
}

impl<'a> From<ChecksumResponse> for System<'a> {
    fn from(i: ChecksumResponse) -> System<'a> {
        System::ChecksumResponse(i)
    }
}

impl<'a> From<ChecksumError> for System<'a> {
    fn from(i: ChecksumError) -> System<'a> {
        System::ChecksumError(i)
    }
}
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
    pub input: ::snap_obj::PlayerInput,
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

#[derive(Clone, Copy)]
pub struct WhatIs {
    pub uuid: Uuid,
}

#[derive(Clone, Copy)]
pub struct ItIs<'a> {
    pub uuid: Uuid,
    pub name: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct IDontKnow {
    pub uuid: Uuid,
}

#[derive(Clone, Copy)]
pub struct RconType {
    pub username_required: bool,
}

#[derive(Clone, Copy)]
pub struct MapDetails<'a> {
    pub name: &'a [u8],
    pub sha256: Sha256,
    pub crc: i32,
}

#[derive(Clone, Copy)]
pub struct Capabilities {
    pub version: i32,
    pub flags: i32,
}

#[derive(Clone, Copy)]
pub struct ClientVersion<'a> {
    pub connection_id: Uuid,
    pub ddnet_version: i32,
    pub ddnet_version_string: &'a [u8],
}

#[derive(Clone, Copy)]
pub struct PingEx {
    pub id: Uuid,
}

#[derive(Clone, Copy)]
pub struct PongEx {
    pub id: Uuid,
}

#[derive(Clone, Copy)]
pub struct ChecksumRequest {
    pub id: Uuid,
    pub start: i32,
    pub length: i32,
}

#[derive(Clone, Copy)]
pub struct ChecksumResponse {
    pub id: Uuid,
    pub sha256: Sha256,
}

#[derive(Clone, Copy)]
pub struct ChecksumError {
    pub id: Uuid,
    pub error: i32,
}

impl<'a> Info<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Info<'a>, Error> {
        let result = Ok(Info {
            version: _p.read_string()?,
            password: _p.read_string().ok(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(self.password.is_some());
        _p.write_string(self.version)?;
        _p.write_string(self.password.unwrap())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for Info<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Info")
            .field("version", &pretty::Bytes::new(&self.version))
            .field("password", &self.password.as_ref().map(|v| pretty::Bytes::new(&v)))
            .finish()
    }
}

impl<'a> MapChange<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapChange<'a>, Error> {
        let result = Ok(MapChange {
            name: _p.read_string()?,
            crc: _p.read_int(warn)?,
            size: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.name)?;
        _p.write_int(self.crc)?;
        _p.write_int(self.size)?;
        Ok(_p.written())
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

impl<'a> MapData<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapData<'a>, Error> {
        let result = Ok(MapData {
            last: _p.read_int(warn)?,
            crc: _p.read_int(warn)?,
            chunk: _p.read_int(warn)?,
            data: _p.read_data(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.last)?;
        _p.write_int(self.crc)?;
        _p.write_int(self.chunk)?;
        _p.write_data(self.data)?;
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

impl ConReady {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ConReady, Error> {
        let result = Ok(ConReady);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ConReady {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConReady")
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
            auth_level: _p.read_int(warn).ok(),
            receive_commands: _p.read_int(warn).ok(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(self.auth_level.is_some());
        assert!(self.receive_commands.is_some());
        _p.write_int(self.auth_level.unwrap())?;
        _p.write_int(self.receive_commands.unwrap())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for RconAuthStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconAuthStatus")
            .field("auth_level", &self.auth_level.as_ref().map(|v| v))
            .field("receive_commands", &self.receive_commands.as_ref().map(|v| v))
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
            input: ::snap_obj::PlayerInput::decode_msg(warn, _p)?,
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
            request_commands: _p.read_int(warn).ok(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        assert!(self.request_commands.is_some());
        _p.write_string(self._unused)?;
        _p.write_string(self.password)?;
        _p.write_int(self.request_commands.unwrap())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for RconAuth<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconAuth")
            .field("_unused", &pretty::Bytes::new(&self._unused))
            .field("password", &pretty::Bytes::new(&self.password))
            .field("request_commands", &self.request_commands.as_ref().map(|v| v))
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

impl<'a> RconCmdAdd<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconCmdAdd<'a>, Error> {
        let result = Ok(RconCmdAdd {
            name: _p.read_string()?,
            help: _p.read_string()?,
            params: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.name)?;
        _p.write_string(self.help)?;
        _p.write_string(self.params)?;
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

impl<'a> RconCmdRemove<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<RconCmdRemove<'a>, Error> {
        let result = Ok(RconCmdRemove {
            name: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.name)?;
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

impl WhatIs {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<WhatIs, Error> {
        let result = Ok(WhatIs {
            uuid: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.uuid.as_bytes())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for WhatIs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WhatIs")
            .field("uuid", &self.uuid)
            .finish()
    }
}

impl<'a> ItIs<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ItIs<'a>, Error> {
        let result = Ok(ItIs {
            uuid: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
            name: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.uuid.as_bytes())?;
        _p.write_string(self.name)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ItIs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ItIs")
            .field("uuid", &self.uuid)
            .field("name", &pretty::Bytes::new(&self.name))
            .finish()
    }
}

impl IDontKnow {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<IDontKnow, Error> {
        let result = Ok(IDontKnow {
            uuid: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.uuid.as_bytes())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for IDontKnow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IDontKnow")
            .field("uuid", &self.uuid)
            .finish()
    }
}

impl RconType {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RconType, Error> {
        let result = Ok(RconType {
            username_required: to_bool(_p.read_int(warn)?)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.username_required as i32)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for RconType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RconType")
            .field("username_required", &self.username_required)
            .finish()
    }
}

impl<'a> MapDetails<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<MapDetails<'a>, Error> {
        let result = Ok(MapDetails {
            name: _p.read_string()?,
            sha256: Sha256::from_slice(_p.read_raw(32)?).unwrap(),
            crc: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_string(self.name)?;
        _p.write_raw(&self.sha256.0)?;
        _p.write_int(self.crc)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for MapDetails<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapDetails")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("sha256", &self.sha256)
            .field("crc", &self.crc)
            .finish()
    }
}

impl Capabilities {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Capabilities, Error> {
        let result = Ok(Capabilities {
            version: _p.read_int(warn)?,
            flags: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_int(self.version)?;
        _p.write_int(self.flags)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Capabilities")
            .field("version", &self.version)
            .field("flags", &self.flags)
            .finish()
    }
}

impl<'a> ClientVersion<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<ClientVersion<'a>, Error> {
        let result = Ok(ClientVersion {
            connection_id: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
            ddnet_version: _p.read_int(warn)?,
            ddnet_version_string: _p.read_string()?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.connection_id.as_bytes())?;
        _p.write_int(self.ddnet_version)?;
        _p.write_string(self.ddnet_version_string)?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for ClientVersion<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ClientVersion")
            .field("connection_id", &self.connection_id)
            .field("ddnet_version", &self.ddnet_version)
            .field("ddnet_version_string", &pretty::Bytes::new(&self.ddnet_version_string))
            .finish()
    }
}

impl PingEx {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PingEx, Error> {
        let result = Ok(PingEx {
            id: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.id.as_bytes())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for PingEx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PingEx")
            .field("id", &self.id)
            .finish()
    }
}

impl PongEx {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<PongEx, Error> {
        let result = Ok(PongEx {
            id: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.id.as_bytes())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for PongEx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PongEx")
            .field("id", &self.id)
            .finish()
    }
}

impl ChecksumRequest {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ChecksumRequest, Error> {
        let result = Ok(ChecksumRequest {
            id: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
            start: _p.read_int(warn)?,
            length: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.id.as_bytes())?;
        _p.write_int(self.start)?;
        _p.write_int(self.length)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ChecksumRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ChecksumRequest")
            .field("id", &self.id)
            .field("start", &self.start)
            .field("length", &self.length)
            .finish()
    }
}

impl ChecksumResponse {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ChecksumResponse, Error> {
        let result = Ok(ChecksumResponse {
            id: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
            sha256: Sha256::from_slice(_p.read_raw(32)?).unwrap(),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.id.as_bytes())?;
        _p.write_raw(&self.sha256.0)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ChecksumResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ChecksumResponse")
            .field("id", &self.id)
            .field("sha256", &self.sha256)
            .finish()
    }
}

impl ChecksumError {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ChecksumError, Error> {
        let result = Ok(ChecksumError {
            id: Uuid::from_slice(_p.read_raw(16)?).unwrap(),
            error: _p.read_int(warn)?,
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(self.id.as_bytes())?;
        _p.write_int(self.error)?;
        Ok(_p.written())
    }
}
impl fmt::Debug for ChecksumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ChecksumError")
            .field("id", &self.id)
            .field("error", &self.error)
            .finish()
    }
}

