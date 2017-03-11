#![cfg(not(test))]

extern crate common;
extern crate datafile as df;
extern crate map;
extern crate ndarray;
extern crate tools;

use common::num::Cast;
use map::format;
use map::reader;
use ndarray::Array;
use std::cmp;
use std::collections::HashMap;
use std::collections::hash_map;
use std::path::Path;

struct Stats {
    tiles: [u64; 256],
    neighbours: [[u64; 256]; 256],
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            tiles: [0; 256],
            neighbours: [[0; 256]; 256],
        }
    }
}

const JUNGLE_DOODADS_5: &'static [(u8, u8)] = &[
    (0x01, 0x21),
    (0x02, 0x22),
    (0x04, 0x14),
    (0x05, 0x15),
    (0x06, 0x16),
    (0x07, 0x17),
    (0x08, 0x08),
    (0x0a, 0x1b),
    (0x0c, 0x0d),
    (0x0e, 0x0e),
    (0x0f, 0x1f),

    (0x10, 0x20),
    (0x13, 0x23),
    (0x18, 0x19),
    (0x1c, 0x1d),
    (0x1e, 0x1e),

    (0x24, 0x24),
    (0x25, 0x25),
    (0x26, 0x26),
    (0x27, 0x28),
    (0x29, 0x2b),
    (0x2c, 0x2d),
    (0x2e, 0x2f),

    (0x30, 0x57),
    (0x38, 0x3a),
    (0x3b, 0x4d),
    (0x3e, 0x4f),

    (0x48, 0x6a),

    (0x5b, 0xaf),

    (0x60, 0x82),
    (0x63, 0x84),

    (0x75, 0xa8),

    (0x90, 0xa4),

    (0xb0, 0xf4),
    (0xb5, 0xb5),
    (0xb6, 0xb6),
    (0xb7, 0xb7),
    (0xb8, 0xb8),
    (0xb9, 0xb9),
    (0xba, 0xfb),
    (0xbc, 0xfd),
    (0xbe, 0xff),

    (0xc5, 0xc5),
    (0xc6, 0xc6),
    (0xc7, 0xc7),
    (0xc8, 0xf9),

    (0xd5, 0xd6),
    (0xd7, 0xd7),

    /*
    (0xe5, 0xe7),
    (0xf5, 0xf7),
    */
];

fn process(path: &Path, dfr: df::Reader, stats: &mut Stats) -> Result<(), map::Error> {
    let mut map = map::Reader::from_datafile(dfr);

    let mut images = HashMap::new();
    let mut found = false;

    for g in map.group_indices() {
        let group = try!(map.group(g));

        for i in group.layer_indices {
            let layer = try!(map.layer(i));
            let tilemap = if let reader::LayerType::Tilemap(t) = layer.t { t } else { continue; };
            let normal = if let Some(n) = tilemap.type_.to_normal() { n } else { continue };
            if tilemap.width == 0 || tilemap.height == 0 {
                return Err(format::Error::MalformedLayerTilemap.into());
            }

            let image_index = if let Some(i) = normal.image { i } else { continue };
            let process_this_layer = match images.entry(image_index) {
                hash_map::Entry::Occupied(o) => *o.into_mut(),
                hash_map::Entry::Vacant(v) => {
                    let image = try!(map.image(image_index));
                    let name = try!(map.image_name(image.name));
                    *v.insert(name == b"jungle_doodads")
                },
            };

            if !process_this_layer {
                continue;
            }
            found = true;

            let height = tilemap.height.usize();
            let width = tilemap.width.usize();
            let tiles = try!(map.layer_tiles(normal.data));
            let tiles = try!(Array::from_shape_vec((height, width), tiles)
                             .map_err(|_| format::Error::MalformedLayerTilemap));

            for y in 0..tilemap.height+1 {
                let above_y = cmp::max(y, 1) - 1;
                let below_y = cmp::min(y + 1, tilemap.height - 1);
                let y = cmp::min(y, tilemap.height - 1);
                for x in 0..tilemap.width+1 {
                    let left_x = cmp::max(x, 1) - 1;
                    let x = cmp::min(x, tilemap.width - 1);

                    // XX.
                    // X:.
                    // X..
                    //
                    // X - tile gets counted
                    // : - main tile
                    // . - tile doesn't get counted
                    //
                    // This way, each tile neighbourhood is counted once.

                    let main = tiles[(y.usize(), x.usize())].index.usize();
                    let above = tiles[(above_y.usize(), x.usize())].index.usize();
                    let above_left = tiles[(above_y.usize(), left_x.usize())].index.usize();
                    let left = tiles[(y.usize(), left_x.usize())].index.usize();
                    let below_left = tiles[(below_y.usize(), left_x.usize())].index.usize();

                    stats.tiles[main] += 1;

                    let mut neighbours = |i: usize, j: usize| {
                        stats.neighbours[i][j] += 1;
                        stats.neighbours[j][i] += 1;
                    };

                    neighbours(main, above);
                    neighbours(main, above_left);
                    neighbours(main, left);
                    neighbours(main, below_left);
                }
            }
        }
    }

    if !found {
        return Ok(());
    }

    println!("{}", path.display());
    for (i, other) in stats.neighbours.iter().enumerate() {
        for (j, &neighbour_count) in other.iter().enumerate() {
            if neighbour_count != 0 {
                let freq = neighbour_count as f32 / stats.tiles[i] as f32 / 8.0;
                println!("{:02x} {:02x} {:.5}", i, j, freq);
            }
        }
    }
    println!("");

    *stats = Default::default();

    Ok(())
}

fn print_stats(_: &Stats) { }

fn main() {
    tools::map_stats::stats(process, print_stats);
}
