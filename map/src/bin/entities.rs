#![cfg(not(test))]

extern crate datafile as df;
extern crate logger;
extern crate map;
extern crate num;

use num::ToPrimitive;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

fn entity_name(index: u8) -> Option<&'static str> {
    Some(match index {
        0x00 => "None",
        0x01 => "Coll",
        0x02 => "Deat",
        0x03 => "Unho",
        0xc0 => "SpBr",
        0xc1 => "SpRe",
        0xc2 => "SpBl",
        0xc3 => "FlRe",
        0xc4 => "FlBl",
        0xc5 => "Shie",
        0xc6 => "Hear",
        0xc7 => "Shot",
        0xc8 => "Gren",
        0xc9 => "Ninj",
        0xca => "Lase",
        _ => return None,
    })
}

#[derive(Clone, Copy)]
struct Entity(u8);

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Entity(inner) = *self;
        match entity_name(inner) {
            Some(name) => write!(f, "{}", name),
            None => write!(f, "{:4x}", inner),
        }
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Default)]
struct ErrorStats {
    map_errors: HashMap<map::format::Error,u64>,
    df_errors: HashMap<df::format::Error,u64>,
    io_errors: Vec<io::Error>,
}

fn process(path: &Path, global_tiles_count: &mut [u64; 256]) -> Result<(),map::Error> {
    let file = try!(File::open(path));
    let dfr = try!(df::Reader::new(file));
    let mut map = map::Reader::from_datafile(dfr);
    let game_layers = try!(map.game_layers());
    let tiles = try!(map.layer_tiles(game_layers.game));

    let mut tiles_count = [0u64; 256];
    for tile in tiles {
        tiles_count[tile.index.to_usize().unwrap()] += 1;
        global_tiles_count[tile.index.to_usize().unwrap()] += 1;
    }
    println!("{}", path.to_string_lossy());
    for (i, &c) in tiles_count.iter().enumerate() {
        let entity = Entity(i.to_u8().unwrap());
        if c != 0 {
            println!("{}: {:5}", entity, c);
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
    logger::init();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut global_tiles_count = [0; 256];
    let mut stats = ErrorStats::default();
    for (_, arg) in args.enumerate() {
        have_args = true;
        match process(Path::new(&arg), &mut global_tiles_count) {
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
    for (i, &c) in global_tiles_count.iter().enumerate() {
        let entity = Entity(i.to_u8().unwrap());
        if c != 0 {
            println!("{}: {:5}", entity, c);
        }
    }
}
