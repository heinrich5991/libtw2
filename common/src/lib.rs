#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)] extern crate quickcheck;

pub use map_iter::MapIterator;
pub use slice::relative_size_of;
pub use slice::relative_size_of_mult;

#[macro_use]
mod macros;

pub mod format_bytes;
pub mod map_iter;
pub mod num;
pub mod slice;
pub mod vec;
