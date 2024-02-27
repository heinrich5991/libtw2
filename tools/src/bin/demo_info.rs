extern crate libtw2_gamenet_teeworlds_0_6 as libtw2_gamenet;

use libtw2_demo::RawChunk;
use libtw2_gamenet::msg::Game;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use warn::Warn;

#[derive(Debug)]
enum Error {
    DemoRead(libtw2_demo::ReadError),
    DemoWrite(libtw2_demo::WriteError),
    Io(io::Error),
    Gamenet(libtw2_gamenet::Error),
}

#[derive(Debug)]
enum Warning {
    Demo(libtw2_demo::Warning),
    Gamenet(libtw2_packer::Warning),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<libtw2_demo::ReadError> for Error {
    fn from(err: libtw2_demo::ReadError) -> Self {
        match err.io_error() {
            Ok(io) => Error::Io(io),
            Err(demo) => Error::DemoRead(demo),
        }
    }
}

impl From<libtw2_demo::WriteError> for Error {
    fn from(err: libtw2_demo::WriteError) -> Error {
        match err.io_error() {
            Ok(io) => Error::Io(io),
            Err(demo) => Error::DemoWrite(demo),
        }
    }
}

impl From<libtw2_gamenet::Error> for Error {
    fn from(e: libtw2_gamenet::Error) -> Error {
        Error::Gamenet(e)
    }
}

impl From<libtw2_demo::Warning> for Warning {
    fn from(w: libtw2_demo::Warning) -> Warning {
        Warning::Demo(w)
    }
}

impl From<libtw2_packer::Warning> for Warning {
    fn from(w: libtw2_packer::Warning) -> Warning {
        Warning::Gamenet(w)
    }
}

#[derive(Default)]
struct ErrorStats {
    demo_warnings: HashMap<libtw2_demo::Warning, u64>,
    demo_read_errors: Vec<libtw2_demo::ReadError>,
    demo_write_errors: Vec<libtw2_demo::WriteError>,
    gamenet_warnings: HashMap<libtw2_packer::Warning, u64>,
    gamenet_errors: HashMap<libtw2_gamenet::Error, u64>,
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
    let mut reader = libtw2_demo::Reader::new(file, warn::wrap(warn))?;
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
            RawChunk::Message(bytes) => {
                let mut u = libtw2_packer::Unpacker::new_from_demo(bytes);
                println!("message {:?}", Game::decode(warn::wrap(warn), &mut u)?);
            }
            RawChunk::Tick { tick, .. } => println!("tick={}", tick),
            RawChunk::Snapshot(_) => println!("snapshot"),
            RawChunk::SnapshotDelta(_) => println!("snapshot_delta"),
            RawChunk::Unknown => println!("Unknown chunk"),
        }
    }
    println!();
    Ok(())
}

fn main() {
    libtw2_logger::init();

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
