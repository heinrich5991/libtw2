#[cfg(test)]
#[macro_use]
extern crate quickcheck;

extern crate arrayvec;
extern crate buffer;
extern crate common;
extern crate packer;
extern crate warn;

#[cfg(test)] extern crate hexdump;
#[cfg(test)] mod test;

pub mod enums;
pub mod error;
pub mod msg;
pub mod snap_obj;

pub use snap_obj::SnapObj;

pub const VERSION: &'static [u8] = b"0.6 626fce9a778df4d4";

mod debug;
