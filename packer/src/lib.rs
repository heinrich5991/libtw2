#![cfg_attr(all(feature = "nightly-test", test), feature(plugin))]
#![cfg_attr(all(feature = "nightly-test", test), plugin(quickcheck_macros))]
#[cfg(all(feature = "nightly-test", test))] extern crate quickcheck;

extern crate arrayvec;
#[macro_use] extern crate common;
extern crate buffer;
extern crate num;
extern crate warn;

use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::BufferRef;
use buffer::CapacityError;
use buffer::with_buffer;
use num::ToPrimitive;
use std::slice;
use warn::Warn;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Warning {
    OverlongIntEncoding,
    IntPadding,
    ExcessData,
}

#[derive(Clone, Copy, Debug)] pub struct ControlCharacters;
#[derive(Clone, Copy, Debug)] pub struct IntOutOfRange;
#[derive(Clone, Copy, Debug)] pub struct UnexpectedEnd;

// Format: ESDD_DDDD EDDD_DDDD EDDD_DDDD EDDD_DDDD PPPP_DDDD
// E - Extend
// S - Sign
// D - Digit (little-endian)
// P - Padding
//
// Padding must be zeroed. The extend bit specifies whether another byte
// follows.
fn read_int<W>(warn: &mut W, iter: &mut slice::Iter<u8>) -> Result<i32, UnexpectedEnd>
    where W: Warn<Warning>
{
    let mut result = 0;
    let mut len = 1;

    let mut src = *unwrap_or_return!(iter.next(), Err(UnexpectedEnd));
    let sign = ((src >> 6) & 1) as i32;

    result |= (src & 0b0011_1111) as i32;

    for i in 0..4 {
        if src & 0b1000_0000 == 0 {
            break;
        }
        src = *unwrap_or_return!(iter.next(), Err(UnexpectedEnd));
        len += 1;
        if i == 3 && src & 0b1111_0000 != 0 {
            warn.warn(Warning::IntPadding);
        }
        result |= ((src & 0b0111_1111) as i32) << (6 + 7 * i);
    }

    if len > 1 && src == 0b0000_0000 {
        warn.warn(Warning::OverlongIntEncoding);
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

fn read_string<'a>(iter: &mut slice::Iter<'a, u8>) -> Result<&'a [u8], UnexpectedEnd> {
    let slice = iter.as_slice();
    // `by_ref` is needed as the iterator is silently copied otherwise.
    for (i, b) in iter.by_ref().cloned().enumerate() {
        if b == 0 {
            return Ok(&slice[..i]);
        }
    }
    Err(UnexpectedEnd)
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
    fn use_up(&mut self) {
        // Advance the iterator to the end.
        self.iter.by_ref().count();
    }
    fn error<T>(&mut self) -> Result<T, UnexpectedEnd> {
        self.use_up();
        Err(UnexpectedEnd)
    }
    pub fn read_string(&mut self) -> Result<&'a [u8], UnexpectedEnd> {
        read_string(&mut self.iter)
    }
    pub fn read_int<W: Warn<Warning>>(&mut self, warn: &mut W) -> Result<i32, UnexpectedEnd> {
        read_int(warn, &mut self.iter)
    }
    pub fn read_data<W: Warn<Warning>>(&mut self, warn: &mut W)
        -> Result<&'a [u8], UnexpectedEnd>
    {
        let len = match self.read_int(warn).map(|l| l.to_usize()) {
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
    pub fn read_rest(&mut self) -> Result<&'a [u8], UnexpectedEnd> {
        let result = Ok(self.iter.as_slice());
        self.use_up();
        result
    }
    pub fn finish<W: Warn<Warning>>(&mut self, warn: &mut W) {
        if !self.as_slice().is_empty() {
            warn.warn(Warning::ExcessData);
        }
        self.use_up();
    }
    pub fn as_slice(&self) -> &'a [u8] {
        self.iter.as_slice()
    }
}

pub fn in_range(v: i32, min: i32, max: i32) -> Result<i32, IntOutOfRange> {
    if min <= v && v <= max {
        Ok(v)
    } else {
        Err(IntOutOfRange)
    }
}

pub fn to_bool(v: i32) -> Result<bool, IntOutOfRange> {
    Ok(try!(in_range(v, 0, 1)) != 0)
}

pub fn sanitize<'a, W: Warn<Warning>>(warn: &mut W, v: &'a [u8])
    -> Result<&'a [u8], ControlCharacters>
{
    if v.iter().any(|&b| b < b' ') {
        return Err(ControlCharacters);
    }
    let _ = warn;
    // TODO: Implement whitespace skipping.
    Ok(v)
}

pub fn positive(v: i32) -> Result<i32, IntOutOfRange> {
    if v >= 0 {
        Ok(v)
    } else {
        Err(IntOutOfRange)
    }
}

#[cfg(test)]
mod test {
    use arrayvec::ArrayVec;
    use std::i32;
    use super::Unpacker;
    use super::Warning::*;
    use super::Warning;
    use super::with_packer;
    use warn::Panic;

    fn assert_int_err(bytes: &[u8]) {
        let mut unpacker = Unpacker::new(bytes);
        unpacker.read_int(&mut Panic).unwrap_err();
    }

    fn assert_int_warnings(bytes: &[u8], int: i32, warnings: &[Warning]) {
        let mut vec = vec![];
        let mut unpacker = Unpacker::new(bytes);
        assert_eq!(unpacker.read_int(&mut vec).unwrap(), int);
        assert!(unpacker.as_slice().is_empty());
        assert_eq!(vec, warnings);

        let mut buf: ArrayVec<[u8; 5]> = ArrayVec::new();
        let written = with_packer(&mut buf, |mut p| {
            p.write_int(int).unwrap();
            p.written()
        });
        if warnings.is_empty() {
            assert_eq!(written, bytes);
        } else {
            assert!(written != bytes);
        }
    }

    fn assert_int_warn(bytes: &[u8], int: i32, warning: Warning) {
        assert_int_warnings(bytes, int, &[warning]);
    }

    fn assert_int(bytes: &[u8], int: i32) {
        assert_int_warnings(bytes, int, &[]);
    }

    fn assert_str(bytes: &[u8], string: &[u8], remaining: &[u8]) {
        let mut unpacker = Unpacker::new(bytes);
        assert_eq!(unpacker.read_string().unwrap(), string);
        assert_eq!(unpacker.as_slice(), remaining);

        let mut buf = Vec::with_capacity(4096);
        let written = with_packer(&mut buf, |mut p| {
            p.write_string(string).unwrap();
            p.write_rest(remaining).unwrap();
            p.written()
        });
        assert_eq!(written, bytes);
    }

    fn assert_str_err(bytes: &[u8]) {
        let mut unpacker = Unpacker::new(bytes);
        unpacker.read_string().unwrap_err();
    }

    #[test] fn int_0() { assert_int(b"\x00", 0) }
    #[test] fn int_1() { assert_int(b"\x01", 1) }
    #[test] fn int_63() { assert_int(b"\x3f", 63) }
    #[test] fn int_m1() { assert_int(b"\x40", -1) }
    #[test] fn int_64() { assert_int(b"\x80\x01", 64) }
    #[test] fn int_m65() { assert_int(b"\xc0\x01", -65) }
    #[test] fn int_m64() { assert_int(b"\x7f", -64) }
    #[test] fn int_min() { assert_int(b"\xff\xff\xff\xff\x0f", i32::min_value()) }
    #[test] fn int_max() { assert_int(b"\xbf\xff\xff\xff\x0f", i32::max_value()) }
    #[test] fn int_quirk1() { assert_int_warn(b"\xff\xff\xff\xff\xff", 0, IntPadding) }
    #[test] fn int_quirk2() { assert_int_warn(b"\xbf\xff\xff\xff\xff", -1, IntPadding) }
    #[test] fn int_empty() { assert_int_err(b"") }
    #[test] fn int_extend_empty() { assert_int_err(b"\x80") }
    #[test] fn int_overlong1() { assert_int_warn(b"\x80\x00", 0, OverlongIntEncoding) }
    #[test] fn int_overlong2() { assert_int_warn(b"\xc0\x00", -1, OverlongIntEncoding) }

    #[test] fn str_empty() { assert_str(b"\0", b"", b"") }
    #[test] fn str_none() { assert_str_err(b"") }
    #[test] fn str_no_nul() { assert_str_err(b"abc") }
    #[test] fn str_rest1() { assert_str(b"abc\0def", b"abc", b"def") }
    #[test] fn str_rest2() { assert_str(b"abc\0", b"abc", b"") }
    #[test] fn str_rest3() { assert_str(b"abc\0\0", b"abc", b"\0") }
    #[test] fn str_rest4() { assert_str(b"\0\0", b"", b"\0") }

    #[test]
    fn excess_data() {
        let mut warnings = vec![];
        let mut unpacker = Unpacker::new(b"\x00");
        unpacker.finish(&mut warnings);
        assert_eq!(warnings, [ExcessData]);
    }
}

#[cfg(all(feature = "nightly-test", test))]
mod test_nightly {
    use arrayvec::ArrayVec;
    use super::Unpacker;
    use super::with_packer;
    use warn::Ignore;
    use warn::Panic;

    #[quickcheck]
    fn int_roundtrip(int: i32) -> bool {
        let mut buf: ArrayVec<[u8; 5]> = ArrayVec::new();
        let mut unpacker = Unpacker::new(with_packer(&mut buf, |mut p| {
            p.write_int(int).unwrap();
            p.written()
        }));
        let read_int = unpacker.read_int(&mut Panic).unwrap();
        int == read_int && unpacker.as_slice().is_empty()
    }

    #[quickcheck]
    fn int_no_panic(data: Vec<u8>) -> bool {
        let mut unpacker = Unpacker::new(&data);
        let _ = unpacker.read_int(&mut Ignore);
        true
    }

    #[quickcheck]
    fn string_no_panic(data: Vec<u8>) -> bool {
        let mut unpacker = Unpacker::new(&data);
        let _ = unpacker.read_string();
        true
    }
}
