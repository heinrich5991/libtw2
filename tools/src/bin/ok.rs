#![cfg(not(test))]

use libtw2_datafile as df;
use std::path::Path;

fn process(p: &Path, _: df::Reader, _: &mut ()) -> Result<(), libtw2_map::Error> {
    println!("{}", p.display());
    Ok(())
}

fn print_stats(_: &()) {}

fn main() {
    libtw2_tools::map_stats::stats(process, print_stats);
}
