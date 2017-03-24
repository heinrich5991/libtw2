use std::mem;
use std::slice;
use raw::CallbackNew;
use raw::CallbackReadError;
use raw::ResultExt;

/// Safe to write arbitrary bytes to this struct.
pub unsafe trait Packed { }

pub fn as_mut_bytes<T: Packed>(x: &mut T) -> &mut [u8] {
    unsafe {
        slice::from_raw_parts_mut(x as *mut _ as *mut _, mem::size_of_val(x))
    }
}

pub trait CallbackNewExt: CallbackNew {
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
}

impl<T: CallbackNew> CallbackNewExt for T { }
