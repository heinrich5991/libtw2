extern crate buffer;
extern crate gamenet;
extern crate packer;
extern crate snapshot;
extern crate warn;

use buffer::CapacityError;
use gamenet::snap_obj::obj_size;
use packer::Unpacker;
use packer::with_packer;
use snapshot::snap::Delta;
use snapshot::snap::DeltaReader;
use snapshot::snap::Snap;
use warn::Panic;

const FIRST_DATA: &'static [i32] = &[0,18,0,4,18,1744,1072,2,3,4,17,1840,912,1,0,4,16,880,880,0,0,4,15,1840,848,1,0,4,14,912,848,0,0,4,13,880,848,1,0,4,12,848,848,0,0,4,11,880,816,0,0,4,9,1264,656,0,0,4,8,1104,656,0,0,4,7,912,656,0,0,4,6,1712,624,2,2,4,5,1840,432,1,0,4,3,1840,336,1,0,9,0,292,1584,305,0,128,0,0,0,-1,0,0,1584,304,0,0,0,10,0,10,1,0,0,6,0,0,0,0,0,20,0,0,1,11,0,-287183387,-320474125,-1594563099,-2139062272,-2139062144,-2139062144,-2139062272,-1,-454695199,-169020288,-2139062144,-2139062144,-2139062144,-2139062272,0,65408,65408,10,0,1,0,0,0,0,];
const FIRST_CRC: i32 = 0x5b96263a;

const SECOND_DATA: &'static [i32] = &[0,1,0,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,];
const SECOND_CRC: i32 = 0x5b96263b;

#[test]
fn simple() {
    let mut buf = Vec::with_capacity(4096);
    let mut reader = DeltaReader::new();
    let mut delta = Delta::new();
    let mut prev = Snap::empty();
    let mut snap = Snap::default();

    with_packer(&mut buf, |mut p| -> Result<_, CapacityError> {
        for &d in &FIRST_DATA[..FIRST_DATA.len()] {
            p.write_int(d)?;
        }
        Ok(p.written())
    }).unwrap();

    reader.read(&mut Panic, &mut delta, obj_size, &mut Unpacker::new(&buf)).unwrap();
    snap.read_with_delta(&mut Panic, &prev, &delta).unwrap();
    println!("{:?}", snap);
    assert_eq!(snap.crc(), FIRST_CRC);

    prev = snap;
    snap = Snap::default();

    buf.clear();
    with_packer(&mut buf, |mut p| -> Result<_, CapacityError> {
        for &d in &SECOND_DATA[..SECOND_DATA.len()] {
            p.write_int(d)?;
        }
        Ok(p.written())
    }).unwrap();

    reader.read(&mut Panic, &mut delta, obj_size, &mut Unpacker::new(&buf)).unwrap();
    snap.read_with_delta(&mut Panic, &prev, &delta).unwrap();
    println!("{:?}", snap);
    assert_eq!(snap.crc(), SECOND_CRC);
}
