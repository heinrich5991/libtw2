#[deriving(Clone, Show, PartialEq, Eq)]
pub struct Rectangle {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[deriving(Clone, Show, PartialEq, Eq)]
pub struct Color {
    r: i32,
    g: i32,
    b: i32,
    a: i32,
}

// TODO: express C strings through the type system
// TODO: express cross-indexes through type system

#[deriving(Clone, Show, PartialEq, Eq)]
pub struct DataIndex(pub uint);


#[deriving(Clone, Show)]
pub struct MapItemVersion {
    version: i32,
}

#[deriving(Clone, Show)]
pub struct MapItemInfo {
    author: Option<DataIndex>,
    map_version: Option<DataIndex>,
    credits: Option<DataIndex>,
    license: Option<DataIndex>,
}

#[deriving(Clone, Show)]
enum ImageFormat {
    ImageFormatRgb,
    ImageFormatRgba,
}

#[deriving(Clone, Show)]
pub struct MapItemImage {
    width: i32,
    height: i32,
    external: bool,
    name: Option<DataIndex>,
    data: Option<DataIndex>,
    format: ImageFormat,
}

#[deriving(Clone, Show)]
pub struct MapItemGroup {
    offset_x: i32,
    offset_y: i32,
    parallax_x: i32,
    parallax_y: i32,

    clipping: Option<Rectangle>,

    start_layer: i32,
    num_layers: u32,

    name: [u8, ..12],
    name_length: uint,
}

#[deriving(Clone, Show)]
pub struct MapItemGroupClip {
    
}

#[deriving(Clone, Show)]
pub struct MapItemLayer {
    flags: u32,
    kind: MapItemLayerKind,
}

#[deriving(Clone, Show)]
pub enum MapItemLayerKind {
    LayerTilemap {
        flags: u32,
        width: i32,
        height: i32,

        color: Color,
        color_env: i32,
        color_env_offset: i32,

        image: i32,
        data: DataIndex,

        name: [u8, ..12],
        name_length: uint,
    }
    LayerQuads {
        flags: u32,

        num_quads: i32,
        data: DataIndex,
        image: i32,

        name: [u8, ..12],
        name_length: uint,
    }
}

#[deriving(Clone, Show)]
pub struct MapItemEnvelope {
    channels: i32,
    start_points: i32,
    num_points: i32,
    synchronized: bool,

    name: [u8, ..32],
    name_length: uint,
}
