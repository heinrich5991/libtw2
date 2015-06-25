#![cfg(not(test))]

extern crate datafile as df;
extern crate env_logger;
extern crate map;
extern crate num;

use num::ToPrimitive;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Default)]
struct ErrorStats {
    map_errors: HashMap<map::format::Error,u64>,
    df_errors: HashMap<df::format::Error,u64>,
    io_errors: Vec<io::Error>,
}

fn process(path: &Path) -> Result<(),map::Error> {
    let file = try!(File::open(path));
    let dfr = try!(df::Reader::new(file));
    let mut map = map::Reader::from_datafile(dfr);
    let (_, _, _, game_layer) = try!(map.game_layer());
    let tiles = try!(map.layer_tiles(game_layer.data));

    let mut tiles_count = [0u64; 256];
    for tile in tiles {
        tiles_count[tile.index.to_usize().unwrap()] += 1;
    }
    println!("{}", path.to_string_lossy());
    for (i, &c) in tiles_count.iter().enumerate() {
        let i = i.to_u8().unwrap();
        if c != 0 {
            println!("{:2x}: {:5}", i, c);
        }
    }
    Ok(())
}

fn update_stats(stats: &mut ErrorStats, err: map::Error) {
    match err {
        map::Error::Map(e) => {
            *stats.map_errors.entry(e).or_insert(0) += 1;
        }
        map::Error::Df(df::Error::Df(e)) => {
            *stats.df_errors.entry(e).or_insert(0) += 1;
        }
        map::Error::Df(df::Error::Io(e)) => {
            stats.io_errors.push(e);
        }
    }
}

fn print_stats(stats: &ErrorStats) {
    for (e, c) in &stats.map_errors {
        println!("{:?}: {}", e, c);
    }
    for (e, c) in &stats.df_errors {
        println!("{:?}: {}", e, c);
    }
    for e in &stats.io_errors {
        println!("{:?}", e);
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut stats = ErrorStats::default();
    for (_, arg) in args.enumerate() {
        have_args = true;
        match process(Path::new(&arg)) {
            Ok(()) => {},
            Err(err) => {
                println!("{}: {:?}", arg.to_string_lossy(), err);
                update_stats(&mut stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <MAP>...", program_name.to_string_lossy());
        return;
    }
    print_stats(&stats);
}
