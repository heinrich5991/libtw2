use num::ToPrimitive;
use std::mem;

use bitmagic::CallbackExt;
use bitmagic::as_mut_i32_slice;
use bitmagic::to_little_endian;
use common::slice::mut_ref_slice;
use raw::Callback;
use raw;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Error {
    WrongMagic([u8; 4]),
    UnsupportedVersion(i32),
    MalformedHeader,
    Malformed,
    CompressionError,
    TooShort,
    TooShortHeaderVersion,
    TooShortHeader,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Header {
    pub hv: HeaderVersion,
    pub hr: HeaderRest,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HeaderVersion {
    pub magic: [u8; 4],
    pub version: i32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct HeaderRest {
    pub size: i32,
    pub _swaplen: i32,
    pub num_item_types: i32,
    pub num_items: i32,
    pub num_data: i32,
    pub size_items: i32,
    pub size_data: i32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ItemType {
    pub type_id: i32,
    pub start: i32,
    pub num: i32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ItemHeader {
    pub type_id_and_id: i32,
    pub size: i32,
}

// A struct may only implement OnlyI32 if it consists entirely of tightly
// packed i32 and does not have a destructor.
pub unsafe trait OnlyI32: Copy { }
unsafe impl OnlyI32 for i32 { }
unsafe impl OnlyI32 for Header { }
unsafe impl OnlyI32 for HeaderVersion { }
unsafe impl OnlyI32 for HeaderRest { }
unsafe impl OnlyI32 for ItemType { }
unsafe impl OnlyI32 for ItemHeader { }

pub static MAGIC: [u8; 4] = *b"DATA";
pub static MAGIC_BIGENDIAN: [u8; 4] = *b"ATAD";
pub static VERSION3: i32 = 3;
pub static VERSION4: i32 = 4;
pub static ITEMTYPE_ID_RANGE: i32 = 0x10000;
impl Header {
    pub fn read<CB:Callback>(cb: &mut CB) -> Result<Header,raw::Error> {
        let mut result: Header = unsafe { mem::uninitialized() };
        let read = try!(cb.read_le_i32s(mut_ref_slice(&mut result)));
        if read < mem::size_of_val(&result.hv) {
            return Err(raw::Error::Df(Error::TooShortHeaderVersion));
        }
        {
            let slice = as_mut_i32_slice(mut_ref_slice(&mut result));
            // Revert endian conversion for magic field.
            unsafe { to_little_endian(&mut slice[..1]); }
        }
        try!(result.hv.check());
        if read < mem::size_of_val(&result) {
            return Err(raw::Error::Df(Error::TooShortHeader));
        }
        try!(result.hr.check());
        try!(result.check());
        debug!("read header={:?}", result);
        Ok(result)
    }
    pub fn check(&self) -> Result<(),Error> {
        let expected_size = try!(self.total_size());
        if self.hr.size != expected_size {
            error!("size does not match expected size, size={} expected={}", self.hr.size, expected_size);
        } else {
            return Ok(())
        }
        Err(Error::MalformedHeader)
    }
    pub fn total_size(&self) -> Result<i32,Error> {
        // These two functions are just used to make the lines in this function
        // shorter. `u` converts an `i32` to an `u64`, and `s` returns the size
        // of the type as `u64`.
        fn u(val: i32) -> u64 { val.to_u64().unwrap() }
        fn s<T>() -> u64 { mem::size_of::<T>().to_u64().unwrap() }

        let result: u64
            // The whole computation won't overflow because we're multiplying
            // small integers with `u32`s.
            = s::<HeaderRest>() - s::<i32>() * 2 // header_rest without size, _swaplen
            + s::<ItemType>() * u(self.hr.num_item_types) // item_types
            + s::<i32>() * u(self.hr.num_items) // item_offsets
            + s::<i32>() * u(self.hr.num_data) // data_offsets
            + if self.hv.version >= 4 { s::<i32>() * u(self.hr.num_data) } else { 0 } // data_sizes (only version 4)
            + u(self.hr.size_items) // items
            + u(self.hr.size_data); // data

        result.to_i32().ok_or(Error::MalformedHeader)
    }
}

impl HeaderVersion {
    pub fn check(&self) -> Result<(),Error> {
        Err(
            if self.magic != MAGIC && self.magic != MAGIC_BIGENDIAN {
                error!("wrong datafile signature, magic={:08x}",
                    ((self.magic[0] as u32) << 24)
                    | ((self.magic[1] as u32) << 16)
                    | ((self.magic[2] as u32) << 8)
                    | (self.magic[3] as u32));
                Error::WrongMagic(self.magic)
            } else if self.version != VERSION3 && self.version != VERSION4 {
                error!("unsupported datafile version, version={}", self.version);
                Error::UnsupportedVersion(self.version)
            } else {
                return Ok(());
            }
        )
    }
}

impl HeaderRest {
    pub fn check(&self) -> Result<(),Error> {
        if self.size < 0 {
            error!("size is negative, size={}", self.size);
        } else if self._swaplen < 0 {
            error!("_swaplen is negative, _swaplen={}", self._swaplen);
        } else if self.num_item_types < 0 {
            error!("num_item_types is negative, num_item_types={}", self.num_item_types);
        } else if self.num_items < 0 {
            error!("num_items is negative, num_items={}", self.num_items);
        } else if self.num_data < 0 {
            error!("num_data is negative, num_data={}", self.num_data);
        } else if self.size_items < 0 {
            error!("size_items is negative, size_items={}", self.size_items);
        } else if self.size_data < 0 {
            error!("size_data is negative, size_data={}", self.size_data);
        } else if self.size_items as u32 % mem::size_of::<i32>() as u32 != 0 {
            error!("size_items not divisible by 4, size_items={}", self.size_items);
        // TODO: make various check about size, swaplen (non-critical)
        } else {
            return Ok(())
        }
        Err(Error::MalformedHeader)
    }
}

impl ItemHeader {
    pub fn new(type_id: u16, id: u16, size: i32) -> ItemHeader {
        let mut result = ItemHeader { type_id_and_id: 0, size: size };
        result.set_type_id_and_id(type_id, id);
        result
    }
    pub fn type_id(&self) -> u16 {
        (((self.type_id_and_id as u32) >> 16) & 0xffff) as u16
    }
    pub fn id(&self) -> u16 {
        ((self.type_id_and_id as u32) & 0xffff) as u16
    }
    pub fn set_type_id_and_id(&mut self, type_id: u16, id: u16) {
        self.type_id_and_id = (((type_id as u32) << 16) | (id as u32)) as i32;
    }
}
