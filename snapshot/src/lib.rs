#[macro_use] extern crate common;
extern crate gamenet;
extern crate num;
extern crate packer;
extern crate vec_map;
extern crate warn;

pub mod format;
pub mod receiver;
pub mod snap;

use num::ToPrimitive;
use std::ops;

fn to_usize(r: ops::Range<u32>) -> ops::Range<usize> {
    r.start.to_usize().unwrap()..r.end.to_usize().unwrap()
}
