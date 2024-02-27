#![cfg(not(test))]

use datafile as df;
use map::format;
use std::path::Path;

#[derive(Default)]
struct Stats {
    author: u64,
    version: u64,
    credits: u64,
    license: u64,
    settings: u64,
    info: u64,
    total: u64,
}

fn process(_: &Path, dfr: df::Reader, stats: &mut Stats) -> Result<(), map::Error> {
    let map = map::Reader::from_datafile(dfr);
    let info = match map.info() {
        Ok(i) => i,
        Err(format::Error::MissingInfo) => {
            stats.total += 1;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    if info.author.is_some() {
        stats.author += 1;
    }
    if info.version.is_some() {
        stats.version += 1;
    }
    if info.credits.is_some() {
        stats.credits += 1;
    }
    if info.license.is_some() {
        stats.license += 1;
    }
    if info.settings.is_some() {
        stats.settings += 1;
    }
    stats.info += 1;
    stats.total += 1;
    Ok(())
}

fn print_stats(stats: &Stats) {
    println!("author:   {:5}", stats.author);
    println!("version:  {:5}", stats.version);
    println!("credits:  {:5}", stats.credits);
    println!("license:  {:5}", stats.license);
    println!("settings: {:5}", stats.settings);
    println!("info:     {:5}", stats.info);
    println!("total:    {:5}", stats.total);
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
