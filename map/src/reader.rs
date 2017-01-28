use common::vec;
use datafile as df;
use num::ToPrimitive;
use std::io;
use std::mem;
use std::ops;

use format::Error as MapError;
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

#[derive(Clone, Copy)]
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

fn get_index(index: i32, indices: ops::Range<usize>) -> Option<usize> {
    let index = unwrap_or_return!(index.to_usize(), None) + indices.start;
    if !(index < indices.end) {
        return None;
    }
    Some(index)
}

impl Group {
    // TODO: Overlong raw?
    fn from_raw(raw: &[i32], layer_indices: ops::Range<usize>) -> Result<Group,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedGroup)
        }
        let v1 = try!(e(format::MapItemGroupV1::from_slice(raw)));
        let v2 = format::MapItemGroupV2::from_slice(raw);
        let v3 = format::MapItemGroupV3::from_slice(raw);
        let layers_start = layer_indices.start + try!(e(v1.start_layer.to_usize()));
        if layers_start > layer_indices.end {
            return Err(MapError::MalformedGroup);
        }
        let layers_end = layers_start + try!(e(v1.num_layers.to_usize()));
        if layers_end > layer_indices.end {
            return Err(MapError::MalformedGroup);
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
    fn from_raw(raw: &[i32], data_indices: ops::Range<usize>, sound_indices: ops::Range<usize>, legacy: bool) -> Result<DdraceLayerSounds,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedDdraceLayerSounds)
        }
        let v1 = try!(e(format::MapItemLayerV1DdraceSoundsV1::from_slice(raw)));
        if !legacy { try!(e(format::MapItemLayerV1DdraceSoundsV2::from_slice(raw))); }
        let sound = if v1.sound == -1 {
            None
        } else {
            Some(try!(e(get_index(v1.sound, sound_indices))))
        };
        Ok(DdraceLayerSounds {
            num_sources: try!(e(v1.num_sources.to_usize())),
            data: try!(e(get_index(v1.data, data_indices))),
            sound: sound,
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
    fn from_raw(raw: &[i32], data_indices: ops::Range<usize>, image_indices: ops::Range<usize>) -> Result<LayerQuads,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedLayerQuads)
        }
        let v1 = try!(e(format::MapItemLayerV1QuadsV1::from_slice(raw)));
        let v2 = format::MapItemLayerV1QuadsV2::from_slice(raw);
        let image = if v1.image == -1 {
            None
        } else {
            Some(try!(e(get_index(v1.image, image_indices))))
        };
        let name = v2.map(|v2| v2.name_get()).unwrap_or([0; 12]);
        Ok(LayerQuads {
            num_quads: try!(e(v1.num_quads.to_usize())),
            data: try!(e(get_index(v1.data, data_indices))),
            image: image,
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
    fn from_raw(raw: &[i32], data_indices: ops::Range<usize>, image_indices: ops::Range<usize>) -> Result<LayerTilemap,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedLayerTilemap)
        }
        let v0 = try!(e(format::MapItemLayerV1CommonV0::from_slice(raw)));
        let v2 = try!(e(format::MapItemLayerV1TilemapV2::from_slice(raw)));
        let v3 = format::MapItemLayerV1TilemapV3::from_slice(raw);
        let flags = v2.flags as u32;
        let ve = format::MapItemLayerV1TilemapExtraRace::from_slice(raw, v0.version, flags);
        // TODO: Standard settings for game group.
        let color = Color {
            red: try!(e(v2.color_red.to_u8())),
            green: try!(e(v2.color_green.to_u8())),
            blue: try!(e(v2.color_blue.to_u8())),
            alpha: try!(e(v2.color_alpha.to_u8())),
        };
        let color_env_and_offset = if v2.color_env == -1 {
            None
        } else {
            Some((try!(e(get_index(v2.color_env, data_indices.clone()))), v2.color_env_offset))
        };
        let image = if v2.image == -1 {
            None
        } else {
            Some(try!(e(get_index(v2.image, image_indices))))
        };
        let data = try!(e(get_index(v2.data, data_indices.clone())));
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
                    try!(e(get_index(try!(e(ve)).data, data_indices.clone()))),
                    data,
                )
            }
            format::TILELAYERFLAG_SPEEDUP => {
                LayerTilemapType::RaceSpeedup(
                    try!(e(get_index(try!(e(ve)).data, data_indices.clone()))),
                    data,
                )
            }
            format::TILELAYERFLAG_FRONT => {
                LayerTilemapType::DdraceFront(
                    try!(e(get_index(try!(e(ve)).data, data_indices.clone()))),
                    data,
                )
            }
            format::TILELAYERFLAG_SWITCH => {
                LayerTilemapType::DdraceSwitch(
                    try!(e(get_index(try!(e(ve)).data, data_indices.clone()))),
                    data,
                )
            }
            format::TILELAYERFLAG_TUNE => {
                LayerTilemapType::DdraceTune(
                    try!(e(get_index(try!(e(ve)).data, data_indices.clone()))),
                    data,
                )
            }
            _ => return Err(MapError::InvalidLayerTilemapFlags(flags)),
        };
        if !normal {
            // TODO: Do some checking on the other fields.
        }
        let name = v3.map(|v3| v3.name_get()).unwrap_or([0; 12]);
        Ok(LayerTilemap {
            width: try!(e(v2.width.to_u32())),
            height: try!(e(v2.height.to_u32())),
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
        image_indices: ops::Range<usize>,
        sound_indices: ops::Range<usize>,
    ) -> Result<Layer,MapError> {
        let (v1, rest) = try!(format::MapItemLayerV1::from_slice_rest(raw)
                              .ok_or(MapError::MalformedLayer));
        let flags = v1.flags as u32;
        if flags & !format::LAYERFLAGS_ALL != 0 {
            return Err(MapError::InvalidLayerFlags(flags));
        }
        let t = match v1.type_ {
            format::MAP_ITEMTYPE_LAYER_V1_TILEMAP =>
                LayerType::Tilemap(try!(LayerTilemap::from_raw(
                    rest, data_indices, image_indices
                ))),
            format::MAP_ITEMTYPE_LAYER_V1_QUADS =>
                LayerType::Quads(try!(LayerQuads::from_raw(
                    rest, data_indices, image_indices
                ))),
            format::MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS
                | format::MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS_LEGACY
            =>  LayerType::DdraceSounds(try!(DdraceLayerSounds::from_raw(
                    rest, data_indices, sound_indices,
                    v1.type_ != format::MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS,
                ))),
            _ => return Err(MapError::InvalidLayerType(v1.type_)),
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
    fn from_raw(raw: &[i32], data_indices: ops::Range<usize>) -> Result<Image,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedImage)
        }
        let v1 = try!(e(format::MapItemImageV1::from_slice(raw)));
        // WARN if external is something other than 0,1
        let data = if v1.external != 0 {
            None
        } else {
            Some(try!(e(get_index(v1.data, data_indices.clone()))))
        };
        Ok(Image {
            width: try!(e(v1.width.to_u32())),
            height: try!(e(v1.height.to_u32())),
            name: try!(e(get_index(v1.name, data_indices.clone()))),
            data: data,
        })
    }
}

pub struct GameLayers {
    pub group: Group,
    pub width: u32,
    pub height: u32,
    pub game: usize,
    pub teleport: Option<usize>,
    pub speedup: Option<usize>,
    pub front: Option<usize>,
    pub switch: Option<usize>,
    pub tune: Option<usize>,
}

pub struct Reader {
    pub reader: df::Reader,
}

impl Reader {
    pub fn from_datafile(reader: df::Reader) -> Reader {
        Reader { reader: reader }
    }
    pub fn check_version(&self) -> Result<(),MapError> {
        let version = try!(self.version());
        if version != 1 {
            return Err(MapError::InvalidVersion(version));
        }
        Ok(())
    }
    pub fn version(&self) -> Result<i32,MapError> {
        let raw = try!(self.reader.find_item(format::MAP_ITEMTYPE_VERSION, 0)
            .ok_or(MapError::MissingVersion));
        let v0 = try!(format::MapItemCommonV0::from_slice(raw.data)
            .ok_or(MapError::MalformedVersion));
        Ok(v0.version)
    }
    pub fn group_indices(&self) -> ops::Range<usize> {
        self.reader.item_type_indices(format::MAP_ITEMTYPE_GROUP)
    }
    pub fn group(&self, index: usize) -> Result<Group,MapError> {
        // Doesn't fail if index is from Reader::groups().
        let raw = self.reader.item(index);
        assert!(raw.type_id == format::MAP_ITEMTYPE_GROUP);
        let layer_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_LAYER);
        Group::from_raw(raw.data, layer_indices)
    }
    pub fn layer(&self, index: usize) -> Result<Layer,MapError> {
        // Doesn't fail if index is from Reader::group().
        let raw = self.reader.item(index);
        assert!(raw.type_id == format::MAP_ITEMTYPE_LAYER);
        let data_indices = 0..self.reader.num_data();
        let image_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_IMAGE);
        let sound_indices = self.reader.item_type_indices(format::MAP_ITEMTYPE_DDRACE_SOUND);
        Layer::from_raw(raw.data, data_indices, image_indices, sound_indices)
    }
    pub fn image(&self, index: usize) -> Result<Image,MapError> {
        let raw = self.reader.item(index);
        let data_indices = 0..self.reader.num_data();
        Image::from_raw(raw.data, data_indices)
    }
    pub fn image_data(&mut self, data_index: usize) -> Result<Vec<u8>,Error> {
        let raw = try!(self.reader.read_data(data_index));
        Ok(raw)
    }
    pub fn game_layers(&self) -> Result<GameLayers,MapError> {
        fn put<T>(opt: &mut Option<T>, new: T) -> Result<(),MapError> {
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
            let group = try!(self.group(i));
            for k in group.layer_indices.clone() {
                // TODO: Just as above, skip this layer in case of failure?
                let layer = try!(self.layer(k));
                if let LayerType::Tilemap(tilemap) = layer.t {
                    match tilemap.type_ {
                        LayerTilemapType::Normal(_) => continue,
                        LayerTilemapType::Game(d) => try!(put(&mut game, d)),
                        LayerTilemapType::RaceTeleport(d, _) => try!(put(&mut teleport, d)),
                        LayerTilemapType::RaceSpeedup(d, _) => try!(put(&mut speedup, d)),
                        LayerTilemapType::DdraceFront(d, _) => try!(put(&mut front, d)),
                        LayerTilemapType::DdraceSwitch(d, _) => try!(put(&mut switch, d)),
                        LayerTilemapType::DdraceTune(d, _) => try!(put(&mut tune, d)),
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
            game: game,
            teleport: teleport,
            speedup: speedup,
            front: front,
            switch: switch,
            tune: tune,
        })
    }
    pub fn image_name(&mut self, data_index: usize) -> Result<Vec<u8>,Error> {
        let mut raw = try!(self.reader.read_data(data_index));
        if raw.pop() != Some(0) {
            return Err(Error::Map(MapError::MalformedImageName))
        }
        for &c in &raw {
            match c {
                b'/' | b'\\' | b'\0' => return Err(Error::Map(MapError::MalformedImageName)),
                _ => {}
            }
        }
        Ok(raw)
    }
    pub fn tune_layer_tiles(&mut self, data_index: usize) -> Result<Vec<format::TuneTile>,Error> {
        let raw = try!(self.reader.read_data(data_index));
        if raw.len() % mem::size_of::<format::TuneTile>() != 0 {
            return Err(Error::Map(MapError::InvalidTuneTilesLength(raw.len())));
        }
        Ok(unsafe { vec::transmute(raw) })
    }
    pub fn layer_tiles(&mut self, data_index: usize) -> Result<Vec<format::Tile>,Error> {
        let raw = try!(self.reader.read_data(data_index));
        if raw.len() % mem::size_of::<format::Tile>() != 0 {
            return Err(Error::Map(MapError::InvalidTilesLength(raw.len())));
        }
        Ok(unsafe { vec::transmute(raw) })
    }
}
