use buffer::Buffer;
use buffer::BufferRef;
use buffer::with_buffer;
use common;
use std::mem;
use std::slice;

use raw::Callback;
use raw::CallbackReadError;
use raw::ResultExt;

/// Safe to write arbitrary bytes to this struct.
pub unsafe trait Packed { }

unsafe impl Packed for common::num::BeI32 { }
unsafe impl Packed for common::num::LeU16 { }
unsafe impl Packed for common::num::BeU32 { }
unsafe impl Packed for u8 { }

pub fn as_mut_bytes<T: Packed>(x: &mut T) -> &mut [u8] {
    unsafe {
        slice::from_raw_parts_mut(x as *mut _ as *mut _, mem::size_of_val(x))
    }
}

pub trait CallbackExt: Callback {
    fn read_raw<T: Packed>(&mut self) -> Result<T, CallbackReadError<Self::Error>> {
        let mut result = unsafe { mem::zeroed() };
        {
            let buffer = as_mut_bytes(&mut result);
            let read = self.read(buffer).wrap()?;
            if read != buffer.len() {
                return Err(CallbackReadError::EndOfFile);
            }
        }
        Ok(result)
    }
    fn read_buffer<'d, B: Buffer<'d>>(&mut self, buf: B)
        -> Result<&'d [u8], Self::Error>
    {
        with_buffer(buf, |buf| self.read_buffer_ref(buf))
    }
    fn read_buffer_ref<'d, 's>(&mut self, mut buf: BufferRef<'d, 's>)
        -> Result<&'d [u8], Self::Error>
    {
        unsafe {
            let read = self.read(buf.uninitialized_mut())?;
            buf.advance(read);
            Ok(buf.initialized())
        }
    }
}

impl<T: Callback> CallbackExt for T { }
