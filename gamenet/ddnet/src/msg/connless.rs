use buffer::CapacityError;
use common::num::BeU16;
use common::pretty;
use error::Error;
use gamenet_common::msg::AddrPackedSliceExt;
use gamenet_common::msg::int_from_string;
use gamenet_common::msg::string_from_int;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::sanitize;
use packer::with_packer;
use std::fmt;
use super::AddrPacked;
use super::ClientsData;
use warn::Panic;
use warn::Warn;
use warn::wrap;

impl<'a> Connless<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Connless<'a>, Error> {
        let id = _p.read_raw(8)?;
        let connless_id = [id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]];
        Connless::decode_connless(warn, connless_id, _p)
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        p.write_raw(&self.connless_id())?;
        with_packer(&mut p, |p| self.encode_connless(p))?;
        Ok(p.written())
    }
}

pub struct Client<'a> {
    pub name: &'a [u8],
    pub clan: &'a [u8],
    pub country: i32,
    pub score: i32,
    pub is_player: i32,
}

impl<'a> Client<'a> {
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>)
        -> Result<&'d [u8], CapacityError>
    {
        _p.write_string(self.name)?;
        _p.write_string(self.clan)?;
        _p.write_string(&string_from_int(self.country))?;
        _p.write_string(&string_from_int(self.score))?;
        _p.write_string(&string_from_int(self.is_player))?;
        Ok(_p.written())
    }
}

impl<'a> fmt::Debug for Client<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Client")
            .field("name", &pretty::Bytes::new(&self.name))
            .field("clan", &pretty::Bytes::new(&self.clan))
            .field("country", &self.country)
            .field("score", &self.score)
            .field("is_player", &self.is_player)
            .finish()
    }
}

pub const INFO_FLAG_PASSWORD: i32 = 1;

pub const REQUEST_LIST: &'static [u8; 8] = b"\xff\xff\xff\xffreq2";
pub const LIST: &'static [u8; 8] = b"\xff\xff\xff\xfflis2";
pub const REQUEST_COUNT: &'static [u8; 8] = b"\xff\xff\xff\xffcou2";
pub const COUNT: &'static [u8; 8] = b"\xff\xff\xff\xffsiz2";
pub const REQUEST_INFO: &'static [u8; 8] = b"\xff\xff\xff\xffgie3";
pub const INFO: &'static [u8; 8] = b"\xff\xff\xff\xffinf3";
pub const INFO_EXTENDED: &'static [u8; 8] = b"\xff\xff\xff\xffiext";
pub const INFO_EXTENDED_MORE: &'static [u8; 8] = b"\xff\xff\xff\xffiex+";
pub const HEARTBEAT: &'static [u8; 8] = b"\xff\xff\xff\xffbea2";
pub const FORWARD_CHECK: &'static [u8; 8] = b"\xff\xff\xff\xfffw??";
pub const FORWARD_RESPONSE: &'static [u8; 8] = b"\xff\xff\xff\xfffw!!";
pub const FORWARD_OK: &'static [u8; 8] = b"\xff\xff\xff\xfffwok";
pub const FORWARD_ERROR: &'static [u8; 8] = b"\xff\xff\xff\xfffwer";

#[derive(Clone, Copy)]
pub enum Connless<'a> {
    RequestList(RequestList),
    List(List<'a>),
    RequestCount(RequestCount),
    Count(Count),
    RequestInfo(RequestInfo),
    Info(Info<'a>),
    InfoExtended(InfoExtended<'a>),
    InfoExtendedMore(InfoExtendedMore<'a>),
    Heartbeat(Heartbeat),
    ForwardCheck(ForwardCheck),
    ForwardResponse(ForwardResponse),
    ForwardOk(ForwardOk),
    ForwardError(ForwardError),
}

impl<'a> Connless<'a> {
    pub fn decode_connless<W: Warn<Warning>>(warn: &mut W, connless_id: [u8; 8], _p: &mut Unpacker<'a>) -> Result<Connless<'a>, Error> {
        Ok(match &connless_id {
            REQUEST_LIST => Connless::RequestList(RequestList::decode(warn, _p)?),
            LIST => Connless::List(List::decode(warn, _p)?),
            REQUEST_COUNT => Connless::RequestCount(RequestCount::decode(warn, _p)?),
            COUNT => Connless::Count(Count::decode(warn, _p)?),
            REQUEST_INFO => Connless::RequestInfo(RequestInfo::decode(warn, _p)?),
            INFO => Connless::Info(Info::decode(warn, _p)?),
            INFO_EXTENDED => Connless::InfoExtended(InfoExtended::decode(warn, _p)?),
            INFO_EXTENDED_MORE => Connless::InfoExtendedMore(InfoExtendedMore::decode(warn, _p)?),
            HEARTBEAT => Connless::Heartbeat(Heartbeat::decode(warn, _p)?),
            FORWARD_CHECK => Connless::ForwardCheck(ForwardCheck::decode(warn, _p)?),
            FORWARD_RESPONSE => Connless::ForwardResponse(ForwardResponse::decode(warn, _p)?),
            FORWARD_OK => Connless::ForwardOk(ForwardOk::decode(warn, _p)?),
            FORWARD_ERROR => Connless::ForwardError(ForwardError::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn connless_id(&self) -> [u8; 8] {
        match *self {
            Connless::RequestList(_) => *REQUEST_LIST,
            Connless::List(_) => *LIST,
            Connless::RequestCount(_) => *REQUEST_COUNT,
            Connless::Count(_) => *COUNT,
            Connless::RequestInfo(_) => *REQUEST_INFO,
            Connless::Info(_) => *INFO,
            Connless::InfoExtended(_) => *INFO_EXTENDED,
            Connless::InfoExtendedMore(_) => *INFO_EXTENDED_MORE,
            Connless::Heartbeat(_) => *HEARTBEAT,
            Connless::ForwardCheck(_) => *FORWARD_CHECK,
            Connless::ForwardResponse(_) => *FORWARD_RESPONSE,
            Connless::ForwardOk(_) => *FORWARD_OK,
            Connless::ForwardError(_) => *FORWARD_ERROR,
        }
    }
    pub fn encode_connless<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            Connless::RequestList(ref i) => i.encode(p),
            Connless::List(ref i) => i.encode(p),
            Connless::RequestCount(ref i) => i.encode(p),
            Connless::Count(ref i) => i.encode(p),
            Connless::RequestInfo(ref i) => i.encode(p),
            Connless::Info(ref i) => i.encode(p),
            Connless::InfoExtended(ref i) => i.encode(p),
            Connless::InfoExtendedMore(ref i) => i.encode(p),
            Connless::Heartbeat(ref i) => i.encode(p),
            Connless::ForwardCheck(ref i) => i.encode(p),
            Connless::ForwardResponse(ref i) => i.encode(p),
            Connless::ForwardOk(ref i) => i.encode(p),
            Connless::ForwardError(ref i) => i.encode(p),
        }
    }
}

impl<'a> fmt::Debug for Connless<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Connless::RequestList(ref i) => i.fmt(f),
            Connless::List(ref i) => i.fmt(f),
            Connless::RequestCount(ref i) => i.fmt(f),
            Connless::Count(ref i) => i.fmt(f),
            Connless::RequestInfo(ref i) => i.fmt(f),
            Connless::Info(ref i) => i.fmt(f),
            Connless::InfoExtended(ref i) => i.fmt(f),
            Connless::InfoExtendedMore(ref i) => i.fmt(f),
            Connless::Heartbeat(ref i) => i.fmt(f),
            Connless::ForwardCheck(ref i) => i.fmt(f),
            Connless::ForwardResponse(ref i) => i.fmt(f),
            Connless::ForwardOk(ref i) => i.fmt(f),
            Connless::ForwardError(ref i) => i.fmt(f),
        }
    }
}

impl<'a> From<RequestList> for Connless<'a> {
    fn from(i: RequestList) -> Connless<'a> {
        Connless::RequestList(i)
    }
}

impl<'a> From<List<'a>> for Connless<'a> {
    fn from(i: List<'a>) -> Connless<'a> {
        Connless::List(i)
    }
}

impl<'a> From<RequestCount> for Connless<'a> {
    fn from(i: RequestCount) -> Connless<'a> {
        Connless::RequestCount(i)
    }
}

impl<'a> From<Count> for Connless<'a> {
    fn from(i: Count) -> Connless<'a> {
        Connless::Count(i)
    }
}

impl<'a> From<RequestInfo> for Connless<'a> {
    fn from(i: RequestInfo) -> Connless<'a> {
        Connless::RequestInfo(i)
    }
}

impl<'a> From<Info<'a>> for Connless<'a> {
    fn from(i: Info<'a>) -> Connless<'a> {
        Connless::Info(i)
    }
}

impl<'a> From<InfoExtended<'a>> for Connless<'a> {
    fn from(i: InfoExtended<'a>) -> Connless<'a> {
        Connless::InfoExtended(i)
    }
}

impl<'a> From<InfoExtendedMore<'a>> for Connless<'a> {
    fn from(i: InfoExtendedMore<'a>) -> Connless<'a> {
        Connless::InfoExtendedMore(i)
    }
}

impl<'a> From<Heartbeat> for Connless<'a> {
    fn from(i: Heartbeat) -> Connless<'a> {
        Connless::Heartbeat(i)
    }
}

impl<'a> From<ForwardCheck> for Connless<'a> {
    fn from(i: ForwardCheck) -> Connless<'a> {
        Connless::ForwardCheck(i)
    }
}

impl<'a> From<ForwardResponse> for Connless<'a> {
    fn from(i: ForwardResponse) -> Connless<'a> {
        Connless::ForwardResponse(i)
    }
}

impl<'a> From<ForwardOk> for Connless<'a> {
    fn from(i: ForwardOk) -> Connless<'a> {
        Connless::ForwardOk(i)
    }
}

impl<'a> From<ForwardError> for Connless<'a> {
    fn from(i: ForwardError) -> Connless<'a> {
        Connless::ForwardError(i)
    }
}
#[derive(Clone, Copy)]
pub struct RequestList;

#[derive(Clone, Copy)]
pub struct List<'a> {
    pub servers: &'a [AddrPacked],
}

#[derive(Clone, Copy)]
pub struct RequestCount;

#[derive(Clone, Copy)]
pub struct Count {
    pub count: u16,
}

#[derive(Clone, Copy)]
pub struct RequestInfo {
    pub token: u8,
}

#[derive(Clone, Copy)]
pub struct Info<'a> {
    pub token: i32,
    pub version: &'a [u8],
    pub name: &'a [u8],
    pub map: &'a [u8],
    pub game_type: &'a [u8],
    pub flags: i32,
    pub num_players: i32,
    pub max_players: i32,
    pub num_clients: i32,
    pub max_clients: i32,
    pub clients: ClientsData<'a>,
}

#[derive(Clone, Copy)]
pub struct InfoExtended<'a> {
    pub token: i32,
    pub version: &'a [u8],
    pub name: &'a [u8],
    pub map: &'a [u8],
    pub map_crc: i32,
    pub map_size: i32,
    pub game_type: &'a [u8],
    pub flags: i32,
    pub num_players: i32,
    pub max_players: i32,
    pub num_clients: i32,
    pub max_clients: i32,
    pub reserved: &'a [u8],
    pub clients: ClientsData<'a>,
}

#[derive(Clone, Copy)]
pub struct InfoExtendedMore<'a> {
    pub token: i32,
    pub packet_no: i32,
    pub reserved: &'a [u8],
    pub clients: ClientsData<'a>,
}

#[derive(Clone, Copy)]
pub struct Heartbeat {
    pub alt_port: u16,
}

#[derive(Clone, Copy)]
pub struct ForwardCheck;

#[derive(Clone, Copy)]
pub struct ForwardResponse;

#[derive(Clone, Copy)]
pub struct ForwardOk;

#[derive(Clone, Copy)]
pub struct ForwardError;

impl RequestList {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RequestList, Error> {
        let result = Ok(RequestList);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for RequestList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RequestList")
            .finish()
    }
}

impl<'a> List<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<List<'a>, Error> {
        let result = Ok(List {
            servers: AddrPackedSliceExt::from_bytes(wrap(warn), _p.read_rest()?),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_rest(self.servers.as_bytes())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for List<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("List")
            .field("servers", &self.servers)
            .finish()
    }
}

impl RequestCount {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RequestCount, Error> {
        let result = Ok(RequestCount);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for RequestCount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RequestCount")
            .finish()
    }
}

impl Count {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Count, Error> {
        let result = Ok(Count {
            count: { let s = _p.read_raw(2)?; BeU16::from_bytes(&[s[0], s[1]]).to_u16() },
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(BeU16::from_u16(self.count).as_bytes())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for Count {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Count")
            .field("count", &self.count)
            .finish()
    }
}

impl RequestInfo {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<RequestInfo, Error> {
        let result = Ok(RequestInfo {
            token: _p.read_raw(1)?[0],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(&[self.token])?;
        Ok(_p.written())
    }
}
impl fmt::Debug for RequestInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RequestInfo")
            .field("token", &self.token)
            .finish()
    }
}

impl<'a> Info<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<Info<'a>, Error> {
        let result = Ok(Info {
            token: int_from_string(_p.read_string()?)?,
            version: sanitize(warn, _p.read_string()?)?,
            name: sanitize(warn, _p.read_string()?)?,
            map: sanitize(warn, _p.read_string()?)?,
            game_type: sanitize(warn, _p.read_string()?)?,
            flags: int_from_string(_p.read_string()?)?,
            num_players: int_from_string(_p.read_string()?)?,
            max_players: int_from_string(_p.read_string()?)?,
            num_clients: int_from_string(_p.read_string()?)?,
            max_clients: int_from_string(_p.read_string()?)?,
            clients: ClientsData::from_bytes(_p.read_rest()?),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.version).unwrap();
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.map).unwrap();
        sanitize(&mut Panic, self.game_type).unwrap();
        _p.write_string(&string_from_int(self.token))?;
        _p.write_string(self.version)?;
        _p.write_string(self.name)?;
        _p.write_string(self.map)?;
        _p.write_string(self.game_type)?;
        _p.write_string(&string_from_int(self.flags))?;
        _p.write_string(&string_from_int(self.num_players))?;
        _p.write_string(&string_from_int(self.max_players))?;
        _p.write_string(&string_from_int(self.num_clients))?;
        _p.write_string(&string_from_int(self.max_clients))?;
        _p.write_rest(self.clients.as_bytes())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for Info<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Info")
            .field("token", &self.token)
            .field("version", &pretty::Bytes::new(&self.version))
            .field("name", &pretty::Bytes::new(&self.name))
            .field("map", &pretty::Bytes::new(&self.map))
            .field("game_type", &pretty::Bytes::new(&self.game_type))
            .field("flags", &self.flags)
            .field("num_players", &self.num_players)
            .field("max_players", &self.max_players)
            .field("num_clients", &self.num_clients)
            .field("max_clients", &self.max_clients)
            .field("clients", &self.clients)
            .finish()
    }
}

impl<'a> InfoExtended<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<InfoExtended<'a>, Error> {
        let result = Ok(InfoExtended {
            token: int_from_string(_p.read_string()?)?,
            version: sanitize(warn, _p.read_string()?)?,
            name: sanitize(warn, _p.read_string()?)?,
            map: sanitize(warn, _p.read_string()?)?,
            map_crc: int_from_string(_p.read_string()?)?,
            map_size: int_from_string(_p.read_string()?)?,
            game_type: sanitize(warn, _p.read_string()?)?,
            flags: int_from_string(_p.read_string()?)?,
            num_players: int_from_string(_p.read_string()?)?,
            max_players: int_from_string(_p.read_string()?)?,
            num_clients: int_from_string(_p.read_string()?)?,
            max_clients: int_from_string(_p.read_string()?)?,
            reserved: sanitize(warn, _p.read_string()?)?,
            clients: ClientsData::from_bytes(_p.read_rest()?),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.version).unwrap();
        sanitize(&mut Panic, self.name).unwrap();
        sanitize(&mut Panic, self.map).unwrap();
        sanitize(&mut Panic, self.game_type).unwrap();
        sanitize(&mut Panic, self.reserved).unwrap();
        _p.write_string(&string_from_int(self.token))?;
        _p.write_string(self.version)?;
        _p.write_string(self.name)?;
        _p.write_string(self.map)?;
        _p.write_string(&string_from_int(self.map_crc))?;
        _p.write_string(&string_from_int(self.map_size))?;
        _p.write_string(self.game_type)?;
        _p.write_string(&string_from_int(self.flags))?;
        _p.write_string(&string_from_int(self.num_players))?;
        _p.write_string(&string_from_int(self.max_players))?;
        _p.write_string(&string_from_int(self.num_clients))?;
        _p.write_string(&string_from_int(self.max_clients))?;
        _p.write_string(self.reserved)?;
        _p.write_rest(self.clients.as_bytes())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for InfoExtended<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InfoExtended")
            .field("token", &self.token)
            .field("version", &pretty::Bytes::new(&self.version))
            .field("name", &pretty::Bytes::new(&self.name))
            .field("map", &pretty::Bytes::new(&self.map))
            .field("map_crc", &self.map_crc)
            .field("map_size", &self.map_size)
            .field("game_type", &pretty::Bytes::new(&self.game_type))
            .field("flags", &self.flags)
            .field("num_players", &self.num_players)
            .field("max_players", &self.max_players)
            .field("num_clients", &self.num_clients)
            .field("max_clients", &self.max_clients)
            .field("reserved", &pretty::Bytes::new(&self.reserved))
            .field("clients", &self.clients)
            .finish()
    }
}

impl<'a> InfoExtendedMore<'a> {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker<'a>) -> Result<InfoExtendedMore<'a>, Error> {
        let result = Ok(InfoExtendedMore {
            token: int_from_string(_p.read_string()?)?,
            packet_no: int_from_string(_p.read_string()?)?,
            reserved: sanitize(warn, _p.read_string()?)?,
            clients: ClientsData::from_bytes(_p.read_rest()?),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        sanitize(&mut Panic, self.reserved).unwrap();
        _p.write_string(&string_from_int(self.token))?;
        _p.write_string(&string_from_int(self.packet_no))?;
        _p.write_string(self.reserved)?;
        _p.write_rest(self.clients.as_bytes())?;
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for InfoExtendedMore<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InfoExtendedMore")
            .field("token", &self.token)
            .field("packet_no", &self.packet_no)
            .field("reserved", &pretty::Bytes::new(&self.reserved))
            .field("clients", &self.clients)
            .finish()
    }
}

impl Heartbeat {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Heartbeat, Error> {
        let result = Ok(Heartbeat {
            alt_port: { let s = _p.read_raw(2)?; BeU16::from_bytes(&[s[0], s[1]]).to_u16() },
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        _p.write_raw(BeU16::from_u16(self.alt_port).as_bytes())?;
        Ok(_p.written())
    }
}
impl fmt::Debug for Heartbeat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Heartbeat")
            .field("alt_port", &self.alt_port)
            .finish()
    }
}

impl ForwardCheck {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ForwardCheck, Error> {
        let result = Ok(ForwardCheck);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ForwardCheck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ForwardCheck")
            .finish()
    }
}

impl ForwardResponse {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ForwardResponse, Error> {
        let result = Ok(ForwardResponse);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ForwardResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ForwardResponse")
            .finish()
    }
}

impl ForwardOk {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ForwardOk, Error> {
        let result = Ok(ForwardOk);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ForwardOk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ForwardOk")
            .finish()
    }
}

impl ForwardError {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<ForwardError, Error> {
        let result = Ok(ForwardError);
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        Ok(_p.written())
    }
}
impl fmt::Debug for ForwardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ForwardError")
            .finish()
    }
}

