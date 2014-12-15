//! Type for a cached value

#![crate_type = "rlib"]
#![crate_type = "dylib"]

use std::fmt;
use std::kinds::marker;
use std::cell::UnsafeCell;

/// A memory location that is initialized once and then kept constant until the
/// end of its life.
pub struct OnceCell<T> {
	value: UnsafeCell<Option<T>>,
	noshare: marker::NoSync,
}

impl<T> OnceCell<T> {
	/// Create a new empty `OnceCell`.
	pub fn new() -> OnceCell<T> {
		OnceCell {
			value: UnsafeCell::new(None),
			noshare: marker::NoSync,
		}
	}

	/// Create a new `OnceCell` already containing the specified value.
	pub fn new_with_value(value: T) -> OnceCell<T> {
		OnceCell {
			value: UnsafeCell::new(Some(value)),
			noshare: marker::NoSync,
		}
	}

	/// Consumes the `OnceCell`, returning the wrapped value.
	pub fn unwrap(self) -> T {
		unsafe { self.value.into_inner().unwrap() }
	}

	/// Attempts to initialize the `OnceCell`.
	///
	/// Returns `Err` if it was already initialized, `Ok` otherwise.
	pub fn try_init(&self, value: T) -> Result<(), ()> {
		match unsafe { &*self.value.get() } {
			&None => {
				unsafe { *self.value.get() = Some(value) };
				Ok(())
			}
			_ => Err(())
		}
	}

	/// Initializes the `OnceCell`.
	///
	/// # Failure
	///
	/// Fails if the `OnceCell` is already initialized.
	pub fn init(&self, value: T) {
		self.try_init(value).unwrap()
	}

	/// Attempts to immutably borrow the wrapped value.
	///
	/// The borrow lasts until the `OnceCell` exits scope.
	///
	/// Returns `None` if the value is not initialized yet.
	pub fn try_borrow<'a>(&'a self) -> Option<&'a T> {
		unsafe { (*self.value.get()).as_ref() }
	}

	/// Immutably borrows the wrapped value.
	///
	/// The borrow lasts until the `OnceCell` exits scope.
	///
	/// # Panics
	///
	/// Panics if the value is not initalized yet.
	pub fn borrow<'a>(&'a self) -> &'a T {
		match self.try_borrow() {
			Some(ptr) => ptr,
			None => panic!("OnceCell<T> not initalized yet")
		}
	}
}

impl<T:PartialEq> PartialEq for OnceCell<T> {
	fn eq(&self, other: &OnceCell<T>) -> bool {
		self.borrow() == other.borrow()
	}
}

impl<T:Eq> Eq for OnceCell<T> { }

impl<T:Clone> Clone for OnceCell<T> {
	fn clone(&self) -> OnceCell<T> {
		let self_value = unsafe { &*self.value.get() };
		OnceCell {
			value: UnsafeCell::new(self_value.clone()),
			noshare: marker::NoSync,
		}
	}
}

impl<T:fmt::Show> fmt::Show for OnceCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r"OnceCell {{ value: {} }}", self.borrow())
    }
}

#[cfg(test)]
mod test {
	use super::OnceCell;

	#[test]
	fn smoketest() {
		let x = OnceCell::new();
		assert_eq!(x.try_borrow(), None);
		assert_eq!(x.try_init(10u32), Ok(()));
		assert_eq!(x, OnceCell::new_with_value(10));
		assert_eq!(x.try_borrow(), Some(&10));
		assert_eq!(x.try_init(20), Err(()));
	}

	#[test]
	#[should_fail]
	fn borrow_without_value() {
		let x: OnceCell<()> = OnceCell::new();
		x.borrow();
	}

	#[test]
	#[should_fail]
	fn init_with_value() {
		let x = OnceCell::new_with_value(());
		x.init(());
	}

	#[test]
	#[should_fail]
	fn compare_before_init() {
		OnceCell::<()>::new() == OnceCell::new();
	}
}
