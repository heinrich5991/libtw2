extern crate buffer;
extern crate common;
extern crate gamenet_teeworlds_0_6 as gamenet;
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

use common::num::Cast;
use std::ops;

fn to_usize(r: ops::Range<u32>) -> ops::Range<usize> {
    r.start.usize()..r.end.usize()
}
