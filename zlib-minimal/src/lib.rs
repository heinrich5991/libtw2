//! A minimal zlib wrapper
//!
//! This wrapper only exposes the `uncompress` method of zlib, both without
//! indirection and as idiomatic Rust function.

extern crate libz_sys as raw;

use libc::c_ulong;
use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Error {
    inner: i32,
}

impl Error {
    pub fn from_raw(val: i32) -> Result<(), Error> {
        if val == raw::Z_OK {
            Ok(())
        } else {
            Err(Error { inner: val })
        }
    }
    pub fn kind(self) -> Result<ErrorKind, ()> {
        Ok(match self.inner {
            raw::Z_MEM_ERROR => ErrorKind::OutOfMemory,
            raw::Z_BUF_ERROR => ErrorKind::OutputBufferTooSmall,
            raw::Z_DATA_ERROR => ErrorKind::InvalidInput,
            _ => return Err(()),
        })
    }
    pub fn raw_error(self) -> i32 {
        self.inner
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    OutOfMemory,
    OutputBufferTooSmall,
    InvalidInput,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind() {
            Ok(k) => k.fmt(f),
            Err(()) => write!(f, "UnknownZlibError({})", self.raw_error()),
        }
    }
}

/// The wrapper for zlib's `uncompress` function.
///
/// Uncompresses the `src` parameter into the `dest` parameter and returning
/// the number of bytes written. If the decompression fails for some reason,
/// Err is returned. In this case, the `dest` buffer may or may not be
/// modified.
pub fn uncompress(dest: &mut [u8], src: &[u8]) -> Result<usize, Error> {
    let mut output_size = dest.len() as c_ulong;
    Error::from_raw(unsafe {
        raw::uncompress(
            dest.as_mut_ptr(),
            &mut output_size,
            src.as_ptr(),
            src.len() as c_ulong,
        )
    })
    .map(|()| output_size as usize)
}

/// The wrapper for zlib's `compress` function.
///
/// Compresses the `src` parameter into the `dest` parameter and returning the
/// number of bytes written. If the compression fails for some reason, Err is
/// returned. In this case, the `dest` buffer may or may not be modified.
pub fn compress(dest: &mut [u8], src: &[u8]) -> Result<usize, Error> {
    let mut output_size = dest.len() as c_ulong;
    Error::from_raw(unsafe {
        raw::compress(
            dest.as_mut_ptr(),
            &mut output_size,
            src.as_ptr(),
            src.len() as c_ulong,
        )
    })
    .map(|()| output_size as usize)
}

/// The wrapper for zlib's `compressBound` function.
///
/// Returns an upper bound on the compressed size for `compress()`.
pub fn compress_bound(source_len: usize) -> usize {
    (unsafe { raw::compressBound(source_len as c_ulong) }) as usize
}

pub fn compress_vec(source: &[u8]) -> Result<Vec<u8>, Error> {
    let upper_bound = compress_bound(source.len());
    let mut dest = Vec::with_capacity(upper_bound);

    // u8 has no destructor, this is safe
    unsafe {
        dest.set_len(upper_bound);
    }

    let output_length = compress(&mut dest, source)?;
    unsafe {
        dest.set_len(output_length);
    }

    Ok(dest)
}
