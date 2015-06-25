#![cfg(not(test))]

extern crate datafile;
extern crate env_logger;
extern crate map;

use datafile::Version as DfVersion;
use std::env;
use std::fs::File;
use std::io::Write;
use std::io;
use std::path::Path;

use map::format::*;

#[derive(Default)]
struct Stats {
    error: u64,
    v3: u64,
    v4_crude: u64,
    v4: u64,
}

fn process(path: &Path, stats: &mut Stats) -> Result<(),datafile::Error> {
    let file = try!(File::open(path));
    let dfr = try!(datafile::Reader::new(file));
    match dfr.version() {
        DfVersion::V3 => stats.v3 += 1,
        DfVersion::V4Crude => stats.v4_crude += 1,
        DfVersion::V4 => stats.v4 += 1,
    }
    Ok(())
}

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut stats = Stats::default();
    for (_, arg) in args.enumerate() {
        have_args = true;
        match process(Path::new(&arg), &mut stats) {
            Ok(()) => {},
            Err(e) => {
                println!("{}: {:?}", arg.to_string_lossy(), e);
                stats.error += 1;
            }
        }
        print!("v3={} v4_crude={} v4={} error={}\r", stats.v3, stats.v4_crude, stats.v4, stats.error);
        io::stdout().flush().unwrap();
    }
    if !have_args {
        println!("USAGE: {} <MAP>...", program_name.to_string_lossy());
        return;
    }
    println!("v3={} v4_crude={} v4={} error={}", stats.v3, stats.v4_crude, stats.v4, stats.error);
}
