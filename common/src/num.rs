use external_num::ToPrimitive;
use std::fmt;

fn overflow<T: fmt::Display>(type_: &str, val: T) -> ! {
    panic!("Overflow casting {} to `{}`", val, type_);
}

fn unwrap_overflow<T: fmt::Display, U>(type_: &str, original: T, val: Option<U>) -> U {
    match val {
        Some(v) => v,
        None => overflow(type_, original),
    }
}

pub trait I32 { }
pub trait U32 { }
pub trait I64 { }
pub trait U64 { }
pub trait Usize { }

pub trait Cast {
    fn i32(self) -> i32 where Self: I32;
    fn u32(self) -> u32 where Self: U32;
    fn i64(self) -> i64 where Self: I64;
    fn u64(self) -> u64 where Self: U64;
    fn usize(self) -> usize where Self: Usize;
    fn assert_u8(self) -> u8;
    fn assert_i32(self) -> i32;
    fn assert_u32(self) -> u32;
    fn assert_i64(self) -> i64;
    fn assert_usize(self) -> usize;
}

impl Cast for u8 {
    fn i32(self) -> i32 { self.to_i32().unwrap() }
    fn u32(self) -> u32 { self.to_u32().unwrap() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn usize(self) -> usize { self.to_usize().unwrap() }
    fn assert_u8(self) -> u8 { self }
    fn assert_i32(self) -> i32 { self.i32() }
    fn assert_u32(self) -> u32 { self.u32() }
    fn assert_i64(self) -> i64 { self.i64() }
    fn assert_usize(self) -> usize { self.usize() }
}

impl Cast for i32 {
    fn i32(self) -> i32 { self }
    fn u32(self) -> u32 { unreachable!() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn usize(self) -> usize { unreachable!() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_i32(self) -> i32 { self.i32() }
    fn assert_u32(self) -> u32 { unwrap_overflow("u32", self, self.to_u32()) }
    fn assert_i64(self) -> i64 { self.i64() }
    fn assert_usize(self) -> usize { unwrap_overflow("u32", self, self.to_usize()) }
}

impl Cast for u32 {
    fn i32(self) -> i32 { unreachable!() }
    fn u32(self) -> u32 { self }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn usize(self) -> usize { self.to_usize().unwrap() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    /*fn assert_u8(self) -> u8 {
        if self > u8::max_value().u32() {
            overflow("u8", self);
        }
        self as u8
    }*/
    fn assert_i32(self) -> i32 { unwrap_overflow("i32", self, self.to_i32()) }
    fn assert_u32(self) -> u32 { self.u32() }
    fn assert_i64(self) -> i64 { self.i64() }
    fn assert_usize(self) -> usize { self.usize() }
}

impl Cast for usize {
    fn i32(self) -> i32 { unreachable!() }
    fn u32(self) -> u32 { unreachable!() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn i64(self) -> i64 { unreachable!() }
    fn usize(self) -> usize { self }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_i32(self) -> i32 { unwrap_overflow("i32", self, self.to_i32()) }
    fn assert_u32(self) -> u32 { unwrap_overflow("u32", self, self.to_u32()) }
    fn assert_i64(self) -> i64 { unwrap_overflow("i64", self, self.to_i64()) }
    fn assert_usize(self) -> usize { self.usize() }
}

impl I32 for u8 { }
impl I32 for i32 { }
impl U32 for u8 { }
impl U32 for u32 { }
impl U64 for u8 { }
impl U64 for i32 { }
impl U64 for u32 { }
impl U64 for usize { }
impl I64 for u8 { }
impl I64 for i32 { }
impl I64 for u32 { }
impl Usize for u8 { }
impl Usize for u32 { }
impl Usize for usize { }

pub trait CastFloat {
    fn round_to_i32(self) -> i32;
    fn trunc_to_i32(self) -> i32;
}

impl CastFloat for f32 {
    fn round_to_i32(self) -> i32 {
        // TODO: Do overflow checking?
        self.round() as i32
    }
    fn trunc_to_i32(self) -> i32 {
        // TODO: Do overflow checking?
        self.trunc() as i32
    }
}


/// Big-endian unsigned 16-bit integer
///
/// Is internally represented as `[u8; 2]`.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BeU16([u8; 2]);

/// Little-endian unsigned 16-bit integer
///
/// Is internally represented as `[u8; 2]`.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LeU16([u8; 2]);

// ======================
// BOILERPLATE CODE BELOW
// ======================

impl BeU16 {
    pub fn from_u16(value: u16) -> BeU16 {
        BeU16([(value >> 8) as u8, value as u8])
    }
    pub fn to_u16(self) -> u16 {
        let BeU16(v) = self;
        (v[0] as u16) << 8 | v[1] as u16
    }
}

impl fmt::Debug for BeU16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_u16().fmt(f)
    }
}

impl LeU16 {
    pub fn from_u16(value: u16) -> LeU16 {
        LeU16([value as u8, (value >> 8) as u8])
    }
    pub fn to_u16(self) -> u16 {
        let LeU16(v) = self;
        (v[1] as u16) << 8 | v[0] as u16
    }
}

impl fmt::Debug for LeU16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_u16().fmt(f)
    }
}

unsafe_boilerplate_packed!(BeU16, 2, test_size_beu16, test_align_beu16);
unsafe_boilerplate_packed!(LeU16, 2, test_size_leu16, test_align_leu16);

#[cfg(all(test, feature="nightly-test"))]
mod test_nigthly {
    use super::BeU16;
    use super::LeU16;

    #[quickcheck]
    fn beu16_roundtrip(val: u16) -> bool { BeU16::from_u16(val).to_u16() == val }
    #[quickcheck]
    fn leu16_roundtrip(val: u16) -> bool { LeU16::from_u16(val).to_u16() == val }

    #[quickcheck]
    fn beu16_unpack((v0, v1): (u8, u8)) -> bool {
        let bytes = &[v0, v1];
        BeU16::from_u16(BeU16::from_bytes(bytes).to_u16()).as_bytes() == bytes
    }
    #[quickcheck]
    fn leu16_unpack((v0, v1): (u8, u8)) -> bool {
        let bytes = &[v0, v1];
        LeU16::from_u16(LeU16::from_bytes(bytes).to_u16()).as_bytes() == bytes
    }
}

#[cfg(test)]
mod test {
    use super::BeU16;
    use super::LeU16;

    #[test]
    fn order() {
        let be = *BeU16::from_u16(0x0120).as_bytes();
        let le = *LeU16::from_u16(0x0120).as_bytes();
        assert_eq!(be[0], 0x01);
        assert_eq!(be[1], 0x20);
        assert_eq!(le[0], 0x20);
        assert_eq!(le[1], 0x01);
    }
}
