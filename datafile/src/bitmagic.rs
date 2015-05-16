pub use common::relative_size_of;
pub use common::relative_size_of_mult;
pub use common::slice::transmute_mut as transmute_mut_slice;
pub use common::slice::transmute as transmute_slice;

use ext::ReadComplete;
use std::io::Read;
use std::io;
use std::mem;

#[cfg(target_endian="little")]
pub unsafe fn from_little_endian<T>(_buffer: &mut [T]) {
}

#[cfg(target_endian="little")]
pub unsafe fn to_little_endian<T>(_buffer: &mut [T]) {
}

#[cfg(target_endian="big")]
pub unsafe fn from_little_endian<T>(buffer: &mut [T]) {
    swap_endian(buffer);
}

#[cfg(target_endian="big")]
pub unsafe fn to_little_endian<T>(buffer: &mut [T]) {
    swap_endian(buffer);
}

// depending on the target's endianness this function might not be needed
#[allow(dead_code)]
pub unsafe fn swap_endian<T>(buffer: &mut [T]) {
    let len = buffer.len();
    let buffer_bytes: &mut [u8] = transmute_mut_slice(buffer);
    for i in 0..len {
        let mut start = i * mem::size_of::<T>();
        let mut end = start + mem::size_of::<T>() - 1;
        while start < end {
            buffer_bytes.swap(start, end);
            start += 1;
            end -= 1;
        }
    }
}

pub unsafe fn read_exact_raw<T>(mut reader: &mut Read, buffer: &mut [T]) -> io::Result<()> {
    reader.read_complete(transmute_mut_slice(buffer))
}

pub unsafe fn read_exact_le_ints<T>(reader: &mut Read, buffer: &mut [T]) -> io::Result<()> {
    try!(read_exact_raw(reader, buffer));
    from_little_endian(buffer);
    Ok(())
}

pub unsafe fn read_exact_le_ints_owned<T>(reader: &mut Read, count: usize) -> io::Result<Vec<T>> {
    let mut result = Vec::with_capacity(count);
    result.set_len(count);
    try!(read_exact_le_ints(reader, &mut result));
    Ok(result)
}
