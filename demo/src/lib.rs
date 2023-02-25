extern crate arrayvec;
extern crate buffer;
extern crate common;
extern crate huffman;
#[macro_use]
extern crate matches;
extern crate packer;
extern crate uuid;
extern crate warn;

pub use crate::file::Reader;
pub use crate::file::Writer;
pub use crate::format::Chunk;
pub use crate::format::Tick;
pub use crate::format::Warning;
pub use crate::raw::Error;

pub mod format;

mod bitmagic;
mod file;
mod raw;
mod writer;
