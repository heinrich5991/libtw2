#![cfg_attr(all(feature = "nightly-test", test), feature(plugin))]
#![cfg_attr(all(feature = "nightly-test", test), plugin(quickcheck_macros))]
#[cfg(all(feature = "nightly-test", test))] extern crate quickcheck;
#[cfg(test)] extern crate hexdump;
#[cfg(test)] extern crate itertools;

extern crate arrayvec;
#[macro_use] extern crate assert_matches;
extern crate buffer;
#[macro_use] extern crate common;
extern crate huffman;
extern crate linear_map;
#[macro_use] extern crate matches;
extern crate num;
extern crate void;
extern crate warn;

pub mod connection;
pub mod net;
pub mod protocol;

pub use connection::Connection;
pub use net::Net;
