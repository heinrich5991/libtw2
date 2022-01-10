use common::num::Cast;
use common::vec;
use datafile as df;
use ndarray::Array2;
use std::io;
use std::mem;
use std::ops;
use std::path::Path;

use format::Error as MapError;
use format::MapItem;
use format::MapItemExt;
use format;

#[derive(Debug)]
pub enum Error {
    Map(MapError),
    Df(df::Error),
}

impl From<MapError> for Error {
    fn from(err: MapError) -> Error {
        Error::Map(err)
    }
}

impl From<df::Error> for Error {
    fn from(err: df::Error) -> Error {
        Error::Df(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Df(df::Error::Io(err))
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Clipping {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone)]
pub struct Group {
    pub offset_x: i32,
    pub offset_y: i32,
    pub parallax_x: i32,
    pub parallax_y: i32,
    pub layer_indices: ops::Range<usize>,
    pub clipping: Option<Clipping>,
    pub name: [u8; 12],
}

fn get_index_impl(index: i32, indices: ops::Range<usize>) -> Option<usize> {
    let index = unwrap_or_return!(index.try_usize(), None) + indices.start;
    if !(index < indices.end) {
        return None;
    }
    Some(index)
}

fn get_index<E, II>(index: i32, indices: ops::Range<usize>, invalid_index: II)
    -> Result<usize, E>
    where II: FnOnce(i32) -> E,
{
    get_index_impl(index, indices).ok_or_else(|| invalid_index(index))
}

fn get_index_opt<E, II>(index: i32, indices: ops::Range<usize>, invalid_index: II)
    -> Result<Option<usize>, E>
    where II: FnOnce(i32) -> E,
{
    if index == -1 {
        return Ok(None);
    }
    get_index_impl(index, indices).ok_or_else(|| invalid_index(index)).map(Some)
}

trait MapItemExtInternal: MapItem {
    fn optional<E, TS>(slice: &[i32], too_short: TS)
        -> Result<Option<&Self>, E>
        where TS: FnOnce(usize) -> E,
    {
        Self::from_slice(slice).map_err(|_| too_short(slice.len()))
    }
    fn mandatory<E, TS, IV>(slice: &[i32], too_short: TS, invalid_version: IV)
        -> Result<&Self, E>
        where TS: FnOnce(usize) -> E,
              IV: FnOnce(i32) -> E,
    {
        Self::optional(slice, too_short)?.ok_or_else(|| invalid_version(slice[0]))
    }
    fn optional_rest<E, TS>(slice: &[i32], too_short: TS)
        -> Result<Option<(&Self, &[i32])>, E>
        where TS: FnOnce(usize) -> E,
    {
        Self::from_slice_rest(slice).map_err(|_| too_short(slice.len()))
    }
    fn mandatory_rest<E, TS, IV>(slice: &[i32], too_short: TS, invalid_version: IV)
        -> Result<(&Self, &[i32]), E>
        where TS: FnOnce(usize) -> E,
              IV: FnOnce(i32) -> E,
    {
        Self::optional_rest(slice, too_short)?.ok_or_else(|| invalid_version(slice[0]))
    }
}

impl<T: MapItem> MapItemExtInternal for T { }

trait AugmentResult {
    type AddIndex;
    fn add_index(self, index: usize) -> Self::AddIndex;
}

impl<T> AugmentResult for Result<T, format::GroupError> {
    type AddIndex = Result<T, MapError>;
    fn add_index(self, index: usize) -> Result<T, MapError> {
        self.map_err(|e| MapError::Group(index, e))
    }
}

impl<T> AugmentResult for Result<T, format::LayerError> {
    type AddIndex = Result<T, MapError>;
    fn add_index(self, index: usize) -> Result<T, MapError> {
        self.map_err(|e| MapError::Layer(index, e))
    }
}

impl<T> AugmentResult for Result<T, format::ImageError> {
    type AddIndex = Result<T, MapError>;
    fn add_index(self, index: usize) -> Result<T, MapError> {
        self.map_err(|e| MapError::Image(index, e))
    }
}

pub struct LayerTilesIndex {
    data_index: usize,
    width: u32,
    height: u32,
}

impl Group {
    // TODO: Overlong raw?
    fn from_raw(raw: &[i32], layer_indices: ops::Range<usize>)
        -> Result<Group, format::GroupError>
    {
        use format::GroupError::*;

        let v1 = format::MapItemGroupV1::mandatory(raw, TooShort, InvalidVersion)?;
        let v2 = format::MapItemGroupV2::optional(raw, TooShort)?;
        let v3 = format::MapItemGroupV3::optional(raw, TooShort)?;

        let sl = InvalidStartLayerIndex(v1.start_layer, v1.num_layers);
        let nl = InvalidNumLayers(v1.start_layer, v1.num_layers);

        let layers_start = layer_indices.start + v1.start_layer.try_usize().ok_or(sl)?;
        if layers_start > layer_indices.end {
            return Err(sl);
        }
        let layers_end = layers_start + v1.num_layers.try_usize().ok_or(nl)?;
        if layers_end > layer_indices.end {
            return Err(nl);
        }
        let clipping = v2.and_then(|v2| {
            if v2.use_clipping != 0 {
                Some(Clipping {
                    x: v2.clip_x,
                    y: v2.clip_y,
                    width: v2.clip_w,
                    height: v2.clip_h,
                })
            } else {
                None
            }
        });
        let name = v3.map(|v3| v3.name_get()).unwrap_or([0; 12]);
        Ok(Group {
            offset_x: v1.offset_x,
            offset_y: v1.offset_y,
            parallax_x: v1.parallax_x,
            parallax_y: v1.parallax_y,
            layer_indices: layers_start..layers_end,
            clipping: clipping,
            name: name,
        })
    }
}

#[derive(Clone, Copy)]
pub struct DdraceLayerSounds {
    pub num_sources: usize,
    pub data: usize,
    pub sound: Option<usize>,
    pub legacy: bool,
    pub name: [u8; 12],
}

impl DdraceLayerSounds {
    fn from_raw(
        raw: &[i32],
        data_indices: ops::Range<usize>,
        sound_indices: ops::Range<usize>,
        legacy: bool
    ) -> Result<DdraceLayerSounds, format::DdraceLayerSoundsError>
    {
        use format::DdraceLayerSoundsError::*;
        let v1 = format::MapItemLayerV1DdraceSoundsV1::mandatory(raw, TooShort, InvalidVersion)?;
        if !legacy {
            format::MapItemLayerV1DdraceSoundsV2::mandatory(raw, TooShortV2, InvalidVersion)?;
        }
        Ok(DdraceLayerSounds {
            num_sources: v1.num_sources.try_usize().ok_or(InvalidNumSources(v1.num_sources))?,
            data: get_index(v1.data, data_indices, InvalidDataIndex)?,
            sound: get_index_opt(v1.sound, sound_indices, InvalidSoundIndex)?,
            legacy: legacy,
            name: v1.name_get(),
        })
    }
}

#[derive(Clone, Copy)]
pub struct LayerQuads {
    pub num_quads: usize,
    pub data: usize,
    pub image: Option<usize>,
    pub name: [u8; 12],
}

impl LayerQuads {
    fn from_raw(
        raw: &[i32],
        data_indices: ops::Range<usize>,
        image_indices: ops::Range<usize>
    ) -> Result<LayerQuads, format::LayerQuadsError>
    {
        use format::LayerQuadsError::*;

        let v1 = format::MapItemLayerV1QuadsV1::mandatory(raw, TooShort, InvalidVersion)?;
        let v2 = format::MapItemLayerV1QuadsV2::optional(raw, TooShortV2)?;
        let name = v2.map(|v2| v2.name_get()).unwrap_or([0; 12]);
        Ok(LayerQuads {
            num_quads: v1.num_quads.try_usize().ok_or(InvalidNumQuads(v1.num_quads))?,
            data: get_index(v1.data, data_indices, InvalidDataIndex)?,
            image: get_index_opt(v1.image, image_indices, InvalidImageIndex)?,
            name: name,
        })
    }
}

#[derive(Clone, Copy)]
pub enum LayerTilemapType {
    // Normal(normal)
    Normal(LayerTilemapNormal),
    // Game(data)
    Game(usize),
    // RaceTeleport(data, zeroes)
    RaceTeleport(usize, usize),
    // RaceSpeedup(data, zeroes)
    RaceSpeedup(usize, usize),
    // DdraceFront(data, zeroes)
    DdraceFront(usize, usize),
    // DdraceSwitch(data, zeroes)
    DdraceSwitch(usize, usize),
    // DdraceTune(data, zeroes)
    DdraceTune(usize, usize),
}

#[derive(Clone, Copy)]
pub struct LayerTilemapNormal {
    pub color: Color,
    pub color_env_and_offset: Option<(usize, i32)>,
    pub image: Option<usize>,
    pub data: usize,
}

impl LayerTilemapType {
    pub fn to_normal(&self) -> Option<&LayerTilemapNormal> {
        match *self {
            LayerTilemapType::Normal(ref n) => Some(n),
            _ => None,
        }
    }
    pub fn tiles(&self) -> Option<usize> {
        match *self {
            LayerTilemapType::Normal(ref n) => Some(n.data),
            LayerTilemapType::Game(d) => Some(d),
            LayerTilemapType::DdraceFront(d, _) => Some(d),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct LayerTilemap {
    pub width: u32,
    pub height: u32,
    pub type_: LayerTilemapType,
    pub name: [u8; 12],
}

impl LayerTilemap {
    pub fn tiles(&self, data_index: usize) -> LayerTilesIndex {
        LayerTilesIndex {
            data_index: data_index,
            width: self.width,
            height: self.height,
        }
    }
}

impl LayerTilemap {
    fn from_raw(
        raw: &[i32],
        data_indices: ops::Range<usize>,
        envelope_indices: ops::Range<usize>,
        image_indices: ops::Range<usize>
    ) -> Result<LayerTilemap, format::LayerTilemapError>
    {
        use format::ColorComponent::*;
        use format::LayerTilemapError::*;

        fn extra<TS>(raw: &[i32], version: i32, flags: u32, too_short: TS)
            -> Result<&format::MapItemLayerV1TilemapExtraRace, format::LayerTilemapError>
            where TS: FnOnce(usize) -> format::LayerTilemapError
        {
            format::MapItemLayerV1TilemapExtraRace::from_slice(raw, version, flags)
                .ok_or_else(|| too_short(raw.len()))
        }

        let v0 = format::MapItemLayerV1CommonV0::mandatory(raw, TooShort, InvalidVersion)?;
        let v2 = format::MapItemLayerV1TilemapV2::mandatory(raw, TooShortV2, InvalidVersion)?;
        let v3 = format::MapItemLayerV1TilemapV3::optional(raw, TooShortV3)?;
        let flags = v2.flags as u32;

        // TODO: Standard settings for game group.
        let color = Color {
            red: v2.color_red.try_u8().ok_or(InvalidColor(Red, v2.color_red))?,
            green: v2.color_green.try_u8().ok_or(InvalidColor(Green, v2.color_green))?,
            blue: v2.color_blue.try_u8().ok_or(InvalidColor(Blue, v2.color_blue))?,
            alpha: v2.color_alpha.try_u8().ok_or(InvalidColor(Alpha, v2.color_alpha))?,
        };
        let color_env_and_offset = if v2.color_env == -1 {
            None
        } else {
            let index = get_index(
                v2.color_env,
                envelope_indices.clone(),
                InvalidColorEnvelopeIndex,
            )?;
            Some((index, v2.color_env_offset))
        };
        let image = get_index_opt(v2.image, image_indices, InvalidImageIndex)?;
        let data = get_index(v2.data, data_indices.clone(), InvalidDataIndex)?;
        let mut normal = false;
        let type_ = match flags {
            0 => {
                normal = true;
                LayerTilemapType::Normal(LayerTilemapNormal {
                    color: color,
                    color_env_and_offset: color_env_and_offset,
                    image: image,
                    data: data,
                })
            }
            format::TILELAYERFLAG_GAME => {
                LayerTilemapType::Game(data)
            }
            format::TILELAYERFLAG_TELEPORT => {
                LayerTilemapType::RaceTeleport(
                    get_index(
                        extra(raw, v0.version, flags, TooShortRaceTeleport)?.data,
                        data_indices.clone(),
                        InvalidRaceTeleportDataIndex,
                    )?,
                    data,
                )
            }
            format::TILELAYERFLAG_SPEEDUP => {
                LayerTilemapType::RaceSpeedup(
                    get_index(
                        extra(raw, v0.version, flags, TooShortRaceSpeedup)?.data,
                        data_indices.clone(),
                        InvalidRaceSpeedupDataIndex,
                    )?,
                    data,
                )
            }
            format::TILELAYERFLAG_FRONT => {
                LayerTilemapType::DdraceFront(
                    get_index(
                        extra(raw, v0.version, flags, TooShortDdraceFront)?.data,
                        data_indices.clone(),
                        InvalidDdraceFrontDataIndex,
                    )?,
                    data,
                )
            }
            format::TILELAYERFLAG_SWITCH => {
                LayerTilemapType::DdraceSwitch(
                    get_index(
                        extra(raw, v0.version, flags, TooShortDdraceSwitch)?.data,
                        data_indices.clone(),
                        InvalidDdraceSwitchDataIndex,
                    )?,
                    data,
                )
            }
            format::TILELAYERFLAG_TUNE => {
                LayerTilemapType::DdraceTune(
                    get_index(
                        extra(raw, v0.version, flags, TooShortDdraceTune)?.data,
                        data_indices.clone(),
                        InvalidDdraceTuneDataIndex,
                    )?,
                    data,
                )
            }
            _ => return Err(InvalidFlags(v2.flags)),
        };
        if !normal {
            // TODO: Do some checking on the other fields.
        }
        let name = v3.map(|v3| v3.name_get()).unwrap_or([0; 12]);
        let width = v2.width.try_u32().ok_or(InvalidWidth(v2.width))?;
        let height = v2.height.try_u32().ok_or(InvalidHeight(v2.height))?;

        if width == 0 { return Err(InvalidWidth(v2.width)); }
        if height == 0 { return Err(InvalidHeight(v2.height)); }

        Ok(LayerTilemap {
            width: width,
            height: height,
            type_: type_,
            name: name,
        })
    }
}

#[derive(Clone, Copy)]
pub struct Layer {
    pub detail: bool,
    pub t: LayerType,
}

#[derive(Clone, Copy)]
pub enum LayerType {
    Quads(LayerQuads),
    Tilemap(LayerTilemap),
    DdraceSounds(DdraceLayerSounds),
}

impl Layer {
    fn from_raw(
        raw: &[i32],
        data_indices: ops::Range<usize>,
        envelope_indices: ops::Range<usize>,
        image_indices: ops::Range<usize>,
        sound_indices: ops::Range<usize>,
    ) -> Result<Layer, format::LayerError> {
        use format::LayerError::*;

        let (v1, rest) = format::MapItemLayerV1::mandatory_rest(
            raw,
            TooShort,
            // MapItemLayerV1 doesn't check the version as it is not set by the
            // reference implementation, and contains arbitrary garbage.
            |_| unreachable!(),
        )?;
        let flags = v1.flags as u32;
        if flags & !format::LAYERFLAGS_ALL != 0 {
            return Err(InvalidFlags(v1.flags));
        }
        let t = match v1.type_ {
            format::MAP_ITEMTYPE_LAYER_V1_TILEMAP =>
                LayerType::Tilemap(LayerTilemap::from_raw(
                    rest, data_indices, envelope_indices, image_indices
                )?),
            format::MAP_ITEMTYPE_LAYER_V1_QUADS =>
                LayerType::Quads(LayerQuads::from_raw(
                    rest, data_indices, image_indices
                )?),
            format::MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS
                | format::MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS_LEGACY
            =>  LayerType::DdraceSounds(DdraceLayerSounds::from_raw(
                    rest, data_indices, sound_indices,
                    v1.type_ != format::MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS,
                )?),
            _ => return Err(InvalidType(v1.type_)),
        };
        Ok(Layer {
            detail: flags & format::LAYERFLAG_DETAIL != 0,
            t: t,
        })
    }
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub name: usize,
    pub data: Option<usize>,
}

impl Image {
    fn from_raw(raw: &[i32], data_indices: ops::Range<usize>)
        -> Result<Image, format::ImageError>
    {
        use format::ImageError::*;

        let v1 = format::MapItemImageV1::mandatory(raw, TooShort, InvalidVersion)?;
        // WARN if external is something other than 0,1
        let data = if v1.external != 0 {
            None
        } else {
            Some(get_index(v1.data, data_indices.clone(), InvalidDataIndex)?)
        };
        Ok(Image {
            width: v1.width.try_u32().ok_or(InvalidWidth(v1.width))?,
            height: v1.height.try_u32().ok_or(InvalidHeight(v1.height))?,
            name: get_index(v1.name, data_indices.clone(), InvalidNameIndex)?,
            data: data,
        })
    }
}

pub struct GameLayers {
    pub group: Group,
    pub width: u32,
    pub height: u32,
    pub game_raw: usize,
    pub teleport_raw: Option<usize>,
    pub speedup_raw: Option<usize>,
    pub front_raw: Option<usize>,
    pub switch_raw: Option<usize>,
    pub tune_raw: Option<usize>,
}

impl GameLayers {
    fn layer_tiles_index(&self, data_index: usize) -> LayerTilesIndex {
        LayerTilesIndex {
            data_index: data_index,
            width: self.width,
            height: self.height,
        }
    }
    pub fn game(&self) -> LayerTilesIndex {
        self.layer_tiles_index(self.game_raw)
    }
    pub fn teleport(&self) -> Option<LayerTilesIndex> {
        self.teleport_raw.map(|i| self.layer_tiles_index(i))
    }
    pub fn speedup(&self) -> Option<LayerTilesIndex> {
        self.speedup_raw.map(|i| self.layer_tiles_index(i))
    }
    pub fn front(&self) -> Option<LayerTilesIndex> {
        self.front_raw.map(|i| self.layer_tiles_index(i))
    }
    pub fn switch(&self) -> Option<LayerTilesIndex> {
        self.switch_raw.map(|i| self.layer_tiles_index(i))
    }
    pub fn tune(&self) -> Option<LayerTilesIndex> {
        self.tune_raw.map(|i| self.layer_tiles_index(i))
    }
}

pub struct Info {
    pub author: Option<usize>,
    pub version: Option<usize>,
    pub credits: Option<usize>,
    pub license: Option<usize>,
    pub settings: Option<usize>,
}

impl Info {
    pub fn from_raw(raw: &[i32], data_indices: ops::Range<usize>)
        -> Result<Info, format::InfoError>
    {
        use format::InfoError::*;

        let v1 = format::MapItemInfoV1::mandatory(raw, TooShort, InvalidVersion)?;
        let v2 = format::MapItemInfoV2::from_slice(raw).ok().and_then(|x| x);
        Ok(Info {
            author: get_index_opt(v1.author, data_indices.clone(), InvalidAuthorIndex)?,
            version: get_index_opt(v1.version, data_indices.clone(), InvalidVersionIndex)?,
            credits: get_index_opt(v1.credits, data_indices.clone(), InvalidCreditsIndex)?,
            license: get_index_opt(v1.license, data_indices.clone(), InvalidLicenseIndex)?,
            settings: if let Some(v2) = v2 {
                get_index_opt(v2.settings, data_indices.clone(), InvalidSettingsIndex)?
            } else {
                None
            },
        })
    }
}

pub struct Settings {
    pub raw: Vec<u8>,
}

#[derive(Clone, Copy)]
pub struct SettingsIter<'a> {
    settings: &'a [u8],
    pos: usize,
}

impl Settings {
    pub fn iter(&self) -> SettingsIter {
        SettingsIter {
            settings: &self.raw,
            pos: 0,
        }
    }
}

impl<'a> Iterator for SettingsIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        let len = self.settings[self.pos..].iter().position(|&b| b == 0)?;
        let cur_pos = self.pos;
        self.pos += len + 1;
        Some(&self.settings[cur_pos..cur_pos + len])
    }
}

pub struct Reader {
    pub reader: df::Reader,
}

impl Reader {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Reader, Error> {
        fn inner(path: &Path) -> Result<Reader, Error> {
            Ok(Reader::from_datafile(df::Reader::open(path)?))
        }
        inner(path.as_ref())
    }
    pub fn from_datafile(reader: df::Reader) -> Reader {
        Reader { reader: reader }
    }
    pub fn check_version(&self) -> Result<(), MapError> {
        let version = self.version()?;
        if version != 1 {
            return Err(MapError::InvalidVersion(version));
        }
        Ok(())
    }
    pub fn version(&self) -> Result<i32, MapError> {
        let raw = self.reader.find_item(format::MAP_ITEMTYPE_VERSION, 0)
            .ok_or(MapError::MissingVersion)?;
        let v0 = format::MapItemCommonV0::mandatory(
            raw.data,
            |_| MapError::EmptyVersion,
            // MapItemCommonV0 doesn't check the version.
            |_| unreachable!(),
        )?;
        Ok(v0.version)
    }
    pub fn info(&self) -> Result<Info, MapError> {
        let raw = self.reader.find_item(format::MAP_ITEMTYPE_INFO, 0)
            .ok_or(MapError::MissingInfo)?;
        let data_indices = 0..self.reader.num_data();
        Ok(Info::from_raw(raw.data, data_indices)?)
    }
    pub fn group_indices(&self) -> ops::Range<usize> {
        self.reader.item_type_indices(format::MAP_ITEMTYPE_GROUP)
    }
    pub fn group(&self, index: usize) -> Result<Group, MapError> {
        // Doesn't fail if index is from Reader::groups().
        let raw = self.reader.item(index);
        assert!(raw.type_id == format::MAP_ITEMTYPE_GROUP);
        let layer_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_LAYER);
        Group::from_raw(raw.data, layer_indices)
            .add_index(index)
    }
    pub fn layer(&self, index: usize) -> Result<Layer, MapError> {
        // Doesn't fail if index is from Reader::group().
        let raw = self.reader.item(index);
        assert!(raw.type_id == format::MAP_ITEMTYPE_LAYER);
        let data_indices = 0..self.reader.num_data();
        let envelope_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_ENVELOPE);
        let image_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_IMAGE);
        let sound_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_DDRACE_SOUND);
        Layer::from_raw(raw.data, data_indices, envelope_indices, image_indices, sound_indices)
            .add_index(index)
    }
    pub fn image(&self, index: usize) -> Result<Image, MapError> {
        let raw = self.reader.item(index);
        let data_indices = 0..self.reader.num_data();
        Image::from_raw(raw.data, data_indices)
            .add_index(index)
    }
    pub fn image_data(&mut self, data_index: usize) -> Result<Vec<u8>, Error> {
        Ok(self.reader.read_data(data_index)?)
    }
    pub fn game_layers(&self) -> Result<GameLayers, MapError> {
        fn put<T>(opt: &mut Option<T>, new: T) -> Result<(), MapError> {
            match mem::replace(opt, Some(new)) {
                None => Ok(()),
                Some(_) => Err(MapError::TooManyGameLayers),
            }
        }
        let mut group_index_width_height = None;
        let mut game_group = None;
        let mut game = None;
        let mut teleport = None;
        let mut speedup = None;
        let mut front = None;
        let mut switch = None;
        let mut tune = None;
        for i in self.group_indices() {
            // TODO: Just skip this group?
            let group = self.group(i)?;
            for k in group.layer_indices.clone() {
                // TODO: Just as above, skip this layer in case of failure?
                let layer = self.layer(k)?;
                if let LayerType::Tilemap(tilemap) = layer.t {
                    match tilemap.type_ {
                        LayerTilemapType::Normal(_) => continue,
                        LayerTilemapType::Game(d) => put(&mut game, d)?,
                        LayerTilemapType::RaceTeleport(d, _) => put(&mut teleport, d)?,
                        LayerTilemapType::RaceSpeedup(d, _) => put(&mut speedup, d)?,
                        LayerTilemapType::DdraceFront(d, _) => put(&mut front, d)?,
                        LayerTilemapType::DdraceSwitch(d, _) => put(&mut switch, d)?,
                        LayerTilemapType::DdraceTune(d, _) => put(&mut tune, d)?,
                    }
                    match group_index_width_height {
                        Some((k, _, _)) if i != k => {
                            return Err(MapError::TooManyGameGroups);
                        }
                        Some((_, w, h)) if w != tilemap.width || h != tilemap.height => {
                            return Err(MapError::InconsistentGameLayerDimensions);
                        }
                        Some(_) => {},
                        None => {
                            game_group = Some(group.clone());
                            group_index_width_height = Some((i, tilemap.width, tilemap.height));
                        }
                    }
                }
            }
        }
        let game = match game {
            Some(g) => g,
            None => return Err(MapError::NoGameLayer),
        };
        let (_, width, height) = group_index_width_height.unwrap();
        let group = game_group.unwrap();
        Ok(GameLayers {
            group: group,
            width: width,
            height: height,
            game_raw: game,
            teleport_raw: teleport,
            speedup_raw: speedup,
            front_raw: front,
            switch_raw: switch,
            tune_raw: tune,
        })
    }
    pub fn image_name(&mut self, data_index: usize) -> Result<Vec<u8>, Error> {
        let mut raw = self.reader.read_data(data_index)?;
        if raw.pop() != Some(0) {
            return Err(Error::Map(MapError::MalformedImageName(data_index)))
        }
        for &c in &raw {
            match c {
                b'/' | b'\\' | b'\0' =>
                    return Err(Error::Map(MapError::MalformedImageName(data_index))),
                _ => {}
            }
        }
        Ok(raw)
    }
    pub fn tune_layer_tiles_raw(&mut self, data_index: usize)
        -> Result<Vec<format::TuneTile>, Error>
    {
        let raw = self.reader.read_data(data_index)?;
        if raw.len() % mem::size_of::<format::TuneTile>() != 0 {
            return Err(Error::Map(MapError::InvalidTuneTilesLength(raw.len())));
        }
        let tiles: Vec<format::TuneTile> = unsafe { vec::transmute(raw) };
        Ok(tiles)
    }
    pub fn tune_layer_tiles(&mut self, index: LayerTilesIndex)
        -> Result<Array2<format::TuneTile>, Error>
    {
        let LayerTilesIndex { data_index, width, height } = index;
        let tiles = self.tune_layer_tiles_raw(data_index)?;
        let len = tiles.len();
        Ok(Array2::from_shape_vec((height.usize(), width.usize()), tiles)
            .map_err(|_| MapError::InvalidTilesDimensions(len, height, width))?)
    }
    pub fn speedup_layer_tiles_raw(&mut self, data_index: usize)
        -> Result<Vec<format::SpeedupTile>, Error>
    {
        let raw = self.reader.read_data(data_index)?;
        if raw.len() % mem::size_of::<format::SpeedupTile>() != 0 {
            return Err(Error::Map(MapError::InvalidTeleTilesLength(raw.len())));
        }
        let tiles: Vec<format::SpeedupTile> = unsafe { vec::transmute(raw) };
        Ok(tiles)
    }
    pub fn speedup_layer_tiles(&mut self, index: LayerTilesIndex)
        -> Result<Array2<format::SpeedupTile>, Error>
    {
        let LayerTilesIndex { data_index, width, height } = index;
        let tiles = self.speedup_layer_tiles_raw(data_index)?;
        let len = tiles.len();
        Ok(Array2::from_shape_vec((height.usize(), width.usize()), tiles)
            .map_err(|_| MapError::InvalidTilesDimensions(len, height, width))?)
    }
    pub fn switch_layer_tiles_raw(&mut self, data_index: usize)
        -> Result<Vec<format::SwitchTile>, Error>
    {
        let raw = self.reader.read_data(data_index)?;
        if raw.len() % mem::size_of::<format::SwitchTile>() != 0 {
            return Err(Error::Map(MapError::InvalidTeleTilesLength(raw.len())));
        }
        let tiles: Vec<format::SwitchTile> = unsafe { vec::transmute(raw) };
        Ok(tiles)
    }
    pub fn switch_layer_tiles(&mut self, index: LayerTilesIndex)
        -> Result<Array2<format::SwitchTile>, Error>
    {
        let LayerTilesIndex { data_index, width, height } = index;
        let tiles = self.switch_layer_tiles_raw(data_index)?;
        let len = tiles.len();
        Ok(Array2::from_shape_vec((height.usize(), width.usize()), tiles)
            .map_err(|_| MapError::InvalidTilesDimensions(len, height, width))?)
    }
    pub fn tele_layer_tiles_raw(&mut self, data_index: usize)
        -> Result<Vec<format::TeleTile>, Error>
    {
        let raw = self.reader.read_data(data_index)?;
        if raw.len() % mem::size_of::<format::TeleTile>() != 0 {
            return Err(Error::Map(MapError::InvalidTeleTilesLength(raw.len())));
        }
        let tiles: Vec<format::TeleTile> = unsafe { vec::transmute(raw) };
        Ok(tiles)
    }
    pub fn tele_layer_tiles(&mut self, index: LayerTilesIndex)
        -> Result<Array2<format::TeleTile>, Error>
    {
        let LayerTilesIndex { data_index, width, height } = index;
        let tiles = self.tele_layer_tiles_raw(data_index)?;
        let len = tiles.len();
        Ok(Array2::from_shape_vec((height.usize(), width.usize()), tiles)
            .map_err(|_| MapError::InvalidTilesDimensions(len, height, width))?)
    }
    pub fn layer_tiles_raw(&mut self, data_index: usize)
        -> Result<Vec<format::Tile>, Error>
    {
        let raw = self.reader.read_data(data_index)?;
        if raw.len() % mem::size_of::<format::Tile>() != 0 {
            return Err(Error::Map(MapError::InvalidTilesLength(raw.len())));
        }
        let tiles: Vec<format::Tile> = unsafe { vec::transmute(raw) };
        Ok(tiles)
    }
    pub fn layer_tiles(&mut self, index: LayerTilesIndex)
        -> Result<Array2<format::Tile>, Error>
    {
        let LayerTilesIndex { data_index, width, height } = index;
        let tiles = self.layer_tiles_raw(data_index)?;
        let len = tiles.len();
        Ok(Array2::from_shape_vec((height.usize(), width.usize()), tiles)
            .map_err(|_| MapError::InvalidTilesDimensions(len, height, width))?)
    }
    pub fn string(&mut self, data_index: usize)
        -> Result<Vec<u8>, Error>
    {
        let mut raw = self.reader.read_data(data_index)?;
        if let Some(0) = raw.pop() {
            // ok
        } else {
            return Err(format::Error::InvalidStringMissingNullTermination.into());
        }
        if raw.iter().any(|&b| b == 0) {
            return Err(format::Error::InvalidStringNullTermination.into());
        }
        Ok(raw)
    }
    pub fn settings(&mut self, data_index: usize)
        -> Result<Settings, Error>
    {
        let raw = self.reader.read_data(data_index)?;
        if raw.len() == 0 || raw[raw.len() - 1] != 0 {
            return Err(format::Error::InvalidSettingsMissingNullTermination.into());
        }
        Ok(Settings {
            raw: raw
        })
    }
}
