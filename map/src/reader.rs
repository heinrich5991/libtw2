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
pub struct LayerQuads {
    pub num_quads: usize,
    pub data: usize,
    pub image: Option<usize>,
    pub name: [u8; 12],
}

impl LayerQuads {
    fn from_raw(raw: &[i32]) -> Result<LayerQuads,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedLayerQuads)
        }
        let v1 = try!(e(format::MapItemLayerV1QuadsV1::from_slice(raw)));
        let v2 = format::MapItemLayerV1QuadsV2::from_slice(raw);
        let image = if v1.image == -1 {
            None
        } else {
            Some(try!(e(v1.image.to_usize())))
        };
        let name = v2.map(|v2| v2.name_get()).unwrap_or([0; 12]);
        Ok(LayerQuads {
            num_quads: try!(e(v1.num_quads.to_usize())),
            data: try!(e(v1.data.to_usize())),
            image: image,
            name: name,
        })
    }
}

#[derive(Clone, Copy)]
pub struct LayerTilemap {
    pub game: bool,
    pub width: u32,
    pub height: u32,
    pub color: Color,
    pub color_env: Option<u16>,
    pub color_env_offset: i32,
    pub image: Option<usize>,
    pub data: usize,
    pub name: [u8; 12],
}

impl LayerTilemap {
    fn from_raw(raw: &[i32]) -> Result<LayerTilemap,MapError> {
        fn e<T>(option: Option<T>) -> Result<T,MapError> {
            option.ok_or(MapError::MalformedLayerTilemap)
        }
        let v2 = try!(e(format::MapItemLayerV1TilemapV2::from_slice(raw)));
        let v3 = format::MapItemLayerV1TilemapV3::from_slice(raw);
        // TODO: Standard settings for game group.
        let flags = v2.flags as u32;
        if flags & !format::TILELAYERFLAGS_ALL != 0 {
            return Err(MapError::InvalidLayerTilemapFlags(flags));
        }
        let color = Color {
            red: try!(e(v2.color_red.to_u8())),
            green: try!(e(v2.color_green.to_u8())),
            blue: try!(e(v2.color_blue.to_u8())),
            alpha: try!(e(v2.color_alpha.to_u8())),
        };
        let color_env = if v2.color_env == -1 {
            None
        } else {
            Some(try!(e(v2.color_env.to_u16())))
        };
        let image = if v2.image == -1 {
            None
        } else {
            Some(try!(e(v2.image.to_usize())))
        };
        let name = v3.map(|v3| v3.name_get()).unwrap_or([0; 12]);
        Ok(LayerTilemap {
            game: flags & format::TILELAYERFLAG_GAME != 0,
            width: try!(e(v2.width.to_u32())),
            height: try!(e(v2.height.to_u32())),
            color: color,
            color_env: color_env,
            color_env_offset: v2.color_env_offset,
            image: image,
            data: try!(e(v2.data.to_usize())),
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
}

impl Layer {
    fn from_raw(raw: &[i32]) -> Result<Layer,MapError> {
        let (v1, rest) = try!(format::MapItemLayerV1::from_slice_rest(raw).ok_or(MapError::MalformedLayer));
        let flags = v1.flags as u32;
        if flags & !format::LAYERFLAGS_ALL != 0 {
            return Err(MapError::InvalidLayerFlags(flags));
        }
        let t = match v1.type_ {
            format::MAP_ITEMTYPE_LAYER_V1_TILEMAP =>
                LayerType::Tilemap(try!(LayerTilemap::from_raw(rest))),
            format::MAP_ITEMTYPE_LAYER_V1_QUADS =>
                LayerType::Quads(try!(LayerQuads::from_raw(rest))),
            _ => return Err(MapError::InvalidLayerType(v1.type_)),
        };
        Ok(Layer {
            detail: flags & format::LAYERFLAG_DETAIL != 0,
            t: t,
        })
    }
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
        Group::from_raw(raw.data, self.reader.item_type_indices(format::MAP_ITEMTYPE_LAYER))
    }
    pub fn layer(&self, index: usize) -> Result<Layer,MapError> {
        // Doesn't fail if index is from Reader::group().
        let raw = self.reader.item(index);
        Layer::from_raw(raw.data)
    }
    // Returns (game group index, game layer index).
    pub fn game_layer(&self) -> Result<(usize,Group,usize,LayerTilemap),MapError> {
        let mut num_game_layers = 0;
        let mut result = None;
        for i in self.group_indices() {
            // TODO: Just skip this group?
            let group = try!(self.group(i));
            for k in group.layer_indices.clone() {
                // TODO: Just as above, skip this layer in case of failure?
                let layer = try!(self.layer(k));
                if let LayerType::Tilemap(tilemap) = layer.t {
                    if tilemap.game {
                        num_game_layers += 1;
                        result = Some((i, group.clone(), k, tilemap))
                    }
                }
            }
        }
        match num_game_layers {
            0 => Err(MapError::NoGameLayers),
            1 => Ok(result.unwrap()),
            _ => Err(MapError::TooManyGameLayers(num_game_layers)),
        }
    }
    pub fn layer_tiles(&mut self, data_index: usize) -> Result<Vec<format::Tile>,Error> {
        let raw = try!(self.reader.read_data(data_index));
        if raw.len() % mem::size_of::<format::Tile>() != 0 {
            return Err(Error::Map(MapError::InvalidTilesLength(raw.len())));
        }
        Ok(unsafe { vec::transmute(raw) })
    }
}
