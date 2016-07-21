extern crate arrayvec;
extern crate buffer;
extern crate gamenet;
extern crate hexdump;
extern crate net;
extern crate packer;
extern crate warn;

use State::*;
use arrayvec::ArrayVec;
use buffer::ReadBuffer;
use gamenet::msg::Connless;
use gamenet::msg::SystemOrGame;
use hexdump::hexdump;
use net::protocol::Packet;
use net::protocol::ConnectedPacketType;
use net::protocol::ChunksIter;
use packer::Unpacker;
use std::fmt;
use std::io;
use warn::Warn;

struct Stdout;

impl<W: fmt::Debug> Warn<W> for Stdout {
    fn warn(&mut self, warning: W) {
        println!("WARN: {:?}", warning);
    }
}

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
                (InsideFirst, b) => if let Some(n) = try!(unhex(b)) {
                    self.state = InsideSecond(n);
                },
                (InsideSecond(_), b'|') => return Err(Error::OddCharacterCount),
                (InsideSecond(f), b) => if let Some(n) = try!(unhex(b)) {
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

fn main() {
    let mut un = Unhexdump::new();
    let mut buf: ArrayVec<[u8; 4096]> = ArrayVec::new();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    while { buf.clear(); stdin.read_buffer(&mut buf).unwrap().len() != 0 } {
        un.feed(&buf).unwrap();
    }

    let bytes = un.into_inner().unwrap();

    println!("packet");
    hexdump(&bytes);
    let p = match Packet::read(&mut Stdout, &bytes, &mut buf) {
        Err(e) => {
            println!("ERROR: {:?}", e);
            return;
        },
        Ok(p) => p,
    };

    let cp = match p {
        Packet::Connless(data) => {
            println!("connless");
            let msg = match Connless::decode(&mut Stdout, &mut Unpacker::new(data)) {
                Err(e) => {
                    println!("ERROR: {:?}", e);
                    return;
                },
                Ok(m) => m,
            };
            println!("{:?}", msg);
            return;
        },
        Packet::Connected(cp) => cp,
    };

    let (request_resend, num_chunks, payload) = match cp.type_ {
        ConnectedPacketType::Control(control) => {
            println!("control ack={}", cp.ack);
            println!("{:?}", control);
            return;
        },
        ConnectedPacketType::Chunks(r, n, p) => (r, n, p),
    };
    println!("chunks ack={} request_resend={} num_chunks={}", cp.ack, request_resend, num_chunks);
    hexdump(payload);
    let mut i = 0;
    let mut chunks_iter = ChunksIter::new(payload, num_chunks);
    loop {
        if chunks_iter.clone().next().is_some() {
            println!("chunk {}", i);
        }
        let chunk = if let Some(chunk) = chunks_iter.next_warn(&mut Stdout) {
            i += 1;
            chunk
        } else {
            break;
        };

        match chunk.vital {
            Some((sequence, resend)) => println!("vital=true sequence={} resend={}", sequence, resend),
            None => println!("vital=false"),
        }
        hexdump(chunk.data);

        let msg = match SystemOrGame::decode(&mut Stdout, &mut Unpacker::new(chunk.data)) {
            Err(e) => {
                println!("ERROR: {:?}", e);
                continue;
            },
            Ok(m) => m,
        };

        println!("{:?}", msg);
    }
}
