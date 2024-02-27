#![cfg(not(test))]

use datafile as df;
use std::path::Path;

fn process(_: &Path, _: df::Reader, _: &mut ()) -> Result<(), map::Error> {
    Ok(())
}

fn print_stats(_: &()) {}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
