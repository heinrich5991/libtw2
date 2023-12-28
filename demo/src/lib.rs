extern crate arrayvec;
extern crate binrw;
extern crate buffer;
extern crate common;
extern crate gamenet_common;
extern crate gamenet_ddnet;
extern crate huffman;
#[macro_use]
extern crate matches;
extern crate packer;
extern crate snapshot;
extern crate thiserror;
extern crate warn;

pub mod ddnet;
mod format;
mod reader;
mod writer;

pub use format::DemoKind;
pub use format::RawChunk;
pub use format::Version;
pub use format::Warning;
pub use reader::ReadError;
pub use reader::Reader;
pub use writer::WriteError;
pub use writer::Writer;
