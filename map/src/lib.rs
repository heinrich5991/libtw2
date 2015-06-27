#[macro_use]
extern crate common;
extern crate datafile;
extern crate num;

pub use reader::Reader;
pub use reader::Error;

pub mod format;
pub mod reader;
