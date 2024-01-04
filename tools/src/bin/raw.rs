#![cfg(not(test))]

extern crate datafile;
extern crate map;
extern crate tools;

use std::fmt::Debug;
use std::path::Path;

use map::format::*;

fn process(_: &Path, dfr: datafile::Reader, _: &mut ()) -> Result<(), map::Error> {
    let mut env_version = None;

    for item in dfr.items() {
        let item: datafile::ItemView = item;
        fn print_map_item<T: MapItem + Debug>(slice: &[i32]) {
            if let Ok(Some(mi)) = T::from_slice(slice) {
                print!(" {:?}", mi);
            }
        }

        print!("{} {} {}", item.type_id, item.id, item.data.len());
        match item.type_id {
            MAP_ITEMTYPE_VERSION => {
                print_map_item::<MapItemCommonV0>(item.data);
                //print_map_item::<MapItemVersionV1>(item.data);
            }
            MAP_ITEMTYPE_INFO => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemInfoV1>(item.data);
            }
            MAP_ITEMTYPE_IMAGE => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemImageV1>(item.data);
                print_map_item::<MapItemImageV2>(item.data);
            }
            MAP_ITEMTYPE_ENVELOPE => {
                print_map_item::<MapItemCommonV0>(item.data);
                if let Ok(Some(c)) = MapItemCommonV0::from_slice(item.data) {
                    match env_version {
                        None => env_version = Some(c.version),
                        Some(v) if v == c.version => {}
                        Some(v) => panic!(
                            "differing versions for envpoints, v1={} v2={}",
                            v, c.version
                        ),
                    }
                }
                print_map_item::<MapItemEnvelopeV1>(item.data);
                print_map_item::<MapItemEnvelopeV2>(item.data);
            }
            MAP_ITEMTYPE_GROUP => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemGroupV1>(item.data);
                print_map_item::<MapItemGroupV2>(item.data);
            }
            MAP_ITEMTYPE_LAYER => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemLayerV1>(item.data);
                if let Ok(Some((layer, rest))) = MapItemLayerV1::from_slice_rest(item.data) {
                    print!(" {} {} {}", layer.type_, layer.flags, rest.len());
                    match layer.type_ {
                        MAP_ITEMTYPE_LAYER_V1_TILEMAP => {
                            print_map_item::<MapItemLayerV1CommonV0>(rest);
                            //print_map_item::<MapItemLayerV1TilemapV1>(rest);
                            print_map_item::<MapItemLayerV1TilemapV2>(rest);
                            print_map_item::<MapItemLayerV1TilemapV3>(rest);
                        }
                        MAP_ITEMTYPE_LAYER_V1_QUADS => {
                            print_map_item::<MapItemLayerV1CommonV0>(rest);
                            print_map_item::<MapItemLayerV1QuadsV1>(rest);
                            print_map_item::<MapItemLayerV1QuadsV2>(rest);
                        }
                        MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS
                        | MAP_ITEMTYPE_LAYER_V1_DDRACE_SOUNDS_LEGACY => {
                            print_map_item::<MapItemLayerV1CommonV0>(rest);
                            print_map_item::<MapItemLayerV1DdraceSoundsV1>(rest);
                            //print_map_item::<MapItemLayerV1DdraceSoundsV2>(rest);
                        }
                        _ => panic!("unknown layer type {}", layer.type_),
                    }
                }
            }
            MAP_ITEMTYPE_ENVPOINTS => {
                let env_version =
                    env_version.unwrap_or_else(|| panic!("envpoints but no envelope"));
                fn print_envpoints<E: Envpoint + Debug>(slice: &[i32], env_version: i32) {
                    if let Some(e) = E::from_slice(slice, env_version) {
                        print!(" {:?}", e);
                    }
                }
                print_envpoints::<MapItemEnvpointV1>(item.data, env_version);
                print_envpoints::<MapItemEnvpointV2>(item.data, env_version);
            }
            MAP_ITEMTYPE_DDRACE_SOUND => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemDdraceSoundV1>(item.data);
            }
            _ => {
                print_map_item::<MapItemCommonV0>(item.data);
                panic!("unknown datafile item type {}", item.type_id);
            }
        }
        println!("");
    }
    Ok(())
}

fn nothing(_: &()) {}

fn main() {
    tools::map_stats::stats(process, nothing);
}
