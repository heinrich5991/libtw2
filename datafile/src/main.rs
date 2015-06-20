#![cfg(not(test))]

extern crate datafile;
extern crate env_logger;

use datafile::DatafileReaderFile;
use std::fs::File;
use std::path::Path;

fn main() {
    env_logger::init().unwrap();
    let file = File::open(&Path::new("../dm1.map")).unwrap();

    let mut df = match DatafileReaderFile::new(file) {
        Ok(df) => df,
        Err(e) => panic!("Error opening datafile: {:?}", e),
    };
    df.debug_dump().unwrap();
}
