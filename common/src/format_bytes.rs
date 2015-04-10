use std::fmt;
use std::str;

use slice::ref_slice;

pub struct Bytes<'a>(pub &'a [u8]);
pub struct Byte(pub u8);
pub struct Escape(pub u8);

fn hex_digit_to_byte(digit: u32) -> u8 {
    match digit {
        0x0 => b'0',
        0x1 => b'1',
        0x2 => b'2',
        0x3 => b'3',
        0x4 => b'4',
        0x5 => b'5',
        0x6 => b'6',
        0x7 => b'7',
        0x8 => b'8',
        0x9 => b'9',
        0xa => b'a',
        0xb => b'b',
        0xc => b'c',
        0xd => b'd',
        0xe => b'e',
        0xf => b'f',
        _ => panic!("digit must be less than 16"),
    }
}

fn generic_escape(byte: u8, buffer: &mut [u8; 4]) -> &str {
    buffer[0] = b'\\';
    buffer[1] = b'x';
    buffer[2] = hex_digit_to_byte((byte / 16) as u32);
    buffer[3] = hex_digit_to_byte((byte % 16) as u32);
    unsafe { str::from_utf8_unchecked(buffer) }
}

impl fmt::Debug for Escape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Taken from http://en.wikipedia.org/wiki/index.html?title=Escape_sequences_in_C&oldid=648380493.
        let Escape(byte) = *self;

        let mut buffer = [0u8; 4];
        f.write_str(match byte {
            b'\0' => "\\0",
            0x07  => "\\a",
            0x08  => "\\b",
            0x0c  => "\\f",
            b'\n' => "\\n",
            b'\r' => "\\r",
            b'\t' => "\\t",
            0x0b  => "\\v",
            b'\\' => "\\\\",
            b'\"' => "\\\"",
            b if b < 32 || b >= 128 => generic_escape(b, &mut buffer),
            // Safe because the byte is less than 128.
            _ => unsafe { str::from_utf8_unchecked(ref_slice(&byte)) },
        })
    }
}

impl<'a> fmt::Debug for Bytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Bytes(bytes) = *self;
        try!(f.write_str("\""));
        for &b in bytes {
            try!(Escape(b).fmt(f));
        }
        try!(f.write_str("\""));
        Ok(())
    }
}
