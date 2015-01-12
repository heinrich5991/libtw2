#![allow(unstable)]
#![feature(int_uint)]

pub use slice::relative_size_of;
pub use slice::relative_size_of_mult;
pub use slice::transmute_mut_slice;
pub use slice::transmute_slice;

mod macros;

pub mod num;
pub mod slice;
