extern crate demo;
extern crate gamenet_teeworlds_0_6 as gamenet;
extern crate hexdump;
extern crate logger;
extern crate packer;
extern crate warn;

use gamenet::msg::Game;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use warn::Warn;

#[derive(Debug)]
enum Error {
    DemoRead(demo::ReadError),
    DemoWrite(demo::WriteError),
    Io(io::Error),
    Gamenet(gamenet::Error),
}

#[derive(Debug)]
enum Warning {
    Demo(demo::Warning),
    Gamenet(packer::Warning),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<demo::ReadError> for Error {
    fn from(err: demo::ReadError) -> Self {
        match err.io_error() {
            Ok(io) => Error::Io(io),
            Err(demo) => Error::DemoRead(demo),
        }
    }
}

impl From<demo::WriteError> for Error {
    fn from(err: demo::WriteError) -> Error {
        match err.io_error() {
            Ok(io) => Error::Io(io),
            Err(demo) => Error::DemoWrite(demo),
        }
    }
}

impl From<gamenet::Error> for Error {
    fn from(e: gamenet::Error) -> Error {
        Error::Gamenet(e)
    }
}

impl From<demo::Warning> for Warning {
    fn from(w: demo::Warning) -> Warning {
        Warning::Demo(w)
    }
}

impl From<packer::Warning> for Warning {
    fn from(w: packer::Warning) -> Warning {
        Warning::Gamenet(w)
    }
}

#[derive(Default)]
struct ErrorStats {
    demo_warnings: HashMap<demo::Warning, u64>,
    demo_read_errors: Vec<demo::ReadError>,
    demo_write_errors: Vec<demo::WriteError>,
    gamenet_warnings: HashMap<packer::Warning, u64>,
    gamenet_errors: HashMap<gamenet::Error, u64>,
    io_errors: Vec<io::Error>,
    ok: u64,
}

fn update_warning_stats(stats: &mut ErrorStats, warning: Warning) {
    match warning {
        Warning::Demo(w) => *stats.demo_warnings.entry(w).or_insert(0) += 1,
        Warning::Gamenet(w) => *stats.gamenet_warnings.entry(w).or_insert(0) += 1,
    }
}

fn update_error_stats(stats: &mut ErrorStats, err: Error) {
    match err {
        Error::DemoRead(e) => stats.demo_read_errors.push(e),
        Error::DemoWrite(e) => stats.demo_write_errors.push(e),
        Error::Gamenet(e) => *stats.gamenet_errors.entry(e).or_insert(0) += 1,
        Error::Io(e) => stats.io_errors.push(e),
    }
}

fn print_error_stats(error_stats: &ErrorStats) {
    for e in &error_stats.demo_read_errors {
        println!("{}", e);
    }
    for e in &error_stats.demo_write_errors {
        println!("{}", e);
    }
    for (w, c) in &error_stats.demo_warnings {
        println!("{:?}: {}", w, c);
    }
    for (e, c) in &error_stats.gamenet_errors {
        println!("{:?}: {}", e, c);
    }
    for (w, c) in &error_stats.gamenet_warnings {
        println!("{:?}: {}", w, c);
    }
    for e in &error_stats.io_errors {
        println!("{:?}", e);
    }
    println!("ok: {}", error_stats.ok);
}

fn process<W: Warn<Warning>>(warn: &mut W, path: &Path) -> Result<(), Error> {
    let file = fs::File::open(path)?;
    let mut reader = demo::Reader::new(warn::wrap(warn), file)?;
    println!("{}", path.display());
    println!("version: {:?}", reader.version());
    println!(
        "net_version: {}",
        String::from_utf8_lossy(reader.net_version())
    );
    println!("map_name: {}", String::from_utf8_lossy(reader.map_name()));
    println!("map_size: {}", reader.map_size());
    println!("map_crc: {:x}", reader.map_crc());
    println!("timestamp: {}", String::from_utf8_lossy(reader.timestamp()));
    while let Some(chunk) = reader.read_chunk(warn::wrap(warn))? {
        match chunk {
            demo::RawChunk::Message(bytes) => {
                let mut u = packer::Unpacker::new_from_demo(bytes);
                println!("message {:?}", Game::decode(warn::wrap(warn), &mut u)?);
            }
            demo::RawChunk::Tick { tick, .. } => println!("tick={}", tick),
            demo::RawChunk::Snapshot(_) => println!("snapshot"),
            demo::RawChunk::SnapshotDelta(_) => println!("snapshot_delta"),
            demo::RawChunk::Unknown => println!("Unknown chunk"),
        }
    }
    println!();
    Ok(())
}

fn main() {
    logger::init();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut error_stats = ErrorStats::default();
    for arg in args {
        have_args = true;
        let path = Path::new(&arg);
        match process(
            warn::closure(&mut |w| {
                println!("{}: {:?}", path.display(), w);
                update_warning_stats(&mut error_stats, w);
            }),
            path,
        ) {
            Ok(()) => error_stats.ok += 1,
            Err(err) => {
                println!("{}: {:?}", path.display(), err);
                update_error_stats(&mut error_stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <DEMO>...", program_name.to_string_lossy());
        return;
    }
    print_error_stats(&error_stats);
}
