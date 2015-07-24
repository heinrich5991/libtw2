use common;
use datafile::OnlyI32;
use std::fmt;
use std::mem;
use std::ops;

pub trait MapItem: OnlyI32 {
    fn version() -> i32;
    fn offset() -> usize;
    fn ignore_version() -> bool;
}

pub trait MapItemExt: MapItem {
    fn len() -> usize {
        mem::size_of::<Self>() / mem::size_of::<i32>()
    }
    fn sum_len() -> usize {
        Self::offset() + Self::len()
    }
    fn from_slice(slice: &[i32]) -> Option<&Self> {
        Self::from_slice_rest(slice).map(|(f, _)| f)
    }
    fn from_slice_mut(slice: &mut [i32]) -> Option<&mut Self> {
        Self::from_slice_rest_mut(slice).map(|(f, _)| f)
    }
    fn from_slice_rest(slice: &[i32]) -> Option<(&Self, &[i32])> {
        if slice.len() < Self::sum_len() {
            return None;
        }
        if !Self::ignore_version() && slice[0] < Self::version() {
            return None;
        }
        let result: &[i32] = &slice[Self::offset()..];
        let (item, rest) = result.split_at(Self::len());
        assert!(item.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Some((unsafe { &*(item.as_ptr() as *const Self) }, rest))
    }
    fn from_slice_rest_mut(slice: &mut [i32]) -> Option<(&mut Self, &mut [i32])> {
        if slice.len() < Self::sum_len() {
            return None;
        }
        if slice[0] < Self::version() {
            return None;
        }
        let result: &mut [i32] = &mut slice[Self::offset()..Self::sum_len()];
        let (item, rest) = result.split_at_mut(Self::len());
        assert!(item.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Some((unsafe { &mut *(item.as_ptr() as *mut Self) }, rest))
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
struct Fixed22_10 {
    value: i32,
}

unsafe impl OnlyI32 for Fixed22_10 { }
impl fmt::Debug for Fixed22_10 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ((self.value as f32) / 1024.0).fmt(f)
    }
}

impl fmt::Display for Fixed22_10 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}


#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemCommonV0 {
    pub version: i32,
}

unsafe impl OnlyI32 for MapItemCommonV0 { }
impl MapItem for MapItemCommonV0 { fn version() -> i32 { 0 } fn offset() -> usize { 0 } fn ignore_version() -> bool { true } }

impl fmt::Debug for MapItemCommonV0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "version={:?}", self.version)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemInfoV1ExtraRace {
    settings: i32,
}

unsafe impl OnlyI32 for MapItemInfoV1ExtraRace { }
impl MapItem for MapItemInfoV1ExtraRace { fn version() -> i32 { 1 } fn offset() -> usize { 5 } fn ignore_version() -> bool { false } }

impl fmt::Debug for MapItemInfoV1ExtraRace {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "settings={:?}", self.settings));
        Ok(())
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemEnvelopeV1Legacy {
    pub channels: i32,
    pub start_points: i32,
    pub num_points: i32,
    pub _name: i32,
}

unsafe impl OnlyI32 for MapItemEnvelopeV1Legacy { }
impl MapItem for MapItemEnvelopeV1Legacy { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }

impl fmt::Debug for MapItemEnvelopeV1Legacy {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "channels={:?}", self.channels));
        try!(write!(_f, " start_points={:?}", self.start_points));
        try!(write!(_f, " num_points={:?}", self.num_points));
        try!(write!(_f, " _name={:?}", self._name));
        Ok(())
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1CommonV0 {
    pub version: i32,
}

unsafe impl OnlyI32 for MapItemLayerV1CommonV0 { }
impl MapItem for MapItemLayerV1CommonV0 { fn version() -> i32 { 0 } fn offset() -> usize { 0 } fn ignore_version() -> bool { true } }

impl fmt::Debug for MapItemLayerV1CommonV0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "version={:?}", self.version)
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemEnvpointV1 {
    time: i32,
    curve_type: i32,
    values: [Fixed22_10; 4],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemEnvpointV2 {
    v1: MapItemEnvpointV1,
    in_tangent_dx: [Fixed22_10; 4],
    in_tangent_dy: [Fixed22_10; 4],
    out_tangent_dx: [Fixed22_10; 4],
    out_tangent_dy: [Fixed22_10; 4],
}

unsafe impl OnlyI32 for MapItemEnvpointV1 { }
unsafe impl OnlyI32 for MapItemEnvpointV2 { }
impl fmt::Debug for MapItemEnvpointV1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "time={:?}", self.time));
        try!(write!(f, " curve_type={:?}", self.curve_type));
        try!(write!(f, " values={:?}", self.values));
        Ok(())
    }
}
impl fmt::Debug for MapItemEnvpointV2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:?}", self.v1));
        try!(write!(f, " in_tangent_dx={:?}", self.in_tangent_dx));
        try!(write!(f, " in_tangent_dy={:?}", self.in_tangent_dy));
        try!(write!(f, " out_tangent_dx={:?}", self.out_tangent_dx));
        try!(write!(f, " out_tangent_dy={:?}", self.out_tangent_dy));
        Ok(())
    }
}
impl Envpoint for MapItemEnvpointV1 { fn envelope_version() -> ops::Range<i32> { 1..2+1 } }
impl Envpoint for MapItemEnvpointV2 { fn envelope_version() -> ops::Range<i32> { 3..3+1 } }

pub trait Envpoint: OnlyI32 {
    fn envelope_version() -> ops::Range<i32>;
}

pub trait EnvpointExt: Envpoint {
    fn from_slice(slice: &[i32], envelope_version: i32) -> Option<&[Self]> {
        if !(Self::envelope_version().start <= envelope_version
            && envelope_version < Self::envelope_version().end)
        {
            return None;
        }
        if mem::size_of::<i32>() * slice.len() % mem::size_of::<Self>() != 0 {
            return None;
        }
        Some(unsafe { common::slice::transmute(slice) })
    }
}

impl<T:Envpoint> EnvpointExt for T { }

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1TilemapExtraRace {
    pub data: i32,
}
impl MapItemLayerV1TilemapExtraRace {
    pub fn offset(version: i32, flags: u32) -> Option<usize> {
        let offset = match version {
            2 => MapItemLayerV1TilemapV2::sum_len(),
            3 => MapItemLayerV1TilemapV3::sum_len(),
            _ => return None,
        };
        Some(offset + match flags {
            TILELAYERFLAG_TELEPORT => 0,
            TILELAYERFLAG_SPEEDUP => 1,
            TILELAYERFLAG_FRONT => 2,
            TILELAYERFLAG_SWITCH => 3,
            TILELAYERFLAG_TUNE => 4,
            _ => return None,
        })
    }
    pub fn from_slice(slice: &[i32], version: i32, flags: u32)
        -> Option<&MapItemLayerV1TilemapExtraRace>
    {
        let offset = unwrap_or_return!(
            MapItemLayerV1TilemapExtraRace::offset(version, flags), None
        );
        if slice.len() <= offset {
            return None;
        }
        Some(&(unsafe { common::slice::transmute(slice) })[offset])
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Tile {
    pub index: u8,
    pub flags: u8,
    pub skip: u8,
    pub reserved: u8,
}

pub const LAYERFLAG_DETAIL: u32 = 1;
pub const LAYERFLAGS_ALL: u32 = 1;

pub const TILELAYERFLAG_GAME: u32 = 1;
pub const TILELAYERFLAG_TELEPORT: u32 = 2;
pub const TILELAYERFLAG_SPEEDUP: u32 = 4;
pub const TILELAYERFLAG_FRONT: u32 = 8;
pub const TILELAYERFLAG_SWITCH: u32 = 16;
pub const TILELAYERFLAG_TUNE: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    InconsistentGameLayerDimensions,
    InvalidLayerFlags(u32),
    InvalidLayerTilemapFlags(u32),
    InvalidLayerType(i32),
    InvalidTilesLength(usize),
    InvalidVersion(i32),
    MalformedDdraceLayerSounds,
    MalformedGroup,
    MalformedImage,
    MalformedImageName,
    MalformedLayer,
    MalformedLayerQuads,
    MalformedLayerTilemap,
    MalformedVersion,
    MissingVersion,
    NoGameLayer,
    TooManyGameGroups,
    TooManyGameLayers,
}

pub const MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS_LEGACY: i32 = 9;

pub const MAP_ITEMTYPE_VERSION: u16 = 0;
pub const MAP_ITEMTYPE_INFO: u16 = 1;
pub const MAP_ITEMTYPE_IMAGE: u16 = 2;
pub const MAP_ITEMTYPE_ENVELOPE: u16 = 3;
pub const MAP_ITEMTYPE_GROUP: u16 = 4;
pub const MAP_ITEMTYPE_LAYER: u16 = 5;
pub const MAP_ITEMTYPE_ENVPOINTS: u16 = 6;
pub const MAP_ITEMTYPE_DDRACE_SOUND: u16 = 7;

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
pub struct MapItemGroupV3 {
    pub name: [i32; 3],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1 {
    pub type_: i32,
    pub flags: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemDdraceSoundV1 {
    pub external: i32,
    pub name: i32,
    pub data: i32,
    pub data_size: i32,
}

unsafe impl OnlyI32 for MapItemVersionV1 { }
unsafe impl OnlyI32 for MapItemInfoV1 { }
unsafe impl OnlyI32 for MapItemImageV1 { }
unsafe impl OnlyI32 for MapItemImageV2 { }
unsafe impl OnlyI32 for MapItemEnvelopeV1 { }
unsafe impl OnlyI32 for MapItemEnvelopeV2 { }
unsafe impl OnlyI32 for MapItemGroupV1 { }
unsafe impl OnlyI32 for MapItemGroupV2 { }
unsafe impl OnlyI32 for MapItemGroupV3 { }
unsafe impl OnlyI32 for MapItemLayerV1 { }
unsafe impl OnlyI32 for MapItemDdraceSoundV1 { }

impl MapItem for MapItemVersionV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemInfoV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemImageV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemImageV2 { fn version() -> i32 { 2 } fn offset() -> usize { 6 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemEnvelopeV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemEnvelopeV2 { fn version() -> i32 { 2 } fn offset() -> usize { 12 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemGroupV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemGroupV2 { fn version() -> i32 { 2 } fn offset() -> usize { 7 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemGroupV3 { fn version() -> i32 { 3 } fn offset() -> usize { 12 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { true } }
impl MapItem for MapItemDdraceSoundV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }

impl MapItemEnvelopeV1 {
    pub fn name_get(&self) -> [u8; 32] {
        let mut result: [u8; 32] = unsafe { mem::uninitialized() };
        i32s_to_bytes(&mut result, &self.name);
        result[32-1] = 0;
        result
    }
}
impl MapItemGroupV3 {
    pub fn name_get(&self) -> [u8; 12] {
        let mut result: [u8; 12] = unsafe { mem::uninitialized() };
        i32s_to_bytes(&mut result, &self.name);
        result[12-1] = 0;
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
impl fmt::Debug for MapItemGroupV3 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "name={:?}", String::from_utf8_lossy(bytes_to_string(&self.name_get()))));
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
impl fmt::Debug for MapItemDdraceSoundV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "external={:?}", self.external));
        try!(write!(_f, " name={:?}", self.name));
        try!(write!(_f, " data={:?}", self.data));
        try!(write!(_f, " data_size={:?}", self.data_size));
        Ok(())
    }
}
pub const MAP_ITEMTYPE_LAYER_V1_TILEMAP: i32 = 2;
pub const MAP_ITEMTYPE_LAYER_V1_QUADS: i32 = 3;
pub const MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS: i32 = 10;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1TilemapV1;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1TilemapV2 {
    pub width: i32,
    pub height: i32,
    pub flags: i32,
    pub color_red: i32,
    pub color_green: i32,
    pub color_blue: i32,
    pub color_alpha: i32,
    pub color_env: i32,
    pub color_env_offset: i32,
    pub image: i32,
    pub data: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1TilemapV3 {
    pub name: [i32; 3],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1QuadsV1 {
    pub num_quads: i32,
    pub data: i32,
    pub image: i32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1QuadsV2 {
    pub name: [i32; 3],
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1DdraceSoundsV1;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MapItemLayerV1DdraceSoundsV2 {
    pub num_sources: i32,
    pub data: i32,
    pub sound: i32,
    pub name: [i32; 3],
}

unsafe impl OnlyI32 for MapItemLayerV1TilemapV1 { }
unsafe impl OnlyI32 for MapItemLayerV1TilemapV2 { }
unsafe impl OnlyI32 for MapItemLayerV1TilemapV3 { }
unsafe impl OnlyI32 for MapItemLayerV1QuadsV1 { }
unsafe impl OnlyI32 for MapItemLayerV1QuadsV2 { }
unsafe impl OnlyI32 for MapItemLayerV1DdraceSoundsV1 { }
unsafe impl OnlyI32 for MapItemLayerV1DdraceSoundsV2 { }

impl MapItem for MapItemLayerV1TilemapV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1TilemapV2 { fn version() -> i32 { 2 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1TilemapV3 { fn version() -> i32 { 3 } fn offset() -> usize { 12 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1QuadsV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1QuadsV2 { fn version() -> i32 { 2 } fn offset() -> usize { 4 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1DdraceSoundsV1 { fn version() -> i32 { 1 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }
impl MapItem for MapItemLayerV1DdraceSoundsV2 { fn version() -> i32 { 2 } fn offset() -> usize { 1 } fn ignore_version() -> bool { false } }

impl MapItemLayerV1TilemapV3 {
    pub fn name_get(&self) -> [u8; 12] {
        let mut result: [u8; 12] = unsafe { mem::uninitialized() };
        i32s_to_bytes(&mut result, &self.name);
        result[12-1] = 0;
        result
    }
}
impl MapItemLayerV1QuadsV2 {
    pub fn name_get(&self) -> [u8; 12] {
        let mut result: [u8; 12] = unsafe { mem::uninitialized() };
        i32s_to_bytes(&mut result, &self.name);
        result[12-1] = 0;
        result
    }
}
impl MapItemLayerV1DdraceSoundsV2 {
    pub fn name_get(&self) -> [u8; 12] {
        let mut result: [u8; 12] = unsafe { mem::uninitialized() };
        i32s_to_bytes(&mut result, &self.name);
        result[12-1] = 0;
        result
    }
}

impl fmt::Debug for MapItemLayerV1TilemapV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1TilemapV2 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "width={:?}", self.width));
        try!(write!(_f, " height={:?}", self.height));
        try!(write!(_f, " flags={:?}", self.flags));
        try!(write!(_f, " color_red={:?}", self.color_red));
        try!(write!(_f, " color_green={:?}", self.color_green));
        try!(write!(_f, " color_blue={:?}", self.color_blue));
        try!(write!(_f, " color_alpha={:?}", self.color_alpha));
        try!(write!(_f, " color_env={:?}", self.color_env));
        try!(write!(_f, " color_env_offset={:?}", self.color_env_offset));
        try!(write!(_f, " image={:?}", self.image));
        try!(write!(_f, " data={:?}", self.data));
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1TilemapV3 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "name={:?}", String::from_utf8_lossy(bytes_to_string(&self.name_get()))));
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1QuadsV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "num_quads={:?}", self.num_quads));
        try!(write!(_f, " data={:?}", self.data));
        try!(write!(_f, " image={:?}", self.image));
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1QuadsV2 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "name={:?}", String::from_utf8_lossy(bytes_to_string(&self.name_get()))));
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1DdraceSoundsV1 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
impl fmt::Debug for MapItemLayerV1DdraceSoundsV2 {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(_f, "num_sources={:?}", self.num_sources));
        try!(write!(_f, " data={:?}", self.data));
        try!(write!(_f, " sound={:?}", self.sound));
        try!(write!(_f, " name={:?}", String::from_utf8_lossy(bytes_to_string(&self.name_get()))));
        Ok(())
    }
}
