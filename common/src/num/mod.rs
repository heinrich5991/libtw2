use std::fmt;

mod cast;

pub use self::cast::Cast;

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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BeU32([u8; 4]);

/// Big-endian signed 32-bit integer
///
/// Is internally represented as `[u8; 4]`.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BeI32([u8; 4]);

/// Little-endian signed 32-bit integer
///
/// Is internally represented as `[u8; 4]`.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LeI32([u8; 4]);

/// Big-endian unsigned 16-bit integer
///
/// Is internally represented as `[u8; 2]`.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BeU16([u8; 2]);

/// Little-endian unsigned 16-bit integer
///
/// Is internally represented as `[u8; 2]`.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LeU16([u8; 2]);

/// Little-endian signed 16-bit integer
///
/// Is internally represented as `[u8; 2]`.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LeI16([u8; 2]);

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

impl LeI16 {
    pub fn from_i16(value: i16) -> LeI16 {
        LeI16([value as u8, (value >> 8) as u8])
    }
    pub fn to_i16(self) -> i16 {
        let LeI16(v) = self;
        (v[1] as i16) << 8 | v[0] as i16
    }
}

impl fmt::Debug for LeI16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_i16().fmt(f)
    }
}

unsafe_boilerplate_packed!(BeI32, 4, test_size_bei32, test_align_bei32);
unsafe_boilerplate_packed!(BeU16, 2, test_size_beu16, test_align_beu16);
unsafe_boilerplate_packed!(BeU32, 4, test_size_beu32, test_align_beu32);
unsafe_boilerplate_packed!(LeI32, 4, test_size_lei32, test_align_lei32);
unsafe_boilerplate_packed!(LeU16, 2, test_size_leu16, test_align_leu16);
unsafe_boilerplate_packed!(LeI16, 2, test_size_lei16, test_align_lei16);

#[cfg(test)]
mod test {
    use super::BeI32;
    use super::BeU16;
    use super::BeU32;
    use super::LeI32;
    use super::LeU16;
    use super::LeI16;

    quickcheck! {
        fn bei32_roundtrip(val: i32) -> bool { BeI32::from_i32(val).to_i32() == val }
        fn beu16_roundtrip(val: u16) -> bool { BeU16::from_u16(val).to_u16() == val }
        fn beu32_roundtrip(val: u32) -> bool { BeU32::from_u32(val).to_u32() == val }
        fn lei32_roundtrip(val: i32) -> bool { LeI32::from_i32(val).to_i32() == val }
        fn leu16_roundtrip(val: u16) -> bool { LeU16::from_u16(val).to_u16() == val }
        fn lei16_roundtrip(val: i16) -> bool { LeI16::from_i16(val).to_i16() == val }

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
        fn lei16_unpack(v: (u8, u8)) -> bool {
            let bytes = &[v.0, v.1];
            LeI16::from_i16(LeI16::from_bytes(bytes).to_i16()).as_bytes() == bytes
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
    fn order_i16() {
        let le = *LeI16::from_i16(0x1234).as_bytes();
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
