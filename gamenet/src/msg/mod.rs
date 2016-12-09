pub mod connless;
pub mod game;
pub mod system;

pub use self::connless::Connless;
pub use self::game::Game;
pub use self::system::System;

use arrayvec::ArrayVec;
use common::num::BeU16;
use common::slice;
use error::Error;
use error::InvalidIntString;
use packer::ExcessData;
use packer::Unpacker;
use packer::Warning;
use std::io::Write;
use std::mem;
use std::str;
use warn::Warn;

pub const CLIENTS_DATA_NONE: ClientsData<'static> = ClientsData { inner: b"" };

#[derive(Clone, Copy, Debug)]
pub struct ClientsData<'a> {
    inner: &'a [u8],
}

impl<'a> ClientsData<'a> {
    fn from_bytes(bytes: &[u8]) -> ClientsData {
        ClientsData {
            inner: bytes,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        self.inner
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct AddrPacked {
    ip_address: [u8; 16],
    port: BeU16,
}

trait AddrPackedSliceExt {
    fn from_bytes<'a, W: Warn<ExcessData>>(warn: &mut W, bytes: &'a [u8]) -> &'a Self;
    fn as_bytes(&self) -> &[u8];
}

impl AddrPackedSliceExt for [AddrPacked] {
    fn from_bytes<'a, W: Warn<ExcessData>>(warn: &mut W, bytes: &'a [u8]) -> &'a [AddrPacked] {
        let remainder = bytes.len() % mem::size_of::<AddrPacked>();
        if remainder != 0 {
            warn.warn(ExcessData);
        }
        let actual_len = bytes.len() - remainder;
        unsafe {
            slice::transmute(&bytes[..actual_len])
        }
    }
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            slice::transmute(self)
        }
    }
}

fn int_from_string(bytes: &[u8]) -> Result<i32, InvalidIntString> {
    str::from_utf8(bytes)
        .map(|s| s.parse().map_err(|_| InvalidIntString))
        .unwrap_or(Err(InvalidIntString))
}

fn string_from_int(int: i32) -> ArrayVec<[u8; 16]> {
    let mut result = ArrayVec::new();
    write!(&mut result, "{}", int).unwrap();
    result
}

#[derive(Clone, Copy, Debug)]
pub enum SystemOrGame<S, G> {
    System(S),
    Game(G),
}

impl<S, G> SystemOrGame<S, G> {
    fn is_game(&self) -> bool {
        match *self {
            SystemOrGame::System(_) => false,
            SystemOrGame::Game(_) => true,
        }
    }
    fn is_system(&self) -> bool {
        !self.is_game()
    }
}

impl SystemOrGame<i32, i32> {
    fn decode_id(id: i32) -> SystemOrGame<i32, i32> {
        let sys = id & 1 != 0;
        let msg = id >> 1;
        if sys {
            SystemOrGame::System(msg)
        } else {
            SystemOrGame::Game(msg)
        }
    }
    fn internal_id(self) -> i32 {
        match self {
            SystemOrGame::System(msg) => msg,
            SystemOrGame::Game(msg) => msg,
        }
    }
    fn encode_id(self) -> i32 {
        let iid = self.internal_id() as u32;
        assert!((iid & (1 << 31)) == 0);
        let flag = self.is_system() as u32;
        ((iid << 1) | flag) as i32
    }
}

impl<'a> SystemOrGame<System<'a>, Game<'a>> {
    pub fn decode<W>(warn: &mut W, p: &mut Unpacker<'a>)
        -> Result<SystemOrGame<System<'a>, Game<'a>>, Error>
        where W: Warn<Warning>
    {
        let msg_id = try!(p.read_int(warn));
        Ok(match SystemOrGame::decode_id(msg_id) {
            SystemOrGame::System(msg_id) =>
                SystemOrGame::System(try!(System::decode_msg(warn, msg_id, p))),
            SystemOrGame::Game(msg_id) =>
                SystemOrGame::Game(try!(Game::decode_msg(warn, msg_id, p))),
        })
    }
}
