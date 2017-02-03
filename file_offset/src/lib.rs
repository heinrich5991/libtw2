use std::fs::File;
use std::io;

mod sys;

pub trait FileExt {
    fn read_offset(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;
    fn write_offset(&self, buf: &[u8], offset: u64) -> io::Result<usize>;
}

impl FileExt for File {
    #[inline]
    fn read_offset(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        sys::read_offset(self, buf, offset)
    }
    #[inline]
    fn write_offset(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        sys::write_offset(self, buf, offset)
    }
}
