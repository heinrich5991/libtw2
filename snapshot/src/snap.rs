use format::DeltaHeader;
use format::Item;
use format::Warning;
use format::key;
use format::key_to_id;
use format::key_to_type_id;
use gamenet::packer::Unpacker;
use gamenet;
use num::ToPrimitive;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map;
use std::fmt;
use std::iter;
use std::ops;
use warn::Warn;
use warn::wrap;

#[derive(Debug)]
pub enum Error {
    Packer(gamenet::error::Error),
    DeletedItemsUnpacking,
    ItemDiffsUnpacking,
    TypeIdRange,
    IdRange,
    NegativeSize,
    TooLongDiff,
    TooLongSnap,
    DeltaDifferingSizes,
}

impl From<gamenet::error::Error> for Error {
    fn from(err: gamenet::error::Error) -> Error {
        Error::Packer(err)
    }
}

fn to_usize(r: ops::Range<u32>) -> ops::Range<usize> {
    r.start.to_usize().unwrap()..r.end.to_usize().unwrap()
}

fn apply_delta(in_: Option<&[i32]>, delta: &[i32], out: &mut [i32])
    -> Result<(), Error>
{
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

// TODO: Select a faster hasher?
#[derive(Clone, Default)]
pub struct Snap {
    offsets: HashMap<i32, ops::Range<u32>>,
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
        self.offsets.get(&key(type_id, id)).map(|o| &self.buf[to_usize(o.clone())])
    }
    pub fn items(&self) -> Items {
        Items {
            snap: self,
            iter: self.offsets.iter(),
        }
    }
    fn prepare_item(&mut self, type_id: u16, id: u16, size: usize)
        -> Result<&mut [i32], Error>
    {
        let offset = match self.offsets.entry(key(type_id, id)) {
            hash_map::Entry::Occupied(o) => o.into_mut(),
            hash_map::Entry::Vacant(v) => {
                let offset = self.buf.len();
                let start = try!(offset.to_u32().ok_or(Error::TooLongSnap));
                let end = try!((offset + size).to_u32().ok_or(Error::TooLongSnap));
                self.buf.extend(iter::repeat(0).take(size));
                v.insert(start..end)
            },
        }.clone();
        Ok(&mut self.buf[to_usize(offset)])
    }
    pub fn read_with_delta<W>(&mut self, warn: &mut W, from: &Snap, delta: &Delta)
        -> Result<(), Error>
        where W: Warn<Warning>,
    {
        self.clear();

        let mut num_deletions = 0;
        for item in from.items() {
            if !delta.deleted_items.contains(&item.key()) {
                let out = try!(self.prepare_item(item.type_id, item.id, item.data.len()));
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
            let out = try!(self.prepare_item(type_id, id, diff.len()));
            let in_ = from.item(type_id, id);

            try!(apply_delta(in_, diff, out));
        }
        Ok(())
    }
    pub fn crc(&self) -> i32 {
        self.buf.iter().fold(0, |s, &a| s.wrapping_add(a))
    }
}

pub struct Items<'a> {
    snap: &'a Snap,
    iter: hash_map::Iter<'a, i32, ops::Range<u32>>,
}

impl<'a> Iterator for Items<'a> {
    type Item = Item<'a>;
    fn next(&mut self) -> Option<Item<'a>> {
        self.iter.next().map(|(&k, o)| {
            Item::from_key(k, self.snap.item_from_offset(o.clone()))
        })
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
        f.debug_map().entries(self.items().map(
            |Item { type_id, id, data }| ((type_id, id), data)
        )).finish()
    }
}

#[derive(Clone, Default)]
pub struct Delta {
    deleted_items: HashSet<i32>,
    updated_items: HashMap<i32, ops::Range<u32>>,
    buf: Vec<i32>,
}

impl Delta {
    pub fn new() -> Delta {
        Default::default()
    }
    fn clear(&mut self) {
        self.deleted_items.clear();
        self.updated_items.clear();
        self.buf.clear();
    }
}

#[derive(Clone, Default)]
pub struct DeltaReader {
    buf: Vec<i32>,
}

impl DeltaReader {
    pub fn new() -> DeltaReader {
        Default::default()
    }
    fn clear(&mut self) {
        self.buf.clear();
    }
    pub fn read<W, O>(&mut self, warn: &mut W, delta: &mut Delta, object_size: O, p: &mut Unpacker)
        -> Result<(), Error>
        where W: Warn<Warning>,
              O: FnMut(u16) -> Option<u32>,
    {
        delta.clear();
        self.clear();

        let mut object_size = object_size;

        let header = try!(DeltaHeader::decode(warn, p));
        while !p.as_slice().is_empty() {
            self.buf.push(try!(p.read_int(wrap(warn))));
        }
        let split = header.num_deleted_items.to_usize().unwrap();
        if split > self.buf.len() {
            return Err(Error::DeletedItemsUnpacking);
        }
        let (deleted_items, buf) = self.buf.split_at(split);
        delta.deleted_items.extend(deleted_items);
        if deleted_items.len() != delta.deleted_items.len() {
            warn.warn(Warning::DuplicateDelete);
        }

        let mut buf = buf.iter();
        while buf.len() != 0 {
            // WARN: num_updates not matching the number of updates
            let type_id = try!(buf.next().ok_or(Error::ItemDiffsUnpacking));
            let id = try!(buf.next().ok_or(Error::ItemDiffsUnpacking));

            let type_id = try!(type_id.to_u16().ok_or(Error::TypeIdRange));
            let id = try!(id.to_u16().ok_or(Error::TypeIdRange));

            let size = match object_size(type_id) {
                Some(s) => s.to_usize().unwrap(),
                None => {
                    let s = try!(buf.next().ok_or(Error::ItemDiffsUnpacking));
                    try!(s.to_usize().ok_or(Error::NegativeSize))
                }
            };

            if size > buf.len() {
                return Err(Error::ItemDiffsUnpacking);
            }
            let (data, b) = buf.as_slice().split_at(size);
            buf = b.iter();

            let offset = delta.buf.len();
            let start = try!(offset.to_u32().ok_or(Error::TooLongDiff));
            let end = try!((offset + data.len()).to_u32().ok_or(Error::TooLongDiff));
            delta.buf.extend(data.iter());

            // In case of conflict, take later update (as the original code does).
            if delta.updated_items.insert(key(type_id, id), start..end).is_some() {
                warn.warn(Warning::DuplicateUpdate);
            }

            if delta.deleted_items.contains(&key(type_id, id)) {
                warn.warn(Warning::DeleteUpdate);
            }
        }

        Ok(())
    }
}
