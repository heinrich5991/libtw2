//#![allow(dead_code)]
#![feature(macro_rules)]

pub static MAPITEMTYPE_VERSION  : u16 = 0;
pub static MAPITEMTYPE_INFO     : u16 = 1;
pub static MAPITEMTYPE_IMAGE    : u16 = 2;
pub static MAPITEMTYPE_ENVELOPE : u16 = 3;
pub static MAPITEMTYPE_GROUP    : u16 = 4;
pub static MAPITEMTYPE_LAYER    : u16 = 5;
pub static MAPITEMTYPE_ENVPOINTS: u16 = 6;
pub static NUM_MAPITEMTYPES     : u16 = 7;


pub static MAP_ITEM_VERSION: i32 = 1;
//#[deriving(Clone, Show)]
//#[repr(packed)]
//pub struct MapItemVersionV1 {
//	pub version: i32,
//}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemInfoV1 {
	pub version: i32,
	pub author: i32,
	pub map_version: i32,
	pub credits: i32,
	pub license: i32,
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemImageV1 {
	pub version: i32,
	pub width: i32,
	pub height: i32,
	pub external: i32,
	pub name: i32,
	pub data: i32,
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemImageV2 {
	pub p: MapItemImageV1,

	pub format: i32,
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemGroupV1 {
	pub version: i32,
	pub offset_x: i32,
	pub offset_y: i32,
	pub parallax_x: i32,
	pub parallax_y: i32,
	pub start_layer: i32,
	pub num_layers: i32,
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemGroupV2 {
	pub p: MapItemGroupV1,

	pub use_clipping: i32,
	pub clip_x: i32,
	pub clip_y: i32,
	pub clip_w: i32,
	pub clip_h: i32,
}

//#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemGroupV3 {
	pub p: MapItemGroupV2,

	pub name: [i32, ..3],
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemLayerV1 {
	pub version: i32,
	pub type_: i32,
	pub flags: i32,
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemLayerV1TilemapV2 {
	pub p: MapItemLayerV1,

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

//#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemLayerV1TilemapV3 {
	pub p: MapItemLayerV1TilemapV2,

	pub name: [i32, ..3],
}

#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemLayerV1QuadsV1 {
	pub p: MapItemLayerV1,

	pub version: i32,
	pub data: i32,
	pub image: i32,
}

//#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemLayerV1QuadsV2 {
	pub p: MapItemLayerV1QuadsV1,

	pub name: [i32, ..3],
}

//#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemEnvelopeV1 {
	pub version: i32,
	pub channels: i32,
	pub start_points: i32,
	pub num_points: i32,
	pub name: [i32, ..8],
}

//#[deriving(Clone, Show)]
#[repr(packed)]
pub struct MapItemEnvelopeV2 {
	pub p: MapItemEnvelopeV1,

	pub synchronized: i32,
}

macro_rules! vec(
    ($foo:ident, $($e:expr),*) => ({
        // leading _ to allow empty construction without a warning.
        let mut _temp = ::vec::Vec::new();
        $(_temp.push($e);)*
        _temp
    });
    ($($e:expr),+,) => (vec!($($e),+))
)

macro_rules! map_item(
	($name:ident, $($($members:ident)*),*) => (
		#[deriving(Clone, Show)]
		#[repr(packed)]
		$(pub struct $name {
			$(pub $members: i32,)*
		})*
	);
)

map_item!(MapItemVersionV1, version)
