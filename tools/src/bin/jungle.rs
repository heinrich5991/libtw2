#![cfg(not(test))]

use libtw2_datafile as df;
use libtw2_map::reader;
use std::collections::HashMap;
use std::path::Path;

fn process(
    _: &Path,
    dfr: df::Reader,
    tilesets: &mut HashMap<Vec<u8>, u64>,
) -> Result<(), libtw2_map::Error> {
    let mut map = libtw2_map::Reader::from_datafile(dfr);
    for i in map.group_indices() {
        let group = map.group(i)?;
        for k in group.layer_indices.clone() {
            let layer = map.layer(k)?;
            let image_index = if let Some(i) = match layer.t {
                reader::LayerType::Quads(q) => q.image,
                reader::LayerType::Tilemap(t) => t.type_.to_normal().and_then(|n| n.image),
                reader::LayerType::DdraceSounds(_) => continue,
            } {
                i
            } else {
                continue;
            };
            let image = map.image(image_index)?;
            let name = map.image_name(image.name)?;
            *tilesets.entry(name).or_insert(0) += 1;
        }
    }
    Ok(())
}

fn print_stats(tilesets: &HashMap<Vec<u8>, u64>) {
    for (name, &c) in tilesets.iter() {
        println!("{:14} {:5}", String::from_utf8_lossy(name), c);
    }
}

fn main() {
    libtw2_tools::map_stats::stats(process, print_stats);
}
