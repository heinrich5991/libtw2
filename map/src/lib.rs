#[macro_use]
extern crate common;
extern crate datafile;
extern crate ndarray;

pub use reader::Reader;
pub use reader::Error;

#[rustfmt::skip]
pub mod format;
pub mod reader;
