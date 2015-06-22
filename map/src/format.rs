extern crate datafile;

use datafile::OnlyI32;

use std::fmt;
use std::mem;

pub trait MapItem: OnlyI32 {
    fn version() -> i32;
    fn offset() -> usize;
}

pub trait MapItemExt: MapItem {
    fn len() -> usize {
        mem::size_of::<Self>() / mem::size_of::<i32>()
    }
    fn sum_len() -> usize {
        Self::offset() + Self::len()
    }
    fn from_slice(slice: &[i32]) -> Option<&Self> {
        if slice.len() < Self::sum_len() {
            return None;
        }
        if slice[0] < Self::version() {
            return None;
        }
        let result: &[i32] = &slice[Self::offset()..Self::sum_len()];
        assert!(result.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Some(unsafe { &*(result.as_ptr() as *const Self) })
    }
    fn from_slice_mut(slice: &mut [i32]) -> Option<&mut Self> {
        if slice.len() < Self::sum_len() {
            return None;
        }
        if slice[0] < Self::version() {
            return None;
        }
        let result: &mut [i32] = &mut slice[Self::offset()..Self::sum_len()];
        assert!(result.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Some(unsafe { &mut *(result.as_ptr() as *mut Self) })
    }
}

impl<T:MapItem> MapItemExt for T { }

pub fn i32s_to_bytes(result: &mut [u8], input: &[i32]) {
    assert!(result.len() == input.len() * mem::size_of::<i32>());
    for (output, input) in result.chunks_mut(mem::size_of::<i32>()).zip(input) {
        output[0] = (((input >> 24) & 0xff) - 0x80) as u8;
        output[1] = (((input >> 16) & 0xff) - 0x80) as u8;
        output[2] = (((input >>  8) & 0xff) - 0x80) as u8;
        output[3] = (((input >>  0) & 0xff) - 0x80) as u8;
    }
}

pub fn bytes_to_string(bytes: &[u8]) -> &[u8] {
    for (i, &b) in bytes.iter().enumerate() {
        if b == 0 {
            return &bytes[..i]
        }
    }
    bytes
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemCommonV0 {
    pub version: i32,
}

unsafe impl OnlyI32 for MapItemCommonV0 { }
impl MapItem for MapItemCommonV0 { fn version() -> i32 { 0 } fn offset() -> usize { 0 } }

impl fmt::Debug for MapItemCommonV0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "version={:?}", self.version)
    }
}

pub const MAP_ITEMTYPE_VERSION: u16 = 0;
pub const MAP_ITEMTYPE_INFO: u16 = 1;
pub const MAP_ITEMTYPE_IMAGE: u16 = 2;
pub const MAP_ITEMTYPE_ENVELOPE: u16 = 3;
pub const MAP_ITEMTYPE_GROUP: u16 = 4;
pub const MAP_ITEMTYPE_LAYER: u16 = 5;
pub const MAP_ITEMTYPE_ENVPOINTS: u16 = 6;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemVersionV1;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemInfoV1 {
    pub author: i32,
    pub map_version: i32,
    pub credits: i32,
    pub license: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemImageV1 {
    pub width: i32,
    pub height: i32,
    pub external: i32,
    pub name: i32,
    pub data: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemImageV2 {
    pub format: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemEnvelopeV1 {
    pub channels: i32,
    pub start_points: i32,
    pub num_points: i32,
    pub name: [i32; 8],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemEnvelopeV2 {
    pub synchronized: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemGroupV1 {
    pub offset_x: i32,
    pub offset_y: i32,
    pub parallax_x: i32,
    pub parallax_y: i32,
    pub start_layer: i32,
    pub num_layers: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemGroupV2 {
    pub use_clipping: i32,
    pub clip_x: i32,
    pub clip_y: i32,
    pub clip_w: i32,
    pub clip_h: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1 {
    pub type_: i32,
    pub flags: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemEnvpointsV1 {
    pub time: i32,
    pub curvetype: i32,
    pub values: [i32; 4],
}

unsafe impl OnlyI32 for MapItemVersionV1 { }
unsafe impl OnlyI32 for MapItemInfoV1 { }
unsafe impl OnlyI32 for MapItemImageV1 { }
unsafe impl OnlyI32 for MapItemImageV2 { }
unsafe impl OnlyI32 for MapItemEnvelopeV1 { }
unsafe impl OnlyI32 for MapItemEnvelopeV2 { }
unsafe impl OnlyI32 for MapItemGroupV1 { }
unsafe impl OnlyI32 for MapItemGroupV2 { }
unsafe impl OnlyI32 for MapItemLayerV1 { }
unsafe impl OnlyI32 for MapItemEnvpointsV1 { }

impl MapItem for MapItemVersionV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }
impl MapItem for MapItemInfoV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }
impl MapItem for MapItemImageV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }
impl MapItem for MapItemImageV2 { fn version() -> i32 { 2 } fn offset() -> usize { 6 } }
impl MapItem for MapItemEnvelopeV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }
impl MapItem for MapItemEnvelopeV2 { fn version() -> i32 { 2 } fn offset() -> usize { 12 } }
impl MapItem for MapItemGroupV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }
impl MapItem for MapItemGroupV2 { fn version() -> i32 { 2 } fn offset() -> usize { 7 } }
impl MapItem for MapItemLayerV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }
impl MapItem for MapItemEnvpointsV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } }

impl MapItemEnvelopeV1 {
    pub fn name_get(&self) -> [u8; 32] {
        let mut result: [u8; 32] = unsafe { mem::uninitialized() };
        i32s_to_bytes(&mut result, &self.name);
        result[32-1] = 0;
        result
    }
}

impl fmt::Debug for MapItemVersionV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
impl fmt::Debug for MapItemInfoV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "author={:?}", self.author));
        try!(write!(_f, " map_version={:?}", self.map_version));
        try!(write!(_f, " credits={:?}", self.credits));
        try!(write!(_f, " license={:?}", self.license));
        Ok(())
    }
}
impl fmt::Debug for MapItemImageV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "width={:?}", self.width));
        try!(write!(_f, " height={:?}", self.height));
        try!(write!(_f, " external={:?}", self.external));
        try!(write!(_f, " name={:?}", self.name));
        try!(write!(_f, " data={:?}", self.data));
        Ok(())
    }
}
impl fmt::Debug for MapItemImageV2 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "format={:?}", self.format));
        Ok(())
    }
}
impl fmt::Debug for MapItemEnvelopeV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "channels={:?}", self.channels));
        try!(write!(_f, " start_points={:?}", self.start_points));
        try!(write!(_f, " num_points={:?}", self.num_points));
        try!(write!(_f, " name={:?}", String::from_utf8_lossy(bytes_to_string(&self.name_get()))));
        Ok(())
    }
}
impl fmt::Debug for MapItemEnvelopeV2 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "synchronized={:?}", self.synchronized));
        Ok(())
    }
}
impl fmt::Debug for MapItemGroupV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "offset_x={:?}", self.offset_x));
        try!(write!(_f, " offset_y={:?}", self.offset_y));
        try!(write!(_f, " parallax_x={:?}", self.parallax_x));
        try!(write!(_f, " parallax_y={:?}", self.parallax_y));
        try!(write!(_f, " start_layer={:?}", self.start_layer));
        try!(write!(_f, " num_layers={:?}", self.num_layers));
        Ok(())
    }
}
impl fmt::Debug for MapItemGroupV2 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "use_clipping={:?}", self.use_clipping));
        try!(write!(_f, " clip_x={:?}", self.clip_x));
        try!(write!(_f, " clip_y={:?}", self.clip_y));
        try!(write!(_f, " clip_w={:?}", self.clip_w));
        try!(write!(_f, " clip_h={:?}", self.clip_h));
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "type_={:?}", self.type_));
        try!(write!(_f, " flags={:?}", self.flags));
        Ok(())
    }
}
impl fmt::Debug for MapItemEnvpointsV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "time={:?}", self.time));
        try!(write!(_f, " curvetype={:?}", self.curvetype));
        try!(write!(_f, " values={:?}", self.values));
        Ok(())
    }
}
