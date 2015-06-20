use std::mem;

use raw::DatafileError;

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DatafileHeaderVersion {
    pub magic: [u8; 4],
    pub version: i32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DatafileHeaderRest {
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
pub struct DatafileItemType {
    pub type_id: i32,
    pub start: i32,
    pub num: i32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DatafileItemHeader {
    pub type_id_and_id: i32,
    pub size: i32,
}

// A struct may only implement OnlyI32 if it consists entirely of tightly
// packed i32 and does not have a destructor.
pub unsafe trait OnlyI32: Copy { }
unsafe impl OnlyI32 for i32 { }
unsafe impl OnlyI32 for DatafileHeaderVersion { }
unsafe impl OnlyI32 for DatafileHeaderRest { }
unsafe impl OnlyI32 for DatafileItemType { }
unsafe impl OnlyI32 for DatafileItemHeader { }

pub static DATAFILE_MAGIC: [u8; 4] = *b"DATA";
pub static DATAFILE_MAGIC_BIGENDIAN: [u8; 4] = *b"ATAD";
pub static DATAFILE_VERSION3: i32 = 3;
pub static DATAFILE_VERSION4: i32 = 4;
pub static DATAFILE_ITEMTYPE_ID_RANGE: i32 = 0x10000;

impl DatafileHeaderVersion {
    pub fn check(&self) -> Result<(),DatafileError> {
        Err(
            if self.magic != DATAFILE_MAGIC && self.magic != DATAFILE_MAGIC_BIGENDIAN {
                error!("wrong datafile signature, magic={:08x}",
                    ((self.magic[0] as u32) << 24)
                    | ((self.magic[1] as u32) << 16)
                    | ((self.magic[2] as u32) << 8)
                    | (self.magic[3] as u32));
                DatafileError::WrongMagic(self.magic)
            } else if self.version != DATAFILE_VERSION3 && self.version != DATAFILE_VERSION4 {
                error!("unsupported datafile version, version={}", self.version);
                DatafileError::UnsupportedVersion(self.version)
            } else {
                return Ok(());
            }
        )
    }
}

impl DatafileHeaderRest {
    pub fn check(&self) -> Result<(),DatafileError> {
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
        Err(DatafileError::MalformedHeader)
    }
}

impl DatafileItemHeader {
    pub fn new(type_id: u16, id: u16, size: i32) -> DatafileItemHeader {
        let mut result = DatafileItemHeader { type_id_and_id: 0, size: size };
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
