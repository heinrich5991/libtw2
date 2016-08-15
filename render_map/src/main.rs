#![cfg(not(test))]

extern crate common;
extern crate datafile as df;
extern crate logger;
extern crate ndarray;
extern crate image;
extern crate map;

use common::num::Cast;
use common::slice;
use common::vec;
use image::ImageError;
use map::format;
use map::reader;
use ndarray::Array;
use ndarray::Ix;
use std::cmp;
use std::collections::HashMap;
use std::collections::hash_map;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::mem;
use std::path::Path;
use std::str;

// TODO: Skip empty tiles

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl From<reader::Color> for Color {
    fn from(c: reader::Color) -> Color {
        Color {
            red: c.red,
            green: c.green,
            blue: c.blue,
            alpha: c.alpha,
        }
    }
}

impl Color {
    fn transparent() -> Color {
        Color::default()
    }
    fn white() -> Color {
        Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }
    }
    fn mask(self, other: Color) -> Color {
        fn mask(a: u8, b: u8) -> u8 {
            (a.u32() * b.u32() / 255).assert_u8()
        }
        Color {
            red: mask(self.red, other.red),
            green: mask(self.green, other.green),
            blue: mask(self.blue, other.blue),
            alpha: mask(self.alpha, other.alpha),
        }
    }
    fn overlay_with(self, other: Color) -> Color {
        // From https://en.wikipedia.org/w/index.php?title=Alpha_compositing&oldid=732001952#Alpha_blending.
        fn mix(mix_a: u32, num_a: u8, mix_b: u32, num_b: u8) -> u8 {
            if mix_a == 0 && mix_b == 0 {
                return 0;
            }
            ((mix_a * num_a.u32() + mix_b * num_b.u32()) / (mix_a + mix_b)).assert_u8()
        }
        let src = other;
        let dst = self;
        let mix_a = src.alpha.u32() * 255;
        let mix_b = dst.alpha.u32() * (255 - src.alpha.u32());
        Color {
            red: mix(mix_a, src.red, mix_b, dst.red),
            green: mix(mix_a, src.green, mix_b, dst.green),
            blue: mix(mix_a, src.blue, mix_b, dst.blue),
            alpha: ((mix_a + mix_b) / 255).assert_u8(),
        }
    }
}

struct Layer {
    detail: bool,
    color: Color,
    image: Option<usize>,
    tiles: Array<format::Tile, (Ix, Ix)>,
}

struct Image {
    data: Array<Color, (Ix, Ix)>,
}

const TILE_LEN: u32 = 1;
const TILE_NUM: u32 = 16;

fn transform_image(tileset: Array<Color, (Ix, Ix)>) -> Array<Color, (Ix, Ix)> {
    let dim = tileset.dim();
    let height = dim.0.assert_u32();
    let width = dim.1.assert_u32();
    let result_len = (TILE_LEN * TILE_NUM).usize();
    let mut result = Array::default((result_len, result_len));
    if height == 0 || width == 0 {
        return result;
    }
    for y in 0..TILE_LEN*TILE_NUM {
        for x in 0..TILE_LEN*TILE_NUM {
            // TODO: Do averaging.
            let low_tx = x * width / (TILE_LEN * TILE_NUM);
            let low_ty = y * height / (TILE_LEN * TILE_NUM);
            let mut high_tx = (x + 1) * width / (TILE_LEN * TILE_NUM);
            let mut high_ty = (y + 1) * height / (TILE_LEN * TILE_NUM);
            if low_tx == high_tx {
                high_tx += 1;
            }
            if low_ty == high_ty {
                high_ty += 1;
            }
            let mut count = 0;
            let mut red = 0;
            let mut green = 0;
            let mut blue = 0;
            let mut alpha = 0;
            for ty in low_ty..high_ty {
                for tx in low_tx..high_tx {
                    count += 1;
                    let c = tileset[(ty.usize(), tx.usize())];
                    red += c.red.u32();
                    green += c.green.u32();
                    blue += c.blue.u32();
                    alpha += c.alpha.u32();
                }
            }

            result[(y.usize(), x.usize())] = Color {
                red: (red / count).assert_u8(),
                green: (green / count).assert_u8(),
                blue: (blue / count).assert_u8(),
                alpha: (alpha / count).assert_u8(),
            };
        }
    }
    for y in 0..TILE_LEN {
        for x in 0..TILE_LEN {
            result[(y.usize(), x.usize())] = Color::transparent();
        }
    }
    result
}

fn sanitize(s: &str) -> Option<&str> {
    let pat: &[char] = &['/', '\\'];
    if !s.contains(pat) {
        Some(s)
    } else {
        None
    }
}

fn swap<T, E>(v: Option<Result<T, E>>) -> Result<Option<T>, E> {
    v.map(|r| r.map(|t| Some(t))).unwrap_or(Ok(None))
}

fn transform_coordinates((mut iy, mut ix): (u32, u32), rotate: bool, vflip: bool, hflip: bool)
    -> (u32, u32)
{
    if vflip {
        ix = (TILE_LEN-1) - ix;
    }
    if hflip {
        iy = (TILE_LEN-1) - iy;
    }
    if rotate {
        ix = (TILE_LEN-1) - mem::replace(&mut iy, ix);
    }
    (iy, ix)
}

fn process<E>(path: &Path, out_path: &Path, mut external: &mut E)
    -> Result<(), Error>
    where E: FnMut(&str) -> Result<Option<Array<Color, (Ix, Ix)>>, Error>,
{
    let file = try!(File::open(path));
    let dfr = try!(df::Reader::new(file));
    let mut map = map::Reader::from_datafile(dfr);

    let game_layers = try!(map.game_layers());
    let mut layers = vec![];
    let mut images = HashMap::new();

    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;
    let mut max_height = 0;
    let mut max_width = 0;

    for i in game_layers.group.layer_indices.clone() {
        let layer = try!(map.layer(i));
        let tilemap = if let reader::LayerType::Tilemap(t) = layer.t { t } else { continue; };
        max_height = cmp::max(max_height, tilemap.height);
        max_width = cmp::max(max_width, tilemap.width);
        let normal = if let Some(n) = tilemap.type_.to_normal() { n } else { continue; };
        let height = tilemap.height.usize();
        let width = tilemap.width.usize();
        let tiles = try!(map.layer_tiles(normal.data));
        let tiles = try!(Array::from_shape_vec((height, width), tiles)
                         .map_err(|_| OwnError::TilemapShape));

        match images.entry(normal.image) {
            hash_map::Entry::Occupied(_) => {},
            hash_map::Entry::Vacant(v) => {
                let data = match normal.image {
                    None => Array::from_elem((1, 1), Color::white()),
                    Some(image_index) => {
                        let image = try!(map.image(image_index));
                        let height = image.height.usize();
                        let width = image.width.usize();
                        match image.data {
                            Some(d) => {
                                let data = try!(map.image_data(d));
                                if data.len() % mem::size_of::<Color>() != 0 {
                                    return Err(OwnError::ImageShape.into());
                                }
                                let data: Vec<Color> = unsafe { vec::transmute(data) };
                                try!(Array::from_shape_vec((height, width), data).map_err(|_| OwnError::ImageShape))
                            }
                            None => {
                                let image_name = try!(map.image_name(image.name));
                                // WARN? Unknown external image
                                // WARN! Wrong dimensions
                                try!(swap(str::from_utf8(&image_name).ok()
                                          .and_then(sanitize)
                                          .map(&mut external)))
                                    .unwrap_or(None)
                                    .unwrap_or_else(|| Array::from_elem((1, 1), Color::white()))
                            }
                        }
                    }
                };
                v.insert(Image {
                    data: transform_image(data),
                });
            },
        }

        for y in 0..tilemap.height {
            for x in 0..tilemap.width {
                if tiles[(y.usize(), x.usize())].index != 0 {
                    min_x = cmp::min(min_x, x);
                    min_y = cmp::min(min_y, y);
                    max_x = cmp::max(max_x, x);
                    max_y = cmp::max(max_y, y);
                }
            }
        }

        layers.push(Layer {
            detail: layer.detail,
            color: normal.color.into(),
            image: normal.image,
            tiles: tiles,
        });
    }

    let result_width = (max_x - min_x).checked_mul(TILE_LEN).unwrap();
    let result_height = (max_y - min_y).checked_mul(TILE_LEN).unwrap();

    let mut result: Array<Color, _> = Array::default((result_height.usize(), result_width.usize()));

    for l in &layers {
        let image = &images[&l.image];
        for y in 0..(max_y - min_y) {
            for x in 0..(max_x - min_x) {
                let tile = l.tiles[((min_y + y).usize(), (min_x + x).usize())];
                let rotate = tile.flags & format::TILEFLAG_ROTATE != 0;
                let vflip = tile.flags & format::TILEFLAG_VFLIP != 0;
                let hflip = tile.flags & format::TILEFLAG_HFLIP != 0;
                let tile_x = tile.index.u32() % TILE_NUM;
                let tile_y = tile.index.u32() / TILE_NUM;
                for iy in 0..TILE_LEN {
                    for ix in 0..TILE_LEN {
                        let p_target = &mut result[((y * TILE_LEN + iy).usize(), (x * TILE_LEN + ix).usize())];
                        let (ty, tx) = transform_coordinates((iy, ix), rotate, vflip, hflip);
                        let p_tile = image.data[((tile_y * TILE_LEN + ty).usize(), (tile_x * TILE_LEN + tx).usize())];
                        *p_target = p_target.overlay_with(p_tile.mask(l.color));
                    }
                }
            }
        }
    }

    let raw: &[Color] = result.as_slice().unwrap();
    let raw: &[u8] = unsafe { slice::transmute(raw) };
    try!(image::save_buffer(out_path, raw, result.dim().1.assert_u32(), result.dim().0.assert_u32(), image::ColorType::RGBA(8)));

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum OwnError {
    TilemapShape,
    ImageShape,
}

#[derive(Debug)]
enum Error {
    Df(df::format::Error),
    Io(io::Error),
    Image(ImageError),
    Map(map::format::Error),
    Own(OwnError),
}

impl From<df::Error> for Error {
    fn from(e: df::Error) -> Error {
        match e {
            df::Error::Df(e) => e.into(),
            df::Error::Io(e) => e.into(),
        }
    }
}

impl From<map::Error> for Error {
    fn from(e: map::Error) -> Error {
        match e {
            map::Error::Df(e) => e.into(),
            map::Error::Map(e) => e.into(),
        }
    }
}

impl From<df::format::Error> for Error {
    fn from(e: df::format::Error) -> Error {
        Error::Df(e)
    }
}

impl From<map::format::Error> for Error {
    fn from(e: map::format::Error) -> Error {
        Error::Map(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Error {
        Error::Image(e)
    }
}

impl From<OwnError> for Error {
    fn from(e: OwnError) -> Error {
        Error::Own(e)
    }
}

#[derive(Default)]
struct ErrorStats {
    map_errors: HashMap<map::format::Error,u64>,
    df_errors: HashMap<df::format::Error,u64>,
    own_errors: HashMap<OwnError,u64>,
    image_errors: Vec<ImageError>,
    io_errors: Vec<io::Error>,
    ok: u64,
}

fn update_error_stats(stats: &mut ErrorStats, err: Error) {
    match err {
        Error::Map(e) => *stats.map_errors.entry(e).or_insert(0) += 1,
        Error::Df(e) => *stats.df_errors.entry(e).or_insert(0) += 1,
        Error::Own(e) => *stats.own_errors.entry(e).or_insert(0) += 1,
        Error::Image(e) => stats.image_errors.push(e),
        Error::Io(e) => stats.io_errors.push(e),
    }
}

fn print_error_stats(error_stats: &ErrorStats) {
    for (e, c) in &error_stats.map_errors {
        println!("{:?}: {}", e, c);
    }
    for (e, c) in &error_stats.df_errors {
        println!("{:?}: {}", e, c);
    }
    for (e, c) in &error_stats.own_errors {
        println!("{:?}: {}", e, c);
    }
    for e in &error_stats.io_errors {
        println!("{:?}", e);
    }
    for e in &error_stats.image_errors {
        println!("{:?}", e);
    }
    println!("ok: {}", error_stats.ok);
}

fn load_external_image(path: &Path) -> Result<Option<Array<Color, (Ix, Ix)>>, Error> {
    let image_result = image::open(path);
    match image_result {
        Err(ImageError::IoError(ref e)) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
        },
        _ => {},
    }
    let image = try!(image_result).to_rgba();
    let (width, height) = image.dimensions();
    let raw: Vec<u8> = image.into_raw();
    let raw: Vec<Color> = unsafe { vec::transmute(raw) };
    Ok(Some(Array::from_shape_vec((width.usize(), height.usize()), raw).unwrap()))
}

fn main() {
    logger::init();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut error_stats = ErrorStats::default();
    let mut out_path_buf = OsString::new();

    let mut external_images: HashMap<String, Option<Array<Color, (Ix, Ix)>>> = HashMap::new();
    let mut external = |name: &str| match external_images.entry(name.into()) {
        hash_map::Entry::Occupied(o) => Ok(o.get().clone()),
        hash_map::Entry::Vacant(v) => {
            let image = try!(load_external_image(Path::new(&format!("mapres/{}.png", name))));
            Ok(v.insert(image).clone())
        },
    };

    for arg in args {
        have_args = true;
        out_path_buf.clear();
        out_path_buf.push(&arg);
        out_path_buf.push(".png");
        match process(Path::new(&arg), Path::new(&out_path_buf), &mut external) {
            Ok(()) => error_stats.ok += 1,
            Err(err) => {
                println!("{}: {:?}", arg.to_string_lossy(), err);
                update_error_stats(&mut error_stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <MAP>...", program_name.to_string_lossy());
        return;
    }
    print_error_stats(&error_stats);
}
