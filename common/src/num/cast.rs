use num_traits::ToPrimitive;
use std::fmt;
use unreachable::unreachable;

trait TypeName {
    fn type_name() -> &'static str;
}

#[cold]
#[inline(never)]
fn overflow<T: fmt::Display, U: TypeName>(original: T) -> ! {
    panic!("Overflow casting {} to `{}`", original, U::type_name());
}

#[inline]
fn unwrap_overflow<T: fmt::Display, U: TypeName>(original: T, val: Option<U>) -> U {
    match val {
        Some(v) => v,
        None => overflow::<T, U>(original),
    }
}

pub trait Cast {
    fn i8(self) -> i8 where Self: I8;
    fn u8(self) -> u8 where Self: U8;
    fn i16(self) -> i16 where Self: I16;
    fn u16(self) -> u16 where Self: U16;
    fn i32(self) -> i32 where Self: I32;
    fn u32(self) -> u32 where Self: U32;
    fn i64(self) -> i64 where Self: I64;
    fn u64(self) -> u64 where Self: U64;
    fn isize(self) -> isize where Self: Isize;
    fn usize(self) -> usize where Self: Usize;
    fn try_i8(self) -> Option<i8> where Self: NI8;
    fn try_u8(self) -> Option<u8> where Self: NU8;
    fn try_i16(self) -> Option<i16> where Self: NI16;
    fn try_u16(self) -> Option<u16> where Self: NU16;
    fn try_i32(self) -> Option<i32> where Self: NI32;
    fn try_u32(self) -> Option<u32> where Self: NU32;
    fn try_i64(self) -> Option<i64> where Self: NI64;
    fn try_u64(self) -> Option<u64> where Self: NU64;
    fn try_isize(self) -> Option<isize> where Self: NIsize;
    fn try_usize(self) -> Option<usize> where Self: NUsize;
    fn assert_i8(self) -> i8 where Self: NI8;
    fn assert_u8(self) -> u8 where Self: NU8;
    fn assert_i16(self) -> i16 where Self: NI16;
    fn assert_u16(self) -> u16 where Self: NU16;
    fn assert_i32(self) -> i32 where Self: NI32;
    fn assert_u32(self) -> u32 where Self: NU32;
    fn assert_i64(self) -> i64 where Self: NI64;
    fn assert_u64(self) -> u64 where Self: NU64;
    fn assert_isize(self) -> isize where Self: NIsize;
    fn assert_usize(self) -> usize where Self: NUsize;
}

pub trait I8 { }
pub trait U8 { }
pub trait I16 { }
pub trait U16 { }
pub trait I32 { }
pub trait U32 { }
pub trait I64 { }
pub trait U64 { }
pub trait Isize { }
pub trait Usize { }

pub trait NI8 { }
pub trait NU8 { }
pub trait NI16 { }
pub trait NU16 { }
pub trait NI32 { }
pub trait NU32 { }
pub trait NI64 { }
pub trait NU64 { }
pub trait NIsize { }
pub trait NUsize { }

impl Cast for i8 {
    #[inline] fn i8(self) -> i8 { self.to_i8().unwrap() }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { self.to_i16().unwrap() }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { self.to_i32().unwrap() }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn isize(self) -> isize { self.to_isize().unwrap() }
    #[inline] fn usize(self) -> usize { unsafe { unreachable() } }
    #[inline] fn try_i8(self) -> Option<i8> { unsafe { unreachable() } }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { unsafe { unreachable() } }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { unsafe { unreachable() } }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { self.to_u64() }
    #[inline] fn try_isize(self) -> Option<isize> { unsafe { unreachable() } }
    #[inline] fn try_usize(self) -> Option<usize> { self.to_usize() }
    #[inline] fn assert_i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unwrap_overflow(self, self.to_u64()) }
    #[inline] fn assert_isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn assert_usize(self) -> usize { unwrap_overflow(self, self.to_usize()) }
}

impl Cast for u8 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { self.to_u8().unwrap() }
    #[inline] fn i16(self) -> i16 { self.to_i16().unwrap() }
    #[inline] fn u16(self) -> u16 { self.to_u16().unwrap() }
    #[inline] fn i32(self) -> i32 { self.to_i32().unwrap() }
    #[inline] fn u32(self) -> u32 { self.to_u32().unwrap() }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { self.to_u64().unwrap() }
    #[inline] fn isize(self) -> isize { self.to_isize().unwrap() }
    #[inline] fn usize(self) -> usize { self.to_usize().unwrap() }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { unsafe { unreachable() } }
    #[inline] fn try_i16(self) -> Option<i16> { unsafe { unreachable() } }
    #[inline] fn try_u16(self) -> Option<u16> { unsafe { unreachable() } }
    #[inline] fn try_i32(self) -> Option<i32> { unsafe { unreachable() } }
    #[inline] fn try_u32(self) -> Option<u32> { unsafe { unreachable() } }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { unsafe { unreachable() } }
    #[inline] fn try_isize(self) -> Option<isize> { unsafe { unreachable() } }
    #[inline] fn try_usize(self) -> Option<usize> { unsafe { unreachable() } }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn assert_i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn assert_u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn assert_i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn assert_u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn assert_isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn assert_usize(self) -> usize { unsafe { unreachable() } }
}

impl Cast for i16 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { self.to_i16().unwrap() }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { self.to_i32().unwrap() }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn isize(self) -> isize { self.to_isize().unwrap() }
    #[inline] fn usize(self) -> usize { unsafe { unreachable() } }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { unsafe { unreachable() } }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { unsafe { unreachable() } }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { self.to_u64() }
    #[inline] fn try_isize(self) -> Option<isize> { unsafe { unreachable() } }
    #[inline] fn try_usize(self) -> Option<usize> { self.to_usize() }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unwrap_overflow(self, self.to_u64()) }
    #[inline] fn assert_isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn assert_usize(self) -> usize { unwrap_overflow(self, self.to_usize()) }
}

impl Cast for u16 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { self.to_u16().unwrap() }
    #[inline] fn i32(self) -> i32 { self.to_i32().unwrap() }
    #[inline] fn u32(self) -> u32 { self.to_u32().unwrap() }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { self.to_u64().unwrap() }
    #[inline] fn isize(self) -> isize { self.to_isize().unwrap() }
    #[inline] fn usize(self) -> usize { self.to_usize().unwrap() }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { unsafe { unreachable() } }
    #[inline] fn try_i32(self) -> Option<i32> { unsafe { unreachable() } }
    #[inline] fn try_u32(self) -> Option<u32> { unsafe { unreachable() } }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { unsafe { unreachable() } }
    #[inline] fn try_isize(self) -> Option<isize> { unsafe { unreachable() } }
    #[inline] fn try_usize(self) -> Option<usize> { unsafe { unreachable() } }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn assert_i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn assert_u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn assert_isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn assert_usize(self) -> usize { unsafe { unreachable() } }
}

impl Cast for i32 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { self.to_i32().unwrap() }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn isize(self) -> isize { self.to_isize().unwrap() }
    #[inline] fn usize(self) -> usize { unsafe { unreachable() } }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { unsafe { unreachable() } }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { self.to_u64() }
    #[inline] fn try_isize(self) -> Option<isize> { unsafe { unreachable() } }
    #[inline] fn try_usize(self) -> Option<usize> { self.to_usize() }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unwrap_overflow(self, self.to_u64()) }
    #[inline] fn assert_isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn assert_usize(self) -> usize { unwrap_overflow(self, self.to_usize()) }
}

impl Cast for u32 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn u32(self) -> u32 { self.to_u32().unwrap() }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { self.to_u64().unwrap() }
    #[inline] fn isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn usize(self) -> usize { self.to_usize().unwrap() }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { self.to_i32() }
    #[inline] fn try_u32(self) -> Option<u32> { unsafe { unreachable() } }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { unsafe { unreachable() } }
    #[inline] fn try_isize(self) -> Option<isize> { self.to_isize() }
    #[inline] fn try_usize(self) -> Option<usize> { unsafe { unreachable() } }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unwrap_overflow(self, self.to_i32()) }
    #[inline] fn assert_u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn assert_isize(self) -> isize { unwrap_overflow(self, self.to_isize()) }
    #[inline] fn assert_usize(self) -> usize { unsafe { unreachable() } }
}

impl Cast for i64 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn usize(self) -> usize { unsafe { unreachable() } }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { self.to_i32() }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { self.to_u64() }
    #[inline] fn try_isize(self) -> Option<isize> { self.to_isize() }
    #[inline] fn try_usize(self) -> Option<usize> { self.to_usize() }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unwrap_overflow(self, self.to_i32()) }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unwrap_overflow(self, self.to_u64()) }
    #[inline] fn assert_isize(self) -> isize { unwrap_overflow(self, self.to_isize()) }
    #[inline] fn assert_usize(self) -> usize { unwrap_overflow(self, self.to_usize()) }
}

impl Cast for u64 {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn u64(self) -> u64 { self.to_u64().unwrap() }
    #[inline] fn isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn usize(self) -> usize { unsafe { unreachable() } }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { self.to_i32() }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { self.to_i64() }
    #[inline] fn try_u64(self) -> Option<u64> { unsafe { unreachable() } }
    #[inline] fn try_isize(self) -> Option<isize> { self.to_isize() }
    #[inline] fn try_usize(self) -> Option<usize> { self.to_usize() }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unwrap_overflow(self, self.to_i32()) }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unwrap_overflow(self, self.to_i64()) }
    #[inline] fn assert_u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn assert_isize(self) -> isize { unwrap_overflow(self, self.to_isize()) }
    #[inline] fn assert_usize(self) -> usize { unwrap_overflow(self, self.to_usize()) }
}

impl Cast for isize {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { self.to_i64().unwrap() }
    #[inline] fn u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn isize(self) -> isize { self.to_isize().unwrap() }
    #[inline] fn usize(self) -> usize { unsafe { unreachable() } }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { self.to_i32() }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { unsafe { unreachable() } }
    #[inline] fn try_u64(self) -> Option<u64> { self.to_u64() }
    #[inline] fn try_isize(self) -> Option<isize> { unsafe { unreachable() } }
    #[inline] fn try_usize(self) -> Option<usize> { self.to_usize() }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unwrap_overflow(self, self.to_i32()) }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn assert_u64(self) -> u64 { unwrap_overflow(self, self.to_u64()) }
    #[inline] fn assert_isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn assert_usize(self) -> usize { unwrap_overflow(self, self.to_usize()) }
}

impl Cast for usize {
    #[inline] fn i8(self) -> i8 { unsafe { unreachable() } }
    #[inline] fn u8(self) -> u8 { unsafe { unreachable() } }
    #[inline] fn i16(self) -> i16 { unsafe { unreachable() } }
    #[inline] fn u16(self) -> u16 { unsafe { unreachable() } }
    #[inline] fn i32(self) -> i32 { unsafe { unreachable() } }
    #[inline] fn u32(self) -> u32 { unsafe { unreachable() } }
    #[inline] fn i64(self) -> i64 { unsafe { unreachable() } }
    #[inline] fn u64(self) -> u64 { self.to_u64().unwrap() }
    #[inline] fn isize(self) -> isize { unsafe { unreachable() } }
    #[inline] fn usize(self) -> usize { self.to_usize().unwrap() }
    #[inline] fn try_i8(self) -> Option<i8> { self.to_i8() }
    #[inline] fn try_u8(self) -> Option<u8> { self.to_u8() }
    #[inline] fn try_i16(self) -> Option<i16> { self.to_i16() }
    #[inline] fn try_u16(self) -> Option<u16> { self.to_u16() }
    #[inline] fn try_i32(self) -> Option<i32> { self.to_i32() }
    #[inline] fn try_u32(self) -> Option<u32> { self.to_u32() }
    #[inline] fn try_i64(self) -> Option<i64> { self.to_i64() }
    #[inline] fn try_u64(self) -> Option<u64> { unsafe { unreachable() } }
    #[inline] fn try_isize(self) -> Option<isize> { self.to_isize() }
    #[inline] fn try_usize(self) -> Option<usize> { unsafe { unreachable() } }
    #[inline] fn assert_i8(self) -> i8 { unwrap_overflow(self, self.to_i8()) }
    #[inline] fn assert_u8(self) -> u8 { unwrap_overflow(self, self.to_u8()) }
    #[inline] fn assert_i16(self) -> i16 { unwrap_overflow(self, self.to_i16()) }
    #[inline] fn assert_u16(self) -> u16 { unwrap_overflow(self, self.to_u16()) }
    #[inline] fn assert_i32(self) -> i32 { unwrap_overflow(self, self.to_i32()) }
    #[inline] fn assert_u32(self) -> u32 { unwrap_overflow(self, self.to_u32()) }
    #[inline] fn assert_i64(self) -> i64 { unwrap_overflow(self, self.to_i64()) }
    #[inline] fn assert_u64(self) -> u64 { unsafe { unreachable() } }
    #[inline] fn assert_isize(self) -> isize { unwrap_overflow(self, self.to_isize()) }
    #[inline] fn assert_usize(self) -> usize { unsafe { unreachable() } }
}

impl I8 for i8 { }
impl NI8 for u8 { }
impl NI8 for i16 { }
impl NI8 for u16 { }
impl NI8 for i32 { }
impl NI8 for u32 { }
impl NI8 for i64 { }
impl NI8 for u64 { }
impl NI8 for isize { }
impl NI8 for usize { }

impl NU8 for i8 { }
impl U8 for u8 { }
impl NU8 for i16 { }
impl NU8 for u16 { }
impl NU8 for i32 { }
impl NU8 for u32 { }
impl NU8 for i64 { }
impl NU8 for u64 { }
impl NU8 for isize { }
impl NU8 for usize { }

impl I16 for i8 { }
impl I16 for u8 { }
impl I16 for i16 { }
impl NI16 for u16 { }
impl NI16 for i32 { }
impl NI16 for u32 { }
impl NI16 for i64 { }
impl NI16 for u64 { }
impl NI16 for isize { }
impl NI16 for usize { }

impl NU16 for i8 { }
impl U16 for u8 { }
impl NU16 for i16 { }
impl U16 for u16 { }
impl NU16 for i32 { }
impl NU16 for u32 { }
impl NU16 for i64 { }
impl NU16 for u64 { }
impl NU16 for isize { }
impl NU16 for usize { }

impl I32 for i8 { }
impl I32 for u8 { }
impl I32 for i16 { }
impl I32 for u16 { }
impl I32 for i32 { }
impl NI32 for u32 { }
impl NI32 for i64 { }
impl NI32 for u64 { }
impl NI32 for isize { }
impl NI32 for usize { }

impl NU32 for i8 { }
impl U32 for u8 { }
impl NU32 for i16 { }
impl U32 for u16 { }
impl NU32 for i32 { }
impl U32 for u32 { }
impl NU32 for i64 { }
impl NU32 for u64 { }
impl NU32 for isize { }
impl NU32 for usize { }

impl I64 for i8 { }
impl I64 for u8 { }
impl I64 for i16 { }
impl I64 for u16 { }
impl I64 for i32 { }
impl I64 for u32 { }
impl I64 for i64 { }
impl NI64 for u64 { }
impl I64 for isize { }
impl NI64 for usize { }

impl NU64 for i8 { }
impl U64 for u8 { }
impl NU64 for i16 { }
impl U64 for u16 { }
impl NU64 for i32 { }
impl U64 for u32 { }
impl NU64 for i64 { }
impl U64 for u64 { }
impl NU64 for isize { }
impl U64 for usize { }

impl Isize for i8 { }
impl Isize for u8 { }
impl Isize for i16 { }
impl Isize for u16 { }
impl Isize for i32 { }
impl NIsize for u32 { }
impl NIsize for i64 { }
impl NIsize for u64 { }
impl Isize for isize { }
impl NIsize for usize { }

impl NUsize for i8 { }
impl Usize for u8 { }
impl NUsize for i16 { }
impl Usize for u16 { }
impl NUsize for i32 { }
impl Usize for u32 { }
impl NUsize for i64 { }
impl NUsize for u64 { }
impl NUsize for isize { }
impl Usize for usize { }

impl TypeName for i8 { fn type_name() -> &'static str { "i8" } }
impl TypeName for u8 { fn type_name() -> &'static str { "u8" } }
impl TypeName for i16 { fn type_name() -> &'static str { "i16" } }
impl TypeName for u16 { fn type_name() -> &'static str { "u16" } }
impl TypeName for i32 { fn type_name() -> &'static str { "i32" } }
impl TypeName for u32 { fn type_name() -> &'static str { "u32" } }
impl TypeName for i64 { fn type_name() -> &'static str { "i64" } }
impl TypeName for u64 { fn type_name() -> &'static str { "u64" } }
impl TypeName for isize { fn type_name() -> &'static str { "isize" } }
impl TypeName for usize { fn type_name() -> &'static str { "usize" } }
