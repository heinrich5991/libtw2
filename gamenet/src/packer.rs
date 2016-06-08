use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::CapacityError;
use buffer::with_buffer;
use buffer;
use error::Error;
use num::ToPrimitive;
use std::slice;

// Format: ESDDDDDD EDDDDDDD EDDDDDDD EDDDDDDD ...
// E - Extend
// S - Sign
// D - Digit (little-endian)
fn read_int(iter: &mut slice::Iter<u8>) -> Result<i32, Error> {
    let mut result = 0;

    let mut src = *unwrap_or_return!(iter.next(), Err(Error::new()));
    let sign = ((src >> 6) & 1) as i32;

    result |= (src & 0b0011_1111) as i32;

    for i in 0..4 {
        // WARN
        if src & 0b1000_0000 == 0 {
            break;
        }
        src = *unwrap_or_return!(iter.next(), Err(Error::new()));
        result |= ((src & 0b0111_1111) as i32) << (6 + 7 * i);
    }

    result ^= -sign;

    Ok(result)
}

fn to_bit(b: bool, bit: u32) -> u8 {
    assert!(bit < 8);
    if b { 1 << bit } else { 0 }
}

fn write_int<E, F: FnMut(&[u8]) -> Result<(), E>>(int: i32, f: F) -> Result<(), E> {
    let mut f = f;
    let mut buf: ArrayVec<[u8; 5]> = ArrayVec::new();
    let sign = if int < 0 { 1 } else { 0 };
    let mut int = (int ^ -sign) as u32;
    let next = (int & 0b0011_1111) as u8;
    int >>= 6;
    assert!(buf.push(to_bit(int != 0, 7) | to_bit(sign != 0, 6) | next).is_none());
    while int != 0 {
        let next = (int & 0b0111_1111) as u8;
        int >>= 7;
        assert!(buf.push(to_bit(int != 0, 7) | next).is_none());
    }
    f(&buf)
}

fn read_string<'a>(iter: &mut slice::Iter<'a, u8>) -> Result<&'a [u8], Error> {
    let slice = iter.as_slice();
    // `by_ref` is needed as the iterator is silently copied otherwise.
    for (i, b) in iter.by_ref().cloned().enumerate() {
        if b == 0 {
            return Ok(&slice[..i]);
        }
    }
    Err(Error::new())
}

fn write_string<E, F: FnMut(&[u8]) -> Result<(), E>>(string: &[u8], f: F) -> Result<(), E> {
    let mut f = f;
    assert!(string.iter().all(|&b| b != 0));
    try!(f(string));
    try!(f(&[0]));
    Ok(())
}

pub struct Packer<'d, 's> {
    buf: BufferRef<'d, 's>,
}

impl<'r, 'd, 's> Buffer<'d> for &'r mut Packer<'d, 's> {
    type Intermediate = buffer::BufferRefBuffer<'r, 'd, 's>;
    fn to_to_buffer_ref(self) -> Self::Intermediate {
        (&mut self.buf).to_to_buffer_ref()
    }
}

impl<'d, 's> Packer<'d, 's> {
    fn new(buf: BufferRef<'d, 's>) -> Packer<'d, 's> {
        Packer {
            buf: buf,
        }
    }
    pub fn write_string(&mut self, string: &[u8]) -> Result<(), CapacityError> {
        write_string(string, |b| self.buf.write(b))
    }
    pub fn write_int(&mut self, int: i32) -> Result<(), CapacityError> {
        write_int(int, |b| self.buf.write(b))
    }
    pub fn write_data(&mut self, data: &[u8]) -> Result<(), CapacityError> {
        try!(self.write_int(try!(data.len().to_i32().ok_or(CapacityError))));
        try!(self.buf.write(data));
        Ok(())
    }
    pub fn write_rest(&mut self, data: &[u8]) -> Result<(), CapacityError> {
        self.buf.write(data)
    }
    pub fn written(self) -> &'d [u8] {
        self.buf.initialized()
    }
}

pub fn with_packer<'a, B: Buffer<'a>, F, R>(buf: B, f: F) -> R
    where F: for<'b> FnOnce(Packer<'a, 'b>) -> R
{
    with_buffer(buf, |b| f(Packer::new(b)))
}

pub struct Unpacker<'a> {
    iter: slice::Iter<'a, u8>,
}

impl<'a> Unpacker<'a> {
    pub fn new(data: &[u8]) -> Unpacker {
        Unpacker {
            iter: data.iter(),
        }
    }
    pub fn error<T>(&mut self) -> Result<T, Error> {
        // Advance the iterator to the end.
        self.iter.by_ref().count();
        Err(Error::new())
    }
    pub fn read_string(&mut self) -> Result<&'a [u8], Error> {
        read_string(&mut self.iter)
    }
    pub fn read_int(&mut self) -> Result<i32, Error> {
        read_int(&mut self.iter)
    }
    pub fn read_data(&mut self) -> Result<&'a [u8], Error> {
        let len = match self.read_int().map(|l| l.to_usize()) {
            Ok(Some(l)) => l,
            _ => return self.error(),
        };
        let slice = self.iter.as_slice();
        if len > slice.len() {
            return self.error();
        }
        let (data, remaining) = slice.split_at(len);
        self.iter = remaining.iter();
        Ok(data)
    }
    pub fn read_rest(&mut self) -> Result<&'a [u8], Error> {
        let result = Ok(self.iter.as_slice());
        // Use up the iterator.
        self.iter.by_ref().count();
        result
    }
    pub fn as_slice(&self) -> &'a [u8] {
        self.iter.as_slice()
    }
}

#[cfg(test)]
mod test {
    use std::i32;
    use super::read_int;
    use super::read_string;

    fn assert_int_err(bytes: &[u8]) {
        let mut iter = bytes.iter();
        read_int(&mut iter).unwrap_err();
    }

    fn assert_int(bytes: &[u8], int: i32) {
        let mut iter = bytes.iter();
        assert_eq!(read_int(&mut iter).unwrap(), int);
        assert!(iter.as_slice().is_empty());
    }

    fn assert_str(bytes: &[u8], string: &[u8], remaining: &[u8]) {
        let mut iter = bytes.iter();
        assert_eq!(read_string(&mut iter).unwrap(), string);
        assert_eq!(iter.as_slice(), remaining);
    }

    fn assert_str_err(bytes: &[u8]) {
        let mut iter = bytes.iter();
        read_string(&mut iter).unwrap_err();
    }

    #[test] fn int_0() { assert_int(b"\x00", 0) }
    #[test] fn int_1() { assert_int(b"\x01", 1) }
    #[test] fn int_63() { assert_int(b"\x3f", 63) }
    #[test] fn int_m1() { assert_int(b"\x40", -1) }
    #[test] fn int_m64() { assert_int(b"\x7f", -64) }
    #[test] fn int_min() { assert_int(b"\xff\xff\xff\xff\x0f", i32::min_value()) }
    #[test] fn int_max() { assert_int(b"\xbf\xff\xff\xff\x0f", i32::max_value()) }
    #[test] fn int_quirk1() { assert_int(b"\xff\xff\xff\xff\xff", 0) }
    #[test] fn int_quirk2() { assert_int(b"\xbf\xff\xff\xff\xff", -1) }
    #[test] fn int_empty() { assert_int_err(b"") }
    #[test] fn int_extend_empty() { assert_int_err(b"\x80") }

    #[test] fn str_empty() { assert_str(b"\0", b"", b"") }
    #[test] fn str_none() { assert_str_err(b"") }
    #[test] fn str_no_nul() { assert_str_err(b"abc") }
    #[test] fn str_rest1() { assert_str(b"abc\0def", b"abc", b"def") }
    #[test] fn str_rest2() { assert_str(b"abc\0", b"abc", b"") }
    #[test] fn str_rest3() { assert_str(b"abc\0\0", b"abc", b"\0") }
    #[test] fn str_rest4() { assert_str(b"\0\0", b"", b"\0") }
}
