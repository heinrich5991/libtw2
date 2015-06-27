#![cfg(not(test))]

extern crate datafile as df;
extern crate env_logger;
extern crate map;

use map::format::*;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;

#[derive(Default)]
struct ErrorStats {
    df_errors: HashMap<df::format::Error,u64>,
    io_errors: Vec<io::Error>,
    ok: u64,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct WeirdItem {
    type_id: u16,
    type_: WeirdItemType,
}

impl WeirdItem {
    fn empty(type_id: u16) -> WeirdItem {
        WeirdItem {
            type_id: type_id,
            type_: WeirdItemType::Empty,
        }
    }
    fn unknown_version(type_id: u16, version: i32, length: usize) -> WeirdItem {
        WeirdItem {
            type_id: type_id,
            type_: WeirdItemType::UnknownVersion(version, length),
        }
    }
    fn wrong_size(type_id: u16, version: i32, length: usize, expected_length: usize) -> WeirdItem {
        WeirdItem {
            type_id: type_id,
            type_: WeirdItemType::WrongSize(version, length, expected_length),
        }
    }
    fn unknown_item_type(type_id: u16, version: i32, length: usize) -> WeirdItem {
        WeirdItem {
            type_id: type_id,
            type_: WeirdItemType::UnknownItemType(version, length),
        }
    }
    fn unknown_layer_type(layer_type: i32, version: i32, length: usize) -> WeirdItem {
        WeirdItem {
            type_id: MAP_ITEMTYPE_LAYER,
            type_: WeirdItemType::UnknownLayerType(layer_type, version, length),
        }
    }
}

impl fmt::Debug for WeirdItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let t = self.type_id;
        match self.type_ {
            WeirdItemType::Empty =>
                write!(f, "empty type_id={}", self.type_id),
            WeirdItemType::UnknownVersion(v, l) =>
                write!(f, "unknown_version type_id={} version={} len={}", t, v, l),
            WeirdItemType::WrongSize(v, l, e) =>
                write!(f, "wrong_size type_id={} version={} len={} expected_len={}", t, v, l, e),
            WeirdItemType::UnknownItemType(v, l) =>
                write!(f, "unknown_item_type type_id={} version={} len={}", t, v, l),
            WeirdItemType::UnknownLayerType(t, v, l) =>
                write!(f, "unknown_layer_type layer_type={} version={} len={}", t, v, l),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum WeirdItemType {
    Empty,
    // UnknownVersion(version, length)
    UnknownVersion(i32, usize),
    // WrongSize(version, length, expected_length)
    WrongSize(i32, usize, usize),
    // UnknownItemType(version, length)
    UnknownItemType(i32, usize),
    // UnknownLayerType(layer_type, version, length)
    UnknownLayerType(i32, i32, usize),
}

fn process(path: &Path, results: &mut HashMap<WeirdItem,u64>) -> Result<(),df::Error> {
    let file = try!(File::open(path));
    let dfr = try!(df::Reader::new(file));

    let mut env_version = None;

    macro_rules! register {
        ($e:expr) => { *results.entry($e).or_insert(0) += 1 };
    }

    for item in dfr.items() {
        let item: df::ItemView = item;

        fn check<T:MapItem>(results: &mut HashMap<WeirdItem,u64>, type_id: u16, slice: &[i32]) {
            if T::version() == slice[0] && !T::ignore_version()
                && slice.len() != T::sum_len()
            {
                *results.entry(WeirdItem::wrong_size(type_id, slice[0], slice.len(), T::sum_len())).or_insert(0) += 1;
            }
        }

        let version = match MapItemCommonV0::from_slice(item.data) {
            Some(v) => v.version,
            None => {
                if item.type_id != MAP_ITEMTYPE_ENVPOINTS {
                    register!(WeirdItem::empty(item.type_id));
                }
                continue;
            }
        };

        match item.type_id {
            MAP_ITEMTYPE_VERSION => {
                if version != 1 {
                    register!(WeirdItem::unknown_version(item.type_id, version, item.data.len()));
                    continue;
                }
                check::<MapItemCommonV0>(results, item.type_id, item.data);
            },
            MAP_ITEMTYPE_INFO => {
                if version != 1 {
                    register!(WeirdItem::unknown_version(item.type_id, version, item.data.len()));
                    continue;
                }
                check::<MapItemInfoV1>(results, item.type_id, item.data);
            },
            MAP_ITEMTYPE_IMAGE => {
                if !(1 <= version && version <= 2) {
                    register!(WeirdItem::unknown_version(item.type_id, version, item.data.len()));
                    continue;
                }
                check::<MapItemImageV1>(results, item.type_id, item.data);
                check::<MapItemImageV2>(results, item.type_id, item.data);
            },
            MAP_ITEMTYPE_ENVELOPE => {
                if !(1 <= version && version <= 2) {
                    register!(WeirdItem::unknown_version(item.type_id, version, item.data.len()));
                    continue;
                }
                if version == 1 && item.data.len() != MapItemEnvelopeV1::sum_len()
                    && item.data.len() != MapItemEnvelopeV1Legacy::sum_len()
                {
                    register!(WeirdItem::wrong_size(item.type_id, version, item.data.len(), MapItemEnvelopeV1::sum_len()));
                }
                check::<MapItemEnvelopeV2>(results, item.type_id, item.data);
                if let Some(c) = MapItemCommonV0::from_slice(item.data) {
                    match env_version {
                        None => env_version = Some(c.version),
                        Some(v) if v == c.version => {},
                        Some(v) => panic!("differing versions for envpoints, v1={} v2={}", v, c.version),
                    }
                }
            },
            MAP_ITEMTYPE_GROUP => {
                if !(1 <= version && version <= 3) {
                    register!(WeirdItem::unknown_version(item.type_id, version, item.data.len()));
                    continue;
                }
                check::<MapItemGroupV1>(results, item.type_id, item.data);
                check::<MapItemGroupV2>(results, item.type_id, item.data);
                check::<MapItemGroupV3>(results, item.type_id, item.data);
            },
            MAP_ITEMTYPE_LAYER => {
                register!(WeirdItem::unknown_version(item.type_id, version, item.data.len()));
                if let Some((layer, rest)) = MapItemLayerV1::from_slice_rest(item.data) {
                    match layer.type_ {
                        MAP_ITEMTYPE_LAYER_V1_TILEMAP => {
                            // Might panic, whatever...
                            let version = rest[0];
                            if !(2 <= version && version <= 3) {
                                register!(WeirdItem::unknown_version(101, version, rest.len()));
                                continue;
                            }
                            check::<MapItemLayerV1TilemapV2>(results, 101, rest);
                            check::<MapItemLayerV1TilemapV3>(results, 101, rest);
                        }
                        MAP_ITEMTYPE_LAYER_V1_QUADS => {
                            // Might panic, whatever...
                            let version = rest[0];
                            if !(1 <= version && version <= 2) {
                                register!(WeirdItem::unknown_version(102, version, rest.len()));
                                continue;
                            }
                            check::<MapItemLayerV1QuadsV1>(results, 102, rest);
                            check::<MapItemLayerV1QuadsV2>(results, 102, rest);
                        }
                        _ => {
                            register!(WeirdItem::unknown_layer_type(layer.type_, rest[0], rest.len()));
                        }
                    }
                }
            },
            MAP_ITEMTYPE_ENVPOINTS => {
                // Not validated.
            },
            _ => {
                register!(WeirdItem::unknown_item_type(item.type_id, version, item.data.len()));
            },
        }
    }
    Ok(())
}

fn update_stats(stats: &mut ErrorStats, err: df::Error) {
    match err {
        df::Error::Df(e) => {
            *stats.df_errors.entry(e).or_insert(0) += 1;
        }
        df::Error::Io(e) => {
            stats.io_errors.push(e);
        }
    }
}

fn print_stats(stats: &ErrorStats) {
    for (e, c) in &stats.df_errors {
        println!("{}: {:?}", c, e);
    }
    for e in &stats.io_errors {
        println!("1: {:?}", e);
    }
    println!("{}: ok", stats.ok);
    println!("-------------");
}

fn print_results(results: &HashMap<WeirdItem,u64>) {
    for (w, c) in results {
        println!("{}: {:?}", c, w)
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut args = env::args_os();
    let mut have_args = false;
    let program_name = args.next().unwrap();

    let mut stats = ErrorStats::default();
    let mut results = HashMap::default();

    for (_, arg) in args.enumerate() {
        have_args = true;
        match process(Path::new(&arg), &mut results) {
            Ok(()) => stats.ok += 1,
            Err(err) => {
                println!("{}: {:?}", arg.to_string_lossy(), err);
                update_stats(&mut stats, err);
            }
        }
    }
    if !have_args {
        println!("USAGE: {} <MAP>...", program_name.to_string_lossy());
        return;
    }
    print_stats(&stats);
    print_results(&results);
}
