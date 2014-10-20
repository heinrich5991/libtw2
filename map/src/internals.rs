use std::fmt;

pub static MAPITEMTYPE_VERSION  : u16 = 0;
pub static MAPITEMTYPE_INFO     : u16 = 1;
pub static MAPITEMTYPE_IMAGE    : u16 = 2;
pub static MAPITEMTYPE_ENVELOPE : u16 = 3;
pub static MAPITEMTYPE_GROUP    : u16 = 4;
pub static MAPITEMTYPE_LAYER    : u16 = 5;
pub static MAPITEMTYPE_ENVPOINTS: u16 = 6;
pub static NUM_MAPITEMTYPES     : u16 = 7;

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemV0 {
    pub version: i32,
}

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemVersionV1;

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemInfoV1 {
    pub author: i32,
    pub map_version: i32,
    pub credits: i32,
    pub license: i32,
}

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemImageV1 {
    pub width: i32,
    pub height: i32,
    pub external: i32,
    pub name: i32,
    pub data: i32,
}

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemImageV2 {
    pub format: i32,
}

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemGroupV1 {
    pub offset_x: i32,
    pub offset_y: i32,
    pub parallax_x: i32,
    pub parallax_y: i32,
    pub start_layer: i32,
    pub num_layers: i32,
}

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemGroupV2 {
    pub use_clipping: i32,
    pub clip_x: i32,
    pub clip_y: i32,
    pub clip_w: i32,
    pub clip_h: i32,
}

#[repr(packed, C)]
pub struct MapItemGroupV3 {
    pub name: [i32, ..3],
}

impl Clone for MapItemGroupV3 { fn clone(&self) -> MapItemGroupV3 { *self } }
impl fmt::Show for MapItemGroupV3 { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "MapItemGroupV3 {{ name: {} }}", self.name.as_slice()) } }

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemLayerV1 {
    pub version: i32,
    pub type_: i32,
    pub flags: i32,
}

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemLayerV1TilemapV2 {
    pub version: i32,
    pub flags: i32,
    pub width: i32,
    pub height: i32,
    pub color_r: i32,
    pub color_g: i32,
    pub color_b: i32,
    pub color_a: i32,
    pub color_env: i32,
    pub color_env_offset: i32,
    pub image: i32,
    pub data: i32,
}

#[repr(packed, C)]
pub struct MapItemLayerV1TilemapV3 {
    pub name: [i32, ..3],
}

impl Clone for MapItemLayerV1TilemapV3 { fn clone(&self) -> MapItemLayerV1TilemapV3 { *self } }
impl fmt::Show for MapItemLayerV1TilemapV3 { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "MapItemLayerV1TilemapV3 {{ name: {} }}", self.name.as_slice()) } }

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemLayerV1QuadsV1 {
    pub version: i32,
    pub data: i32,
    pub image: i32,
}

#[repr(packed, C)]
pub struct MapItemLayerV1QuadsV2 {
    pub name: [i32, ..3],
}

impl Clone for MapItemLayerV1QuadsV2 { fn clone(&self) -> MapItemLayerV1QuadsV2 { *self } }
impl fmt::Show for MapItemLayerV1QuadsV2 { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "MapItemLayerV1QuadsV2 {{ name: {} }}", self.name.as_slice()) } }

#[repr(packed, C)]
pub struct MapItemEnvelopeV1 {
    pub version: i32,
    pub channels: i32,
    pub start_points: i32,
    pub num_points: i32,
    pub name: [i32, ..8],
}

impl Clone for MapItemEnvelopeV1 { fn clone(&self) -> MapItemEnvelopeV1 { *self } }
impl fmt::Show for MapItemEnvelopeV1 { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "MapItemEnvelopeV1 {{ version: {}, channels: {}, start_points: {}, num_points: {}, name: {} }}", self.version, self.channels, self.start_points, self.num_points, self.name.as_slice()) } }

#[deriving(Clone, Show)]
#[repr(packed, C)]
pub struct MapItemEnvelopeV2 {
    pub synchronized: i32,
}

