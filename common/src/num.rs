use num_traits::ToPrimitive;
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

pub trait NU8 { }
pub trait NU16 { }
pub trait NI32 { }
pub trait NU32 { }
pub trait NI64 { }
pub trait NU64 { }
pub trait NUsize { }

pub trait U8 { }
pub trait U16 { }
pub trait I32 { }
pub trait U32 { }
pub trait I64 { }
pub trait U64 { }
pub trait Usize { }

pub trait Cast {
    fn u16(self) -> u16 where Self: U16;
    fn i32(self) -> i32 where Self: I32;
    fn u32(self) -> u32 where Self: U32;
    fn i64(self) -> i64 where Self: I64;
    fn u64(self) -> u64 where Self: U64;
    fn usize(self) -> usize where Self: Usize;
    fn try_u8(self) -> Option<u8> where Self: NU8;
    fn try_u16(self) -> Option<u16> where Self: NU16;
    fn try_i32(self) -> Option<i32> where Self: NI32;
    fn try_u32(self) -> Option<u32> where Self: NU32;
    fn try_i64(self) -> Option<i64> where Self: NI64;
    fn try_u64(self) -> Option<u64> where Self: NU64;
    fn try_usize(self) -> Option<usize> where Self: NUsize;
    fn assert_u8(self) -> u8 where Self: NU8;
    fn assert_u16(self) -> u16 where Self: NU16;
    fn assert_i32(self) -> i32 where Self: NI32;
    fn assert_u32(self) -> u32 where Self: NU32;
    fn assert_i64(self) -> i64 where Self: NI64;
    fn assert_u64(self) -> u64 where Self: NU64;
    fn assert_usize(self) -> usize where Self: NUsize;
}

impl Cast for u8 {
    fn u16(self) -> u16 { self.to_u16().unwrap() }
    fn i32(self) -> i32 { self.to_i32().unwrap() }
    fn u32(self) -> u32 { self.to_u32().unwrap() }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn usize(self) -> usize { self.to_usize().unwrap() }
    fn try_u8(self) -> Option<u8> { unreachable!() }
    fn try_u16(self) -> Option<u16> { unreachable!() }
    fn try_i32(self) -> Option<i32> { unreachable!() }
    fn try_u32(self) -> Option<u32> { unreachable!() }
    fn try_i64(self) -> Option<i64> { unreachable!() }
    fn try_u64(self) -> Option<u64> { unreachable!() }
    fn try_usize(self) -> Option<usize> { unreachable!() }
    fn assert_u8(self) -> u8 { unreachable!() }
    fn assert_u16(self) -> u16 { unreachable!() }
    fn assert_i32(self) -> i32 { unreachable!() }
    fn assert_u32(self) -> u32 { unreachable!() }
    fn assert_i64(self) -> i64 { unreachable!() }
    fn assert_u64(self) -> u64 { unreachable!() }
    fn assert_usize(self) -> usize { unreachable!() }
}

impl Cast for u16 {
    fn u16(self) -> u16 { self }
    fn i32(self) -> i32 { self.to_i32().unwrap() }
    fn u32(self) -> u32 { self.to_u32().unwrap() }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn usize(self) -> usize { self.to_usize().unwrap() }
    fn try_u8(self) -> Option<u8> { self.to_u8() }
    fn try_u16(self) -> Option<u16> { unreachable!() }
    fn try_i32(self) -> Option<i32> { unreachable!() }
    fn try_u32(self) -> Option<u32> { unreachable!() }
    fn try_i64(self) -> Option<i64> { unreachable!() }
    fn try_u64(self) -> Option<u64> { unreachable!() }
    fn try_usize(self) -> Option<usize> { unreachable!() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_u16(self) -> u16 { unreachable!() }
    fn assert_i32(self) -> i32 { unreachable!() }
    fn assert_u32(self) -> u32 { unreachable!() }
    fn assert_i64(self) -> i64 { unreachable!() }
    fn assert_u64(self) -> u64 { unreachable!() }
    fn assert_usize(self) -> usize { unreachable!() }
}

impl Cast for i32 {
    fn u16(self) -> u16 { unreachable!() }
    fn i32(self) -> i32 { self }
    fn u32(self) -> u32 { unreachable!() }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn u64(self) -> u64 { unreachable!() }
    fn usize(self) -> usize { unreachable!() }
    fn try_u8(self) -> Option<u8> { self.to_u8() }
    fn try_u16(self) -> Option<u16> { self.to_u16() }
    fn try_i32(self) -> Option<i32> { unreachable!() }
    fn try_u32(self) -> Option<u32> { self.to_u32() }
    fn try_i64(self) -> Option<i64> { unreachable!() }
    fn try_u64(self) -> Option<u64> { self.to_u64() }
    fn try_usize(self) -> Option<usize> { self.to_usize() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_u16(self) -> u16 { unwrap_overflow("u16", self, self.to_u16()) }
    fn assert_i32(self) -> i32 { unreachable!() }
    fn assert_u32(self) -> u32 { unwrap_overflow("u32", self, self.to_u32()) }
    fn assert_i64(self) -> i64 { unreachable!() }
    fn assert_u64(self) -> u64 { unwrap_overflow("i64", self, self.to_u64()) }
    fn assert_usize(self) -> usize { unwrap_overflow("u32", self, self.to_usize()) }
}

impl Cast for u32 {
    fn u16(self) -> u16 { unreachable!() }
    fn i32(self) -> i32 { unreachable!() }
    fn u32(self) -> u32 { self }
    fn i64(self) -> i64 { self.to_i64().unwrap() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn usize(self) -> usize { self.to_usize().unwrap() }
    fn try_u8(self) -> Option<u8> { self.to_u8() }
    fn try_u16(self) -> Option<u16> { self.to_u16() }
    fn try_i32(self) -> Option<i32> { self.to_i32() }
    fn try_u32(self) -> Option<u32> { unreachable!() }
    fn try_i64(self) -> Option<i64> { unreachable!() }
    fn try_u64(self) -> Option<u64> { unreachable!() }
    fn try_usize(self) -> Option<usize> { unreachable!() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_u16(self) -> u16 { unwrap_overflow("u16", self, self.to_u16()) }
    fn assert_i32(self) -> i32 { unwrap_overflow("i32", self, self.to_i32()) }
    fn assert_u32(self) -> u32 { self.u32() }
    fn assert_i64(self) -> i64 { self.i64() }
    fn assert_u64(self) -> u64 { self.u64() }
    fn assert_usize(self) -> usize { self.usize() }
}

impl Cast for i64 {
    fn u16(self) -> u16 { unreachable!() }
    fn i32(self) -> i32 { unreachable!() }
    fn u32(self) -> u32 { unreachable!() }
    fn i64(self) -> i64 { self }
    fn u64(self) -> u64 { unreachable!() }
    fn usize(self) -> usize { unreachable!() }
    fn try_u8(self) -> Option<u8> { self.to_u8() }
    fn try_u16(self) -> Option<u16> { self.to_u16() }
    fn try_i32(self) -> Option<i32> { self.to_i32() }
    fn try_u32(self) -> Option<u32> { self.to_u32() }
    fn try_i64(self) -> Option<i64> { unreachable!() }
    fn try_u64(self) -> Option<u64> { self.to_u64() }
    fn try_usize(self) -> Option<usize> { self.to_usize() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_u16(self) -> u16 { unwrap_overflow("u16", self, self.to_u16()) }
    fn assert_i32(self) -> i32 { unwrap_overflow("i32", self, self.to_i32()) }
    fn assert_u32(self) -> u32 { unwrap_overflow("u32", self, self.to_u32()) }
    fn assert_i64(self) -> i64 { unreachable!() }
    fn assert_u64(self) -> u64 { unwrap_overflow("u64", self, self.to_u64()) }
    fn assert_usize(self) -> usize { unwrap_overflow("usize", self, self.to_usize()) }
}

impl Cast for u64 {
    fn u16(self) -> u16 { unreachable!() }
    fn i32(self) -> i32 { unreachable!() }
    fn u32(self) -> u32 { unreachable!() }
    fn i64(self) -> i64 { unreachable!() }
    fn u64(self) -> u64 { self }
    fn usize(self) -> usize { unreachable!() }
    fn try_u8(self) -> Option<u8> { self.to_u8() }
    fn try_u16(self) -> Option<u16> { self.to_u16() }
    fn try_i32(self) -> Option<i32> { self.to_i32() }
    fn try_u32(self) -> Option<u32> { self.to_u32() }
    fn try_i64(self) -> Option<i64> { self.to_i64() }
    fn try_u64(self) -> Option<u64> { unreachable!() }
    fn try_usize(self) -> Option<usize> { self.to_usize() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_u16(self) -> u16 { unwrap_overflow("u16", self, self.to_u16()) }
    fn assert_i32(self) -> i32 { unwrap_overflow("i32", self, self.to_i32()) }
    fn assert_u32(self) -> u32 { unwrap_overflow("u32", self, self.to_u32()) }
    fn assert_i64(self) -> i64 { unwrap_overflow("i64", self, self.to_i64()) }
    fn assert_u64(self) -> u64 { unreachable!() }
    fn assert_usize(self) -> usize { unwrap_overflow("usize", self, self.to_usize()) }
}

impl Cast for usize {
    fn u16(self) -> u16 { unreachable!() }
    fn i32(self) -> i32 { unreachable!() }
    fn u32(self) -> u32 { unreachable!() }
    fn i64(self) -> i64 { unreachable!() }
    fn u64(self) -> u64 { self.to_u64().unwrap() }
    fn usize(self) -> usize { self }
    fn try_u8(self) -> Option<u8> { self.to_u8() }
    fn try_u16(self) -> Option<u16> { self.to_u16() }
    fn try_i32(self) -> Option<i32> { self.to_i32() }
    fn try_u32(self) -> Option<u32> { self.to_u32() }
    fn try_i64(self) -> Option<i64> { self.to_i64() }
    fn try_u64(self) -> Option<u64> { unreachable!() }
    fn try_usize(self) -> Option<usize> { unreachable!() }
    fn assert_u8(self) -> u8 { unwrap_overflow("u8", self, self.to_u8()) }
    fn assert_u16(self) -> u16 { unwrap_overflow("u16", self, self.to_u16()) }
    fn assert_i32(self) -> i32 { unwrap_overflow("i32", self, self.to_i32()) }
    fn assert_u32(self) -> u32 { unwrap_overflow("u32", self, self.to_u32()) }
    fn assert_i64(self) -> i64 { unwrap_overflow("i64", self, self.to_i64()) }
    fn assert_u64(self) -> u64 { unwrap_overflow("u64", self, self.to_u64()) }
    fn assert_usize(self) -> usize { unreachable!() }
}

impl U8 for u8 { }
impl NU8 for u16 { }
impl NU8 for i32 { }
impl NU8 for u32 { }
impl NU8 for i64 { }
impl NU8 for u64 { }
impl NU8 for usize { }

impl U16 for u8 { }
impl U16 for u16 { }
impl NU16 for i32 { }
impl NU16 for u32 { }
impl NU16 for i64 { }
impl NU16 for u64 { }
impl NU16 for usize { }

impl I32 for u8 { }
impl I32 for u16 { }
impl I32 for i32 { }
impl NI32 for u32 { }
impl NI32 for i64 { }
impl NI32 for u64 { }
impl NI32 for usize { }

impl U32 for u8 { }
impl U32 for u16 { }
impl NU32 for i32 { }
impl U32 for u32 { }
impl NU32 for i64 { }
impl NU32 for u64 { }
impl NU32 for usize { }

impl U64 for u8 { }
impl U64 for u16 { }
impl NU64 for i32 { }
impl U64 for u32 { }
impl NU64 for i64 { }
impl U64 for u64 { }
impl U64 for usize { }

impl I64 for u8 { }
impl I64 for u16 { }
impl I64 for i32 { }
impl I64 for u32 { }
impl I64 for i64 { }
impl NI64 for u64 { }
impl NI64 for usize { }

impl Usize for u8 { }
impl Usize for u16 { }
impl NUsize for i32 { }
impl Usize for u32 { }
impl NUsize for i64 { }
impl NUsize for u64 { }
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

/// Big-endian unsigned 32-bit integer
///
/// Is internally represented as `[u8; 4]`.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BeU32([u8; 4]);

/// Big-endian signed 32-bit integer
///
/// Is internally represented as `[u8; 4]`.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BeI32([u8; 4]);

/// Little-endian signed 32-bit integer
///
/// Is internally represented as `[u8; 4]`.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LeI32([u8; 4]);

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

impl BeI32 {
    pub fn from_i32(value: i32) -> BeI32 {
        BeI32([
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])
    }
    pub fn to_i32(self) -> i32 {
        let BeI32(v) = self;
        (v[0] as i32) << 24
            | (v[1] as i32) << 16
            | (v[2] as i32) << 8
            | v[3] as i32
    }
}

impl LeI32 {
    pub fn from_i32(value: i32) -> LeI32 {
        LeI32([
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8,
        ])
    }
    pub fn to_i32(self) -> i32 {
        let LeI32(v) = self;
        (v[3] as i32) << 24
            | (v[2] as i32) << 16
            | (v[1] as i32) << 8
            | v[0] as i32
    }
}

impl BeU32 {
    pub fn from_u32(value: u32) -> BeU32 {
        BeU32([
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])
    }
    pub fn to_u32(self) -> u32 {
        let BeU32(v) = self;
        (v[0] as u32) << 24
            | (v[1] as u32) << 16
            | (v[2] as u32) << 8
            | v[3] as u32
    }
}

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

unsafe_boilerplate_packed!(BeI32, 4, test_size_bei32, test_align_bei32);
unsafe_boilerplate_packed!(BeU16, 2, test_size_beu16, test_align_beu16);
unsafe_boilerplate_packed!(BeU32, 4, test_size_beu32, test_align_beu32);
unsafe_boilerplate_packed!(LeI32, 4, test_size_lei32, test_align_lei32);
unsafe_boilerplate_packed!(LeU16, 2, test_size_leu16, test_align_leu16);

#[cfg(test)]
mod test {
    use super::BeI32;
    use super::BeU16;
    use super::BeU32;
    use super::LeI32;
    use super::LeU16;

    quickcheck! {
        fn bei32_roundtrip(val: i32) -> bool { BeI32::from_i32(val).to_i32() == val }
        fn beu16_roundtrip(val: u16) -> bool { BeU16::from_u16(val).to_u16() == val }
        fn beu32_roundtrip(val: u32) -> bool { BeU32::from_u32(val).to_u32() == val }
        fn lei32_roundtrip(val: i32) -> bool { LeI32::from_i32(val).to_i32() == val }
        fn leu16_roundtrip(val: u16) -> bool { LeU16::from_u16(val).to_u16() == val }

        fn bei32_unpack(v: (u8, u8, u8, u8)) -> bool {
            let bytes = &[v.0, v.1, v.2, v.3];
            BeI32::from_i32(BeI32::from_bytes(bytes).to_i32()).as_bytes() == bytes
        }
        fn beu16_unpack(v: (u8, u8)) -> bool {
            let bytes = &[v.0, v.1];
            BeU16::from_u16(BeU16::from_bytes(bytes).to_u16()).as_bytes() == bytes
        }
        fn beu32_unpack(v: (u8, u8, u8, u8)) -> bool {
            let bytes = &[v.0, v.1, v.2, v.3];
            BeU32::from_u32(BeU32::from_bytes(bytes).to_u32()).as_bytes() == bytes
        }
        fn lei32_unpack(v: (u8, u8, u8, u8)) -> bool {
            let bytes = &[v.0, v.1, v.2, v.3];
            LeI32::from_i32(LeI32::from_bytes(bytes).to_i32()).as_bytes() == bytes
        }
        fn leu16_unpack(v: (u8, u8)) -> bool {
            let bytes = &[v.0, v.1];
            LeU16::from_u16(LeU16::from_bytes(bytes).to_u16()).as_bytes() == bytes
        }
    }
    #[test]
    fn order_u16() {
        let be = *BeU16::from_u16(0x1234).as_bytes();
        let le = *LeU16::from_u16(0x1234).as_bytes();
        assert_eq!(be, [0x12, 0x34]);
        assert_eq!(le, [0x34, 0x12]);
    }

    #[test]
    fn order_i32() {
        let be = *BeI32::from_i32(0x12345678).as_bytes();
        let le = *LeI32::from_i32(0x12345678).as_bytes();
        assert_eq!(be, [0x12, 0x34, 0x56, 0x78]);
        assert_eq!(le, [0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn order_u32() {
        let be = *BeU32::from_u32(0x12345678).as_bytes();
        assert_eq!(be, [0x12, 0x34, 0x56, 0x78]);
    }
}
