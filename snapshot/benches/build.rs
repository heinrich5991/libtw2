#![allow(unused_mut)]

use self::traits::Delta as _;
use self::traits::Implementation;
use self::traits::Libtw2;
use self::traits::RawBuilder;
use self::traits::RawSnap;
use self::traits::Reference;
use bencher::benchmark_group;
use bencher::benchmark_main;
use bencher::black_box;
use bencher::Bencher;
use itertools::Itertools as _;
use libtw2_common::num::Cast as _;
use libtw2_gamenet::snap_obj::obj_size;
use rand::Rng as _;
use rand::SeedableRng as _;
use rand_chacha::ChaCha8Rng as DeterministicRng;

mod traits {
    use libtw2_buffer::CapacityError;
    use libtw2_snapshot::snap as libtw2_snap;
    use libtw2_snapshot::snap::BuilderError;
    use libtw2_snapshot_reference::snap as reference_snap;

    pub trait Implementation {
        type Delta: Delta<RawSnap = Self::RawSnap>;
        type RawBuilder: RawBuilder<RawSnap = Self::RawSnap>;
        type RawSnap: RawSnap<RawBuilder = Self::RawBuilder>;
    }

    pub struct Libtw2;
    pub struct Reference;

    impl Implementation for Libtw2 {
        type Delta = libtw2_snap::Delta;
        type RawBuilder = libtw2_snap::RawBuilder;
        type RawSnap = libtw2_snap::RawSnap;
    }

    impl Implementation for Reference {
        type Delta = reference_snap::Delta;
        type RawBuilder = reference_snap::RawBuilder;
        type RawSnap = reference_snap::RawSnap;
    }

    pub trait Delta: Default {
        type RawSnap: RawSnap;
        fn create_raw_and_write_to_ints<'a>(
            &mut self,
            from: &Self::RawSnap,
            to: &Self::RawSnap,
            obj_size: fn(u16) -> Option<u32>,
            result: &'a mut [i32],
        ) -> Result<&'a [i32], CapacityError>;
    }
    pub trait RawBuilder: Default {
        type RawSnap: RawSnap<RawBuilder = Self>;
        fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError>;
        fn finish(self) -> Self::RawSnap;
    }
    pub trait RawSnap {
        type RawBuilder: RawBuilder<RawSnap = Self>;
        fn write_to_ints<'a>(&mut self, result: &'a mut [i32]) -> Result<&'a [i32], CapacityError>;
        fn recycle(self) -> Self::RawBuilder;
    }

    impl Delta for libtw2_snap::Delta {
        type RawSnap = libtw2_snap::RawSnap;
        fn create_raw_and_write_to_ints<'a>(
            &mut self,
            from: &libtw2_snap::RawSnap,
            to: &libtw2_snap::RawSnap,
            obj_size: fn(u16) -> Option<u32>,
            result: &'a mut [i32],
        ) -> Result<&'a [i32], CapacityError> {
            self.create_raw(from, to);
            self.write_to_ints(obj_size, result)
        }
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
        fn write_to_ints<'a>(&mut self, result: &'a mut [i32]) -> Result<&'a [i32], CapacityError> {
            (&*self).write_to_ints(result)
        }
        fn recycle(self) -> libtw2_snap::RawBuilder {
            self.recycle()
        }
    }

    impl Delta for reference_snap::Delta {
        type RawSnap = reference_snap::RawSnap;
        fn create_raw_and_write_to_ints<'a>(
            &mut self,
            from: &reference_snap::RawSnap,
            to: &reference_snap::RawSnap,
            obj_size: fn(u16) -> Option<u32>,
            result: &'a mut [i32],
        ) -> Result<&'a [i32], CapacityError> {
            self.create_raw_and_write_to_ints(from, to, obj_size, result)
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
        fn write_to_ints<'a>(&mut self, result: &'a mut [i32]) -> Result<&'a [i32], CapacityError> {
            self.write_to_ints(result)
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

#[rustfmt::skip]
fn add_items<B: RawBuilder>(builder: &mut B, items: &[Item]) {
    for &Item { type_id, id, ref data } in items {
        builder.add_item(type_id, id, data).unwrap();
    }
}

fn snap_from_items<S: RawSnap>(items: &[Item]) -> S {
    let mut builder = S::RawBuilder::default();
    add_items(&mut builder, items);
    builder.finish()
}

fn bench_snapwrite<I: Implementation>(bencher: &mut Bencher, items: Vec<Item>) {
    let mut out = (0..16384).map(|_| 0).collect_vec();
    let mut builder_buf = Some(I::RawBuilder::default());
    bencher.iter(|| {
        let mut builder = builder_buf.take().unwrap();
        add_items(&mut builder, black_box(&items));
        let mut snap = builder.finish();
        black_box(snap.write_to_ints(&mut out).unwrap());
        builder_buf = Some(snap.recycle());
    });
}

fn bench_snap<I: Implementation>(bencher: &mut Bencher, items: Vec<Item>) {
    let mut builder_buf = Some(I::RawBuilder::default());
    bencher.iter(|| {
        let mut builder = builder_buf.take().unwrap();
        add_items(&mut builder, black_box(&items));
        let mut snap = builder.finish();
        black_box(&snap);
        builder_buf = Some(snap.recycle());
    });
}

fn bench_delta<I: Implementation>(
    bencher: &mut Bencher,
    from_items: Vec<Item>,
    to_items: Vec<Item>,
) {
    let mut out = (0..16384).map(|_| 0).collect_vec();

    let mut delta = I::Delta::default();
    let from: I::RawSnap = snap_from_items(&from_items);
    let to: I::RawSnap = snap_from_items(&to_items);

    bencher.iter(|| {
        black_box(
            delta
                .create_raw_and_write_to_ints(black_box(&from), black_box(&to), obj_size, &mut out)
                .unwrap(),
        );
    });
}

fn bench_snapdelta<I: Implementation>(
    bencher: &mut Bencher,
    from_items: Vec<Item>,
    to_items: Vec<Item>,
) {
    let mut out = (0..16384).map(|_| 0).collect_vec();

    let mut delta = I::Delta::default();
    let from: I::RawSnap = snap_from_items(&from_items);

    let mut to_buf = Some(I::RawBuilder::default());
    bencher.iter(|| {
        let mut to = to_buf.take().unwrap();
        add_items(&mut to, black_box(&to_items));
        let mut to = to.finish();
        black_box(
            delta
                .create_raw_and_write_to_ints(black_box(&from), &to, obj_size, &mut out)
                .unwrap(),
        );
        to_buf = Some(to.recycle());
    });
}

fn empty() -> Vec<Item> {
    Vec::new()
}

fn _300_items() -> Vec<Item> {
    let mut rng = deterministic_rng();

    let mut result = Vec::new();
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

fn _300_items_modified() -> Vec<Item> {
    let mut rng = deterministic_rng();

    let mut result = _300_items();
    for item in &mut result {
        if rng.gen_bool(0.1) {
            item.data = vec![rng.gen(); item.data.len()];
        }
    }
    result
}

macro_rules! benches {
    ($($fn:ident($($args:tt)*), $libtw2:ident, $reference:ident;)*) => {
        $(
            fn $libtw2(bencher: &mut Bencher) {
                $fn::<Libtw2>(bencher, $($args)*);
            }
            fn $reference(bencher: &mut Bencher) {
                $fn::<Reference>(bencher, $($args)*);
            }
        )+
        benchmark_group!(
            building,
            $($libtw2, $reference,)+
        );
    }
}

benches! {
    bench_snap(empty()), snap_empty_libtw2, snap_empty_reference;
    bench_snap(_300_items()), snap_300_libtw2, snap_300_reference;
    bench_snapwrite(empty()), snapwrite_empty_libtw2, snapwrite_empty_reference;
    bench_snapwrite(_300_items()), snapwrite_300_libtw2, snapwrite_300_reference;
    bench_delta(empty(), empty()), delta_empty_empty_libtw2, delta_empty_empty_reference;
    bench_delta(_300_items(), _300_items()), delta_300_300_libtw2, delta_300_300_reference;
    bench_delta(_300_items(), _300_items_modified()), delta_300_300m_libtw2, delta_300_300m_reference;
    bench_snapdelta(empty(), empty()), snapdelta_empty_empty_libtw2, snapdelta_empty_empty_reference;
    bench_snapdelta(_300_items(), _300_items()), snapdelta_300_300_libtw2, snapdelta_300_300_reference;
    bench_snapdelta(_300_items(), _300_items_modified()), snapdelta_300_300m_libtw2, snapdelta_300_300m_reference;
}
benchmark_main!(building);
