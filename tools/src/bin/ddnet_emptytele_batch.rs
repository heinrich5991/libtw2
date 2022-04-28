extern crate clap;
extern crate datafile;
extern crate logger;
extern crate map;
extern crate walkdir;

use map::Error;
use std::path::Path;
use std::process;
use walkdir::WalkDir;

fn tele_tile_name(index: u8) -> Option<&'static str> {
    Some(match index {
        10 => "TELEINEVIL",
        14 => "TELEINWEAPON",
        15 => "TELEINHOOK",
        26 => "TELEIN",
        27 => "TELEOUT",
        29 => "TELECHECK",
        30 => "TELECHECKOUT",
        31 => "TELECHECKIN",
        63 => "TELECHECKINEVIL",
        _ => return None,
    })
}

fn process(path: &Path) -> Result<(), Error> {
    let mut map = map::Reader::open(path)?;
    let game_layers = map.game_layers()?;

    let tele = if let Some(t) = game_layers.teleport() {
        t
    } else {
        return Ok(());
    };
    let tele_tiles = map.tele_layer_tiles(tele)?;
    for ((y, x), &t) in tele_tiles.indexed_iter() {
        if t.index != 0 && t.number == 0 {
            if let Some(name) = tele_tile_name(t.index) {
                println!("{}: {}: ({}, {})", path.display(), name, x, y);
            } else {
                println!("{}: unknown ({}): ({}, {})", path.display(), t.index, x, y);
            }
        }
    }
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("DDNet teleporter scanner")
        .about("Scans map files for weird teleporters.")
        .arg(Arg::with_name("MAPS")
            .help("Sets the path to the directory that contain map files to analyse")
            .multiple(false)
            .required(true)
        )
        .get_matches();

    let maps = matches.value_of_os("MAPS").unwrap();

    let mut error = false;
    for file in WalkDir::new(maps).into_iter().filter_map(|file| file.ok()) {
        if file.metadata().unwrap().is_file() && file.path().extension().unwrap() == "map" {
            let map = file.path();
            match process(map) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("{}: {:?}", map.display(), err);
                    error = true;
                }
            }
        }
    }
 
    if error {
        process::exit(1);
    }
}
