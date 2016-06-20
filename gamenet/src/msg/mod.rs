pub mod system;
pub mod game;

pub use self::game::Game;
pub use self::system::System;

use error::Error;
use packer::Unpacker;
use packer::Warning;
use warn::Warn;

#[derive(Clone, Copy, Debug)]
pub struct InputData<'a> {
    inner: &'a [u8],
}

impl<'a> InputData<'a> {
    fn from_bytes(bytes: &[u8]) -> InputData {
        InputData {
            inner: bytes,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        self.inner
    }
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
