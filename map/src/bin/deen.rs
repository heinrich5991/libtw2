#![cfg(not(test))]

extern crate datafile as df;
extern crate env_logger;
extern crate map;
extern crate num;

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

#[derive(Default)]
struct Stats {
    game: u64,
    teleport: u64,
    speedup: u64,
    front: u64,
    switch: u64,
    tune: u64,
}

fn process(path: &Path, stats: &mut Stats) -> Result<(),map::Error> {
    let file = try!(File::open(path));
    let dfr = try!(df::Reader::new(file));
    let map = map::Reader::from_datafile(dfr);
    let game_layers = try!(map.game_layers());
    stats.game += 1;
    if game_layers.teleport.is_some() { stats.teleport += 1; }
    if game_layers.speedup.is_some() { stats.speedup += 1; }
    if game_layers.front.is_some() { stats.front += 1; }
    if game_layers.switch.is_some() { stats.switch += 1; }
    if game_layers.tune.is_some() { stats.tune += 1; }
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

fn print_stats(error_stats: &ErrorStats, stats: &Stats) {
    for (e, c) in &error_stats.map_errors {
        println!("{:?}: {}", e, c);
    }
    for (e, c) in &error_stats.df_errors {
        println!("{:?}: {}", e, c);
    }
    for e in &error_stats.io_errors {
        println!("{:?}", e);
    }
    println!("ok: {}", error_stats.ok);
    println!("--------");
    println!("game: {}", stats.game);
    println!("teleport: {}", stats.teleport);
    println!("speedup: {}", stats.speedup);
    println!("front: {}", stats.front);
    println!("switch: {}", stats.switch);
    println!("tune: {}", stats.tune);
}

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut error_stats = ErrorStats::default();
    let mut stats = Stats::default();
    for (_, arg) in args.enumerate() {
        have_args = true;
        match process(Path::new(&arg), &mut stats) {
            Ok(()) => error_stats.ok += 1,
            Err(err) => {
                println!("{}: {:?}", arg.to_string_lossy(), err);
                update_stats(&mut error_stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <MAP>...", program_name.to_string_lossy());
        return;
    }
    print_stats(&error_stats, &stats);
}
