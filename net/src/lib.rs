#![cfg_attr(all(feature = "nightly-test", test), feature(plugin))]
#![cfg_attr(all(feature = "nightly-test", test), plugin(quickcheck_macros))]
#[cfg(all(feature = "nightly-test", test))] extern crate quickcheck;

extern crate arrayvec;
extern crate buffer;
#[macro_use] extern crate common;
extern crate huffman;
extern crate linear_map;
extern crate num;
extern crate void;

pub mod connection;
pub mod net;
pub mod protocol;
