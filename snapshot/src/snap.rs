use crate::format::key;
use crate::format::key_to_id;
use crate::format::key_to_type_id;
use crate::format::DeltaHeader;
use crate::format::Item;
use crate::format::SnapHeader;
use crate::format::Warning;
use crate::to_usize;
use buffer::CapacityError;
use libtw2_common::num::Cast;
use libtw2_gamenet_snap as msg;
use libtw2_gamenet_snap::SnapMsg;
use libtw2_gamenet_snap::MAX_SNAPSHOT_PACKSIZE;
use libtw2_packer::with_packer;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Packer;
use libtw2_packer::UnexpectedEnd;
use libtw2_packer::Unpacker;
use std::cmp;
use std::collections::btree_map;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::iter;
use std::mem;
use std::ops;
use warn::wrap;
use warn::Warn;

// TODO: Actually obey this the same way as Teeworlds does.
pub const MAX_SNAPSHOT_SIZE: usize = 64 * 1024; // 64 KB

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    UnexpectedEnd,
    IntOutOfRange,
    DeletedItemsUnpacking,
    ItemDiffsUnpacking,
    TypeIdRange,
    IdRange,
    NegativeSize,
    TooLongDiff,
    TooLongSnap,
    DeltaDifferingSizes,
    OffsetsUnpacking,
    InvalidOffset,
    ItemsUnpacking,
    DuplicateKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuilderError {
    DuplicateKey,
    TooLongSnap,
}

impl From<BuilderError> for Error {
    fn from(err: BuilderError) -> Error {
        match err {
            BuilderError::DuplicateKey => Error::DuplicateKey,
            BuilderError::TooLongSnap => Error::TooLongSnap,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TooLongSnap;

impl From<TooLongSnap> for Error {
    fn from(_: TooLongSnap) -> Error {
        Error::TooLongSnap
    }
}

impl From<TooLongSnap> for BuilderError {
    fn from(_: TooLongSnap) -> BuilderError {
        BuilderError::TooLongSnap
    }
}

impl From<libtw2_packer::IntOutOfRange> for Error {
    fn from(_: libtw2_packer::IntOutOfRange) -> Error {
        Error::IntOutOfRange
    }
}

impl From<UnexpectedEnd> for Error {
    fn from(UnexpectedEnd: UnexpectedEnd) -> Error {
        Error::UnexpectedEnd
    }
}

fn apply_delta(in_: Option<&[i32]>, delta: &[i32], out: &mut [i32]) -> Result<(), Error> {
    assert!(delta.len() == out.len());
    match in_ {
        Some(in_) => {
            if in_.len() != out.len() {
                return Err(Error::DeltaDifferingSizes);
            }
            for i in 0..out.len() {
                out[i] = in_[i].wrapping_add(delta[i]);
            }
        }
        None => out.copy_from_slice(delta),
    }
    Ok(())
}

fn create_delta(from: Option<&[i32]>, to: &[i32], out: &mut [i32]) {
    assert!(to.len() == out.len());
    match from {
        Some(from) => {
            assert!(from.len() == to.len());
            for i in 0..out.len() {
                out[i] = to[i].wrapping_sub(from[i]);
            }
        }
        None => out.copy_from_slice(to),
    }
}

#[derive(Clone, Default)]
pub struct Snap {
    offsets: BTreeMap<i32, ops::Range<u32>>,
    buf: Vec<i32>,
}

impl Snap {
    pub fn empty() -> Snap {
        Default::default()
    }
    fn clear(&mut self) {
        self.offsets.clear();
        self.buf.clear();
    }
    fn item_from_offset(&self, offset: ops::Range<u32>) -> &[i32] {
        &self.buf[to_usize(offset)]
    }
    pub fn item(&self, type_id: u16, id: u16) -> Option<&[i32]> {
        self.offsets
            .get(&key(type_id, id))
            .map(|o| &self.buf[to_usize(o.clone())])
    }
    pub fn items(&self) -> Items {
        Items {
            snap: self,
            iter: self.offsets.iter(),
        }
    }
    fn prepare_item_vacant<'a>(
        entry: btree_map::VacantEntry<'a, i32, ops::Range<u32>>,
        buf: &mut Vec<i32>,
        size: usize,
    ) -> Result<&'a mut ops::Range<u32>, TooLongSnap> {
        let offset = buf.len();
        if offset + size > MAX_SNAPSHOT_SIZE {
            return Err(TooLongSnap);
        }
        let start = offset.assert_u32();
        let end = (offset + size).assert_u32();
        buf.extend(iter::repeat(0).take(size));
        Ok(entry.insert(start..end))
    }
    fn add_item_raw(
        &mut self,
        type_id: u16,
        id: u16,
        size: usize,
    ) -> Result<&mut [i32], BuilderError> {
        let offset = match self.offsets.entry(key(type_id, id)) {
            btree_map::Entry::Occupied(..) => return Err(BuilderError::DuplicateKey),
            btree_map::Entry::Vacant(v) => Snap::prepare_item_vacant(v, &mut self.buf, size)?,
        }
        .clone();
        Ok(&mut self.buf[to_usize(offset)])
    }
    fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError> {
        self.add_item_raw(type_id, id, data.len())?
            .copy_from_slice(data);
        Ok(())
    }
    fn prepare_item(&mut self, type_id: u16, id: u16, size: usize) -> Result<&mut [i32], Error> {
        let offset = match self.offsets.entry(key(type_id, id)) {
            btree_map::Entry::Occupied(o) => o.into_mut(),
            btree_map::Entry::Vacant(v) => Snap::prepare_item_vacant(v, &mut self.buf, size)?,
        }
        .clone();
        Ok(&mut self.buf[to_usize(offset)])
    }
    pub fn read<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
        buf: &mut Vec<i32>,
        data: &[u8],
    ) -> Result<(), Error> {
        self.clear();
        buf.clear();

        let mut unpacker = Unpacker::new(data);
        while !unpacker.is_empty() {
            match unpacker.read_int(wrap(warn)) {
                Ok(int) => buf.push(int),
                Err(UnexpectedEnd) => {
                    warn.warn(Warning::ExcessSnapData);
                    break;
                }
            }
        }

        self.read_from_ints(warn, &buf)
    }
    pub fn read_from_ints<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
        data: &[i32],
    ) -> Result<(), Error> {
        self.clear();

        let mut unpacker = IntUnpacker::new(data);
        let header = SnapHeader::decode_obj(&mut unpacker)?;
        let data = unpacker.as_slice();

        let offsets_len = header.num_items.assert_usize();
        if data.len() < offsets_len {
            return Err(Error::OffsetsUnpacking);
        }
        if header.data_size % 4 != 0 {
            return Err(Error::InvalidOffset);
        }
        let items_len = (header.data_size / 4).assert_usize();
        match (offsets_len + items_len).cmp(&data.len()) {
            cmp::Ordering::Less => warn.warn(Warning::ExcessSnapData),
            cmp::Ordering::Equal => {}
            cmp::Ordering::Greater => return Err(Error::ItemsUnpacking),
        }

        let (offsets, item_data) = data.split_at(offsets_len);
        let item_data = &item_data[..items_len];

        let mut offsets = offsets.iter();
        let mut prev_offset = None;
        loop {
            let offset = offsets.next().copied();
            if let Some(offset) = offset {
                if offset < 0 {
                    return Err(Error::InvalidOffset);
                }
                if offset % 4 != 0 {
                    return Err(Error::InvalidOffset);
                }
            }
            let finished = offset.is_none();
            let offset = offset.map(|o| o.assert_usize() / 4).unwrap_or(items_len);

            if let Some(prev_offset) = prev_offset {
                if offset <= prev_offset {
                    return Err(Error::InvalidOffset);
                }
                if offset > items_len {
                    return Err(Error::InvalidOffset);
                }
                let type_id = key_to_type_id(item_data[prev_offset]);
                let id = key_to_id(item_data[prev_offset]);
                self.add_item(type_id, id, &item_data[prev_offset + 1..offset])?;
            } else if offset != 0 {
                // First offset must be 0.
                return Err(Error::InvalidOffset);
            }

            prev_offset = Some(offset);

            if finished {
                break;
            }
        }
        Ok(())
    }
    pub fn read_with_delta<W>(
        &mut self,
        warn: &mut W,
        from: &Snap,
        delta: &Delta,
    ) -> Result<(), Error>
    where
        W: Warn<Warning>,
    {
        self.clear();

        let mut num_deletions = 0;
        for item in from.items() {
            if !delta.deleted_items.contains(&item.key()) {
                let out = self.prepare_item(item.type_id, item.id, item.data.len())?;
                out.copy_from_slice(item.data);
            } else {
                num_deletions += 1;
            }
        }
        if num_deletions != delta.deleted_items.len() {
            warn.warn(Warning::UnknownDelete);
        }

        for (&key, offset) in &delta.updated_items {
            let type_id = key_to_type_id(key);
            let id = key_to_id(key);
            let diff = &delta.buf[to_usize(offset.clone())];
            let out = self.prepare_item(type_id, id, diff.len())?;
            let in_ = from.item(type_id, id);

            apply_delta(in_, diff, out)?;
        }
        Ok(())
    }
    pub fn write<'d, 's>(
        &self,
        buf: &mut Vec<i32>,
        mut p: Packer<'d, 's>,
    ) -> Result<&'d [u8], CapacityError> {
        let keys = buf;
        keys.clear();
        keys.extend(self.offsets.keys().cloned());
        keys.sort_unstable_by_key(|&k| k as u32);
        let data_size = self
            .buf
            .len()
            .checked_add(self.offsets.len())
            .expect("snap size overflow")
            .checked_mul(mem::size_of::<i32>())
            .expect("snap size overflow")
            .assert_i32();
        p.write_int(data_size)?;
        let num_items = self.offsets.len().assert_i32();
        p.write_int(num_items)?;

        let mut offset = 0;
        for &key in &*keys {
            p.write_int(offset)?;
            let key_offset = self.offsets[&key].clone();
            offset = offset
                .checked_add(
                    (key_offset.end - key_offset.start + 1)
                        .usize()
                        .checked_mul(mem::size_of::<i32>())
                        .expect("item size overflow")
                        .assert_i32(),
                )
                .expect("offset overflow");
        }
        for &key in &*keys {
            p.write_int(key)?;
            for &i in &self.buf[to_usize(self.offsets[&key].clone())] {
                p.write_int(i)?;
            }
        }
        Ok(p.written())
    }
    pub fn crc(&self) -> i32 {
        self.buf.iter().fold(0, |s, &a| s.wrapping_add(a))
    }
    pub fn recycle(mut self) -> Builder {
        self.clear();
        Builder { snap: self }
    }
}

fn read_int_err<W: Warn<Warning>>(p: &mut Unpacker, w: &mut W, e: Error) -> Result<i32, Error> {
    p.read_int(wrap(w)).map_err(|_| e)
}

pub struct Items<'a> {
    snap: &'a Snap,
    iter: btree_map::Iter<'a, i32, ops::Range<u32>>,
}

impl<'a> Iterator for Items<'a> {
    type Item = Item<'a>;
    fn next(&mut self) -> Option<Item<'a>> {
        self.iter
            .next()
            .map(|(&k, o)| Item::from_key(k, self.snap.item_from_offset(o.clone())))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for Items<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl fmt::Debug for Snap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(
                self.items()
                    .map(|Item { type_id, id, data }| ((type_id, id), data)),
            )
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct Delta {
    deleted_items: BTreeSet<i32>,
    updated_items: BTreeMap<i32, ops::Range<u32>>,
    buf: Vec<i32>,
}

impl Delta {
    pub fn new() -> Delta {
        Default::default()
    }
    pub fn clear(&mut self) {
        self.deleted_items.clear();
        self.updated_items.clear();
        self.buf.clear();
    }
    fn prepare_update_item(&mut self, type_id: u16, id: u16, size: usize) -> &mut [i32] {
        let key = key(type_id, id);

        let offset = self.buf.len();
        let start = offset.assert_u32();
        let end = (offset + size).assert_u32();
        self.buf.extend(iter::repeat(0).take(size));
        assert!(self.updated_items.insert(key, start..end).is_none());
        &mut self.buf[to_usize(start..end)]
    }
    pub fn create(&mut self, from: &Snap, to: &Snap) {
        self.clear();
        for Item { type_id, id, .. } in from.items() {
            if to.item(type_id, id).is_none() {
                assert!(self.deleted_items.insert(key(type_id, id)));
            }
        }
        for Item { type_id, id, data } in to.items() {
            let from_data = from.item(type_id, id);
            let out_delta = self.prepare_update_item(type_id, id, data.len());
            create_delta(from_data, data, out_delta);
        }
    }
    pub fn write<'d, 's, O>(
        &self,
        object_size: O,
        mut p: Packer<'d, 's>,
    ) -> Result<&'d [u8], CapacityError>
    where
        O: FnMut(u16) -> Option<u32>,
    {
        let mut object_size = object_size;
        with_packer(&mut p, |p| {
            DeltaHeader {
                num_deleted_items: self.deleted_items.len().assert_i32(),
                num_updated_items: self.updated_items.len().assert_i32(),
            }
            .encode(p)
        })?;
        for &key in &self.deleted_items {
            p.write_int(key)?;
        }
        for (&key, range) in &self.updated_items {
            let data = &self.buf[to_usize(range.clone())];
            let type_id = key_to_type_id(key);
            let id = key_to_id(key);
            p.write_int(type_id.i32())?;
            p.write_int(id.i32())?;
            match object_size(type_id) {
                Some(size) => assert!(size.usize() == data.len()),
                None => p.write_int(data.len().assert_i32())?,
            }
            for &d in data {
                p.write_int(d)?;
            }
        }
        Ok(p.written())
    }

    pub fn read<W, O>(
        &mut self,
        warn: &mut W,
        object_size: O,
        p: &mut Unpacker,
    ) -> Result<(), Error>
    where
        W: Warn<Warning>,
        O: FnMut(u16) -> Option<u32>,
    {
        self.clear();

        let mut object_size = object_size;

        let header = DeltaHeader::decode(warn, p)?;

        for _ in 0..header.num_deleted_items {
            self.deleted_items
                .insert(read_int_err(p, warn, Error::DeletedItemsUnpacking)?);
        }
        if header.num_deleted_items.assert_usize() != self.deleted_items.len() {
            warn.warn(Warning::DuplicateDelete);
        }

        let mut num_updates = 0;

        while !p.is_empty() {
            let type_id = read_int_err(p, warn, Error::ItemDiffsUnpacking)?;
            let id = read_int_err(p, warn, Error::ItemDiffsUnpacking)?;

            let type_id = type_id.try_u16().ok_or(Error::TypeIdRange)?;
            let id = id.try_u16().ok_or(Error::IdRange)?;

            let size = match object_size(type_id) {
                Some(s) => s,
                None => {
                    let s = read_int_err(p, warn, Error::ItemDiffsUnpacking)?;
                    s.try_u32().ok_or(Error::NegativeSize)?
                }
            };
            let start = self.buf.len().try_u32().ok_or(Error::TooLongDiff)?;
            let end = start.checked_add(size).ok_or(Error::TooLongDiff)?;
            for _ in 0..size {
                self.buf
                    .push(read_int_err(p, warn, Error::ItemDiffsUnpacking)?);
            }

            // In case of conflict, take later update (as the original code does).
            if self
                .updated_items
                .insert(key(type_id, id), start..end)
                .is_some()
            {
                warn.warn(Warning::DuplicateUpdate);
            }

            if self.deleted_items.contains(&key(type_id, id)) {
                warn.warn(Warning::DeleteUpdate);
            }
            num_updates += 1;
        }

        if num_updates != header.num_updated_items {
            warn.warn(Warning::NumUpdatedItems);
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Builder {
    snap: Snap,
}

impl Builder {
    pub fn new() -> Builder {
        Default::default()
    }
    pub fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError> {
        self.snap.add_item(type_id, id, data)
    }
    pub fn finish(self) -> Snap {
        self.snap
    }
}

pub fn delta_chunks(tick: i32, delta_tick: i32, data: &[u8], crc: i32) -> DeltaChunks {
    DeltaChunks {
        tick: tick,
        delta_tick: tick - delta_tick,
        crc: crc,
        cur_part: if !data.is_empty() { 0 } else { -1 },
        num_parts: ((data.len() + MAX_SNAPSHOT_PACKSIZE as usize - 1)
            / MAX_SNAPSHOT_PACKSIZE as usize)
            .assert_i32(),
        data: data,
    }
}

pub struct DeltaChunks<'a> {
    tick: i32,
    delta_tick: i32,
    crc: i32,
    cur_part: i32,
    num_parts: i32,
    data: &'a [u8],
}

impl<'a> Iterator for DeltaChunks<'a> {
    type Item = SnapMsg<'a>;
    fn next(&mut self) -> Option<SnapMsg<'a>> {
        if self.cur_part == self.num_parts {
            return None;
        }
        let result = if self.num_parts == 0 {
            SnapMsg::SnapEmpty(msg::SnapEmpty {
                tick: self.tick,
                delta_tick: self.delta_tick,
            })
        } else if self.num_parts == 1 {
            SnapMsg::SnapSingle(msg::SnapSingle {
                tick: self.tick,
                delta_tick: self.delta_tick,
                crc: self.crc,
                data: self.data,
            })
        } else {
            let index = self.cur_part.assert_usize();
            let start = MAX_SNAPSHOT_PACKSIZE as usize * index;
            let end = cmp::min(
                MAX_SNAPSHOT_PACKSIZE as usize * (index + 1),
                self.data.len(),
            );
            SnapMsg::Snap(msg::Snap {
                tick: self.tick,
                delta_tick: self.delta_tick,
                num_parts: self.num_parts,
                part: self.cur_part,
                crc: self.crc,
                data: &self.data[start..end],
            })
        };
        self.cur_part += 1;
        Some(result)
    }
}
