extern crate arrayvec;
extern crate common;
extern crate packer;
extern crate warn;

pub use file::Error;
pub use file::Reader;
pub use format::Warning;

pub mod format;

mod bitmagic;
mod file;
mod raw;
