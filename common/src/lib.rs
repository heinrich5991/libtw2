#![cfg_attr(all(test, feature="nightly-test"), feature(plugin))]
#![cfg_attr(all(test, feature="nightly-test"), plugin(quickcheck_macros))]
#[cfg(all(test, feature="nightly-test"))] extern crate quickcheck;

extern crate arrayvec;
extern crate ref_slice;

pub use map_iter::MapIterator;
pub use slice::relative_size_of;
pub use slice::relative_size_of_mult;

#[macro_use]
mod macros;

pub mod map_iter;
pub mod num;
pub mod pretty;
pub mod slice;
pub mod vec;
