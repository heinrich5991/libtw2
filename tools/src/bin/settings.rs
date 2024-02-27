#![cfg(not(test))]

use libtw2_common::pretty;
use libtw2_datafile as df;
use libtw2_map::format;
use std::path::Path;

fn process(_: &Path, dfr: df::Reader, _: &mut ()) -> Result<(), libtw2_map::Error> {
    let mut map = libtw2_map::Reader::from_datafile(dfr);
    let maybe_info = map.info();
    if let Err(format::Error::MissingInfo) = maybe_info {
        return Ok(());
    }
    let info = maybe_info?;
    if let Some(s) = info.settings {
        let settings = map.settings(s)?;
        for line in settings.iter() {
            println!("{:?}", pretty::AlmostString::new(line));
        }
    }
    Ok(())
}

fn nothing(_: &()) {}

fn main() {
    libtw2_tools::map_stats::stats(process, nothing);
}
