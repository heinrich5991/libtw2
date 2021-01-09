use buffer::CapacityError;
use common::pretty;
use error::Error;
use gamenet_common::msg::string_from_int;
use packer::Packer;
use packer::Unpacker;
use packer::Warning;
use packer::with_packer;
use std::fmt;
use warn::Warn;

impl Connless {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, _p: &mut Unpacker) -> Result<Connless, Error> {
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

pub const FORWARD_CHECK: &'static [u8; 8] = b"\xff\xff\xff\xfffw??";
pub const FORWARD_RESPONSE: &'static [u8; 8] = b"\xff\xff\xff\xfffw!!";
pub const FORWARD_OK: &'static [u8; 8] = b"\xff\xff\xff\xfffwok";
pub const FORWARD_ERROR: &'static [u8; 8] = b"\xff\xff\xff\xfffwer";

#[derive(Clone, Copy)]
pub enum Connless {
    ForwardCheck(ForwardCheck),
    ForwardResponse(ForwardResponse),
    ForwardOk(ForwardOk),
    ForwardError(ForwardError),
}

impl Connless {
    pub fn decode_connless<W: Warn<Warning>>(warn: &mut W, connless_id: [u8; 8], _p: &mut Unpacker) -> Result<Connless, Error> {
        Ok(match &connless_id {
            FORWARD_CHECK => Connless::ForwardCheck(ForwardCheck::decode(warn, _p)?),
            FORWARD_RESPONSE => Connless::ForwardResponse(ForwardResponse::decode(warn, _p)?),
            FORWARD_OK => Connless::ForwardOk(ForwardOk::decode(warn, _p)?),
            FORWARD_ERROR => Connless::ForwardError(ForwardError::decode(warn, _p)?),
            _ => return Err(Error::UnknownId),
        })
    }
    pub fn connless_id(&self) -> [u8; 8] {
        match *self {
            Connless::ForwardCheck(_) => *FORWARD_CHECK,
            Connless::ForwardResponse(_) => *FORWARD_RESPONSE,
            Connless::ForwardOk(_) => *FORWARD_OK,
            Connless::ForwardError(_) => *FORWARD_ERROR,
        }
    }
    pub fn encode_connless<'d, 's>(&self, p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        match *self {
            Connless::ForwardCheck(ref i) => i.encode(p),
            Connless::ForwardResponse(ref i) => i.encode(p),
            Connless::ForwardOk(ref i) => i.encode(p),
            Connless::ForwardError(ref i) => i.encode(p),
        }
    }
}

impl fmt::Debug for Connless {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Connless::ForwardCheck(ref i) => i.fmt(f),
            Connless::ForwardResponse(ref i) => i.fmt(f),
            Connless::ForwardOk(ref i) => i.fmt(f),
            Connless::ForwardError(ref i) => i.fmt(f),
        }
    }
}

impl From<ForwardCheck> for Connless {
    fn from(i: ForwardCheck) -> Connless {
        Connless::ForwardCheck(i)
    }
}

impl From<ForwardResponse> for Connless {
    fn from(i: ForwardResponse) -> Connless {
        Connless::ForwardResponse(i)
    }
}

impl From<ForwardOk> for Connless {
    fn from(i: ForwardOk) -> Connless {
        Connless::ForwardOk(i)
    }
}

impl From<ForwardError> for Connless {
    fn from(i: ForwardError) -> Connless {
        Connless::ForwardError(i)
    }
}
#[derive(Clone, Copy)]
pub struct ForwardCheck;

#[derive(Clone, Copy)]
pub struct ForwardResponse;

#[derive(Clone, Copy)]
pub struct ForwardOk;

#[derive(Clone, Copy)]
pub struct ForwardError;

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

