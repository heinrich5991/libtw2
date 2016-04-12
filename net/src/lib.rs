#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(quickcheck_macros))]
#[cfg(test)] extern crate quickcheck;

extern crate arrayvec;
extern crate buffer;
#[macro_use] extern crate common;
extern crate huffman;
extern crate num;
extern crate void;

pub mod connection;
pub mod protocol;
