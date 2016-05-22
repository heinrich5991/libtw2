use arrayvec::ArrayVec;
use std::ascii;
use std::fmt;
use std::mem;
use std::ops;
use std::str;

pub struct PrettyBytes([u8]);

impl PrettyBytes {
    pub fn new(bytes: &[u8]) -> &PrettyBytes {
        unsafe {
            mem::transmute(bytes)
        }
    }
}

struct PrettyByte {
    string: ArrayVec<[u8; 4]>,
}

impl PrettyByte {
    fn new(byte: u8) -> PrettyByte {
        let mut string = ArrayVec::new();
        if byte == b'\\' || byte == b'\"' {
            string.push(b'\\');
            string.push(byte);
        } else {
            string.extend(ascii::escape_default(byte));
        }
        PrettyByte {
            string: string,
        }
    }
}

impl ops::Deref for PrettyByte {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(&self.string)
        }
    }
}

impl fmt::Debug for PrettyBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str("b\""));
        for &byte in &self.0 {
            try!(f.write_str(&PrettyByte::new(byte)));
        }
        try!(f.write_str("\""));
        Ok(())
    }
}
