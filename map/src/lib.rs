#[macro_use]
extern crate common;
extern crate datafile;
extern crate ndarray;

pub use reader::Error;
pub use reader::Reader;

#[rustfmt::skip]
pub mod format;
pub mod reader;
