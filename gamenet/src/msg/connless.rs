use buffer::CapacityError;
use common::num::BeU16;
use common::pretty;
use error::Error;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use std::fmt;
use super::AddrPacked;
use super::AddrPackedSliceExt;
use super::ClientsData;
use super::int_from_string;
use super::string_from_int;
use warn::Warn;
use warn::wrap;

pub const REQUEST_LIST: &'static [u8; 8] = b"\xff\xff\xff\xffreq2";
pub const LIST: &'static [u8; 8] = b"\xff\xff\xff\xfflis2";
pub const REQUEST_COUNT: &'static [u8; 8] = b"\xff\xff\xff\xffcou2";
pub const COUNT: &'static [u8; 8] = b"\xff\xff\xff\xffsiz2";
pub const REQUEST_INFO: &'static [u8; 8] = b"\xff\xff\xff\xffgie3";
pub const INFO: &'static [u8; 8] = b"\xff\xff\xff\xffinf3";

#[derive(Clone, Copy)]
pub enum Connless<'a> {
    RequestList(RequestList),
    List(List<'a>),
    RequestCount(RequestCount),
    Count(Count),
    RequestInfo(RequestInfo),
    Info(Info<'a>),
}

impl<'a> Connless<'a> {
    pub fn decode_connless<W: Warn<Warning>>(warn: &mut W, connless_id: [u8; 8], _p: &mut Unpacker<'a>) -> Result<Connless<'a>, Error> {
        Ok(match &connless_id {
            REQUEST_LIST => Connless::RequestList(try!(RequestList::decode(warn, _p))),
            LIST => Connless::List(try!(List::decode(warn, _p))),
            REQUEST_COUNT => Connless::RequestCount(try!(RequestCount::decode(warn, _p))),
            COUNT => Connless::Count(try!(Count::decode(warn, _p))),
            REQUEST_INFO => Connless::RequestInfo(try!(RequestInfo::decode(warn, _p))),
            INFO => Connless::Info(try!(Info::decode(warn, _p))),
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
    pub game_type: &'a [u8],
    pub flags: i32,
    pub num_players: i32,
    pub max_players: i32,
    pub num_clients: i32,
    pub max_clients: i32,
    pub clients: ClientsData<'a>,
}

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
            servers: AddrPackedSliceExt::from_bytes(wrap(warn), try!(_p.read_rest())),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_rest(self.servers.as_bytes()));
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
            count: { let s = try!(_p.read_raw(2)); BeU16::from_bytes(&[s[0], s[1]]).to_u16() },
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_raw(BeU16::from_u16(self.count).as_bytes()));
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
            token: try!(_p.read_raw(1))[0],
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_raw(&[self.token]));
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
            token: try!(int_from_string(try!(_p.read_string()))),
            version: try!(_p.read_string()),
            name: try!(_p.read_string()),
            game_type: try!(_p.read_string()),
            flags: try!(int_from_string(try!(_p.read_string()))),
            num_players: try!(int_from_string(try!(_p.read_string()))),
            max_players: try!(int_from_string(try!(_p.read_string()))),
            num_clients: try!(int_from_string(try!(_p.read_string()))),
            max_clients: try!(int_from_string(try!(_p.read_string()))),
            clients: ClientsData::from_bytes(try!(_p.read_rest())),
        });
        _p.finish(warn);
        result
    }
    pub fn encode<'d, 's>(&self, mut _p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        try!(_p.write_string(&string_from_int(self.token)));
        try!(_p.write_string(self.version));
        try!(_p.write_string(self.name));
        try!(_p.write_string(self.game_type));
        try!(_p.write_string(&string_from_int(self.flags)));
        try!(_p.write_string(&string_from_int(self.num_players)));
        try!(_p.write_string(&string_from_int(self.max_players)));
        try!(_p.write_string(&string_from_int(self.num_clients)));
        try!(_p.write_string(&string_from_int(self.max_clients)));
        try!(_p.write_rest(self.clients.as_bytes()));
        Ok(_p.written())
    }
}
impl<'a> fmt::Debug for Info<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Info")
            .field("token", &self.token)
            .field("version", &pretty::Bytes::new(&self.version))
            .field("name", &pretty::Bytes::new(&self.name))
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

