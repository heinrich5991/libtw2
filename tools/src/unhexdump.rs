use self::State::*;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    UnknownCharacter,
    OddCharacterCount,
}

#[derive(Clone, Copy, Debug)]
pub struct Unfinished;

#[derive(Clone, Copy)]
enum State {
    Before,
    InsideFirst,
    InsideSecond(u8),
    After,
}

impl Default for State {
    fn default() -> State {
        Before
    }
}

fn unhex(byte: u8) -> Result<Option<u8>, Error> {
    match byte {
        b'0'...b'9' => Ok(Some(byte - b'0')),
        b'a'...b'f' => Ok(Some(byte - b'a' + 10)),
        b' ' => Ok(None),
        _ => Err(Error::UnknownCharacter),
    }
}

#[derive(Default)]
pub struct Unhexdump {
    buf: Vec<u8>,
    state: State,
}

impl Unhexdump {
    pub fn new() -> Unhexdump {
        Default::default()
    }
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for &b in bytes {
            match (self.state, b) {
                (Before, b'|') => self.state = InsideFirst,
                (Before, _) => {},
                (InsideFirst, b'|') => self.state = After,
                (InsideFirst, b) => if let Some(n) = unhex(b)? {
                    self.state = InsideSecond(n);
                },
                (InsideSecond(_), b'|') => return Err(Error::OddCharacterCount),
                (InsideSecond(f), b) => if let Some(n) = unhex(b)? {
                    self.buf.push((f << 4) | n);
                    self.state = InsideFirst;
                },
                (After, b'\n') => self.state = Before,
                (After, _) => {},
            }
        }
        Ok(())
    }
    pub fn into_inner(self) -> Result<Vec<u8>, Unfinished> {
        match self.state {
            After | Before => Ok(self.buf),
            _ => Err(Unfinished),
        }
    }
}
