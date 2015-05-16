//! A minimal zlib wrapper
//!
//! This wrapper only exposes the `uncompress` method of zlib, both without
//! indirection and as idiomatic Rust function.

extern crate libc;

use libc::c_ulong;

/// The raw interface to zlib
pub mod raw {
    use libc::{c_ulong, c_int};

    #[link(name="z")]
    extern {
        pub fn uncompress(dest: *mut u8, dest_len: *mut c_ulong, source: *const u8, source_len: c_ulong) -> c_int;
        pub fn compress(dest: *mut u8, dest_len: *mut c_ulong, source: *const u8, source_len: c_ulong) -> c_int;
        pub fn compressBound(source_len: c_ulong) -> c_ulong;
    }

    pub static Z_OK: c_int = 0;
}

/// The wrapper for zlib's `uncompress` function.
///
/// Uncompresses the `src` parameter into the `dest` parameter and returning
/// the number of bytes written. If the decompression fails for some reason,
/// Err is returned. In this case, the `dest` buffer may or may not be
/// modified.
pub fn uncompress(dest: &mut [u8], src: &[u8]) -> Result<usize,()> {
    let mut output_size = dest.len() as c_ulong;
    if unsafe { raw::uncompress(dest.as_mut_ptr(), &mut output_size, src.as_ptr(), src.len() as c_ulong) } == raw::Z_OK {
        Ok(output_size as usize)
    } else {
        Err(())
    }
}

/// The wrapper for zlib's `compress` function.
///
/// Compresses the `src` parameter into the `dest` parameter and returning the
/// number of bytes written. If the compression fails for some reason, Err is
/// returned. In this case, the `dest` buffer may or may not be modified.
pub fn compress(dest: &mut [u8], src: &[u8]) -> Result<usize,()> {
    let mut output_size = dest.len() as c_ulong;
    if unsafe { raw::compress(dest.as_mut_ptr(), &mut output_size, src.as_ptr(), src.len() as c_ulong) } == raw::Z_OK {
        Ok(output_size as usize)
    } else {
        Err(())
    }
}

/// The wrapper for zlib's `compressBound` function.
///
/// Returns an upper bound on the compressed size for `compress()`.
pub fn compress_bound(source_len: usize) -> usize {
    (unsafe { raw::compressBound(source_len as c_ulong) }) as usize
}

pub fn compress_vec(source: &[u8]) -> Result<Vec<u8>,()> {
    let upper_bound = compress_bound(source.len());
    let mut dest = Vec::with_capacity(upper_bound);

    // u8 has no destructor, this is safe
    unsafe { dest.set_len(upper_bound); }

    let output_length = try!(compress(&mut dest, source));
    unsafe { dest.set_len(output_length); }

    Ok(dest)
}
