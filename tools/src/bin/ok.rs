#![cfg(not(test))]

use datafile as df;
use std::path::Path;

fn process(p: &Path, _: df::Reader, _: &mut ()) -> Result<(), map::Error> {
    println!("{}", p.display());
    Ok(())
}

fn print_stats(_: &()) {}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
