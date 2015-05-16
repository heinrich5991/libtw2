use std::mem;
use std::num::Int;
use std::raw::Repr;
use std::raw::Slice;

#[deriving(Eq, Hash, Ord, PartialEq, PartialOrd, Show)]
pub struct Align<T>([T, ..0]);

impl<T:Copy> Copy for Align<T> { }
impl<T> Clone for Align<T> {
    fn clone(&self) -> Align<T> {
        Align([])
    }
}

/// Big-endian aligned signed 32-bit integer.
#[repr(C, packed)]
#[deriving(Clone, Copy)]
pub struct BeaI32(Align<i32>, [u8, ..4]);

trait FlatArray<T> {
    fn num_elements(_unused_self: Option<Self>) -> usize {
        if mem::size_of::<T>() != 0 {
            mem::size_of::<Self>() / mem::size_of::<T>()
        } else {
            panic!("num_elements must be manually implemented for zero-sized types");
        }
    }
}

impl FlatArray<u8> for BeaI32 { }

pub trait SliceExt<T> for Sized? {
    fn flatten_ref(&self) -> &[T];
    fn flatten_mut(&mut self) -> &mut [T];
}

impl<T,U:FlatArray<T>> SliceExt<T> for [U] {
    fn flatten_ref(&self) -> &[T] {
        let repr = self.repr();
        let new_len;
        let num_elements = FlatArray::num_elements(None::<U>);
        if mem::size_of::<T>() != 0 {
            // can't overflow, we have it in memory
            new_len = repr.len * num_elements;
        } else {
            new_len = repr.len.checked_mul(num_elements).expect("capacity overflow");
        }
        unsafe { mem::transmute(Slice {
            data: repr.data,
            len: new_len,
        })}
    }
    fn flatten_mut(&mut self) -> &mut [T] {
        let repr = self.repr();
        let new_len;
        let num_elements = FlatArray::num_elements(None::<U>);
        if mem::size_of::<T>() != 0 {
            // can't overflow, we have it in memory
            new_len = repr.len * num_elements;
        } else {
            new_len = repr.len.checked_mul(num_elements).expect("capacity overflow");
        }
        unsafe { mem::transmute(Slice {
            data: repr.data,
            len: new_len,
        })}
    }
}

// ======================
// BOILERPLATE CODE BELOW
// ======================

const S32: usize = 4;
#[test] fn check_s32() { use std::mem; assert_eq!(S32, mem::size_of::<BeaU32>()); }
#[test] fn check_align_beai32() { use std::mem; assert_eq!(BeaI32, mem::min_align_of::<BeaI32>()); }

fn u32_to_bytes(value: u32) -> [u8, ..S32] {
    [
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >>  8) as u8,
        (value >>  0) as u8,
    ]
}

fn bytes_to_u32(bytes: [u8, ..S32]) -> u32 {
    (bytes[0] as u32 << 24) |
    (bytes[1] as u32 << 16) |
    (bytes[2] as u32 <<  8) |
    (bytes[3] as u32 <<  0)
}

impl BeaI32 {
    pub fn from_i32(value: i32) -> BeaI32 {
        BeaI32(Align([]), u32_to_bytes(value as u32))
    }
    pub fn to_i32(self) -> i32 {
        let BeaI32(_, bytes) = self;
        bytes_to_u32(bytes) as i32
    }
    pub fn as_bytes(&self) -> &[u8, ..S32] {
        &self.1
    }
}
