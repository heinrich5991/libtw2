#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(unsafe_destructor)]

use std::mem;
use std::ptr;

// TODO: T, U same size
struct PartialVec<T,U> {
	vec: Vec<T>,

	start_u: *mut U,
	end_u: *mut U,
	start_t: *mut T,
	end_t: *mut T,
}

impl<T,U> PartialVec<T,U> {
	pub fn new(mut vec: Vec<T>) -> PartialVec<T,U> {
		// TODO: do this statically
		assert!(mem::size_of::<T>() != 0);
		assert!(mem::size_of::<U>() != 0);
		assert!(mem::size_of::<T>() == mem::size_of::<U>());

		let start = vec.as_mut_ptr();
		let offset = vec.len().to_int().expect("integer overflow");

		let start_u = start as *mut U;
		let end_u = start as *mut U;
		let start_t = start;
		let end_t = unsafe { start_t.offset(offset) };

		PartialVec {
			vec: vec,
			start_u: start_u,
			end_u: end_u,
			start_t: start_t,
			end_t: end_t,
		}
	}

	pub fn push(&mut self, value: U) {
		assert!(self.end_u as *() < self.start_t as *(),
			"writing more elements to PartialVec than reading from it")
		unsafe {
			ptr::write(self.end_u, value);
			self.end_u = self.end_u.offset(1);
		}
	}

	pub fn unwrap(self) -> Vec<U> {
		assert!(self.end_u as *() == self.end_t as *(),
			"trying to unwrap a PartialVec before completing the writes to it");
		unsafe {
			let vec = ptr::read(&self.vec);
			mem::forget(self);
			mem::transmute::<Vec<T>,Vec<U>>(vec)
		}
	}
}

impl<T,U> Iterator<T> for PartialVec<T,U> {
	fn next(&mut self) -> Option<T> {
		if self.start_t < self.end_t {
			let result;
			unsafe {
				result = ptr::read(self.start_t as *T);
				self.start_t = self.start_t.offset(1);
			}
			Some(result)
		} else {
			None
		}
	}
}

#[unsafe_destructor]
impl<T,U> Drop for PartialVec<T,U> {
	fn drop(&mut self) {
		unsafe {
			// first, prevent the vector from running destructors on the elements
			self.vec.set_len(0);

			while self.start_u < self.end_u {
				ptr::read(self.start_u as *U);
				self.start_u = self.start_u.offset(1);
			}
			while self.start_t < self.end_t {
				ptr::read(self.start_t as *T);
				self.start_t = self.start_t.offset(1);
			}
		}
	}
}

pub trait MapInplace<T,U,V:Vector<U>> : Vector<T> {
	fn map_inplace(self, f: |T| -> U) -> V;
}

impl<T,U> MapInplace<T,U,Vec<U>> for Vec<T> {
	fn map_inplace(self, f: |T| -> U) -> Vec<U> {
		let mut pv = PartialVec::new(self);
		loop {
			let new_value = match pv.next() {
				Some(x) => f(x),
				None => return pv.unwrap(),
			};
			pv.push(new_value);
		}
	}
}

#[cfg(test)]
mod test {
	use super::MapInplace;

	#[test]
	#[should_fail]
	fn incompatible_types_fail() {
		let v = vec![0u, 1, 2];
		v.map_inplace(|_| ());
	}

	#[test]
	fn compatible_types_nofail() {
		let v = vec![0u, 1, 2];
		assert_eq!(v.map_inplace(|i: uint| i as int - 1), vec![-1i, 0, 1]);
	}
}
