#![feature(core)]
#![feature(slice_patterns)]

pub use slice::relative_size_of;
pub use slice::relative_size_of_mult;

mod macros;

pub mod format_bytes;
pub mod num;
pub mod slice;
