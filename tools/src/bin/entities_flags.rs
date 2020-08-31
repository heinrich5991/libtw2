#![cfg(not(test))]

extern crate common;
extern crate datafile as df;
extern crate map;
extern crate tools;

use common::num::Cast;
use std::path::Path;
use tools::map_stats::Entity;

pub struct Stats {
    tiles: [[u64; 256]; 256],
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            tiles: [[0; 256]; 256],
        }
    }
}

fn process(path: &Path, dfr: df::Reader, stats: &mut Stats) -> Result<(), map::Error> {
    let mut map = map::Reader::from_datafile(dfr);
    let game_layers = map.game_layers()?;
    let mut tiles_count = [[0u64; 256]; 256];

    let tiles = map.layer_tiles(game_layers.game())?;
    for tile in tiles.iter() {
        let index = tile.index.usize();
        let flags = tile.flags.usize();
        tiles_count[index][flags] += 1;
        stats.tiles[index][flags] += 1;
    }

    if let Some(front) = game_layers.front() {
        let tiles = map.layer_tiles(front)?;
        for tile in tiles.iter() {
            let index = tile.index.usize();
            let flags = tile.flags.usize();
            tiles_count[index][flags] += 1;
            stats.tiles[index][flags] += 1;
        }
    }

    println!("{}", path.to_string_lossy());
    for (i, inner) in tiles_count.iter().enumerate() {
        let entity = Entity(i.assert_u8());
        for (flags, &c) in inner.iter().enumerate() {
            if c != 0 {
                println!("{}: {:4b}: {:5}", entity, flags.assert_u8(), c);
            }
        }
    }
    Ok(())
}

fn print_stats(stats: &Stats) {
    for (i, inner) in stats.tiles.iter().enumerate() {
        let entity = Entity(i.assert_u8());
        for (flags, &c) in inner.iter().enumerate() {
            if c != 0 {
                println!("{}: {:4b}: {:5}", entity, flags.assert_u8(), c);
            }
        }
    }
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
