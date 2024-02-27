#![cfg(not(test))]

use libtw2_datafile::Version as DfVersion;
use std::io;
use std::io::Write;
use std::path::Path;

#[derive(Default)]
struct Stats {
    v3: u64,
    v4_crude: u64,
    v4: u64,
}

fn process(
    _: &Path,
    dfr: libtw2_datafile::Reader,
    stats: &mut Stats,
) -> Result<(), libtw2_map::Error> {
    match dfr.version() {
        DfVersion::V3 => stats.v3 += 1,
        DfVersion::V4Crude => stats.v4_crude += 1,
        DfVersion::V4 => stats.v4 += 1,
    }
    print!(
        "v3={} v4_crude={} v4={}\r",
        stats.v3, stats.v4_crude, stats.v4
    );
    io::stdout().flush().unwrap();
    Ok(())
}

fn print_stats(stats: &Stats) {
    println!(
        "v3={} v4_crude={} v4={}",
        stats.v3, stats.v4_crude, stats.v4
    );
}

fn main() {
    libtw2_tools::map_stats::stats(process, print_stats);
}
