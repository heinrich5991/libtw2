#![cfg_attr(all(feature = "nightly-test", test), feature(plugin))]
#![cfg_attr(all(feature = "nightly-test", test), plugin(quickcheck_macros))]
#[cfg(all(feature = "nightly-test", test))] extern crate quickcheck;

extern crate arrayvec;
extern crate buffer;
#[macro_use] extern crate common;
extern crate num;
extern crate warn;

#[cfg(test)] extern crate hexdump;
#[cfg(test)] mod test;

pub mod error;
pub mod msg;
pub mod packer;

// TODO: Remove me!
pub mod bytes;
