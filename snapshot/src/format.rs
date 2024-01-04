use buffer::CapacityError;
use packer;
use packer::Packer;
use packer::Unpacker;
use snap::Error;
use warn::wrap;
use warn::Warn;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Warning {
    Packer(packer::Warning),
    NonZeroPadding,
    DuplicateDelete,
    DuplicateUpdate,
    UnknownDelete,
    DeleteUpdate,
    NumUpdatedItems,
}

impl From<packer::Warning> for Warning {
    fn from(w: packer::Warning) -> Warning {
        Warning::Packer(w)
    }
}

pub fn key_to_type_id(key: i32) -> u16 {
    ((key as u32 >> 16) & 0xffff) as u16
}

pub fn key_to_id(key: i32) -> u16 {
    ((key as u32) & 0xffff) as u16
}

pub fn key(type_id: u16, id: u16) -> i32 {
    (((type_id as u32) << 16) | (id as u32)) as i32
}

#[derive(Clone, Copy, Debug)]
pub struct Item<'a> {
    pub type_id: u16,
    pub id: u16,
    pub data: &'a [i32],
}

impl<'a> Item<'a> {
    pub fn from_key(key: i32, data: &[i32]) -> Item {
        Item {
            type_id: key_to_type_id(key),
            id: key_to_id(key),
            data: data,
        }
    }
    pub fn key(&self) -> i32 {
        key(self.type_id, self.id)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DeltaHeader {
    pub num_deleted_items: i32,
    pub num_updated_items: i32,
}

impl DeltaHeader {
    pub fn decode<W: Warn<Warning>>(warn: &mut W, p: &mut Unpacker) -> Result<DeltaHeader, Error> {
        let result = DeltaHeader {
            num_deleted_items: packer::positive(p.read_int(wrap(warn))?)?,
            num_updated_items: packer::positive(p.read_int(wrap(warn))?)?,
        };
        if p.read_int(wrap(warn))? != 0 {
            warn.warn(Warning::NonZeroPadding);
        }
        Ok(result)
    }
    pub fn encode<'d, 's>(&self, mut p: Packer<'d, 's>) -> Result<&'d [u8], CapacityError> {
        p.write_int(self.num_deleted_items)?;
        p.write_int(self.num_updated_items)?;
        p.write_int(0)?;
        Ok(p.written())
    }
}
