#![cfg(not(test))]

use datafile as df;
use std::path::Path;

#[derive(Default)]
struct Stats {
    game: u64,
    teleport: u64,
    speedup: u64,
    front: u64,
    switch: u64,
    tune: u64,
}

fn process(_: &Path, dfr: df::Reader, stats: &mut Stats) -> Result<(), map::Error> {
    let map = map::Reader::from_datafile(dfr);
    let game_layers = map.game_layers()?;
    stats.game += 1;
    if game_layers.teleport_raw.is_some() {
        stats.teleport += 1;
    }
    if game_layers.speedup_raw.is_some() {
        stats.speedup += 1;
    }
    if game_layers.front_raw.is_some() {
        stats.front += 1;
    }
    if game_layers.switch_raw.is_some() {
        stats.switch += 1;
    }
    if game_layers.tune_raw.is_some() {
        stats.tune += 1;
    }
    Ok(())
}

fn print_stats(stats: &Stats) {
    println!("game: {}", stats.game);
    println!("teleport: {}", stats.teleport);
    println!("speedup: {}", stats.speedup);
    println!("front: {}", stats.front);
    println!("switch: {}", stats.switch);
    println!("tune: {}", stats.tune);
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
