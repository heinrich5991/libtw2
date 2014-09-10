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
			self.end_u.offset(1);
		}
	}

	pub fn unwrap(self) -> Vec<U> {
		assert!(self.end_u as *() == self.end_t as *(),
			"trying to unwrap a PartialVec before completing the writes to it");
		unsafe {
			let PartialVec { vec, ..  };
			mem::transmute(vec)
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

impl<T,U> Drop for PartialVec<T,U> {
	fn drop(&mut self) {
		unsafe {
			while self.start_u < self.end_u {
				ptr::read(self.start_u as *U);
				self.start_u = self.start_u.offset(1);
			}
			while self.start_t < self.end_t {
				ptr::read(self.start_t as *T);
				self.start_t = self.start_t.offset(1);
			}

			// prevent the vector from running destructors on the elements
			self.vec.set_len(0);
		}
	}
}

trait MapInplace<T,U,V:Vector<U>> : Vector<T> {
	fn map_inplace(self, f: |T| -> U) -> V;
}

impl<T,U> MapInplace<T,U,Vec<U>> for Vec<T> {
	fn map_inplace(self, f: |T| -> U) -> Vec<U> {
		let mut pv = PartialVec::new(self);
		for value in pv {
			pv.push(f(value));
		}
		pv.unwrap()
	}
}
