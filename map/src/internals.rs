
extern crate datafile;

use datafile::UnsafeOnlyI32;

use std::mem;

pub trait MapItem: UnsafeOnlyI32 {
    fn version(unused_self: Option<Self>) -> i32;
    fn offset(unused_self: Option<Self>) -> uint;
}

pub trait MapItemExt {
    fn len(unused_self: Option<Self>) -> uint;
    fn sum_len(unused_self: Option<Self>) -> uint;

    fn from_slice(slice: &[i32]) -> Option<&Self>;
    //fn from_slice_mut(slice: &mut [i32]) -> Option<&mut Self>;
}

impl<T:MapItem> MapItemExt for T {
    fn len(_: Option<T>) -> uint {
        mem::size_of::<T>() / mem::size_of::<i32>()
    }
    fn sum_len(_: Option<T>) -> uint {
        MapItem::offset(None::<T>) + MapItemExt::len(None::<T>)
    }

    fn from_slice(slice: &[i32]) -> Option<&T> {
        if slice.len() < MapItemExt::sum_len(None::<T>) {
            return None;
        }
        if slice[0] < MapItem::version(None::<T>) {
            return None;
        }
        let result: &[i32] = slice.slice(MapItem::offset(None::<T>), MapItemExt::sum_len(None::<T>));
        assert!(result.len() * mem::size_of::<i32>() == mem::size_of::<T>());
        Some(unsafe { &*(result.as_ptr() as *const T) })
    }
}

#[deriving(Clone, Copy, Show)]
#[repr(C, packed)]
pub struct MapItemCommonV0 {
    pub version: i32,
}

impl UnsafeOnlyI32 for MapItemCommonV0 { }
impl MapItem for MapItemCommonV0 { fn version(_: Option<MapItemCommonV0>) -> i32 { 0 } fn offset(_: Option<MapItemCommonV0>) -> uint { 0 } }

pub static MAP_ITEMTYPE_VERSION: u16 = 0;
pub static MAP_ITEMTYPE_INFO: u16 = 1;
pub static MAP_ITEMTYPE_IMAGE: u16 = 2;
pub static MAP_ITEMTYPE_ENVELOPE: u16 = 3;
pub static MAP_ITEMTYPE_GROUP: u16 = 4;
pub static MAP_ITEMTYPE_LAYER: u16 = 5;
pub static MAP_ITEMTYPE_ENVPOINTS: u16 = 6;

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemVersionV1;

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemInfoV1 {
    pub author: i32,
    pub map_version: i32,
    pub credits: i32,
    pub license: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemImageV1 {
    pub width: i32,
    pub height: i32,
    pub external: i32,
    pub name: i32,
    pub data: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemImageV2 {
    pub format: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemEnvelopeV1 {
    pub channels: i32,
    pub start_points: i32,
    pub num_points: i32,
    pub name: [i32, ..8],
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemEnvelopeV2 {
    pub synchronized: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemGroupV1 {
    pub offset_x: i32,
    pub offset_y: i32,
    pub parallax_x: i32,
    pub parallax_y: i32,
    pub start_layer: i32,
    pub num_layers: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemGroupV2 {
    pub use_clipping: i32,
    pub clip_x: i32,
    pub clip_y: i32,
    pub clip_w: i32,
    pub clip_h: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemLayerV1 {
    pub type_: i32,
    pub flags: i32,
}

#[deriving(Clone, Copy, Show)]
#[repr(packed, C)]
pub struct MapItemEnvpointsV1 {
    pub time: i32,
    pub curvetype: i32,
    pub values: [i32, ..4],
}

impl UnsafeOnlyI32 for MapItemVersionV1 { }
impl UnsafeOnlyI32 for MapItemInfoV1 { }
impl UnsafeOnlyI32 for MapItemImageV1 { }
impl UnsafeOnlyI32 for MapItemImageV2 { }
impl UnsafeOnlyI32 for MapItemEnvelopeV1 { }
impl UnsafeOnlyI32 for MapItemEnvelopeV2 { }
impl UnsafeOnlyI32 for MapItemGroupV1 { }
impl UnsafeOnlyI32 for MapItemGroupV2 { }
impl UnsafeOnlyI32 for MapItemLayerV1 { }
impl UnsafeOnlyI32 for MapItemEnvpointsV1 { }

impl MapItem for MapItemVersionV1 { fn version(_: Option<MapItemVersionV1>) -> i32 { 1 } fn offset(_: Option<MapItemVersionV1>) -> uint { 1 } }
impl MapItem for MapItemInfoV1 { fn version(_: Option<MapItemInfoV1>) -> i32 { 1 } fn offset(_: Option<MapItemInfoV1>) -> uint { 1 } }
impl MapItem for MapItemImageV1 { fn version(_: Option<MapItemImageV1>) -> i32 { 1 } fn offset(_: Option<MapItemImageV1>) -> uint { 1 } }
impl MapItem for MapItemImageV2 { fn version(_: Option<MapItemImageV2>) -> i32 { 2 } fn offset(_: Option<MapItemImageV2>) -> uint { 6 } }
impl MapItem for MapItemEnvelopeV1 { fn version(_: Option<MapItemEnvelopeV1>) -> i32 { 1 } fn offset(_: Option<MapItemEnvelopeV1>) -> uint { 1 } }
impl MapItem for MapItemEnvelopeV2 { fn version(_: Option<MapItemEnvelopeV2>) -> i32 { 2 } fn offset(_: Option<MapItemEnvelopeV2>) -> uint { 12 } }
impl MapItem for MapItemGroupV1 { fn version(_: Option<MapItemGroupV1>) -> i32 { 1 } fn offset(_: Option<MapItemGroupV1>) -> uint { 1 } }
impl MapItem for MapItemGroupV2 { fn version(_: Option<MapItemGroupV2>) -> i32 { 2 } fn offset(_: Option<MapItemGroupV2>) -> uint { 7 } }
impl MapItem for MapItemLayerV1 { fn version(_: Option<MapItemLayerV1>) -> i32 { 1 } fn offset(_: Option<MapItemLayerV1>) -> uint { 1 } }
impl MapItem for MapItemEnvpointsV1 { fn version(_: Option<MapItemEnvpointsV1>) -> i32 { 1 } fn offset(_: Option<MapItemEnvpointsV1>) -> uint { 1 } }

