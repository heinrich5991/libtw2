#![feature(phase)]

#[phase(plugin)]
extern crate map_macros;

extern crate datafile;

extern crate debug;

use std::io::File;

use datafile::Datafile;
use datafile::DatafileItem;
use datafile::DatafileReader;

fn main() {
    let file = box File::open(&Path::new("../dm1.map")).unwrap();
    let dfr = DatafileReader::read(file).unwrap().unwrap();

    for item in dfr.items() {
        match item.type_id {
            0 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemVersionV0::from_slice(item.data));
                print!("{:?} ", MapItemVersionV1::from_slice(item.data));
                println!("");
            },
            1 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemInfoV0::from_slice(item.data));
                print!("{:?} ", MapItemInfoV1::from_slice(item.data));
                println!("");
            },
            2 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemImageV0::from_slice(item.data));
                print!("{:?} ", MapItemImageV1::from_slice(item.data));
                print!("{:?} ", MapItemImageV2::from_slice(item.data));
                println!("");
            },
            3 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemEnvelopeV0::from_slice(item.data));
                print!("{:?} ", MapItemEnvelopeV1::from_slice(item.data));
                print!("{:?} ", MapItemEnvelopeV2::from_slice(item.data));
                println!("");
            },
            4 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemGroupV0::from_slice(item.data));
                print!("{:?} ", MapItemGroupV1::from_slice(item.data));
                print!("{:?} ", MapItemGroupV2::from_slice(item.data));
                println!("");
            },
            5 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemLayerV0::from_slice(item.data));
                print!("{:?} ", MapItemLayerV1::from_slice(item.data));
                println!("");
            },
            6 => {
                print!("{} {} ", item.type_id, item.id);
                print!("{:?} ", MapItemEnvPointsV0::from_slice(item.data));
                print!("{:?} ", MapItemEnvPointsV1::from_slice(item.data));
                println!("");
            },
            _ => {
                println!("{} {}", item.type_id, item.id);
                fail!("unknown datafile item type");
            },
        }
        //println!("{}", item);
    }
}
