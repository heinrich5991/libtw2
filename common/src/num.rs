#[cfg(test)]
use std::mem;

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

const S16: usize = 2;
#[test] fn check_size_beu16() { assert_eq!(mem::size_of::<BeU16>(), S16); }
#[test] fn check_size_leu16() { assert_eq!(mem::size_of::<LeU16>(), S16); }
#[test] fn check_align_beu16() { assert_eq!(mem::align_of::<BeU16>(), 1); }
#[test] fn check_align_leu16() { assert_eq!(mem::align_of::<LeU16>(), 1); }

impl BeU16 {
    pub fn from_u16(value: u16) -> BeU16 {
        BeU16([(value >> 8) as u8, value as u8])
    }
    pub fn to_u16(self) -> u16 {
        let BeU16(v) = self;
        (v[0] as u16) << 8 | v[1] as u16
    }
    pub fn as_bytes(&self) -> &[u8; S16] {
        let BeU16(ref v) = *self;
        v
    }
    pub fn from_bytes(bytes: &[u8; S16]) -> &BeU16 {
        unsafe { &*(bytes as *const _ as *const BeU16) }
    }
    pub fn from_byte_slice(bytes: &[u8]) -> Option<(&BeU16, &[u8])> {
        if bytes.len() < S16 {
            return None;
        }
        let (my_bytes, more_bytes) = bytes.split_at(S16);
        let me = BeU16::from_bytes(unsafe {
            &*(&my_bytes[0] as *const _ as *const [u8; S16])
        });
        Some((me, more_bytes))
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
    pub fn as_bytes(&self) -> &[u8; S16] {
        let LeU16(ref v) = *self;
        v
    }
    pub fn from_bytes(bytes: &[u8; S16]) -> &LeU16 {
        unsafe { &*(bytes as *const _ as *const LeU16) }
    }
    pub fn from_byte_slice(bytes: &[u8]) -> Option<(&LeU16, &[u8])> {
        if bytes.len() < S16 {
            return None;
        }
        let (my_bytes, more_bytes) = bytes.split_at(S16);
        let me = LeU16::from_bytes(unsafe {
            &*(&my_bytes[0] as *const _ as *const [u8; S16])
        });
        Some((me, more_bytes))
    }
}
