pub use self::system::System;

pub mod system;
pub mod game;

#[derive(Clone, Copy, Debug)]
pub struct IntegerData<'a> {
    inner: &'a [u8],
}

impl<'a> IntegerData<'a> {
    fn from_bytes(bytes: &[u8]) -> IntegerData {
        IntegerData {
            inner: bytes,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        self.inner
    }
}

#[derive(Clone, Copy, Debug)]
enum SystemOrGame<S, G> {
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
