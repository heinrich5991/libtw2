#[macro_export]
macro_rules! unwrap_or_return {
    ($e:expr, $r:expr) => (match $e { Some(e) => e, None => return $r, })
}

#[macro_export]
macro_rules! unsafe_boilerplate_packed {
    ($t:ty, $size:expr, $ts:ident, $ta:ident) => {
        #[test] fn $ts() { assert_eq!(::std::mem::size_of::<$t>(), $size); }
        #[test] fn $ta() { assert_eq!(::std::mem::align_of::<$t>(), 1); }
        impl $t {
            pub fn as_bytes(&self) -> &[u8; $size] {
                unsafe { &*(self as *const _ as *const [u8; $size]) }
            }
            pub fn from_bytes(bytes: &[u8; $size]) -> &$t {
                unsafe { &*(bytes as *const _ as *const $t) }
            }
            pub fn from_byte_slice(bytes: &[u8]) -> Option<(&$t, &[u8])> {
                if bytes.len() < $size {
                    return None;
                }
                let (struct_bytes, rest) = bytes.split_at($size);
                let struct_ = Self::from_bytes(unsafe {
                    &*(&struct_bytes[0] as *const _ as *const [u8; $size])
                });
                Some((struct_, rest))
            }
        }
    }
}
