use rustc_serialize::base64;
use rustc_serialize::base64::ToBase64;

use std::fmt;

use serverbrowse::protocol::PString64;
use serverbrowse::protocol::NzU8SliceExt;

/// A struct for printing a byte array as [Base64][wiki].
///
/// [wiki]: https://en.wikipedia.org/wiki/Base64
#[derive(Copy, Clone)]
pub struct B64<'a>(&'a [u8]);

/// Returns a `B64` for a `PString64`.
pub fn b64(string: &PString64) -> B64 {
    B64(string.as_slice().as_bytes())
}

impl<'a> fmt::String for B64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let B64(bytes) = *self;
        const CONFIG: base64::Config = base64::Config {
            char_set: base64::CharacterSet::Standard,
            newline: base64::Newline::LF,
            pad: true,
            line_length: None,
        };
        //write!(f, "{}", String::from_utf8_lossy(bytes))
        write!(f, "{}", bytes.to_base64(CONFIG))
    }
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl<'a> fmt::Show for B64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
	fmt::String::fmt(self, f)
    }
}
