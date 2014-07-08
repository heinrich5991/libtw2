//! An owned, partially type-converted vector.

#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(unsafe_destructor)]

use std::mem;
use std::ptr;

/// An owned, partially type-converted vector.
///
/// No allocations are performed by usage, only a deallocation happens in the
/// destructor which should only run when unwinding.
///
/// It can be used to convert a vector of `T`s into a vector of `U`s, by
/// converting the individual elements one-by-one.
///
/// You may call the `push` method as often as you get a `Some(t)` from `pop`.
/// After pushing the same number of `U`s as you got `T`s, you can `unwrap` the
/// vector.
///
/// # Example
///
/// ```rust
/// let pv = PartialVec::new(vec![0u, 1]);
/// assert_eq!(pv.pop(), Some(0));
/// assert_eq!(pv.pop(), Some(1));
/// assert_eq!(pv.pop(), None);
/// pv.push(2u);
/// pv.push(3);
/// assert_eq!(pv.into_vec(), vec![2, 3]);
/// ```
//
// Upheld invariants:
//
// (a) `vec` isn't modified except when the `PartialVec` goes out of scope, the
//     only thing it is used for is keeping the memory which the `PartialVec`
//     uses for the inplace conversion.
//
// (b) `start_u` points to the start of the vector.
//
// (c) `end_u` points to one element beyond the vector.
//
// (d) `start_u` <= `end_u` <= `start_t` <= `end_t`.
//
// (e) From `start_u` (incl.) to `end_u` (excl.) there are sequential instances
//     of type `U`.
//
// (f) From `start_t` (incl.) to `end_t` (excl.) there are sequential instances
//     of type `T`.

pub struct PartialVec<T,U> {
    vec: Vec<T>,

    start_u: *mut U,
    end_u: *mut U,
    start_t: *mut T,
    end_t: *mut T,
}

impl<T,U> PartialVec<T,U> {
    /// Creates a `PartialVec` from a `Vec`.
    pub fn new(mut vec: Vec<T>) -> PartialVec<T,U> {
        // FIXME: Assert that the types `T` and `U` have the same size.
        assert!(mem::size_of::<T>() != 0);
        assert!(mem::size_of::<U>() != 0);
        assert!(mem::size_of::<T>() == mem::size_of::<U>());

        let start = vec.as_mut_ptr();

        // This `as int` cast is safe, because the size of the elements of the
        // vector is not 0, and:
        //
        // 1) If the size of the elements in the vector is 1, the `int` may
        //    overflow, but it has the correct bit pattern so that the
        //    `.offset()` function will work.
        //
        //    Example:
        //        Address space 0x0-0xF.
        //        `u8` array at: 0x1.
        //        Size of `u8` array: 0x8.
        //        Calculated `offset`: -0x8.
        //        After `array.offset(offset)`: 0x9.
        //        (0x1 + 0x8 = 0x1 - 0x8)
        //
        // 2) If the size of the elements in the vector is >1, the `uint` ->
        //    `int` conversion can't overflow.
        let offset = vec.len() as int;

        let start_u = start as *mut U;
        let end_u = start as *mut U;
        let start_t = start;
        let end_t = unsafe { start_t.offset(offset) };

        // (b) is satisfied, `start_u` points to the start of `vec`.

        // (c) is also satisfied, `end_t` points to the end of `vec`.

        // `start_u == end_u == start_t <= end_t`, so also `start_u <= end_u <=
        // start_t <= end_t`, thus (b).

        // As `start_u == end_u`, it is represented correctly that there are no
        // instances of `U` in `vec`, thus (e) is satisfied.

        // At start, there are only elements of type `T` in `vec`, so (f) is
        // satisfied, as `start_t` points to the start of `vec` and `end_t` to
        // the end of it.

        // This points inside the vector, as the vector has length `offset`.

        PartialVec {
            // (a) is satisfied, `vec` isn't modified in the function.
            vec: vec,
            start_u: start_u,
            end_u: end_u,
            start_t: start_t,
            end_t: end_t,
        }
    }

    /// Pops a `T` from the `PartialVec`.
    ///
    /// Returns `Some(t)` if there are more `T`s in the vector, otherwise
    /// `None`.
    fn pop(&mut self) -> Option<T> {
        // The `if` ensures that there are more `T`s in `vec`.
        if self.start_t < self.end_t {
            let result;
            unsafe {
                // (f) is satisfied before, so in this if branch there actually
                // is a `T` at `start_t`.  After shifting the pointer by one,
                // (f) is again satisfied.
                result = ptr::read(self.start_t as *const T);
                self.start_t = self.start_t.offset(1);
            }
            Some(result)
        } else {
            None
        }
    }

    /// Pushes a new `U` to the `PartialVec`.
    ///
    /// # Failure
    ///
    /// Fails if not enough `T`s were popped to have enough space for the new
    /// `U`.
    pub fn push(&mut self, value: U) {
        // The assert assures that still `end_u <= start_t` (d) after
        // the function.
        assert!(self.end_u as *const () < self.start_t as *const (),
            "writing more elements to PartialVec than reading from it")
        unsafe {
            // (e) is satisfied before, and after writing one `U`
            // to `end_u` and shifting it by one, it's again
            // satisfied.
            ptr::write(self.end_u, value);
            self.end_u = self.end_u.offset(1);
        }
    }

    /// Unwraps the new `Vec` of `U`s after having pushed enough `U`s and
    /// popped all `T`s.
    ///
    /// # Failure
    ///
    /// Fails if not all `T`s were popped, also fails if not the same amount of
    /// `U`s was pushed before calling `unwrap`.
    pub fn into_vec(self) -> Vec<U> {
        // If `self.end_u == self.end_t`, we know from (e) that there are no
        // more `T`s in `vec`, we also know that the whole length of `vec` is
        // now used by `U`s, thus we can just transmute `vec` from a vector of
        // `T`s to a vector of `U`s safely.

        assert!(self.end_u as *const () == self.end_t as *const (),
            "trying to unwrap a PartialVec before completing the writes to it");

        // Extract `vec` and prevent the destructor of `PartialVec` from
        // running.
        unsafe {
            let vec = ptr::read(&self.vec);
            mem::forget(self);
            mem::transmute(vec)
        }
    }
}

#[unsafe_destructor]
impl<T,U> Drop for PartialVec<T,U> {
    fn drop(&mut self) {
        unsafe {
            // As per (a) `vec` hasn't been modified until now. As it has a
            // length currently, this would run destructors of `T`s which might
            // not be there. So at first, set `vec`s length to `0`. This must
            // be done at first to remain memory-safe as the destructors of `U`
            // or `T` might cause unwinding where `vec`s destructor would be
            // executed.
            self.vec.set_len(0);

            // As per (e) and (f) we have instances of `U`s and `T`s in `vec`.
            // Destruct them.
            while self.start_u < self.end_u {
                let _ = ptr::read(self.start_u as *const U); // Run a `U` destructor.
                self.start_u = self.start_u.offset(1);
            }
            while self.start_t < self.end_t {
                let _ = ptr::read(self.start_t as *const T); // Run a `T` destructor.
                self.start_t = self.start_t.offset(1);
            }
            // After this destructor ran, the destructor of `vec` will run,
            // deallocating the underlying memory.
        }
    }
}

impl<T,U> Iterator<T> for PartialVec<T,U> {
    fn next(&mut self) -> Option<T> {
        self.pop()
    }
}

pub trait MapInplace<T,U,V:Vector<U>> : Vector<T> {
    fn map_inplace(self, f: |T| -> U) -> V;
}

impl<T,U> MapInplace<T,U,Vec<U>> for Vec<T> {
    /// Converts a `Vec<T>` to a `Vec<U>` where `T` and `U` have the same size.
    ///
    /// # Example
    ///
    /// ```rust
    /// let v = vec![0u, 1, 2];
    /// let w = v.map_inplace(|i| i + 3);
    /// assert_eq!(w.as_slice() == &[3, 4, 5]);
    ///
    /// let big_endian_u16s = vec![0x1122u16, 0x3344];
    /// let u8s = big_endian_u16s.map_inplace(|x| [
    ///     ((x >> 8) & 0xff) as u8,
    ///     (x & 0xff) as u8
    /// ]);
    /// assert_eq!(u8s.as_slice() == &[[0x11, 0x22], [0x33, 0x44]]);
    /// ```
    fn map_inplace(self, f: |T| -> U) -> Vec<U> {
        let mut pv = PartialVec::new(self);
        loop {
            let maybe_t = pv.pop();
            match maybe_t {
                Some(t) => pv.push(f(t)),
                None => return pv.into_vec(),
            };
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    #[should_fail]
    fn test_vec_truncate_fail() {
        struct BadElem(int);
        impl Drop for BadElem {
            fn drop(&mut self) {
                let BadElem(ref mut x) = *self;
                if *x == 0xbadbeef {
                    fail!("BadElem failure: 0xbadbeef")
                }
            }
        }

        let mut v = vec![BadElem(1), BadElem(2), BadElem(0xbadbeef), BadElem(4)];
        v.truncate(0);
    }

    #[test]
    #[should_fail]
    fn test_map_inplace_incompatible_types_fail() {
        let v = vec![0u, 1, 2];
        v.map_inplace(|_| ());
    }

    #[test]
    fn test_map_inplace() {
        let v = vec![0u, 1, 2];
        assert_eq!(v.map_inplace(|i: uint| i as int - 1).as_slice, &[-1i, 0, 1]);
    }
}
