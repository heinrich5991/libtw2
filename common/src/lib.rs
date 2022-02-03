#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate arrayvec;
extern crate file_offset;
extern crate num_traits;
extern crate ref_slice;
#[cfg(feature = "serde")] extern crate serde;
extern crate unreachable;

pub use map_iter::MapIterator;
pub use slice::relative_size_of;
pub use slice::relative_size_of_mult;
pub use takeable::Takeable;

#[macro_use]
mod macros;

pub mod digest;
pub mod io;
pub mod map_iter;
pub mod num;
pub mod pretty;
pub mod slice;
pub mod str;
pub mod takeable;
pub mod vec;
