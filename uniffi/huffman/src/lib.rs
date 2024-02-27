use std::error;
use std::fmt;

pub use libtw2_huffman::compress;

#[derive(Debug)]
pub enum DecompressionError {
    InvalidInput,
}

impl fmt::Display for DecompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        libtw2_huffman::InvalidInput.fmt(f)
    }
}

impl error::Error for DecompressionError {}

pub fn decompress(bytes: &[u8]) -> Result<Vec<u8>, DecompressionError> {
    libtw2_huffman::decompress(bytes)
        .map_err(|libtw2_huffman::InvalidInput| DecompressionError::InvalidInput)
}

uniffi::include_scaffolding!("libtw2_huffman");
