#![cfg(not(test))]

#[macro_use]
extern crate clap;
extern crate common;
extern crate datafile as df;
extern crate image;
extern crate logger;
extern crate map;
extern crate ndarray;
extern crate num_traits;

use clap::App;
use clap::Arg;
use common::num::Cast;
use common::slice;
use common::vec;
use image::ImageError;
use image::RgbaImage;
use image::imageops;
use map::format;
use map::reader;
use ndarray::Array2;
use num_traits::ToPrimitive;
use std::cmp;
use std::collections::HashMap;
use std::collections::hash_map;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::mem;
use std::path::Path;
use std::process;
use std::str;

// TODO: Skip empty tiles (i.e. don't count tiles that have index != 0, but are
//       graphically empty.

#[derive(Clone, Copy)]
struct Rect {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl Rect {
    fn width(&self) -> u32 {
        self.max_x - self.min_x
    }

    fn height(&self) -> u32 {
        self.max_y - self.min_y
    }

    fn is_empty(&self) -> bool {
        return self.min_y >= self.max_y || self.min_x >= self.max_x
    }
}

#[derive(Clone, Copy)]
struct Config {
    size: u32,
    render_detail: bool,
    crop: Option<Rect>,
}

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
    color: Color,
    image: Option<usize>,
    tiles: Array2<format::Tile>,
}

const TILE_NUM: u32 = 16;

/// Scales `tileset` to `tile_len` * TILE_NUM pixels, clears first (air) tile.
fn normalize_tileset(tileset: Array2<Color>, tile_len: u32)
    -> Array2<Color>
{
    let dim = tileset.dim();
    let height = dim.0.assert_u32();
    let width = dim.1.assert_u32();
    let result_len = (tile_len * TILE_NUM).usize();
    let mut result = Array2::default((result_len, result_len));
    if height == 0 || width == 0 {
        return result;
    }
    for y in 0..tile_len*TILE_NUM {
        for x in 0..tile_len*TILE_NUM {
            let low_tx = x * width / (tile_len * TILE_NUM);
            let low_ty = y * height / (tile_len * TILE_NUM);
            let mut high_tx = (x + 1) * width / (tile_len * TILE_NUM);
            let mut high_ty = (y + 1) * height / (tile_len * TILE_NUM);
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
    for y in 0..tile_len {
        for x in 0..tile_len {
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

fn transform_coordinates((mut iy, mut ix): (u32, u32), rotate: bool, vflip: bool, hflip: bool, tile_len: u32)
    -> (u32, u32)
{
    if rotate {
        iy = (tile_len-1) - mem::replace(&mut ix, iy);
    }
    if vflip {
        ix = (tile_len-1) - ix;
    }
    if hflip {
        iy = (tile_len-1) - iy;
    }
    (iy, ix)
}

fn select_layers(map: &mut map::Reader, config: &Config)
    -> Result<Vec<Layer>, Error>
{
    let mut layers = vec![];

    for group_idx in map.group_indices() {
        let group = map.group(group_idx)?;

        if group.parallax_x != 100 || group.parallax_y != 100
            || group.offset_x != 0 || group.offset_y != 0
            || group.clipping.is_some()
        {
            continue;
        }

        for layer_idx in group.layer_indices {
            let layer = map.layer(layer_idx)?;

            if layer.detail && !config.render_detail { continue }
            let tilemap = if let reader::LayerType::Tilemap(t) = layer.t { t } else { continue };
            let normal = if let Some(n) = tilemap.type_.to_normal() { n } else { continue };
            let tiles = map.layer_tiles(tilemap.tiles(normal.data))?;

            layers.push(Layer {
                color: normal.color.into(),
                image: normal.image,
                tiles: tiles,
            });
        }
    }

    Ok(layers)
}

fn prepare_tilesets<E>(
    layers: &[Layer],
    map: &mut map::Reader,
    mut external_tileset_loader: &mut E,
    tile_len: u32,
) -> Result<HashMap<Option<usize>, Array2<Color>>, Error>
    where E: FnMut(&str) -> Result<Option<Array2<Color>>, Error>
{
    let mut tilesets = HashMap::new();

    for layer in layers {
        match tilesets.entry(layer.image) {
            hash_map::Entry::Occupied(_) => {},
            hash_map::Entry::Vacant(v) => {
                let data = match layer.image {
                    None => Array2::from_elem((1, 1), Color::white()),
                    Some(image_idx) => {
                        let image = map.image(image_idx)?;
                        let height = image.height.usize();
                        let width = image.width.usize();
                        match image.data {
                            Some(d) => {
                                let data = map.image_data(d)?;
                                if data.len() % mem::size_of::<Color>() != 0 {
                                    return Err(OwnError::ImageShape.into());
                                }
                                let data: Vec<Color> = unsafe { vec::transmute(data) };
                                Array2::from_shape_vec((height, width), data)
                                     .map_err(|_| OwnError::ImageShape)?
                            }
                            None => {
                                let image_name = map.image_name(image.name)?;
                                // WARN? Unknown external image
                                // WARN! Wrong dimensions
                                str::from_utf8(&image_name).ok()
                                    .and_then(sanitize)
                                    .map(&mut external_tileset_loader)
                                    .transpose()?
                                    .unwrap_or(None)
                                    .unwrap_or_else(|| Array2::from_elem((1, 1), Color::white()))
                            }
                        }
                    },
                };
                v.insert(normalize_tileset(data, tile_len));
            },
        }
    }

    Ok(tilesets)
}

fn crop_to_fit_nonair_tiles(layers: &[Layer]) -> Rect {
    let mut crop = Rect {
        min_x: u32::max_value(),
        max_x: 0,
        min_y: u32::max_value(),
        max_y: 0,
    };

    for layer in layers {
        for ((y, x), tile) in layer.tiles.indexed_iter() {
            if tile.index != 0 {
                crop.min_y = cmp::min(crop.min_y, y.assert_u32());
                crop.min_x = cmp::min(crop.min_x, x.assert_u32());
                crop.max_y = cmp::max(crop.max_y, (y + 1).assert_u32());
                crop.max_x = cmp::max(crop.max_x, (x + 1).assert_u32());
            }
        }
    }

    crop
}

fn scale_tile_len(crop: &Rect, config: &Config) -> u32 {
    let mut tile_len = 64;
    // TODO: Fix overflow on huge maps like Back in Time 2
    while tile_len != 1 && tile_len * tile_len * crop.width() * crop.height() > 16 * config.size * config.size {
        tile_len /= 2;
    }
    tile_len
}

fn render_layers(
    layers: &[Layer],
    tilesets: &HashMap<Option<usize>, Array2<Color>>,
    crop: &Rect,
    tile_len: u32
) -> Array2<Color> {

    let result_width = crop.width().checked_mul(tile_len).unwrap();
    let result_height = crop.height().checked_mul(tile_len).unwrap();

    let mut result: Array2<Color> = Array2::default((result_height.usize(), result_width.usize()));

    for l in layers {
        let tileset = &tilesets[&l.image];
        let layer_max_y = cmp::min(l.tiles.dim().0.assert_u32(), crop.max_y);
        let layer_max_x = cmp::min(l.tiles.dim().1.assert_u32(), crop.max_x);

        for layer_y in crop.min_y..layer_max_y {
            for layer_x in crop.min_x..layer_max_x {
                let target_y = layer_y - crop.min_y;
                let target_x = layer_x - crop.min_x;

                let tile = l.tiles[(layer_y.usize(), layer_x.usize())];

                let rotate = tile.flags & format::TILEFLAG_ROTATE != 0;
                let vflip = tile.flags & format::TILEFLAG_VFLIP != 0;
                let hflip = tile.flags & format::TILEFLAG_HFLIP != 0;
                let tile_x = tile.index.u32() % TILE_NUM;
                let tile_y = tile.index.u32() / TILE_NUM;

                // First tile is guaranteed to be empty (air)
                if tile_x == 0 && tile_y == 0 {
                    continue;
                }

                for iy in 0..tile_len {
                    for ix in 0..tile_len {
                        let p_target = &mut result[((target_y * tile_len + iy).usize(), (target_x * tile_len + ix).usize())];
                        let (ty, tx) = transform_coordinates((iy, ix), rotate, vflip, hflip, tile_len);
                        let p_tile = tileset[((tile_y * tile_len + ty).usize(), (tile_x * tile_len + tx).usize())];
                        *p_target = p_target.overlay_with(p_tile.mask(l.color));
                    }
                }
            }
        }
    }

    result
}

fn process<E>(path: &Path, out_path: &Path, mut external_tileset_loader: &mut E, config: &Config)
    -> Result<(), Error>
    where E: FnMut(&str) -> Result<Option<Array2<Color>>, Error>,
{
    let dfr = df::Reader::open(path)?;
    let mut map = map::Reader::from_datafile(dfr);

    let layers = select_layers(&mut map, &config)?;

    let crop = match config.crop {
        Some(crop) => crop,
        None => crop_to_fit_nonair_tiles(&layers),
    };
    if crop.is_empty() {
        return Err(OwnError::EmptyMap.into());
    }

    let tile_len = scale_tile_len(&crop, &config);
    let tilesets = prepare_tilesets(&layers, &mut map, &mut external_tileset_loader, tile_len)?;
    let result = render_layers(&layers, &tilesets, &crop, tile_len);

    let image = {
        let raw: &[Color] = result.as_slice().unwrap();
        let raw: &[u8] = unsafe { slice::transmute(raw) };
        RgbaImage::from_raw(result.dim().1.assert_u32(), result.dim().0.assert_u32(), raw.into()).unwrap()
    };
    mem::drop(result);

    let width = crop.width();
    let height = crop.height();
    let (mut new_width, mut new_height) = if width / height < 6 && height / width < 6 {
        let sqrt = (height * width).to_f32().unwrap().sqrt().to_u32().unwrap();
        (width * config.size / sqrt, height * config.size / sqrt)
    } else {
        let size = cmp::max(height, width);
        let result_size = (config.size.to_f32().unwrap() * 6.to_f32().unwrap().sqrt()).to_u32().unwrap();
        (width * result_size / size, height * result_size / size)
    };
    if new_width == 0 { new_width = 1; }
    if new_height == 0 { new_height = 1; }
    let resized = imageops::resize(&image, new_width, new_height, imageops::CatmullRom);
    mem::drop(image);
    resized.save(out_path)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum OwnError {
    EmptyMap,
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref e) => return e.fmt(f),
            // TODO: Improve error output
            _ => fmt::Debug::fmt(self, f),
        }
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

impl ErrorStats {
    fn has_errors(&self) -> bool {
        false
            || !self.map_errors.is_empty()
            || !self.df_errors.is_empty()
            || !self.own_errors.is_empty()
            || !self.image_errors.is_empty()
            || !self.io_errors.is_empty()
    }
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

fn load_external_image(path: &Path) -> Result<Option<Array2<Color>>, Error> {
    let image_result = image::open(path);
    match image_result {
        Err(ImageError::IoError(ref e)) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
        },
        _ => {},
    }
    let image = image_result?.to_rgba();
    let (width, height) = image.dimensions();
    let raw: Vec<u8> = image.into_raw();
    let raw: Vec<Color> = unsafe { vec::transmute(raw) };
    Ok(Some(Array2::from_shape_vec((width.usize(), height.usize()), raw).unwrap()))
}

fn main() {
    logger::init();

    let matches = App::new("Teeworlds map renderer")
        .about("Reads a Teeworlds map file and renders a PNG thumbnail.")
        .arg(Arg::with_name("size")
            .help("Sets the approximate area of the thumbnail to size*size pixels")
            .long("size")
            .takes_value(true)
            .value_name("SIZE")
            .default_value("200")
        )
        .arg(Arg::with_name("no-detail")
            .help("Don't render layers marked as \"Detail\" in the map editor")
            .long("no-detail")
        )
        .arg(Arg::with_name("map")
            .help("Map to render (output file is the same with \".png\" appended)")
            .multiple(true)
            .required(true)
            .value_name("MAP")
        )
        .arg(Arg::with_name("crop")
            .help("Crop to these tile coordinates (min_x,min_y,max_x,max_y)")
            .long("crop")
            .use_delimiter(true)
            .number_of_values(4)
            .value_name("CROP")
        )
        .get_matches();

    let crop = if !matches.is_present("crop") {
        None
    } else {
        let crop = values_t!(matches, "crop", u32).unwrap_or_else(|e| e.exit());
        if crop[0] > crop[2] {
            clap::Error::with_description(
                "min_x must be smaller or equal to max_x",
                clap::ErrorKind::ValueValidation
            ).exit();
        }
        if crop[1] > crop[3] {
            clap::Error::with_description(
                "min_y must be smaller or equal to max_y",
                clap::ErrorKind::ValueValidation
            ).exit();
        }
        Some(Rect {
            min_x: crop[0],
            min_y: crop[1],
            max_x: crop[2] + 1,
            max_y: crop[3] + 1,
        })
    };

    let config = Config {
        size: value_t!(matches, "size", u32).unwrap_or_else(|e| e.exit()),
        render_detail: !matches.is_present("no-detail"),
        crop: crop,
    };

    let args = matches.values_of_os("map").unwrap();
    let mut num_args: u64 = 0;

    let mut error_stats = ErrorStats::default();
    let mut out_path_buf = OsString::new();

    let mut external_images: HashMap<String, Option<Array2<Color>>> = HashMap::new();
    let mut external = |name: &str| match external_images.entry(name.into()) {
        hash_map::Entry::Occupied(o) => Ok(o.get().clone()),
        hash_map::Entry::Vacant(v) => {
            let image = load_external_image(Path::new(&format!("mapres/{}.png", name)))?;
            Ok(v.insert(image).clone())
        },
    };

    for arg in args {
        num_args += 1;
        out_path_buf.clear();
        out_path_buf.push(&arg);
        out_path_buf.push(".png");
        let path = Path::new(&arg);
        match process(path, Path::new(&out_path_buf), &mut external, &config) {
            Ok(()) => error_stats.ok += 1,
            Err(err) => {
                println!("{}: {}", path.display(), err);
                update_error_stats(&mut error_stats, err);
            }
        }
    }
    if num_args != 1 {
        print_error_stats(&error_stats);
    }
    if error_stats.has_errors() {
        process::exit(1);
    }
}
