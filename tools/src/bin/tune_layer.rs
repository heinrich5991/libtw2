#![cfg(not(test))]

#[macro_use]
extern crate common;
extern crate datafile as df;
extern crate map;
extern crate tools;

use common::num::Cast;
use std::fmt;
use std::path::Path;

#[derive(Clone, Copy)]
struct Entity(u8);

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Entity(inner) = *self;
        write!(f, "{:4x}", inner)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub struct Stats {
    tune_layers: u64,
    tiles: [u64; 256],
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            tune_layers: 0,
            tiles: [0; 256],
        }
    }
}

fn process(path: &Path, dfr: df::Reader, stats: &mut Stats) -> Result<(), map::Error> {
    let mut map = map::Reader::from_datafile(dfr);
    let game_layers = try!(map.game_layers());
    let tune_layer = unwrap_or_return!(game_layers.tune(), Ok(()));
    let tiles = try!(map.tune_layer_tiles(tune_layer));

    stats.tune_layers += 1;
    let mut tiles_count = [0u64; 256];
    for tile in tiles.iter() {
        tiles_count[tile.index.usize()] += 1;
        stats.tiles[tile.index.usize()] += 1;
    }
    println!("{}", path.to_string_lossy());
    for (i, &c) in tiles_count.iter().enumerate() {
        let entity = Entity(i.assert_u8());
        if c != 0 {
            println!("{}: {:5}", entity, c);
        }
    }
    Ok(())
}

fn print_stats(stats: &Stats) {
    for (i, &c) in stats.tiles.iter().enumerate() {
        let entity = Entity(i.assert_u8());
        if c != 0 {
            println!("{}: {:5}", entity, c);
        }
    }
    println!("total: {}", stats.tune_layers);
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
