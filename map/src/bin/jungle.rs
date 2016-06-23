#![cfg(not(test))]

extern crate datafile as df;
extern crate logger;
extern crate map;
extern crate num;

use map::reader;
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
    ok: u64,
}

fn process(path: &Path, tilesets: &mut HashMap<Vec<u8>,u64>) -> Result<(),map::Error> {
    let file = try!(File::open(path));
    let dfr = try!(df::Reader::new(file));
    let mut map = map::Reader::from_datafile(dfr);
    for i in map.group_indices() {
        let group = try!(map.group(i));
        for k in group.layer_indices.clone() {
            let layer = try!(map.layer(k));
            let image_index = if let Some(i) = match layer.t {
                reader::LayerType::Quads(q) => q.image,
                reader::LayerType::Tilemap(t) => t.type_.to_normal().and_then(|n| n.image),
                reader::LayerType::DdraceSounds(_) => continue,
            } { i } else { continue; };
            let image = try!(map.image(image_index));
            let name = try!(map.image_name(image.name));
            *tilesets.entry(name).or_insert(0) += 1;
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
    println!("ok: {}", stats.ok);
}

fn main() {
    logger::init();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut tilesets = Default::default();
    let mut stats = ErrorStats::default();
    for (_, arg) in args.enumerate() {
        have_args = true;
        match process(Path::new(&arg), &mut tilesets) {
            Ok(()) => stats.ok += 1,
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
    for (name, &c) in tilesets.iter() {
        println!("{:14} {:5}", String::from_utf8_lossy(name), c);
    }
}
