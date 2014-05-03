//! A minimal zlib wrapper
//!
//! This wrapper only exposes the `uncompress` method of zlib, both without
//! indirection and as idiomatic Rust function.

#![crate_type = "rlib"]
#![crate_type = "dylib"]

extern crate libc;

use libc::c_ulong;

/// The raw interface to zlib
pub mod raw {
	use libc::{c_ulong, c_int};

	#[link(name="z")]
	extern {
		pub fn uncompress(dest: *mut u8, dest_len: *mut c_ulong, source: *u8, source_len: c_ulong) -> c_int;
	}

	pub static Z_OK: c_int = 0;
}

/// The wrapper for zlib's `uncompress` function.
///
/// Uncompresses the `src` parameter into the `dest` parameter and returning
/// the number of bytes written. If the decompression fails for some reason,
/// Err is returned. In this case, the `dest` buffer may or may not be
/// modified.
pub fn uncompress(dest: &mut [u8], src: &[u8]) -> Result<uint,()> {
	let mut output_size = dest.len() as c_ulong;
	if unsafe { raw::uncompress(dest.as_mut_ptr(), &mut output_size, src.as_ptr(), src.len() as c_ulong) } == raw::Z_OK {
		Ok(output_size as uint)
	}
	else {
		Err(())
	}
}
