#![cfg(not(test))]

extern crate datafile as df;
extern crate logger;
extern crate map;
extern crate num;
extern crate tools;

use num::ToPrimitive;
use std::fmt;
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

pub struct Stats {
    tiles: [u64; 256],
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            tiles: [0; 256],
        }
    }
}

fn process(path: &Path, dfr: df::Reader, stats: &mut Stats) -> Result<(), map::Error> {
    let mut map = map::Reader::from_datafile(dfr);
    let game_layers = try!(map.game_layers());
    let tiles = try!(map.layer_tiles(game_layers.game));

    let mut tiles_count = [0u64; 256];
    for tile in tiles {
        tiles_count[tile.index.to_usize().unwrap()] += 1;
        stats.tiles[tile.index.to_usize().unwrap()] += 1;
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

fn print_stats(stats: &Stats) {
    for (i, &c) in stats.tiles.iter().enumerate() {
        let entity = Entity(i.to_u8().unwrap());
        if c != 0 {
            println!("{}: {:5}", entity, c);
        }
    }
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
