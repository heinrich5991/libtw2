#[macro_use]
extern crate log;

extern crate common;
extern crate itertools;
extern crate num;
extern crate zlib_minimal as zlib;

pub use file::DatafileReaderFile;

mod bitmagic;
pub mod raw;
mod file;
pub mod format;
