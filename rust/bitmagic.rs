
use std::cast;
use std::io::IoResult;
use std::raw;
use std::mem;

pub unsafe fn transmute_slice<'a,T,U>(x: &'a [T]) -> &'a [U] {
	cast::transmute(raw::Slice { data: x.as_ptr(), len: relative_size_of_mult::<T,U>(x.len()) })
}

pub unsafe fn transmute_mut_slice<'a,T,U>(x: &'a mut [T]) -> &'a mut [U] {
	cast::transmute(raw::Slice { data: x.as_ptr(), len: relative_size_of_mult::<T,U>(x.len()) })
}

#[cfg(target_endian="little")]
pub unsafe fn from_little_endian<T:Int>(_buffer: &mut [T]) {
}

#[cfg(target_endian="little")]
pub unsafe fn to_little_endian<T:Int>(_buffer: &mut [T]) {
}

#[cfg(target_endian="big")]
pub unsafe fn from_little_endian<T:Int>(buffer: &mut [T]) {
	swap_endian(buffer);
}

#[cfg(target_endian="big")]
pub unsafe fn to_little_endian<T:Int>(buffer: &mut [T]) {
	swap_endian(buffer);
}

// depending on the target's endianness this function might not be needed
#[allow(dead_code)]
pub unsafe fn swap_endian<T:Int>(buffer: &mut [T]) {
	let len = buffer.len();
	let buffer_bytes: &mut [u8] = transmute_mut_slice(buffer);
	for i in range(0, len) {
		let mut start = i * mem::size_of::<T>();
		let mut end = start + mem::size_of::<T>() - 1;
		while start < end {
			buffer_bytes.swap(start, end);
			start += 1;
			end -= 1;
		}
	}
}

pub unsafe fn read_exact_raw<T>(reader: &mut Reader, buffer: &mut [T]) -> IoResult<()> {
	reader.fill(transmute_mut_slice(buffer))
}

pub fn read_exact_le_ints<T:Int>(reader: &mut Reader, buffer: &mut [T]) -> IoResult<()> {
	try!(unsafe { read_exact_raw(reader, buffer) } );
	unsafe { from_little_endian(buffer) };
	Ok(())
}

pub fn read_exact_le_ints_owned<T:Int>(reader: &mut Reader, count: uint) -> IoResult<Vec<T>> {
	let mut result = Vec::with_capacity(count);
	unsafe { result.set_len(count); }
	try!(read_exact_le_ints(reader, result.as_mut_slice()));
	Ok(result)
}

pub fn relative_size_of_mult<T,U>(mult: uint) -> uint {
	assert!(mult * mem::size_of::<T>() % mem::size_of::<U>() == 0);
	mult * mem::size_of::<T>() / mem::size_of::<U>()
}

pub fn relative_size_of<T,U>() -> uint {
	relative_size_of_mult::<T,U>(1)
}
