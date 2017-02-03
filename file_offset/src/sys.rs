#[cfg(unix)]
pub use self::unix::*;

#[cfg(windows)]
pub use self::windows::*;

#[cfg(unix)]
mod unix {
    use std::fs::File;
    use std::io;
    use std::os::unix::fs::FileExt;

    #[inline]
    pub fn read_offset(file: &File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        file.read_at(buf, offset)
    }

    #[inline]
    pub fn write_offset(file: &File, buf: &[u8], offset: u64) -> io::Result<usize> {
        file.write_at(buf, offset)
    }
}

#[cfg(windows)]
mod windows {
    use std::fs::File;
    use std::io;
    use std::os::windows::fs::FileExt;

    #[inline]
    pub fn read_offset(file: &File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        file.seek_read(buf, offset)
    }

    #[inline]
    pub fn write_offset(file: &File, buf: &[u8], offset: u64) -> io::Result<usize> {
        file.seek_write(buf, offset)
    }
}
