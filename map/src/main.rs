#![cfg(not(test))]

extern crate datafile;
extern crate map;

use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

use map::format::*;

fn main() {
    let file = File::open(&Path::new("../dm1.map")).unwrap();
    let dfr = datafile::Reader::new(file).unwrap();

    for item in dfr.items() {
        let item: datafile::ItemView = item;
        fn print_map_item<T:MapItem+Debug>(slice: &[i32]) {
            if let Some(mi) = T::from_slice(slice) {
                print!(" {:?}", mi);
            }
        }

        print!("{} {} {}", item.type_id, item.id, item.data.len());
        match item.type_id {
            MAP_ITEMTYPE_VERSION => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemVersionV1>(item.data);
            },
            MAP_ITEMTYPE_INFO => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemInfoV1>(item.data);
            },
            MAP_ITEMTYPE_IMAGE => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemImageV1>(item.data);
                print_map_item::<MapItemImageV2>(item.data);
            },
            MAP_ITEMTYPE_ENVELOPE => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemEnvelopeV1>(item.data);
                print_map_item::<MapItemEnvelopeV2>(item.data);
            },
            MAP_ITEMTYPE_GROUP => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemGroupV1>(item.data);
                print_map_item::<MapItemGroupV2>(item.data);
            },
            MAP_ITEMTYPE_LAYER => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemLayerV1>(item.data);
                if let Some((layer, rest)) = MapItemLayerV1::from_slice_rest(item.data) {
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
                        _ => panic!("unknown layer type {}", layer.type_),
                    }
                }
            },
            MAP_ITEMTYPE_ENVPOINTS => {
                print_map_item::<MapItemCommonV0>(item.data);
                print_map_item::<MapItemEnvpointsV1>(item.data);
            },
            _ => {
                print_map_item::<MapItemCommonV0>(item.data);
                panic!("unknown datafile item type {}", item.type_id);
            },
        }
        println!("");
    }
}
