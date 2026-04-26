use crate::format::apply_item_delta;
use crate::format::create_item_delta;
use crate::format::item_data_to_uuid;
use crate::format::key;
use crate::format::key_to_id;
use crate::format::key_to_raw_type_id;
use crate::format::uuid_to_item_data;
use crate::format::DeltaDifferingSizes;
use crate::format::DeltaHeader;
use crate::format::Item;
use crate::format::RawItem;
use crate::format::SnapHeader;
use crate::format::TypeId;
use crate::format::Warning;
use crate::format::OFFSET_EXTENDED_TYPE_ID;
use crate::format::TYPE_ID_EX;
use crate::to_usize;
use crate::ReadInt;
use libtw2_buffer::CapacityError;
use libtw2_common::num::Cast;
use libtw2_gamenet_snap as msg;
use libtw2_gamenet_snap::SnapMsg;
use libtw2_gamenet_snap::MAX_SNAPSHOT_PACKSIZE;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Packer;
use libtw2_packer::UnexpectedEnd;
use libtw2_packer::Unpacker;
use libtw2_warn::wrap;
use libtw2_warn::Ignore;
use libtw2_warn::Warn;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::cmp;
use std::collections::hash_map;
use std::fmt;
use std::mem;
use std::ops;
use uuid::Uuid;

pub const MAX_SNAPSHOT_SIZE: usize = 64 * 1024; // 64 KB
pub const MAX_SNAPSHOT_ITEMS: usize = 1024;

#[derive(Clone, Default)]
struct SeenKeys {
    keys: Vec<u32>,
    gens: Vec<u32>,
    gen: u32,
    len: usize,
}

impl SeenKeys {
    #[inline]
    fn ensure_allocated(&mut self) {
        if self.keys.is_empty() {
            const CAP: usize = MAX_SNAPSHOT_ITEMS * 2;
            self.keys = vec![0; CAP];
            self.gens = vec![0; CAP];
            self.gen = 1;
            self.len = 0;
        }
    }

    #[inline]
    fn clear(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        self.len = 0;
        self.gen = self.gen.wrapping_add(1);
        if self.gen == 0 {
            self.gens.fill(0);
            self.gen = 1;
        }
    }

    #[inline]
    fn insert(&mut self, key: i32) -> bool {
        self.ensure_allocated();
        debug_assert!(self.len * 2 < self.keys.len(), "SeenKeys is over capacity");

        let key_u32 = key as u32;
        let mask = self.keys.len() - 1;
        debug_assert!(self.keys.len().is_power_of_two());

        // Very cheap multiplicative hash; key already has decent entropy.
        let mut idx = (key_u32.wrapping_mul(0x9E37_79B1) as usize) & mask;
        loop {
            if self.gens[idx] != self.gen {
                self.keys[idx] = key_u32;
                self.gens[idx] = self.gen;
                self.len += 1;
                return true;
            }
            if self.keys[idx] == key_u32 {
                return false;
            }
            idx = (idx + 1) & mask;
        }
    }
}

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
    TooManyItems,
    DeltaDifferingSizes,
    OffsetsUnpacking,
    InvalidOffset,
    ItemsUnpacking,
    DuplicateKey,
    DuplicateUuidType,
    InvalidUuidType,
    MissingUuidType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuilderError {
    DuplicateKey,
    TooLongSnap,
    TooManyItems,
}

impl From<BuilderError> for Error {
    fn from(err: BuilderError) -> Error {
        match err {
            BuilderError::DuplicateKey => Error::DuplicateKey,
            BuilderError::TooLongSnap => Error::TooLongSnap,
            BuilderError::TooManyItems => Error::TooManyItems,
        }
    }
}

impl From<DeltaDifferingSizes> for Error {
    fn from(_: DeltaDifferingSizes) -> Error {
        Error::DeltaDifferingSizes
    }
}

impl From<libtw2_packer::IntOutOfRange> for Error {
    fn from(_: libtw2_packer::IntOutOfRange) -> Error {
        Error::IntOutOfRange
    }
}

impl From<UnexpectedEnd> for Error {
    fn from(_: UnexpectedEnd) -> Error {
        Error::UnexpectedEnd
    }
}

#[derive(Default)]
pub struct RawSnap {
    items: Vec<RawEntry>,
    buf: Vec<i32>,
    seen: SeenKeys,
}

impl Clone for RawSnap {
    fn clone(&self) -> RawSnap {
        RawSnap {
            items: self.items.clone(),
            buf: self.buf.clone(),
            seen: SeenKeys::default(),
        }
    }
}

#[derive(Clone)]
struct RawEntry {
    key: i32,
    start: u32,
    end: u32,
}

impl RawSnap {
    pub fn empty() -> RawSnap {
        Default::default()
    }
    fn clear(&mut self) {
        self.items.clear();
        self.buf.clear();
        self.seen.clear();
    }
    #[inline]
    fn item_from_entry(&self, entry: &RawEntry) -> &[i32] {
        &self.buf[to_usize(entry.start..entry.end)]
    }
    pub fn item(&self, raw_type_id: u16, id: u16) -> Option<&[i32]> {
        self.items
            .binary_search_by_key(&(key(raw_type_id, id) as u32), |e| e.key as u32)
            .ok()
            .map(|idx| self.item_from_entry(&self.items[idx]))
    }
    pub fn items(&self) -> RawItems<'_> {
        RawItems {
            snap: self,
            iter: self.items.iter(),
        }
    }
    fn sort_items(&mut self) {
        if self.items.len() >= 2 {
            let mut grouped = true;
            let mut prev = key_to_raw_type_id(self.items[0].key);
            for e in &self.items[1..] {
                let cur = key_to_raw_type_id(e.key);
                if cur < prev {
                    grouped = false;
                    break;
                }
                prev = cur;
            }
            if grouped {
                let mut start = 0usize;
                while start < self.items.len() {
                    let t = key_to_raw_type_id(self.items[start].key);
                    let mut end = start + 1;
                    while end < self.items.len() && key_to_raw_type_id(self.items[end].key) == t {
                        end += 1;
                    }
                    if end - start > 1 {
                        self.items[start..end].sort_unstable_by_key(|e| key_to_id(e.key) as u32);
                    }
                    start = end;
                }
                return;
            }
        }

        self.items.sort_unstable_by_key(|e| e.key as u32);
    }
    #[inline]
    fn would_fit(num_items: usize, num_item_data_i32s: usize) -> bool {
        const MAX_INTS: usize = MAX_SNAPSHOT_SIZE / mem::size_of::<i32>();
        2 + (2 * num_items) + num_item_data_i32s <= MAX_INTS
    }
    fn push_item_uninit(&mut self, key: i32, size: usize) -> Result<ops::Range<u32>, BuilderError> {
        let num_items = self.items.len();
        if num_items + 1 > MAX_SNAPSHOT_ITEMS {
            return Err(BuilderError::TooManyItems);
        }

        let offset = self.buf.len();
        if !RawSnap::would_fit(num_items + 1, offset + size) {
            return Err(BuilderError::TooLongSnap);
        }
        let start = offset.assert_u32();
        let end = (offset + size).assert_u32();
        self.buf.resize(offset + size, 0);
        self.items.push(RawEntry { key, start, end });
        Ok(start..end)
    }
    fn push_item_copy(&mut self, key: i32, data: &[i32]) -> Result<(), BuilderError> {
        let num_items = self.items.len();
        if num_items + 1 > MAX_SNAPSHOT_ITEMS {
            return Err(BuilderError::TooManyItems);
        }

        let offset = self.buf.len();
        let size = data.len();
        if !RawSnap::would_fit(num_items + 1, offset + size) {
            return Err(BuilderError::TooLongSnap);
        }
        let start = offset.assert_u32();
        let end = (offset + size).assert_u32();
        self.items.push(RawEntry { key, start, end });
        self.buf.extend_from_slice(data);
        Ok(())
    }
    fn push_copy_key(&mut self, key: i32, data: &[i32]) -> Result<(), Error> {
        self.push_item_copy(key, data).map_err(Error::from)
    }
    fn push_apply_key(
        &mut self,
        key: i32,
        in_data: Option<&[i32]>,
        diff: &[i32],
    ) -> Result<(), Error> {
        let range = self
            .push_item_uninit(key, diff.len())
            .map_err(Error::from)?;
        let out = &mut self.buf[to_usize(range)];
        apply_item_delta(in_data, diff, out)?;
        Ok(())
    }
    fn push_item(&mut self, raw_type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError> {
        self.push_item_copy(key(raw_type_id, id), data)
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

        self.read_from_ints(warn, buf)
    }
    pub fn read_from_ints<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
        ints: &[i32],
    ) -> Result<(), Error> {
        self.clear();

        let mut unpacker = IntUnpacker::new(ints);
        let header = SnapHeader::decode_obj(&mut unpacker)?;
        let ints = unpacker.as_slice();

        let offsets_len = header.num_items.assert_usize();
        if ints.len() < offsets_len {
            return Err(Error::OffsetsUnpacking);
        }
        if header.data_size % 4 != 0 {
            return Err(Error::InvalidOffset);
        }
        let items_len = (header.data_size / 4).assert_usize();
        match (offsets_len + items_len).cmp(&ints.len()) {
            cmp::Ordering::Less => warn.warn(Warning::ExcessSnapData),
            cmp::Ordering::Equal => {}
            cmp::Ordering::Greater => return Err(Error::ItemsUnpacking),
        }

        let (offsets, item_data) = ints.split_at(offsets_len);
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
                let raw_type_id = key_to_raw_type_id(item_data[prev_offset]);
                let id = key_to_id(item_data[prev_offset]);
                let item_payload = &item_data[prev_offset + 1..offset];
                let range = self
                    .push_item_uninit(key(raw_type_id, id), item_payload.len())
                    .map_err(Error::from)?;
                let start = range.start.usize();
                let end = range.end.usize();
                let desired_end = start + item_payload.len();
                if end != desired_end {
                    // `push_item_uninit` appends, so the written range must end at the buffer end.
                    if end != self.buf.len() {
                        return Err(Error::ItemsUnpacking);
                    }
                    if !RawSnap::would_fit(self.items.len(), desired_end) {
                        return Err(Error::TooLongSnap);
                    }
                    self.buf.resize(desired_end, 0);
                    let last = self.items.last_mut().ok_or(Error::ItemsUnpacking)?;
                    if last.start != range.start || last.end != range.end {
                        return Err(Error::ItemsUnpacking);
                    }
                    last.end = desired_end.assert_u32();
                }
                let dst = self
                    .buf
                    .get_mut(start..desired_end)
                    .ok_or(Error::ItemsUnpacking)?;
                dst.copy_from_slice(item_payload);
            } else if offset != 0 {
                // First offset must be 0.
                return Err(Error::InvalidOffset);
            }

            prev_offset = Some(offset);

            if finished {
                break;
            }
        }
        self.sort_items();
        for w in self.items.windows(2) {
            if w[0].key == w[1].key {
                return Err(Error::DuplicateKey);
            }
        }
        Ok(())
    }
    pub fn read_with_delta<W>(
        &mut self,
        warn: &mut W,
        from: &RawSnap,
        delta: &Delta,
    ) -> Result<(), Error>
    where
        W: Warn<Warning>,
    {
        self.clear();

        let mut index = 0;
        let mut updates_index = 0;
        let mut num_deletions = 0;

        while index < from.items.len() || updates_index < delta.updated_items.len() {
            let from_key = from.items.get(index).map(|e| e.key);
            let update_key = delta.updated_items.get(updates_index).map(|e| e.key);

            match (from_key, update_key) {
                (Some(from_key), Some(update_key)) => match (from_key as u32)
                    .cmp(&(update_key as u32))
                {
                    cmp::Ordering::Less => {
                        if delta.deleted_items.contains_key(from_key) {
                            num_deletions += 1;
                        } else {
                            self.push_copy_key(from_key, from.item_from_entry(&from.items[index]))?;
                        }
                        index += 1;
                    }
                    cmp::Ordering::Greater => {
                        let diff =
                            &delta.buf[to_usize(delta.updated_items[updates_index].range.clone())];
                        self.push_apply_key(update_key, None, diff)?;
                        updates_index += 1;
                    }
                    cmp::Ordering::Equal => {
                        if delta.deleted_items.contains_key(from_key) {
                            num_deletions += 1;
                        }
                        let diff =
                            &delta.buf[to_usize(delta.updated_items[updates_index].range.clone())];
                        self.push_apply_key(
                            from_key,
                            Some(from.item_from_entry(&from.items[index])),
                            diff,
                        )?;
                        index += 1;
                        updates_index += 1;
                    }
                },
                (Some(fk), None) => {
                    if delta.deleted_items.contains_key(fk) {
                        num_deletions += 1;
                    } else {
                        self.push_copy_key(fk, from.item_from_entry(&from.items[index]))?;
                    }
                    index += 1;
                }
                (None, Some(uk)) => {
                    let diff =
                        &delta.buf[to_usize(delta.updated_items[updates_index].range.clone())];
                    self.push_apply_key(uk, None, diff)?;
                    updates_index += 1;
                }
                (None, None) => break,
            }
        }

        if num_deletions != delta.deleted_items.len() {
            warn.warn(Warning::UnknownDelete);
        }
        self.sort_items();
        Ok(())
    }
    fn write_impl<F: FnMut(i32) -> Result<(), CapacityError>>(
        &self,
        buf: &mut Vec<i32>,
        mut write_int: F,
    ) -> Result<(), CapacityError> {
        assert!(self.items.len() <= MAX_SNAPSHOT_ITEMS);
        let mut written = 0;
        let mut write_int = |i| {
            written += mem::size_of::<i32>();
            write_int(i)
        };
        buf.clear();
        let data_size = self
            .buf
            .len()
            .checked_add(self.items.len())
            .expect("snap size overflow")
            .checked_mul(mem::size_of::<i32>())
            .expect("snap size overflow")
            .assert_i32();
        write_int(data_size)?;
        let num_items = self.items.len().assert_i32();
        write_int(num_items)?;

        let mut offset = 0;
        for entry in &self.items {
            write_int(offset)?;
            let data_len = (entry.end - entry.start).usize();
            offset = offset
                .checked_add(
                    (data_len + 1)
                        .checked_mul(mem::size_of::<i32>())
                        .expect("item size overflow")
                        .assert_i32(),
                )
                .expect("offset overflow");
        }
        for entry in &self.items {
            write_int(entry.key)?;
            for &i in self.item_from_entry(entry) {
                write_int(i)?;
            }
        }
        assert!(written <= MAX_SNAPSHOT_SIZE);
        Ok(())
    }
    pub fn write<'d, 's>(
        &self,
        buf: &mut Vec<i32>,
        mut p: Packer<'d, 's>,
    ) -> Result<&'d [u8], CapacityError> {
        self.write_impl(buf, |int| p.write_int(int))?;
        Ok(p.written())
    }
    pub fn write_to_ints<'a>(
        &self,
        buf: &mut Vec<i32>,
        result: &'a mut [i32],
    ) -> Result<&'a [i32], CapacityError> {
        if !buf.is_empty() {
            buf.clear();
        }

        let num_items = self.items.len();
        let total_len = 2 + (num_items * 2) + self.buf.len();
        if result.len() < total_len {
            return Err(CapacityError);
        }

        let mut idx = 0;
        let data_size = ((self.buf.len() + num_items) * mem::size_of::<i32>()).assert_i32();
        result[idx] = data_size;
        idx += 1;
        result[idx] = num_items.assert_i32();
        idx += 1;

        let mut offset: i32 = 0;
        for entry in &self.items {
            result[idx] = offset;
            idx += 1;
            let data_len = (entry.end - entry.start).usize();
            offset += ((data_len + 1) * mem::size_of::<i32>()).assert_i32();
        }

        for entry in &self.items {
            result[idx] = entry.key;
            idx += 1;
            let start = entry.start.usize();
            let len = (entry.end - entry.start).usize();
            result[idx..idx + len].copy_from_slice(&self.buf[start..start + len]);
            idx += len;
        }

        Ok(&result[..idx])
    }
    pub fn crc(&self) -> i32 {
        self.buf.iter().fold(0, |s, &a| s.wrapping_add(a))
    }
    pub fn recycle(mut self) -> RawBuilder {
        self.clear();
        RawBuilder { snap: self }
    }
}

fn read_int_err<R: ReadInt, W: Warn<Warning>>(
    reader: &mut R,
    w: &mut W,
    e: Error,
) -> Result<i32, Error> {
    reader.read_int(w).map_err(|_| e)
}

pub struct RawItems<'a> {
    snap: &'a RawSnap,
    iter: std::slice::Iter<'a, RawEntry>,
}

impl<'a> Iterator for RawItems<'a> {
    type Item = RawItem<'a>;
    fn next(&mut self) -> Option<RawItem<'a>> {
        self.iter
            .next()
            .map(|entry| RawItem::from_key(entry.key, self.snap.item_from_entry(entry)))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for RawItems<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl fmt::Debug for RawSnap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(self.items().map(
                |RawItem {
                     raw_type_id,
                     id,
                     data,
                 }| ((raw_type_id, id), data),
            ))
            .finish()
    }
}

#[derive(Clone, Default)]
pub struct Snap {
    raw: RawSnap,
    extended_types: FxHashMap<Uuid, u16>,
}

impl Snap {
    pub fn empty() -> Snap {
        Default::default()
    }
    fn raw_type_id(&self, type_id: TypeId) -> Option<u16> {
        match type_id {
            TypeId::Ordinal(ordinal) => {
                assert!(0 < ordinal && ordinal < OFFSET_EXTENDED_TYPE_ID);
                Some(ordinal)
            }
            TypeId::Uuid(uuid) => self.extended_types.get(&uuid).copied(),
        }
    }
    fn type_id(&self, raw_type_id: u16) -> Option<TypeId> {
        if raw_type_id == TYPE_ID_EX {
            None
        } else if raw_type_id < OFFSET_EXTENDED_TYPE_ID {
            Some(TypeId::Ordinal(raw_type_id))
        } else {
            // `build_from_raw()` should have checked the UUID type IDs.
            Some(TypeId::Uuid(item_data_to_uuid(
                &mut Ignore,
                self.raw.item(TYPE_ID_EX, raw_type_id).unwrap(),
            )?))
        }
    }
    pub fn item(&self, type_id: TypeId, id: u16) -> Option<&[i32]> {
        self.raw.item(self.raw_type_id(type_id)?, id)
    }
    pub fn items(&self) -> Items<'_> {
        let raw = self.raw.items();
        let remaining = self.raw.items().len() - self.extended_types.len();
        Items {
            raw,
            snap: self,
            remaining,
        }
    }
    fn build_from_raw<W: Warn<Warning>>(&mut self, warn: &mut W) -> Result<(), Error> {
        self.extended_types.clear();
        let mut seen_extended_raw_type_ids = FxHashSet::<u16>::default();
        for entry in &self.raw.items {
            let item_key = entry.key;
            let raw_type_id = key_to_raw_type_id(item_key);
            if raw_type_id == TYPE_ID_EX {
                let id = key_to_id(item_key);
                let item_data = self.raw.item_from_entry(entry);
                let uuid = item_data_to_uuid(warn, item_data).ok_or(Error::InvalidUuidType)?;
                if self.extended_types.insert(uuid, id).is_some() {
                    return Err(Error::DuplicateUuidType);
                }
            } else if raw_type_id >= OFFSET_EXTENDED_TYPE_ID {
                seen_extended_raw_type_ids.insert(raw_type_id);
            }
        }
        for raw_type_id in seen_extended_raw_type_ids {
            if self.raw.item(TYPE_ID_EX, raw_type_id).is_none() {
                return Err(Error::MissingUuidType);
            }
        }
        Ok(())
    }
    pub fn read<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
        buf: &mut Vec<i32>,
        data: &[u8],
    ) -> Result<(), Error> {
        self.raw.read(warn, buf, data)?;
        self.build_from_raw(warn)?;
        Ok(())
    }
    pub fn read_from_ints<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
        data: &[i32],
    ) -> Result<(), Error> {
        self.raw.read_from_ints(warn, data)?;
        self.build_from_raw(warn)?;
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
        self.raw.read_with_delta(warn, &from.raw, delta)?;
        self.build_from_raw(warn)?;
        Ok(())
    }
    pub fn write<'d, 's>(
        &self,
        buf: &mut Vec<i32>,
        p: Packer<'d, 's>,
    ) -> Result<&'d [u8], CapacityError> {
        self.raw.write(buf, p)
    }
    pub fn write_to_ints<'a>(
        &self,
        buf: &mut Vec<i32>,
        result: &'a mut [i32],
    ) -> Result<&'a [i32], CapacityError> {
        self.raw.write_to_ints(buf, result)
    }
    pub fn crc(&self) -> i32 {
        self.raw.crc()
    }
    /// Recycle the snap to build another one.
    ///
    /// This remembers the extended item types inserted into the snap, to keep
    /// the snapshot delta smaller.
    pub fn recycle(mut self) -> Builder {
        let mut next_type_id = OFFSET_EXTENDED_TYPE_ID;
        let mut ex_ids: Vec<u16> = self
            .raw
            .items
            .iter()
            .filter(|e| key_to_raw_type_id(e.key) == TYPE_ID_EX)
            .map(|e| key_to_id(e.key))
            .collect();
        ex_ids.sort_unstable();
        for id in ex_ids {
            // Make sure we'll have space for at least 256 additional extended types.
            if id < next_type_id + 256 {
                next_type_id = id + 1;
            }
        }
        self.raw.clear();
        for (&uuid, &raw_type_id) in &self.extended_types {
            // It fit last time, it's going to fit this time.
            self.raw
                .push_item(TYPE_ID_EX, raw_type_id, &uuid_to_item_data(uuid))
                .unwrap();
            self.raw.seen.insert(key(TYPE_ID_EX, raw_type_id));
        }
        self.raw.sort_items();
        Builder {
            snap: self,
            next_type_id,
        }
    }
}

pub struct Items<'a> {
    raw: RawItems<'a>,
    snap: &'a Snap,
    remaining: usize,
}

impl<'a> Iterator for Items<'a> {
    type Item = Item<'a>;
    fn next(&mut self) -> Option<Item<'a>> {
        loop {
            match self.raw.next() {
                None => return None,
                Some(RawItem {
                    raw_type_id,
                    id,
                    data,
                }) => {
                    if let Some(type_id) = self.snap.type_id(raw_type_id) {
                        self.remaining -= 1;
                        return Some(Item { type_id, id, data });
                    } else {
                        // Skip items with ill-defined types.
                        continue;
                    }
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for Items<'a> {
    fn len(&self) -> usize {
        self.remaining
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
    deleted_items: DeletedItems,
    updated_items: Vec<UpdateEntry>,
    buf: Vec<i32>,
}

#[derive(Clone, Default)]
struct DeletedItems {
    by_type: FxHashMap<u16, Vec<u64>>,
    keys: Vec<i32>,
}

impl DeletedItems {
    fn clear(&mut self) {
        self.by_type.clear();
        self.keys.clear();
    }
    fn len(&self) -> usize {
        self.keys.len()
    }
    fn sort_keys(&mut self) {
        self.keys.sort_unstable_by_key(|&k| k as u32);
    }
    fn insert_key(&mut self, k: i32) -> bool {
        let raw_type_id = key_to_raw_type_id(k);
        let id = key_to_id(k);
        let bits = self.by_type.entry(raw_type_id).or_default();
        let word = (id as usize) >> 6;
        if bits.len() <= word {
            bits.resize(word + 1, 0);
        }
        let mask = 1u64 << (id as u32 & 63);
        let existed = (bits[word] & mask) != 0;
        if !existed {
            bits[word] |= mask;
            self.keys.push(k);
        }
        !existed
    }
    fn contains_key(&self, k: i32) -> bool {
        let raw_type_id = key_to_raw_type_id(k);
        let id = key_to_id(k);
        let bits = match self.by_type.get(&raw_type_id) {
            Some(bits) => bits,
            None => return false,
        };
        let word = (id as usize) >> 6;
        let v = match bits.get(word) {
            Some(&v) => v,
            None => return false,
        };
        (v & (1u64 << (id as u32 & 63))) != 0
    }
}

#[derive(Clone)]
struct UpdateEntry {
    key: i32,
    range: ops::Range<u32>,
    order: u32,
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
    fn prepare_update_item(&mut self, key: i32, size: usize, order: u32) -> ops::Range<u32> {
        let offset = self.buf.len();
        let start = offset.assert_u32();
        let end = (offset + size).assert_u32();
        self.buf.resize(offset + size, 0);
        let range = start..end;
        self.updated_items.push(UpdateEntry {
            key,
            range: range.clone(),
            order,
        });
        range
    }
    pub fn create(&mut self, from: &Snap, to: &Snap) {
        self.create_raw(&from.raw, &to.raw)
    }
    pub fn create_raw(&mut self, from: &RawSnap, to: &RawSnap) {
        self.clear();
        let from_items = &from.items;
        let to_items = &to.items;
        let mut index = 0;
        let mut j = 0;

        self.updated_items.reserve(to_items.len().min(16));

        while index < from_items.len() || j < to_items.len() {
            let from_key = from_items.get(index).map(|e| e.key);
            let item_key = to_items.get(j).map(|e| e.key);
            match (from_key, item_key) {
                (Some(fk), Some(tk)) => {
                    if (fk as u32) < (tk as u32) {
                        assert!(self.deleted_items.insert_key(fk));
                        index += 1;
                    } else if (fk as u32) > (tk as u32) {
                        let data = to.item_from_entry(&to_items[j]);
                        let range = self.prepare_update_item(tk, data.len(), 0);
                        let out_delta = &mut self.buf[to_usize(range.clone())];
                        create_item_delta(None, data, out_delta)
                            .expect("item sizes can't be mismatched for self-created snapshots");
                        j += 1;
                    } else {
                        let from_data = from.item_from_entry(&from_items[index]);
                        let to_data = to.item_from_entry(&to_items[j]);
                        if from_data != to_data {
                            let range = self.prepare_update_item(tk, to_data.len(), 0);
                            let out_delta = &mut self.buf[to_usize(range.clone())];
                            create_item_delta(Some(from_data), to_data, out_delta).expect(
                                "item sizes can't be mismatched for self-created snapshots",
                            );
                            // but they can be different for snapshots received over the network…
                        }
                        index += 1;
                        j += 1;
                    }
                }
                (Some(fk), None) => {
                    assert!(self.deleted_items.insert_key(fk));
                    index += 1;
                }
                (None, Some(tk)) => {
                    let data = to.item_from_entry(&to_items[j]);
                    let range = self.prepare_update_item(tk, data.len(), 0);
                    let out_delta = &mut self.buf[to_usize(range.clone())];
                    create_item_delta(None, data, out_delta)
                        .expect("item sizes can't be mismatched for self-created snapshots");
                    j += 1;
                }
                (None, None) => break,
            }
        }
    }

    fn write_impl<O, F>(&self, mut object_size: O, mut write_int: F) -> Result<(), CapacityError>
    where
        O: FnMut(u16) -> Option<u32>,
        F: FnMut(i32) -> Result<(), CapacityError>,
    {
        {
            let header = DeltaHeader {
                num_deleted_items: self.deleted_items.len().assert_i32(),
                num_updated_items: self.updated_items.len().assert_i32(),
            };
            for int in header.encode_obj() {
                write_int(int)?;
            }
        }
        for &key in &self.deleted_items.keys {
            write_int(key)?;
        }
        for update_entry in &self.updated_items {
            let key = update_entry.key;
            let data = &self.buf[to_usize(update_entry.range.clone())];
            let raw_type_id = key_to_raw_type_id(key);
            let id = key_to_id(key);
            write_int(raw_type_id.i32())?;
            write_int(id.i32())?;
            match object_size(raw_type_id) {
                Some(size) => assert_eq!(size.usize(), data.len()),
                None => write_int(data.len().assert_i32())?,
            }
            for &d in data {
                write_int(d)?;
            }
        }
        Ok(())
    }
    pub fn write<'d, 's, O>(
        &self,
        object_size: O,
        p: Packer<'d, 's>,
    ) -> Result<&'d [u8], CapacityError>
    where
        O: FnMut(u16) -> Option<u32>,
    {
        let mut p = p;
        self.write_impl(object_size, |int| p.write_int(int))?;
        Ok(p.written())
    }
    pub fn write_to_ints<'a, O>(
        &self,
        object_size: O,
        result: &'a mut [i32],
    ) -> Result<&'a [i32], CapacityError>
    where
        O: FnMut(u16) -> Option<u32>,
    {
        let mut object_size = object_size;
        let mut idx = 0usize;

        let header = DeltaHeader {
            num_deleted_items: self.deleted_items.len().assert_i32(),
            num_updated_items: self.updated_items.len().assert_i32(),
        };
        for &int in &header.encode_obj() {
            *result.get_mut(idx).ok_or(CapacityError)? = int;
            idx += 1;
        }
        for &key in &self.deleted_items.keys {
            *result.get_mut(idx).ok_or(CapacityError)? = key;
            idx += 1;
        }
        for update_entry in &self.updated_items {
            let key = update_entry.key;
            let raw_type_id = key_to_raw_type_id(key);
            let id = key_to_id(key);
            *result.get_mut(idx).ok_or(CapacityError)? = raw_type_id.i32();
            idx += 1;
            *result.get_mut(idx).ok_or(CapacityError)? = id.i32();
            idx += 1;

            let data = &self.buf[to_usize(update_entry.range.clone())];
            match object_size(raw_type_id) {
                Some(size) => {
                    assert_eq!(size.usize(), data.len());
                }
                None => {
                    *result.get_mut(idx).ok_or(CapacityError)? = data.len().assert_i32();
                    idx += 1;
                }
            }

            let end = idx + data.len();
            if end > result.len() {
                return Err(CapacityError);
            }
            result[idx..end].copy_from_slice(data);
            idx = end;
        }

        Ok(&result[..idx])
    }

    fn read_impl<W, O, R>(&mut self, warn: &mut W, object_size: O, p: &mut R) -> Result<(), Error>
    where
        W: Warn<Warning>,
        O: FnMut(u16) -> Option<u32>,
        R: ReadInt,
    {
        self.clear();

        let mut object_size = object_size;

        let header = DeltaHeader::decode_impl(warn, p)?;

        let mut dup_delete = false;
        for _ in 0..header.num_deleted_items {
            if !self
                .deleted_items
                .insert_key(read_int_err(p, warn, Error::DeletedItemsUnpacking)?)
            {
                dup_delete = true;
            }
        }
        if dup_delete {
            warn.warn(Warning::DuplicateDelete);
        }
        self.deleted_items.sort_keys();

        let expected_updates = header.num_updated_items.assert_usize();
        for order in 0..expected_updates {
            let raw_type_id = read_int_err(p, warn, Error::ItemDiffsUnpacking)?;
            let id = read_int_err(p, warn, Error::ItemDiffsUnpacking)?;

            let raw_type_id = raw_type_id.try_u16().ok_or(Error::TypeIdRange)?;
            let id = id.try_u16().ok_or(Error::IdRange)?;

            let size = match object_size(raw_type_id) {
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

            let key = key(raw_type_id, id);
            self.updated_items.push(UpdateEntry {
                key,
                range: start..end,
                order: order as u32,
            });

            if self.deleted_items.contains_key(key) {
                warn.warn(Warning::DeleteUpdate);
            }
        }

        while !p.is_empty() {
            if read_int_err(p, warn, Error::ItemDiffsUnpacking)? != 0 {
                warn.warn(Warning::NumUpdatedItems);
                break;
            }
        }

        self.updated_items
            .sort_unstable_by_key(|u| (u.key as u32, u.order));
        if self.updated_items.windows(2).any(|w| w[0].key == w[1].key) {
            warn.warn(Warning::DuplicateUpdate);
        }
        self.updated_items.dedup_by(|a, b| {
            if a.key == b.key {
                a.clone_from(b);
                true
            } else {
                false
            }
        });

        Ok(())
    }
    pub fn read_from_ints<W, O>(
        &mut self,
        warn: &mut W,
        object_size: O,
        p: &mut IntUnpacker,
    ) -> Result<(), Error>
    where
        W: Warn<Warning>,
        O: FnMut(u16) -> Option<u32>,
    {
        self.read_impl(warn, object_size, p)
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
        self.read_impl(warn, object_size, p)
    }
}

#[derive(Default)]
pub struct RawBuilder {
    snap: RawSnap,
}

impl RawBuilder {
    pub fn new() -> RawBuilder {
        Default::default()
    }
    pub fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(), BuilderError> {
        if !self.snap.seen.insert(key(type_id, id)) {
            return Err(BuilderError::DuplicateKey);
        }
        self.snap.push_item(type_id, id, data)
    }
    pub fn finish(self) -> RawSnap {
        let mut snap = self.snap;
        snap.sort_items();
        snap
    }
}

pub struct Builder {
    snap: Snap,
    next_type_id: u16,
}

impl Default for Builder {
    fn default() -> Builder {
        Builder {
            snap: Default::default(),
            next_type_id: OFFSET_EXTENDED_TYPE_ID,
        }
    }
}

impl Builder {
    pub fn new() -> Builder {
        Default::default()
    }
    pub fn add_item(&mut self, type_id: TypeId, id: u16, data: &[i32]) -> Result<(), BuilderError> {
        let raw_type_id = match type_id {
            TypeId::Ordinal(ordinal) => {
                assert!(0 < ordinal && ordinal < OFFSET_EXTENDED_TYPE_ID);
                ordinal
            }
            TypeId::Uuid(uuid) => match self.snap.extended_types.entry(uuid) {
                hash_map::Entry::Occupied(o) => *o.get(),
                hash_map::Entry::Vacant(v) => {
                    let raw_type_id = self.next_type_id;
                    assert!(OFFSET_EXTENDED_TYPE_ID <= raw_type_id, "invalid type ID");
                    assert!(raw_type_id < 0x8000, "invalid type ID");
                    let ex_key = key(TYPE_ID_EX, raw_type_id);
                    if !self.snap.raw.seen.insert(ex_key) {
                        return Err(BuilderError::DuplicateKey);
                    }
                    self.snap
                        .raw
                        .push_item(TYPE_ID_EX, raw_type_id, &uuid_to_item_data(uuid))?;
                    // Only increment `self.next_type_id` after successful
                    // insertion.
                    self.next_type_id += 1;
                    v.insert(raw_type_id);
                    raw_type_id
                }
            },
        };
        if !self.snap.raw.seen.insert(key(raw_type_id, id)) {
            return Err(BuilderError::DuplicateKey);
        }
        self.snap.raw.push_item(raw_type_id, id, data)
    }
    pub fn finish(self) -> Snap {
        let mut snap = self.snap;
        snap.raw.sort_items();
        snap
    }
}

pub fn delta_chunks(tick: i32, delta_tick: i32, data: &[u8], crc: i32) -> DeltaChunks<'_> {
    DeltaChunks {
        tick,
        delta_tick: tick - delta_tick,
        crc,
        cur_part: if !data.is_empty() { 0 } else { -1 },
        num_parts: ((data.len() + MAX_SNAPSHOT_PACKSIZE as usize - 1)
            / MAX_SNAPSHOT_PACKSIZE as usize)
            .assert_i32(),
        data,
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

#[cfg(test)]
mod test {
    use super::Builder;
    use super::Delta;
    use super::Error;
    use super::Item;
    use super::RawBuilder;
    use super::RawSnap;
    use super::Snap;
    use libtw2_buffer::CapacityError;
    use libtw2_packer::with_packer;
    use libtw2_packer::Unpacker;
    use libtw2_warn::Panic;
    use uuid::Uuid;

    #[test]
    fn smoke_test() {
        let uuid: Uuid = "1a3fcc94-1e53-461e-912e-21200882024b".parse().unwrap();

        let mut builder = Builder::new();
        builder
            .add_item(uuid.into(), 1337, &[0x1234, 0x567890ab])
            .unwrap();
        let snap = builder.finish();

        assert_eq!(
            snap.item(uuid.into(), 1337),
            Some(&[0x1234, 0x567890ab][..])
        );
        let item = Item {
            type_id: uuid.into(),
            id: 1337,
            data: &[0x1234, 0x567890ab],
        };
        assert_eq!(snap.items().collect::<Vec<_>>(), &[item][..]);
    }

    #[rustfmt::skip]
    const RAW_SNAP_SINGLE_ITEM_INTS: &'static [i32] = &[
        // SnapHeader: data_size bytes, num_items
        12, 1,
        // item_offsets (in bytes)
        0,
        // items: key, data...
        65538, 10, 20,
    ];

    #[test]
    fn raw_snap_roundtrip_from_ints_single_item() {
        let mut snap = RawSnap::empty();
        snap.read_from_ints(&mut Panic, RAW_SNAP_SINGLE_ITEM_INTS)
            .unwrap();
        assert_eq!(snap.item(1, 2), Some(&[10, 20][..]));

        let mut tmp = Vec::new();
        let mut out = [0i32; 64];
        let written = snap.write_to_ints(&mut tmp, &mut out).unwrap();
        assert_eq!(written, RAW_SNAP_SINGLE_ITEM_INTS);
    }

    #[test]
    fn raw_snap_read_rejects_invalid_first_offset() {
        // First item offset must be 0.
        let data = [
            12, 1, //
            4, // invalid first offset (bytes)
            65538, 10, 20,
        ];
        let mut snap = RawSnap::empty();
        assert_eq!(
            snap.read_from_ints(&mut Panic, &data).unwrap_err(),
            Error::InvalidOffset
        );
    }

    #[test]
    fn raw_snap_read_rejects_non_monotonic_offsets() {
        // Offsets must be strictly increasing and within bounds.
        let data = [
            20, 2, //
            0, 0, // second offset can't be equal to first
            65538, 10, 20, //
            65539, 30,
        ];
        let mut snap = RawSnap::empty();
        assert_eq!(
            snap.read_from_ints(&mut Panic, &data).unwrap_err(),
            Error::InvalidOffset
        );
    }

    #[rustfmt::skip]
    const SNAP_WITH_UUID_TYPE_INTS: &'static [i32] = &[
        // SnapHeader: data_size bytes, num_items
        32, 2,
        // item_offsets (in bytes)
        0, 20,
        // TYPE_ID_EX item: key(TYPE_ID_EX=0, raw_type_id=0x4000=16384) then uuid_to_item_data(...)
        16384, 440388756, 508773918, -1859247840, 142737995,
        // actual item: key(raw_type_id=0x4000, id=7) then data
        1073741831, 4660, 1450744507,
    ];

    #[test]
    fn snap_write_to_ints_matches_server_layout_for_uuid_type() {
        let uuid: Uuid = "1a3fcc94-1e53-461e-912e-21200882024b".parse().unwrap();
        let mut builder = Builder::new();
        builder
            .add_item(uuid.into(), 7, &[0x1234, 0x56789abb])
            .unwrap();
        let snap = builder.finish();

        let mut tmp = Vec::new();
        let mut out = [0i32; 128];
        let written = snap.write_to_ints(&mut tmp, &mut out).unwrap();
        assert_eq!(written, SNAP_WITH_UUID_TYPE_INTS);

        // And it must be readable again.
        let mut parsed = Snap::empty();
        parsed
            .read_from_ints(&mut Panic, SNAP_WITH_UUID_TYPE_INTS)
            .unwrap();
        assert_eq!(parsed.item(uuid.into(), 7), Some(&[0x1234, 0x56789abb][..]));
    }

    #[test]
    fn raw_builder_write_to_ints_matches_expected_text() {
        let mut b = RawBuilder::new();
        b.add_item(1, 2, &[10, 20]).unwrap();
        b.add_item(1, 3, &[30]).unwrap();
        let snap = b.finish();

        #[rustfmt::skip]
        const EXPECTED: &'static [i32] = &[
            20, 2,
            0, 12,
            65538, 10, 20,
            65539, 30,
        ];

        let mut tmp = Vec::new();
        let mut out = [0i32; 64];
        let written = snap.write_to_ints(&mut tmp, &mut out).unwrap();
        assert_eq!(written, EXPECTED);
    }

    #[rustfmt::skip]
    const FIRST_DELTA_INTS: &'static [i32] = &[
        0,18,0,4,18,1744,1072,2,3,4,17,1840,912,1,0,4,16,880,880,0,0,4,15,1840,848,1,0,4,14,912,848,0,0,4,13,880,848,1,0,4,12,848,848,0,0,4,11,880,816,0,0,4,9,1264,656,0,0,4,8,1104,656,0,0,4,7,912,656,0,0,4,6,1712,624,2,2,4,5,1840,432,1,0,4,3,1840,336,1,0,9,0,292,1584,305,0,128,0,0,0,-1,0,0,1584,304,0,0,0,10,0,10,1,0,0,6,0,0,0,0,0,20,0,0,1,11,0,-287183387,-320474125,-1594563099,-2139062272,-2139062144,-2139062144,-2139062272,-1,-454695199,-169020288,-2139062144,-2139062144,-2139062144,-2139062272,0,65408,65408,10,0,1,0,0,0,0,
    ];
    const FIRST_CRC: i32 = 0x5b96263a;

    #[rustfmt::skip]
    const SECOND_DELTA_INTS: &'static [i32] = &[
        0,1,0,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,
    ];
    const SECOND_CRC: i32 = 0x5b96263b;

    #[test]
    fn delta_apply_vectors_crc_matches() {
        // Minimal static-size table for the object types used in the vectors below.
        // This matches what the server uses (items with a known size omit the explicit size field).
        fn obj_size(raw_type_id: u16) -> Option<u32> {
            match raw_type_id {
                4 => Some(4),
                6 => Some(8),
                9 => Some(16),
                10 => Some(4),
                11 => Some(24),
                _ => None,
            }
        }

        let mut buf = Vec::with_capacity(4096);
        let mut delta = Delta::new();
        let mut prev = Snap::empty();
        let mut snap = Snap::default();

        with_packer(&mut buf, |mut p| -> Result<_, CapacityError> {
            for &d in FIRST_DELTA_INTS {
                p.write_int(d)?;
            }
            Ok(p.written())
        })
        .unwrap();

        delta
            .read(&mut Panic, obj_size, &mut Unpacker::new(&buf))
            .unwrap();
        snap.read_with_delta(&mut Panic, &prev, &delta).unwrap();
        assert_eq!(snap.crc(), FIRST_CRC);

        prev = snap;
        snap = Snap::default();

        buf.clear();
        with_packer(&mut buf, |mut p| -> Result<_, CapacityError> {
            for &d in SECOND_DELTA_INTS {
                p.write_int(d)?;
            }
            Ok(p.written())
        })
        .unwrap();

        delta
            .read(&mut Panic, obj_size, &mut Unpacker::new(&buf))
            .unwrap();
        snap.read_with_delta(&mut Panic, &prev, &delta).unwrap();
        assert_eq!(snap.crc(), SECOND_CRC);
    }
}
