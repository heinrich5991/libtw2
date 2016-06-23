#![cfg(not(test))]

extern crate datafile;
extern crate logger;

use std::fs::File;
use std::path::Path;

fn main() {
    logger::init();
    let file = File::open(&Path::new("../dm1.map")).unwrap();

    let mut df = match datafile::Reader::new(file) {
        Ok(df) => df,
        Err(e) => panic!("Error opening datafile: {:?}", e),
    };
    df.debug_dump().unwrap();
}
