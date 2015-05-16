#![cfg(not(test))]

extern crate datafile;

use datafile::DatafileReader;
use datafile::DatafileBuffer;
use datafile::SeekReaderCast;

use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io;
use std::path::Path;

struct SeekReaderRef<'a>(&'a mut (SeekReaderCast+'a));

impl<'a> Read for SeekReaderRef<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.as_reader_mut().read(buf)
    }
}

impl<'a> Seek for SeekReaderRef<'a> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.0.as_seek_mut().seek(from)
    }
}

fn main() {
    let mut file = File::open(&Path::new("../dm1.map")).unwrap();
    let dfr = match DatafileReader::read(SeekReaderRef(&mut file)) {
        Ok(Ok(x)) => x,
        Ok(Err(x)) => panic!("datafile error {:?}", x),
        Err(x) => panic!("IO error {:?}", x),
    };
    dfr.debug_dump();

    let _dfb = match DatafileBuffer::from_datafile(&dfr) {
        Some(x) => x,
        None => panic!("datafile error ..."),
    };
}
