import re

ITEMS = [
    (0, "version", [
        [],
    ]),
    (1, "info", [
        ["author", "version", "credits", "license"],
        ["settings"]
    ]),
    (2, "image", [
        ["width", "height", "external", "name", "data"],
        ["format"],
    ]),
    (3, "envelope", [
        ["channels", "start_points", "num_points", "name[8s]"],
        ["synchronized"],
    ]),
    (4, "group", [
        ["offset_x", "offset_y", "parallax_x", "parallax_y", "start_layer", "num_layers"],
        ["use_clipping", "clip_x", "clip_y", "clip_w", "clip_h"],
        ["name[3s]"],
    ]),
    (5, "layer", [
        ["type_", "flags"],
    ]),
    # Different format:
    (6, "envpoints", []),
    (7, "ddrace_sound", [
        ["external", "name", "data", "data_size"],
    ]),
]

LAYER_V1_ITEMS = [
    (2, "tilemap", [
        # TODO: What is version 1?
        [],
        ["width", "height", "flags", "color_red", "color_green", "color_blue",
         "color_alpha", "color_env", "color_env_offset", "image", "data"],
        ["name[3s]"],
    ]),
    (3, "quads", [
        ["num_quads", "data", "image"],
        ["name[3s]"],
    ]),
    (10, "ddrace_sounds", [
        ["num_sources", "data", "sound", "name[3s]"],
        [],
    ]),
]

header = """\
use common;
use common::num::LeI16;
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
    fn from_slice(slice: &[i32]) -> Result<Option<&Self>, TooShort> {
        Self::from_slice_rest(slice).map(|o| o.map(|(f, _)| f))
    }
    fn from_slice_mut(slice: &mut [i32]) -> Result<Option<&mut Self>, TooShort> {
        Self::from_slice_rest_mut(slice).map(|o| o.map(|(f, _)| f))
    }
    fn from_slice_rest(slice: &[i32]) -> Result<Option<(&Self, &[i32])>, TooShort> {
        if !Self::ignore_version() {
            if slice.len() == 0 {
                return Err(TooShort);
            }
            if slice[0] < Self::version() {
                return Ok(None);
            }
        }
        if slice.len() < Self::sum_len() {
            return Err(TooShort);
        }
        let result: &[i32] = &slice[Self::offset()..];
        let (item, rest) = result.split_at(Self::len());
        assert!(item.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Ok(Some((unsafe { &*(item.as_ptr() as *const Self) }, rest)))
    }
    fn from_slice_rest_mut(slice: &mut [i32])
        -> Result<Option<(&mut Self, &mut [i32])>, TooShort>
    {
        if !Self::ignore_version() {
            if slice.len() == 0 {
                return Err(TooShort);
            }
            if slice[0] < Self::version() {
                return Ok(None);
            }
        }
        if slice.len() < Self::sum_len() {
            return Err(TooShort);
        }
        let result: &mut [i32] = &mut slice[Self::offset()..];
        let (item, rest) = result.split_at_mut(Self::len());
        assert!(item.len() * mem::size_of::<i32>() == mem::size_of::<Self>());
        Ok(Some((unsafe { &mut *(item.as_mut_ptr() as *mut Self) }, rest)))
    }
}

impl<T: MapItem> MapItemExt for T { }

pub struct TooShort;

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
    pub settings: i32,
}

unsafe impl OnlyI32 for MapItemInfoV1ExtraRace { }
impl MapItem for MapItemInfoV1ExtraRace { fn version() -> i32 { 1 } fn offset() -> usize { 5 } fn ignore_version() -> bool { false } }

impl fmt::Debug for MapItemInfoV1ExtraRace {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        write!(_f, "settings={:?}", self.settings)?;
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
        write!(_f, "channels={:?}", self.channels)?;
        write!(_f, " start_points={:?}", self.start_points)?;
        write!(_f, " num_points={:?}", self.num_points)?;
        write!(_f, " _name={:?}", self._name)?;
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
        write!(f, "time={:?}", self.time)?;
        write!(f, " curve_type={:?}", self.curve_type)?;
        write!(f, " values={:?}", self.values)?;
        Ok(())
    }
}
impl fmt::Debug for MapItemEnvpointV2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.v1)?;
        write!(f, " in_tangent_dx={:?}", self.in_tangent_dx)?;
        write!(f, " in_tangent_dy={:?}", self.in_tangent_dy)?;
        write!(f, " out_tangent_dx={:?}", self.out_tangent_dx)?;
        write!(f, " out_tangent_dy={:?}", self.out_tangent_dy)?;
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

impl<T: Envpoint> EnvpointExt for T { }

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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct TeleTile {
    pub number: u8,
    pub index: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SpeedupTile {
    pub force: u8,
    pub max_speed: u8,
    pub index: u8,
    pub padding: u8,
    pub angle: LeI16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct SwitchTile {
    pub number: u8,
    pub index: u8,
    pub flags: u8,
    pub delay: u8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(C)]
pub struct TuneTile {
    pub number: u8,
    pub index: u8,
}

pub const TILEFLAG_VFLIP: u8 = 1 << 0;
pub const TILEFLAG_HFLIP: u8 = 1 << 1;
pub const TILEFLAG_OPAQUE: u8 = 1 << 2;
pub const TILEFLAG_ROTATE: u8 = 1 << 3;

pub const LAYERFLAG_DETAIL: u32 = 1;
pub const LAYERFLAGS_ALL: u32 = 1;

pub const TILELAYERFLAG_GAME: u32 = 1;
pub const TILELAYERFLAG_TELEPORT: u32 = 2;
pub const TILELAYERFLAG_SPEEDUP: u32 = 4;
pub const TILELAYERFLAG_FRONT: u32 = 8;
pub const TILELAYERFLAG_SWITCH: u32 = 16;
pub const TILELAYERFLAG_TUNE: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GroupError {
    TooShort(usize),
    TooShortV2(usize),
    TooShortV3(usize),
    InvalidVersion(i32),
    // InvalidStartLayerIndex(start_layer, num_layers)
    InvalidStartLayerIndex(i32, i32),
    // InvalidNumLayers(start_layer, num_layers)
    InvalidNumLayers(i32, i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DdraceLayerSoundsError {
    TooShort(usize),
    TooShortV2(usize),
    InvalidVersion(i32),
    InvalidSoundIndex(i32),
    InvalidNumSources(i32),
    InvalidDataIndex(i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LayerQuadsError {
    TooShort(usize),
    TooShortV2(usize),
    InvalidVersion(i32),
    InvalidImageIndex(i32),
    InvalidNumQuads(i32),
    InvalidDataIndex(i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ColorComponent {
    Red,
    Green,
    Blue,
    Alpha,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LayerTilemapError {
    TooShort(usize),
    TooShortV2(usize),
    TooShortV3(usize),
    TooShortRaceTeleport(usize),
    TooShortRaceSpeedup(usize),
    TooShortDdraceFront(usize),
    TooShortDdraceSwitch(usize),
    TooShortDdraceTune(usize),
    InvalidVersion(i32),
    InvalidColor(ColorComponent, i32),
    InvalidColorEnvelopeIndex(i32),
    InvalidImageIndex(i32),
    InvalidDataIndex(i32),
    InvalidRaceTeleportDataIndex(i32),
    InvalidRaceSpeedupDataIndex(i32),
    InvalidDdraceFrontDataIndex(i32),
    InvalidDdraceSwitchDataIndex(i32),
    InvalidDdraceTuneDataIndex(i32),
    InvalidFlags(i32),
    InvalidWidth(i32),
    InvalidHeight(i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LayerError {
    Tilemap(LayerTilemapError),
    Quads(LayerQuadsError),
    DdraceSounds(DdraceLayerSoundsError),
    TooShort(usize),
    InvalidFlags(i32),
    InvalidType(i32),
}

impl From<LayerTilemapError> for LayerError {
    fn from(e: LayerTilemapError) -> LayerError {
        LayerError::Tilemap(e)
    }
}

impl From<LayerQuadsError> for LayerError {
    fn from(e: LayerQuadsError) -> LayerError {
        LayerError::Quads(e)
    }
}

impl From<DdraceLayerSoundsError> for LayerError {
    fn from(e: DdraceLayerSoundsError) -> LayerError {
        LayerError::DdraceSounds(e)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImageError {
    TooShort(usize),
    InvalidVersion(i32),
    InvalidDataIndex(i32),
    InvalidWidth(i32),
    InvalidHeight(i32),
    InvalidNameIndex(i32),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InfoError {
    TooShort(usize),
    InvalidVersion(i32),
    InvalidAuthorIndex(i32),
    InvalidVersionIndex(i32),
    InvalidCreditsIndex(i32),
    InvalidLicenseIndex(i32),
    InvalidSettingsIndex(i32),
}

impl From<InfoError> for Error {
    fn from(e: InfoError) -> Error {
        Error::Info(e)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    Group(usize, GroupError),
    Layer(usize, LayerError),
    Image(usize, ImageError),
    Info(InfoError),

    InconsistentGameLayerDimensions,
    InvalidTilesLength(usize),
    InvalidTeleTilesLength(usize),
    InvalidTuneTilesLength(usize),
    InvalidVersion(i32),
    MalformedImageName(usize),
    // InvalidTilesDimensions(length, width, height)
    InvalidTilesDimensions(usize, u32, u32),
    // InvalidTeleTilesDimensions(length, width, height)
    InvalidTeleTilesDimensions(usize, u32, u32),
    // InvalidTuneTilesDimensions(length, width, height)
    InvalidTuneTilesDimensions(usize, u32, u32),
    EmptyVersion,
    MissingVersion,
    MissingInfo,
    InvalidStringMissingNullTermination,
    InvalidStringNullTermination,
    InvalidSettingsMissingNullTermination,
    NoGameLayer,
    TooManyGameGroups,
    TooManyGameLayers,
}

pub const MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS_LEGACY: i32 = 9;
"""

def make_items(items):
    MEMBER_NORMAL=re.compile(r'^(?P<name>[a-z_]+)$')
    MEMBER_ARRAY=re.compile(r'^(?P<name>[a-z_]+)\[(?P<size>[1-9][0-9]*)\]$')
    MEMBER_STRING=re.compile(r'^(?P<name>[a-z_]+)\[(?P<size>[1-9][0-9]*)s\]$')

    result = []
    for (type_id, name, versions) in items:
        result_versions = []
        for version in versions:
            result_version = []
            for member in version:
                m = MEMBER_NORMAL.match(member)
                if m is not None:
                    result_version.append((m.group('name'), None, None))
                else:
                    m = MEMBER_ARRAY.match(member)
                    if m is not None:
                        result_version.append((m.group('name'), int(m.group('size')), None))
                    else:
                        m = MEMBER_STRING.match(member)
                        if m is not None:
                            result_version.append((m.group('name'), int(m.group('size')), 's'))
                        else:
                            raise ValueError("Invalid member '{}'.".format(member))
            result_versions.append(result_version)
        result.append((type_id, name, result_versions))

    return result

def struct_name(name, i):
    return "MapItem{}V{}".format(name.title().replace('_', ''), i + 1)

def generate_header():
    return header

def generate_constants(items):
    result = []
    for (type_id, name, _) in items:
        # TODO: Remove this weird special-casing.
        if name.startswith("layer_v1_"):
            constant_type = "i32"
        else:
            constant_type = "u16"
        result.append("pub const MAP_ITEMTYPE_{}: {} = {};".format(name.upper(), constant_type, type_id))
    result.append("")
    return "\n".join(result)

def generate_structs(items):
    result = []
    for (_, name, versions) in items:
        for (i, version) in enumerate(versions):
            result.append("#[derive(Clone, Copy)]")
            result.append("#[repr(C)]")
            if version:
                result.append("pub struct {s} {{".format(s=struct_name(name, i)))
                for (member, size, _) in version:
                    if size is None:
                        result.append("    pub {}: i32,".format(member))
                    else:
                        result.append("    pub {}: [i32; {}],".format(member, size))
                result.append("}")
            else:
                result.append("pub struct {s};".format(s=struct_name(name, i)))
            result.append("")
    return "\n".join(result)

def generate_impl_unsafe_i32_only(items):
    result = []
    for (_, name, versions) in items:
        for (i, version) in enumerate(versions):
            result.append("unsafe impl OnlyI32 for {s} {{ }}".format(s=struct_name(name, i)))
    result.append("")
    return "\n".join(result)

def generate_impl_map_item(items):
    result = []
    for (_, name, versions) in items:
        offset = 1
        for (i, version) in enumerate(versions):
            ignore_version = False
            if name == "layer":
                ignore_version = True
            elif i == 1 and name == "info":
                ignore_version = True
            ignore_version = "true" if ignore_version else "false"
            result.append("""\
impl MapItem for {s} {{ \
fn version() -> i32 {{ {v} }} \
fn offset() -> usize {{ {o} }} \
fn ignore_version() -> bool {{ {iv} }} \
}}""".format(s=struct_name(name, i), v=i+1, o=offset, iv=ignore_version))
            for (_, size, _) in version:
                if size is None:
                    offset += 1
                else:
                    offset += size
    result.append("")
    return "\n".join(result)

def generate_impl_string(items):
    result = []
    for (_, name, versions) in items:
        offset = 1
        for (i, version) in enumerate(versions):
            for (member, size, type) in version:
                if size is None or type is None:
                    continue
                if type != 's':
                    raise ValueError("Invalid type: {}".format(type))
                result.append("""\
impl {s} {{
    pub fn {m}_get(&self) -> [u8; {num_bytes}] {{
        let mut result = [0u8; {num_bytes}];
        i32s_to_bytes(&mut result, &self.{m});
        result[{num_bytes}-1] = 0;
        result
    }}
}}""".format(s=struct_name(name, i), m=member, num_bytes=size*4))

    result.append("")
    return "\n".join(result)

def generate_impl_debug(items):
    result = []
    for (_, name, versions) in items:
        for (i, version) in enumerate(versions):
            result.append("impl fmt::Debug for {s} {{".format(s=struct_name(name, i)))
            result.append("    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {")
            first = ""
            for (member, size, type) in version:
                if size is None or type is None:
                    result.append("        write!(_f, \"{first}{m}={{:?}}\", self.{m})?;".format(m=member, first=first));
                else:
                    if type != 's':
                        raise ValueError("Invalid type: {}".format(type))
                    result.append("        write!(_f, \"{first}{m}={{:?}}\", String::from_utf8_lossy(bytes_to_string(&self.{m}_get())))?;".format(m=member, first=first));
                first = " "
            result.append("        Ok(())")
            result.append("    }")
            result.append("}")
    return "\n".join(result)

def preprocess_layer_v1_items(items):
    return [(id, "layer_v1_" + name, versions) for (id, name, versions) in items]

def main():
    items = make_items(ITEMS)
    layer_v1_items = make_items(preprocess_layer_v1_items(LAYER_V1_ITEMS))
    steps = [
        generate_constants,
        generate_structs,
        generate_impl_unsafe_i32_only,
        generate_impl_map_item,
        generate_impl_string,
        generate_impl_debug,
    ]

    print(generate_header())
    for i in (items, layer_v1_items):
        for g in steps:
            print(g(i))

if __name__ == '__main__':
    import sys
    sys.exit(main())
