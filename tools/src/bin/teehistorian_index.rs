extern crate clap;
extern crate csv;
extern crate logger;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate teehistorian;
extern crate uuid;
extern crate walkdir;

use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process;
use teehistorian::Buffer;
use teehistorian::Reader;
use teehistorian::format;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug)]
enum Error {
    Csv(csv::Error),
    Io(io::Error),
    Teehistorian(format::Error),
    WalkDir(walkdir::Error),
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Error {
        Error::Csv(e)
    }
}

impl From<teehistorian::Error> for Error {
    fn from(e: teehistorian::Error) -> Error {
        use teehistorian::Error::*;
        match e {
            Teehistorian(i) => Error::Teehistorian(i),
            Io(i) => Error::Io(i),
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}

#[derive(Debug)]
struct HexU32(u32);

impl serde::Serialize for HexU32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:08x}", self.0))
    }
}

impl From<u32> for HexU32 {
    fn from(i: u32) -> HexU32 {
        HexU32(i)
    }
}

#[derive(Debug, Serialize)]
struct Record<'a> {
    path: &'a Path,
    game_uuid: Uuid,
    timestamp: Cow<'a, str>,
    map_name: Cow<'a, str>,
    map_crc: HexU32,
    map_size: u32,
}

fn handle_dir(dir: &OsStr) -> Result<(), ()> {
    fn helper(dir: &OsStr) -> Result<(), Error> {
        let mut buffer = Buffer::new();
        let mut writer = csv::Writer::from_writer(io::stdout());
        for entry in WalkDir::new(Path::new(dir))
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension() != Some(OsStr::new("teehistorian")) {
                continue;
            }
            buffer.clear();
            match Reader::open(entry.path(), &mut buffer) {
                Ok((header, _)) => {
                    writer.serialize(Record {
                        path: entry.path(),
                        game_uuid: header.game_uuid,
                        timestamp: header.timestamp,
                        map_name: header.map_name,
                        map_crc: header.map_crc.into(),
                        map_size: header.map_size,
                    })?;
                },
                Err(e) => {
                    eprintln!("{}: {:?}", entry.path().display(), e);
                }
            }
        }
        Ok(())
    }
    helper(dir).map_err(|e| eprintln!("{}: {:?}", Path::new(dir).display(), e))
}

fn handle_args(args: Option<clap::OsValues>) -> Result<(), ()> {
    if let Some(paths) = args {
        for path in paths {
            handle_dir(path)?;
        }
    } else {
        handle_dir(OsStr::new("."))?;
    }
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian indexer")
        .about("Indexes folders of teehistorian files and dumps the index into\
                a CSV file")
        .arg(Arg::with_name("DIRECTORY")
            .help("Directories to scan (current directory if none are given)")
            .multiple(true)
        )
        .get_matches();

    let paths = matches.values_of_os("DIRECTORY");

    if handle_args(paths).is_err() {
        process::exit(1);
    }
}
