#![cfg(not(test))]

extern crate datafile;

use datafile::DatafileReader;
use datafile::DatafileBuffer;
use datafile::SeekReaderCast;

use std::io::File;
use std::io::IoResult;
use std::io::SeekStyle;

struct SeekReaderRef<'a>(&'a mut (SeekReaderCast+'a));

impl<'a> Reader for SeekReaderRef<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.0.as_reader_mut().read(buf)
    }
}

impl<'a> Seek for SeekReaderRef<'a> {
    fn tell(&self) -> IoResult<u64> {
        self.0.as_seek_ref().tell()
    }
    fn seek(&mut self, pos: i64, seek_style: SeekStyle) -> IoResult<()> {
        self.0.as_seek_mut().seek(pos, seek_style)
    }
}

fn main() {
    let mut file = File::open(&Path::new("../dm1.map")).unwrap();
    let dfr = match DatafileReader::read(SeekReaderRef(&mut file)) {
        Ok(Ok(x)) => x,
        Ok(Err(x)) => panic!("datafile error {}", x),
        Err(x) => panic!("IO error {}", x),
    };
    dfr.debug_dump();

    let _dfb = match DatafileBuffer::from_datafile(&dfr) {
        Some(x) => x,
        None => panic!("datafile error ..."),
    };
}
