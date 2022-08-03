use base64_dep;

use std::fmt;

/// A struct for printing a byte array as [Base64][wiki].
///
/// [wiki]: https://en.wikipedia.org/wiki/Base64
#[derive(Copy, Clone)]
pub struct B64<'a>(pub &'a [u8]);

impl<'a> fmt::Debug for B64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let B64(bytes) = *self;
        fmt::Display::fmt(&base64_dep::encode(bytes), f)
    }
}

// ---------------------------------------
// Boilerplate trait implementations below
// ---------------------------------------

impl<'a> fmt::Display for B64<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
