extern crate clap;
extern crate common;
extern crate datafile;
extern crate logger;
extern crate map;
extern crate rmp;
extern crate walkdir;

use common::num::Cast;
use map::format::Tile;
use std::path::Path;
use common::pretty;
use map::format;
use std::process;
use walkdir::WalkDir;

#[derive(Debug)]
struct Error(map::Error);


impl From<datafile::Error> for Error {
    fn from(e: datafile::Error) -> Error {
        Error(e.into())
    }
}

impl From<map::format::Error> for Error {
    fn from(e: map::format::Error) -> Error {
        Error(e.into())
    }
}

impl From<map::Error> for Error {
    fn from(e: map::Error) -> Error {
        Error(e)
    }
}


fn count<'a, I: Iterator<Item=&'a Tile>>(tiles: I, count: &mut [u64; 256]) {
    for tile in tiles {
        count[tile.index.usize()] += 1;
    }
}
fn tile_solo(index: u8) -> Option<&'static str> {
    Some(match index {
        21 => "SOLO_START",

        _ => return None,
    })
}
fn tile_turret(index: u8) -> Option<&'static str> {
    Some(match index {
        220 => "PLASMAE",
        221 => "PLASMAF",
        222 => "PLASMA",
        223 => "PLASMAU",

        _ => return None,
    })
}
fn tile_switch_on(index: u8) -> Option<&'static str> {
    Some(match index {
        22 => "SWITCH_TIMED",
        24 => "SWITCH",

        _ => return None,
    })
}

fn tile_switch_off(index: u8) -> Option<&'static str> {
    Some(match index {
        23 => "SWITCH_TIMED_OFF",
        25 => "SWITCH_OFF",

        _ => return None,
    })
}

fn process(path: &Path) -> Result<(), Error> {
    let mut map = map::Reader::open(path)?;
    let game_layers = map.game_layers()?;

    let mut tiles_count = [0u64; 256];
    let mut turret_switch_nr = [false; 256];
    let mut got_switch_on = [false; 256];
    let mut got_switch_off = [false; 256];

    let switch_layer = if let Some(t) = game_layers.switch() {
        t
    } else {
        return Ok(());
    };

    let mut len_turrets = 0;
    for tile in map.switch_layer_tiles(switch_layer)?.iter()
    {
        if tile_turret(tile.index).is_some()
        {
            turret_switch_nr[tile.number as usize] = true;
            len_turrets += 1;
        }
        if tile_switch_on(tile.index).is_some()
        {
            got_switch_on[tile.number as usize] = true;
        }
        if tile_switch_off(tile.index).is_some()
        {
            got_switch_off[tile.number as usize] = true;
        }
    }


    count(map.layer_tiles(game_layers.game())?.iter(), &mut tiles_count);
    if let Some(f) = game_layers.front() {
        count(map.layer_tiles(f)?.iter(), &mut tiles_count);
    }


    let len_solo = tiles_count.iter().enumerate().filter(|&(i, &c)| {
        c != 0 && tile_solo(i.assert_u8()).is_some()
    }).count();

    let mut is_solo = false;
    if len_turrets > 0
    {
        if path.to_string_lossy().contains("solo") || path.to_string_lossy().contains("race")
        {
            is_solo = true;
        }

        let maybe_info = map.info();
        if let Err(format::Error::MissingInfo) = maybe_info {
            eprintln!("{}: Missing map info.", path.display());
        }

        let info = maybe_info?;
        if let Some(s) = info.settings {
            let settings = map.settings(s)?;
            for line in settings.iter() {
                let str_line = pretty::AlmostString::new(line);
                if str_line.to_string() == "sv_solo_server 1"
                {
                    is_solo = true;
                }
            }
        }
        if is_solo
        {
            println!("{}: Found {} plasma turret(s) in switch layer on solo map", path.display(), len_turrets);
        }
        else if len_solo > 0
        {
            println!("{}: Found {} plasma turret(s) in switch layer on team map with solo tiles", path.display(), len_turrets);
        }
    }

    if len_turrets > 0 && (is_solo || len_solo > 0)
    {
        let maybe_info = map.info();

        let info = maybe_info?;
        if let Some(s) = info.settings {
            let settings = map.settings(s)?;
            for line in settings.iter() {
                let str_line = pretty::AlmostString::new(line).to_string();
                if str_line.starts_with("switch_open") {
                    let swtich_nr = str_line.split(" ").collect::<Vec<_>>()[1];
                    if turret_switch_nr[(swtich_nr.parse::<i32>().unwrap()) as usize]
                    {
                        println!("{}: Setting: {}", path.display(), str_line);
                    }
                }
            }
        }

        let switch = if let Some(t) = game_layers.switch() {
            t
        } else {
            return Ok(());
        };
        let switch_tiles = map.switch_layer_tiles(switch)?;
        for ((y, x), &t) in switch_tiles.indexed_iter() {
            if let Some(name) = tile_turret(t.index) {
                println!("{}: {}: ({}, {}), Nr: {}, OnSw: {}, OffSw: {}", path.display(), name, x, y, t.number, got_switch_on[t.number as usize], got_switch_off[t.number as usize]);
            }

            if let Some(name) = tile_switch_on(t.index) {
                if turret_switch_nr[t.number as usize]
                {
                    println!("{}: {}: ({}, {}), Nr: {}", path.display(), name, x, y, t.number);
                }
            }            

            if let Some(name) = tile_switch_off(t.index) {
                if turret_switch_nr[t.number as usize]
                {
                    println!("{}: {}: ({}, {}), Nr: {}", path.display(), name, x, y, t.number);
                }
            }
        }
    }
    Ok(())
}

fn main() {
    use clap::App;
    use clap::Arg;

    logger::init();

    let matches = App::new("DDNet map switched solo turrets finder")
        .about("Checks if a map uses switched turrets in maps that have solo parts.")
        .arg(Arg::with_name("MAPS")
             .help("Sets the path to the directory that contain map files to analyse")
             .multiple(false)
             .required(true))
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
