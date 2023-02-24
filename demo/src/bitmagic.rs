use common;
use std::io;
use std::mem;
use std::slice;

use writer;

/// Safe to write arbitrary bytes to this struct.
pub unsafe trait Packed {}

unsafe impl Packed for common::num::BeI32 {}
unsafe impl Packed for common::num::LeU16 {}
unsafe impl Packed for common::num::BeU32 {}
unsafe impl Packed for u8 {}
unsafe impl Packed for [u8; 16] {}
unsafe impl Packed for [u8; 32] {}

pub fn as_mut_bytes<T: Packed>(x: &mut T) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut(x as *mut _ as *mut _, mem::size_of_val(x)) }
}

pub fn as_bytes<T: Packed>(x: &T) -> &[u8] {
    unsafe { slice::from_raw_parts(x as *const _ as *const _, mem::size_of_val(x)) }
}

pub(crate) trait ReadExt: io::Read + Sized {
    fn read_packed<T: Packed>(&mut self) -> io::Result<T> {
        let mut result: T = unsafe { mem::zeroed() };
        self.read_exact(as_mut_bytes(&mut result))?;
        Ok(result)
    }
}

impl<T: io::Read> ReadExt for T {}

pub trait WriteCallbackExt: writer::Callback {
    fn write_raw<T: Packed>(&mut self, t: &T) -> Result<(), Self::Error> {
        self.write(as_bytes(t))
    }
}

impl<T: writer::Callback> WriteCallbackExt for T {}
