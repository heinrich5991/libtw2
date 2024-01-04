#![cfg(not(test))]

extern crate datafile as df;
extern crate map;
extern crate tools;

use std::path::Path;

fn process(_: &Path, _: df::Reader, _: &mut ()) -> Result<(), map::Error> {
    Ok(())
}

fn print_stats(_: &()) {}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
