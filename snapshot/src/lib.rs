extern crate buffer;
#[macro_use] extern crate common;
extern crate gamenet;
extern crate num;
extern crate packer;
extern crate vec_map;
extern crate warn;

pub mod format;
pub mod manager;
pub mod receiver;
pub mod snap;
pub mod storage;

pub use manager::Manager;
pub use receiver::DeltaReceiver;
pub use receiver::ReceivedDelta;
pub use snap::Delta;
pub use snap::DeltaReader;
pub use snap::Snap;
pub use storage::Storage;

use num::ToPrimitive;
use std::ops;

fn to_usize(r: ops::Range<u32>) -> ops::Range<usize> {
    r.start.to_usize().unwrap()..r.end.to_usize().unwrap()
}
