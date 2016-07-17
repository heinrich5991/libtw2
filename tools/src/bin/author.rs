#![cfg(not(test))]

#[macro_use] extern crate common;
extern crate datafile as df;
extern crate logger;
extern crate map;
extern crate num;
extern crate tools;

use map::format;
use map::format::MapItemExt;
use std::path::Path;

#[derive(Default)]
struct Stats {
    author: u64,
    map_version: u64,
    credits: u64,
    license: u64,
    info: u64,
    total: u64,
}

fn process(_: &Path, dfr: df::Reader, stats: &mut Stats)
    -> Result<(), map::Error>
{
    let e = Err(map::Error::Map(format::Error::MalformedInfo));

    let item = unwrap_or_return!(dfr.find_item(format::MAP_ITEMTYPE_INFO, 0), {
        stats.total += 1;
        Ok(())
    });
    let common = unwrap_or_return!(format::MapItemCommonV0::from_slice(item.data), e);
    if common.version != 1 { return e; }
    let info = unwrap_or_return!(format::MapItemInfoV1::from_slice(item.data), e);

    if info.author != -1 { stats.author += 1; }
    if info.map_version != -1 { stats.map_version += 1; }
    if info.credits != -1 { stats.credits += 1; }
    if info.license != -1 { stats.license += 1; }
    stats.info += 1;
    stats.total += 1;
    Ok(())
}

fn print_stats(stats: &Stats) {
    println!("author:      {:5}", stats.author);
    println!("map_version: {:5}", stats.map_version);
    println!("credits:     {:5}", stats.credits);
    println!("license:     {:5}", stats.license);
    println!("info:        {:5}", stats.info);
    println!("total:       {:5}", stats.total);
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
