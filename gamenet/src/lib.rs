extern crate arrayvec;
extern crate buffer;
#[macro_use] extern crate common;
extern crate num;

#[cfg(test)] extern crate hexdump;
#[cfg(test)] mod test;

pub mod error;
pub mod msg;
pub mod packer;
