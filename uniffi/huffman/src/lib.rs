use std::error;
use std::fmt;

pub use huffman::compress;

#[derive(Debug)]
pub enum DecompressionError {
    InvalidInput
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        huffman::InvalidInput.fmt(f)
    }
}

impl error::Error for DecompressionError {}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, DecompressionError> {
    huffman::decompress(bytes)
        .map_err(|huffman::InvalidInput| DecompressionError::InvalidInput)
}

uniffi::include_scaffolding!("libtw2_huffman");
