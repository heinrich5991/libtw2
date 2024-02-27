#![cfg(not(test))]

use common::pretty;
use datafile as df;
use map::format;
use std::path::Path;

fn process(_: &Path, dfr: df::Reader, _: &mut ()) -> Result<(), map::Error> {
    let mut map = map::Reader::from_datafile(dfr);
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
    tools::map_stats::stats(process, nothing);
}
