#![cfg(not(test))]

extern crate datafile as df;
extern crate map;
extern crate tools;

use std::path::Path;

fn process(_: &Path, mut df: df::Reader, _: &mut ())
    -> Result<(), map::Error>
{
    df.debug_dump()?;
    Ok(())
}

fn print_stats(_: &()) {
}

fn main() {
    tools::map_stats::stats(process, print_stats);
}
