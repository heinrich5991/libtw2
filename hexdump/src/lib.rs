#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)] extern crate quickcheck;

extern crate arrayvec;
extern crate itertools;

mod imp;

pub use imp::Buffer;
pub use imp::Hexdump;
pub use imp::hexdump;
pub use imp::hexdump_iter;
