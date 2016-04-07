#![cfg_attr(all(test, feature="nightly-test"), feature(plugin))]
#![cfg_attr(all(test, feature="nightly-test"), plugin(quickcheck_macros))]
#[cfg(all(test, feature="nightly-test"))] extern crate quickcheck;
#[cfg(all(test))] extern crate num;

extern crate arrayvec;
extern crate itertools;

mod imp;

pub use imp::Buffer;
pub use imp::Hexdump;
pub use imp::hexdump;
pub use imp::hexdump_iter;
pub use imp::sanitize_byte;
