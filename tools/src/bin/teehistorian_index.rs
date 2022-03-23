extern crate chrono;
extern crate clap;
extern crate csv;
extern crate itertools;
extern crate logger;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate teehistorian;
extern crate uuid;
extern crate walkdir;

use chrono::DateTime;
use chrono::FixedOffset;
use itertools::Itertools;
use itertools::sorted;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::slice;
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct HexU32(u32);

impl serde::Serialize for HexU32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:08x}", self.0))
    }
}

struct HexU32Visitor;

impl<'de> serde::de::Visitor<'de> for HexU32Visitor {
    type Value = HexU32;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("8 character hex value")
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<HexU32, E> {
        let len = v.chars().count();
        if len != 8 {
            return Err(E::invalid_length(len, &self));
        }
        let value = u32::from_str_radix(v, 16).map_err(|_| {
            E::invalid_value(serde::de::Unexpected::Str(v), &self)
        })?;
        Ok(HexU32(value))
    }
}

impl<'de> serde::Deserialize<'de> for HexU32 {
    fn deserialize<D>(deserializer: D) -> Result<HexU32, D::Error>
        where D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(HexU32Visitor)
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
    timestamp: DateTime<FixedOffset>,
    server_port: u16,
    map_name: Cow<'a, str>,
    map_crc: HexU32,
    map_size: u32,
}

impl<'a> From<&'a ReadRecord> for Record<'a> {
    fn from(r: &'a ReadRecord) -> Record<'a> {
        Record {
            path: &r.path,
            game_uuid: r.game_uuid,
            timestamp: r.timestamp,
            server_port: r.server_port,
            map_name: Cow::from(&r.map_name[..]),
            map_crc: r.map_crc.into(),
            map_size: r.map_size,
        }
    }
}

fn contains<'a>(
    base: &mut slice::Iter<'a, ReadRecord>,
    writer: &mut csv::Writer<Box<dyn Write>>,
    path: &Path,
) -> Result<bool, Error>
{
    let mut found = false;
    for record in base.peeking_take_while(|r| r.path <= path) {
        if record.path == path {
            found = true;
        }
        writer.serialize(Record::from(record))?;
    }
    Ok(found)
}

struct Config {
    ignore_ext: bool,
}

fn handle_dir<'a>(
    base: &mut slice::Iter<'a, ReadRecord>,
    writer: &mut csv::Writer<Box<dyn Write>>,
    dir: &Path,
    config: &Config,
) -> Result<(), ()>
{
    fn helper<'a>(
        base: &mut slice::Iter<'a, ReadRecord>,
        writer: &mut csv::Writer<Box<dyn Write>>,
        dir: &Path,
        config: &Config,
    ) -> Result<(), Error>
    {
        let mut buffer = Buffer::new();
        for entry in WalkDir::new(dir)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        {
            let entry = entry?;
            if !config.ignore_ext &&
                entry.path().extension() != Some(OsStr::new("teehistorian"))
            {
                continue;
            }
            if entry.file_type().is_dir() {
                continue;
            }
            if contains(base, writer, entry.path())? {
                continue;
            }
            buffer.clear();
            match Reader::open(entry.path(), &mut buffer) {
                Ok((header, _)) => {
                    writer.serialize(Record {
                        path: entry.path(),
                        game_uuid: header.game_uuid,
                        timestamp: header.timestamp,
                        server_port: header.server_port,
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
    helper(base, writer, dir, config)
        .map_err(|e| eprintln!("{}: {:?}", dir.display(), e))
}

// Why do I need a separate one for this. :(
//
// `Ord` is implemented using mainly `path`.
#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct ReadRecord {
    path: PathBuf,
    game_uuid: Uuid,
    timestamp: DateTime<FixedOffset>,
    server_port: u16,
    map_name: String,
    map_crc: HexU32,
    map_size: u32,
}

fn read_index(path: &Path) -> Result<Vec<ReadRecord>, Error> {
    fn read(path: &Path) -> Result<Vec<ReadRecord>, Error> {
        csv::Reader::from_path(path)?
            .into_deserialize()
            .map(|r| r.map_err(|e| e.into()))
            .collect()
    }
    read(path).map(|mut v| { v.sort(); v })
}

fn handle_args(
    base: Option<&Path>,
    output: Option<&Path>,
    dirs: Vec<PathBuf>,
    config: &Config,
) -> Result<(), ()>
{
    let base = base.map(|b| {
        read_index(b).map_err(|e| eprintln!("{}: {:?}", b.display(), e))
    }).transpose()?.unwrap_or(Vec::new());

    let mut base_iter = base.iter();

    let mut csv_out: csv::Writer<Box<dyn Write>> = csv::Writer::from_writer(match output {
        Some(o) => Box::new(File::create(o).map_err(|e| {
            eprintln!("{}: {:?}", o.display(), e)
        })?),
        None => Box::new(io::stdout()),
    });

    for dir in dirs {
        handle_dir(&mut base_iter, &mut csv_out, &dir, config)?;
    }

    for record in base_iter {
        csv_out.serialize(Record::from(record)).map_err(|e| eprintln!("{:?}", e))?;
    }

    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("Teehistorian indexer")
        .about("Indexes folders of teehistorian files and dumps the index into \
                a CSV file")
        .arg(Arg::with_name("base")
            .short("b")
            .long("base")
            .value_name("BASE")
            .help("Sets a base index file")
        )
        .arg(Arg::with_name("inplace")
            .short("i")
            .long("in-place")
            .value_name("INDEX")
            .help("Sets the index file to update")
            .conflicts_with("base")
        )
        .arg(Arg::with_name("DIRECTORY")
            .help("Directories to scan (current directory if none are given)")
            .multiple(true)
        )
        .arg(Arg::with_name("ignore-ext")
            .long("--ignore-ext")
            .help("Don't check for the .teehistorian file extension before \
                   indexing a file")
        )
        .get_matches();

    let paths = matches.values_of_os("DIRECTORY");
    let base = matches.value_of_os("base");
    let inplace = matches.value_of_os("inplace");
    let (input, output) = match (base, inplace) {
        (None, None) => (None, None),
        (Some(b), None) => (Some(Path::new(b)), None),
        (None, Some(i)) => (Some(Path::new(i)), Some(Path::new(i))),
        (Some(_), Some(_)) => unreachable!(),
    };
    let config = Config {
        ignore_ext: matches.is_present("ignore-ext"),
    };

    let dirs = if let Some(p) = paths {
        sorted(p.into_iter().map(|p| PathBuf::from(p)))
    } else {
        vec![PathBuf::from(".")]
    };

    if handle_args(input, output, dirs, &config).is_err() {
        process::exit(1);
    }
}
