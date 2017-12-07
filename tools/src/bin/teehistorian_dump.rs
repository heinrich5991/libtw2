extern crate buffer;
extern crate clap;
extern crate gamenet;
extern crate logger;
extern crate packer;
extern crate teehistorian;
extern crate warn;

use buffer::ReadBuffer;
use gamenet::msg::SystemOrGame;
use packer::Unpacker;
use std::fs::File;
use std::io;
use std::path::Path;
use std::process;
use teehistorian::format;
use warn::Ignore;

const BUFSIZE: usize = 4096;

#[derive(Debug)]
enum Error {
    Magic(format::MagicError),
    Header(format::HeaderError),
    Item(format::item::Error),
    Io(io::Error),

    InvalidVersion(i32),
}

impl From<format::MagicError> for Error {
    fn from(e: format::MagicError) -> Error {
        Error::Magic(e)
    }
}

impl From<format::HeaderError> for Error {
    fn from(e: format::HeaderError) -> Error {
        Error::Header(e)
    }
}

impl From<format::item::Error> for Error {
    fn from(e: format::item::Error) -> Error {
        Error::Item(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

enum State {
    Magic,
    Header,
    Items,
}

fn process(path: &Path) -> Result<(), Error> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::with_capacity(BUFSIZE);
    let mut state = State::Magic;
    let mut offset = 0;
    loop {
        if buffer.len() == buffer.capacity() {
            let len = buffer.len();
            buffer.reserve(len);
        }
        match file.read_buffer(&mut buffer).map(|x| x.len()) {
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Ok(0) => break,
            x => x,
        }?;
        let mut processed = 0;
        loop {
            let mut unpacker = Unpacker::new(&mut buffer[processed..]);
            match state {
                State::Magic => {
                    match format::read_magic(&mut unpacker) {
                        Err(format::MagicError::UnexpectedEnd) => break,
                        x => x?,
                    }
                    state = State::Header;
                },
                State::Header => {
                    let header = match format::read_header(&mut unpacker) {
                        Err(format::HeaderError::UnexpectedEnd) => break,
                        x => x?,
                    };
                    println!("{:?}", header);
                    if header.version != 1 {
                        return Err(Error::InvalidVersion(header.version));
                    }
                    state = State::Items;
                },
                State::Items => {
                    let item = match format::Item::decode(&mut unpacker) {
                        Err(format::item::Error::UnexpectedEnd) => break,
                        x => { if x.is_err() { println!("error offset {}", offset + processed); } x? },
                    };
                    println!("{:?}", item);
                    if let format::Item::Message(ref m) = item {
                        match SystemOrGame::decode(&mut Ignore, &mut Unpacker::new(m.msg)) {
                            Ok(m) => println!("    {:?}", m),
                            e => println!("    {:?}", e),
                        }
                    }
                }
            }
            processed += unpacker.num_bytes_read();
        }
        buffer.drain(..processed);
        offset += processed;
    }
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian reader")
        .about("Reads teehistorian file and dumps its contents in a human-readable\
                text stream")
        .arg(Arg::with_name("TEEHISTORIAN")
            .help("Sets the teehistorian file to dump")
            .required(true)
        )
        .get_matches();

    let path = Path::new(matches.value_of_os("TEEHISTORIAN").unwrap());

    match process(path) {
        Ok(()) => {},
        Err(err) => {
            println!("{}: {:?}", path.display(), err);
            process::exit(1);
        }
    }
}
