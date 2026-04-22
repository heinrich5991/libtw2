#![allow(unused_mut)]

use self::traits::RawBuilder as Impl;
use self::traits::RawSnap as _;
use bencher::benchmark_group;
use bencher::benchmark_main;
use bencher::black_box;
use bencher::Bencher;
use itertools::Itertools as _;
use libtw2_common::num::Cast as _;
use libtw2_snapshot::snap::RawBuilder as Libtw2;
use libtw2_snapshot_reference::snap::RawBuilder as Reference;
use libtw2_gamenet::snap_obj::obj_size;
use rand::Rng as _;
use rand::SeedableRng as _;
use rand_chacha::ChaCha8Rng as DeterministicRng;

mod traits {
    use libtw2_buffer::CapacityError;
    use libtw2_snapshot::snap as libtw2_snap;
    use libtw2_snapshot::snap::BuilderError;
    use libtw2_snapshot_reference::snap as reference_snap;

    pub trait RawBuilder: Default {
        type RawSnap: RawSnap<RawBuilder = Self>;
        fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError>;
        fn finish(self) -> Self::RawSnap;
    }
    pub trait RawSnap {
        type RawBuilder: RawBuilder<RawSnap = Self>;
        fn write_to_ints<'a>(
            &mut self,
            buf: &mut Vec<i32>,
            result: &'a mut [i32],
        ) -> Result<&'a [i32], CapacityError>;
        fn recycle(self) -> Self::RawBuilder;
    }

    impl RawBuilder for libtw2_snap::RawBuilder {
        type RawSnap = libtw2_snap::RawSnap;
        fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError> {
            self.add_item(type_id, id, data)
        }
        fn finish(self) -> libtw2_snap::RawSnap {
            self.finish()
        }
    }
    impl RawSnap for libtw2_snap::RawSnap {
        type RawBuilder = libtw2_snap::RawBuilder;
        fn write_to_ints<'a>(
            &mut self,
            buf: &mut Vec<i32>,
            result: &'a mut [i32],
        ) -> Result<&'a [i32], CapacityError> {
            (&*self).write_to_ints(buf, result)
        }
        fn recycle(self) -> libtw2_snap::RawBuilder {
            self.recycle()
        }
    }

    impl RawBuilder for reference_snap::RawBuilder {
        type RawSnap = reference_snap::RawSnap;
        fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError> {
            self.add_item(type_id, id, data).map_err(|i| match i {})
        }
        fn finish(self) -> reference_snap::RawSnap {
            self.finish()
        }
    }
    impl RawSnap for reference_snap::RawSnap {
        type RawBuilder = reference_snap::RawBuilder;
        fn write_to_ints<'a>(
            &mut self,
            buf: &mut Vec<i32>,
            result: &'a mut [i32],
        ) -> Result<&'a [i32], CapacityError> {
            self.write_to_ints(buf, result)
        }
        fn recycle(self) -> reference_snap::RawBuilder {
            self.recycle()
        }
    }
}

fn deterministic_rng() -> DeterministicRng {
    DeterministicRng::from_seed([0; 32])
}

struct Item {
    type_id: u16,
    id: u16,
    data: Vec<i32>,
}

fn bench<I: Impl>(bencher: &mut Bencher, items: Vec<Item>) {
    let mut out = &mut (0..16384).map(|_| 0).collect_vec();
    let mut buffer = Vec::new();
    let mut builder_buf = Some(I::default());
    bencher.iter(|| {
        let mut builder = builder_buf.take().unwrap();
        for &Item {
            type_id,
            id,
            ref data,
        } in black_box(&items)
        {
            builder.add_item(type_id, id, data).unwrap();
        }
        let mut snap = builder.finish();
        black_box(snap.write_to_ints(&mut buffer, &mut out).unwrap());
        builder_buf = Some(snap.recycle());
    });
}

fn empty() -> Vec<Item> {
    Vec::new()
}

fn _300_items() -> Vec<Item> {
    let mut result = Vec::new();
    let mut rng = deterministic_rng();
    for i in 0..300 {
        let type_id = i / 16 + 1;
        result.push(Item {
            type_id,
            id: rng.gen(),
            data: vec![rng.gen(); obj_size(type_id).expect("known obj size").usize()],
        });
    }
    result
}

macro_rules! benches {
    ($($items:ident, $libtw2:ident, $reference:ident;)*) => {
        $(
            fn $libtw2(bencher: &mut Bencher) {
                bench::<Libtw2>(bencher, $items());
            }
            fn $reference(bencher: &mut Bencher) {
                bench::<Reference>(bencher, $items());
            }
        )+
        benchmark_group!(
            building,
            $($libtw2, $reference,)+
        );
    }
}

benches! {
    empty, empty_libtw2, empty_reference;
    _300_items, _300_items_libtw2, _300_items_reference;
}
benchmark_main!(building);
