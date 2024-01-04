extern crate arrayvec;
extern crate buffer;
extern crate common;
extern crate huffman;
#[macro_use]
extern crate matches;
extern crate packer;
extern crate uuid;
extern crate warn;

pub use file::Error;
pub use file::Reader;
pub use file::Writer;
pub use format::Chunk;
pub use format::Tick;
pub use format::Warning;

pub mod format;

mod bitmagic;
mod file;
mod raw;
mod writer;
