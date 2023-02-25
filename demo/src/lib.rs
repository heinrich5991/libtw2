extern crate arrayvec;
extern crate binrw;
extern crate buffer;
extern crate common;
extern crate huffman;
#[macro_use]
extern crate matches;
extern crate packer;
extern crate thiserror;
extern crate warn;

mod format;
mod reader;
mod writer;

pub use format::DemoKind;
pub use format::RawChunk;
pub use format::Warning;
pub use reader::ReadError;
pub use reader::Reader;
pub use writer::WriteError;
pub use writer::Writer;
