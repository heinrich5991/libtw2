#![cfg(not(test))]

extern crate datafile;

use datafile::DatafileReader;
use datafile::DatafileBuffer;

use std::io::File;

fn main() {
    let file = box File::open(&Path::new("../dm1.map")).unwrap();
    let dfr = match DatafileReader::read(file) {
        Ok(Ok(x)) => x,
        Ok(Err(x)) => panic!("datafile error {}", x),
        Err(x) => panic!("IO error {}", x),
    };
    //println!("{:?}", df);
    dfr.debug_dump();

    let _dfb = match DatafileBuffer::from_datafile(&dfr) {
        Some(x) => x,
        None => panic!("datafile error ..."),
    };
}
