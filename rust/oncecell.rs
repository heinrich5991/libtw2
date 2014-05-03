//! Type for a cached value

#![crate_type = "rlib"]
#![crate_type = "dylib"]

use std::kinds::marker;
use std::ty::Unsafe;

/// A memory location that is initialized once and then kept constant until the
/// end of its life.
pub struct OnceCell<T> {
	value: Unsafe<Option<T>>,
	noshare: marker::NoShare,
}

impl<T> OnceCell<T> {
	/// Create a new empty `OnceCell`.
	pub fn new() -> OnceCell<T> {
		OnceCell {
			value: Unsafe::new(None),
			noshare: marker::NoShare,
		}
	}

	/// Create a new `OnceCell` already containing the specified value.
	pub fn new_with_value(value: T) -> OnceCell<T> {
		OnceCell {
			value: Unsafe::new(Some(value)),
			noshare: marker::NoShare,
		}
	}

	/// Consumes the `OnceCell`, returning the wrapped value.
	pub fn unwrap(self) -> T {
		unsafe { self.value.unwrap().unwrap() }
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
	/// # Failure
	///
	/// Fails if the value is not initalized yet.
	pub fn borrow<'a>(&'a self) -> &'a T {
		match self.try_borrow() {
			Some(ptr) => ptr,
			None => fail!("OnceCell<T> not initalized yet")
		}
	}
}

impl<T:Eq> Eq for OnceCell<T> {
	fn eq(&self, other: &OnceCell<T>) -> bool {
		self.borrow() == other.borrow()
	}
}

impl<T:Clone> Clone for OnceCell<T> {
	fn clone(&self) -> OnceCell<T> {
		let self_value = unsafe { &*self.value.get() };
		OnceCell {
			value: Unsafe::new(self_value.clone()),
			noshare: marker::NoShare,
		}
	}
}

mod test {
	/*
	use super::*;

	#[test]
	fn smoketest_cell() {
		let x = Cell::new(10);
		assert_eq!(x, Cell::new(10));
		assert_eq!(x.get(), 10);
		x.set(20);
		assert_eq!(x, Cell::new(20));
		assert_eq!(x.get(), 20);

		let y = Cell::new((30, 40));
		assert_eq!(y, Cell::new((30, 40)));
		assert_eq!(y.get(), (30, 40));
	}

	#[test]
	fn cell_has_sensible_show() {
		use str::StrSlice;

		let x = Cell::new("foo bar");
		assert!(format!("{}", x).contains(x.get()));

		x.set("baz qux");
		assert!(format!("{}", x).contains(x.get()));
	}

	#[test]
	fn double_imm_borrow() {
		let x = OnceCell::new(0);
		let _b1 = x.borrow();
		x.borrow();
	}

	#[test]
	fn no_mut_then_imm_borrow() {
		let x = OnceCell::new(0);
		let _b1 = x.borrow_mut();
		assert!(x.try_borrow().is_none());
	}

	#[test]
	fn no_imm_then_borrow_mut() {
		let x = OnceCell::new(0);
		let _b1 = x.borrow();
		assert!(x.try_borrow_mut().is_none());
	}

	#[test]
	fn no_double_borrow_mut() {
		let x = OnceCell::new(0);
		let _b1 = x.borrow_mut();
		assert!(x.try_borrow_mut().is_none());
	}

	#[test]
	fn imm_release_borrow_mut() {
		let x = OnceCell::new(0);
		{
			let _b1 = x.borrow();
		}
		x.borrow_mut();
	}

	#[test]
	fn mut_release_borrow_mut() {
		let x = OnceCell::new(0);
		{
			let _b1 = x.borrow_mut();
		}
		x.borrow();
	}

	#[test]
	fn double_borrow_single_release_no_borrow_mut() {
		let x = OnceCell::new(0);
		let _b1 = x.borrow();
		{
			let _b2 = x.borrow();
		}
		assert!(x.try_borrow_mut().is_none());
	}

	#[test]
	#[should_fail]
	fn discard_doesnt_unborrow() {
		let x = OnceCell::new(0);
		let _b = x.borrow();
		let _ = _b;
		let _b = x.borrow_mut();
	}
	*/
}
