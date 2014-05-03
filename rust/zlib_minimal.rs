#![crate_type = "rlib"]
#![crate_type = "dylib"]

extern crate libc;

use libc::c_ulong;

pub mod raw {
	use libc::{c_ulong, c_int};

	#[link(name="z")]
	extern {
		pub fn uncompress(dest: *mut u8, dest_len: *mut c_ulong, source: *u8, source_len: c_ulong) -> c_int;
	}

	pub static Z_OK: c_int = 0;
}

pub fn uncompress(dest: &mut [u8], src: &[u8]) -> Result<uint,()> {
	let mut output_size = dest.len() as c_ulong;
	if unsafe { raw::uncompress(dest.as_mut_ptr(), &mut output_size, src.as_ptr(), src.len() as c_ulong) } == raw::Z_OK {
		Ok(output_size as uint)
	}
	else {
		Err(())
	}
}
