#[deriving(Clone, Copy, Show)]
#[repr(C, packed)]
pub struct DfHeaderVersion {
    pub magic: i32,
    pub version: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(C, packed)]
pub struct DfHeader {
    pub _size: i32,
    pub _swaplen: i32,
    pub num_item_types: i32,
    pub num_items: i32,
    pub num_data: i32,
    pub size_items: i32,
    pub size_data: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(C, packed)]
pub struct DfItemType {
    pub type_id: i32,
    pub start: i32,
    pub num: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(C, packed)]
pub struct DfItemHeader {
    pub type_id_and_id: i32,
    pub size: i32,
}

pub trait UnsafeOnlyI32: Copy { }
impl UnsafeOnlyI32 for i32 { }
impl UnsafeOnlyI32 for DfHeaderVersion { }
impl UnsafeOnlyI32 for DfHeader { }
impl UnsafeOnlyI32 for DfItemType { }
impl UnsafeOnlyI32 for DfItemHeader { }

// ----------------------
// BOILERPLATE CODE BELOW
// ----------------------

const SDHV: usize = 2;
const SDH:  usize = 7;
const SDIT: usize = 3;
const SDIH: usize = 2;
#[test] fn check_sdhv() { use std::mem::size_of; assert_eq!(SDHV * size_of::<i32>(), size_of::<DfHeaderVersion>()); }
#[test] fn check_sdh()  { use std::mem::size_of; assert_eq!(SDH  * size_of::<i32>(), size_of::<DfHeader>());        }
#[test] fn check_sdit() { use std::mem::size_of; assert_eq!(SDIT * size_of::<i32>(), size_of::<DfItemType>());      }
#[test] fn check_sdih() { use std::mem::size_of; assert_eq!(SDIH * size_of::<i32>(), size_of::<DfItemHeader>());    }

impl DfHeaderVersion {
    pub fn as_i32s(&self) -> &[i32, ..SDHV] {
        unsafe { &*(self as *const _ as *const [i32, ..SDHV]) }
    }
    pub fn from_i32s(ints: &[i32, ..SDHV]) -> &DfHeaderVersion {
        unsafe { &*(ints as *const _ as *const DfHeaderVersion) }
    }
    pub fn from_i32_slice(i32s: &[i32]) -> Option<(&DfHeaderVersion, &[i32])> {
        if i32s.len() < SDHV {
            return None;
        }
        let (data_i32s, other_i32s) = i32s.split_at(SDHV);
        let me = DfHeaderVersion::from_i32s(unsafe {
            &*(&data_i32s[0] as *const _ as *const [i32, ..SDHV])
        });
        Some((me, data_i32s))
    }
}

impl DfHeader {
    pub fn as_i32s(&self) -> &[i32, ..SDH] {
        unsafe { &*(self as *const _ as *const [i32, ..SDH]) }
    }
    pub fn from_i32s(ints: &[i32, ..SDH]) -> &DfHeader {
        unsafe { &*(ints as *const _ as *const DfHeader) }
    }
    pub fn from_i32_slice(i32s: &[i32]) -> Option<(&DfHeader, &[i32])> {
        if i32s.len() < SDH {
            return None;
        }
        let (data_i32s, other_i32s) = i32s.split_at(SDH);
        let me = DfHeader::from_i32s(unsafe {
            &*(&data_i32s[0] as *const _ as *const [i32, ..SDH])
        });
        Some((me, data_i32s))
    }
}

impl DfItemType {
    pub fn as_i32s(&self) -> &[i32, ..SDIT] {
        unsafe { &*(self as *const _ as *const [i32, ..SDIT]) }
    }
    pub fn from_i32s(ints: &[i32, ..SDIT]) -> &DfItemType {
        unsafe { &*(ints as *const _ as *const DfItemType) }
    }
    pub fn from_i32_slice(i32s: &[i32]) -> Option<(&DfItemType, &[i32])> {
        if i32s.len() < SDIT {
            return None;
        }
        let (data_i32s, other_i32s) = i32s.split_at(SDIT);
        let me = DfItemType::from_i32s(unsafe {
            &*(&data_i32s[0] as *const _ as *const [i32, ..SDIT])
        });
        Some((me, data_i32s))
    }
}

impl DfItemHeader {
    pub fn as_i32s(&self) -> &[i32, ..SDIT] {
        unsafe { &*(self as *const _ as *const [i32, ..SDIT]) }
    }
    pub fn from_i32s(ints: &[i32, ..SDIT]) -> &DfItemHeader {
        unsafe { &*(ints as *const _ as *const DfItemHeader) }
    }
    pub fn from_i32_slice(i32s: &[i32]) -> Option<(&DfItemHeader, &[i32])> {
        if i32s.len() < SDIT {
            return None;
        }
        let (data_i32s, other_i32s) = i32s.split_at(SDIT);
        let me = DfItemHeader::from_i32s(unsafe {
            &*(&data_i32s[0] as *const _ as *const [i32, ..SDIT])
        });
        Some((me, data_i32s))
    }
}
