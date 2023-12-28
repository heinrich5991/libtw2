use gamenet_common::snap_obj::TypeId;
use gamenet_ddnet::snap_obj;
use std::collections::HashMap;
use std::io;
use std::mem;
use std::slice;
use thiserror::Error;
use uuid::Uuid;
use warn::wrap;
use warn::Warn;

use crate::format;
use crate::reader;
use crate::RawChunk;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error(transparent)]
    Inner(#[from] reader::ReadError),
    #[error("{0:?}")]
    Snap(snapshot::snap::Error),
    #[error("Uuid item has an incorrect size")]
    UuidItemLength,
    #[error("Uuid type id that is not registered")]
    UnregisteredUuidTypeId,
    #[error("Chunk types are not in the proper order")]
    ChunkOrder,
}

pub struct DemoReader {
    raw: reader::Reader,
    delta: snapshot::Delta,
    snap: snapshot::Snap,
    old_snap: snapshot::Snap,
    snap_reader: snapshot::SnapReader,
    snapshot: Snapshot,
}

#[derive(Debug)]
pub enum Warning {
    Demo(format::Warning),
    Snapshot(snapshot::format::Warning),
    Packer(packer::Warning),
    ExcessItemData,
    Gamenet(gamenet_common::error::Error),
    GamenetDdnet(gamenet_ddnet::Error),
}

impl From<format::Warning> for Warning {
    fn from(w: format::Warning) -> Self {
        Warning::Demo(w)
    }
}
impl From<snapshot::format::Warning> for Warning {
    fn from(w: snapshot::format::Warning) -> Self {
        Warning::Snapshot(w)
    }
}
impl From<packer::Warning> for Warning {
    fn from(w: packer::Warning) -> Self {
        Warning::Packer(w)
    }
}
impl From<packer::ExcessData> for Warning {
    fn from(_: packer::ExcessData) -> Self {
        Warning::ExcessItemData
    }
}

pub enum Chunk<'a> {
    Message(gamenet_ddnet::msg::Game<'a>),
    Snapshot(slice::Iter<'a, (snap_obj::SnapObj, u16)>),
    Tick(i32),
    Invalid,
}

impl DemoReader {
    pub fn new<R, W>(data: R, warn: &mut W) -> Result<Self, ReadError>
    where
        R: io::Read + io::Seek + 'static,
        W: Warn<Warning>,
    {
        let reader = reader::Reader::new(data, wrap(warn))?;
        Ok(DemoReader {
            raw: reader,
            delta: snapshot::Delta::new(),
            snap: snapshot::Snap::empty(),
            old_snap: snapshot::Snap::empty(),
            snap_reader: snapshot::SnapReader::new(),
            snapshot: Snapshot::default(),
        })
    }

    pub fn next_chunk<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
    ) -> Result<Option<Chunk>, ReadError> {
        match self.raw.read_chunk(wrap(warn))? {
            None => return Ok(None),
            Some(RawChunk::Unknown) => Ok(Some(Chunk::Invalid)),
            Some(RawChunk::Tick { tick, .. }) => Ok(Some(Chunk::Tick(tick))),
            Some(RawChunk::Message(msg)) => {
                let mut unpacker = packer::Unpacker::new_from_demo(msg);
                match gamenet_ddnet::msg::Game::decode(wrap(warn), &mut unpacker) {
                    Ok(msg) => Ok(Some(Chunk::Message(msg))),
                    Err(err) => {
                        warn.warn(Warning::Gamenet(err));
                        Ok(Some(Chunk::Invalid))
                    }
                }
            }
            Some(RawChunk::Snapshot(snap)) => {
                let mut unpacker = packer::Unpacker::new(snap);
                let mut swap = snapshot::Snap::empty();
                mem::swap(&mut self.snap, &mut swap);
                self.snap = self
                    .snap_reader
                    .read(wrap(warn), swap, &mut unpacker)
                    .unwrap();
                self.snapshot.build(warn, &self.snap)?;
                Ok(Some(Chunk::Snapshot(self.snapshot.objects.iter())))
            }
            Some(RawChunk::SnapshotDelta(dt)) => {
                let mut unpacker = packer::Unpacker::new(dt);
                let obj_size = snap_obj::obj_size;
                self.delta
                    .read(wrap(warn), obj_size, &mut unpacker)
                    .map_err(ReadError::Snap)?;
                self.old_snap
                    .read_with_delta(wrap(warn), &self.snap, &self.delta)
                    .map_err(ReadError::Snap)?;
                mem::swap(&mut self.old_snap, &mut self.snap);
                self.snapshot.build(warn, &self.snap)?;
                Ok(Some(Chunk::Snapshot(self.snapshot.objects.iter())))
            }
        }
    }

    pub fn inner(&self) -> &reader::Reader {
        &self.raw
    }
}

#[derive(Default)]
struct Snapshot {
    uuid_index: HashMap<u16, Uuid>,
    pub objects: Vec<(snap_obj::SnapObj, u16)>,
}

impl Snapshot {
    fn build<W>(&mut self, warn: &mut W, snap: &snapshot::Snap) -> Result<(), ReadError>
    where
        W: Warn<Warning>,
    {
        self.uuid_index.clear();
        self.objects.clear();

        // First we build the uuid item index
        for item in snap.items().filter(|item| item.type_id == 0) {
            let mut uuid_bytes = [0; 16];
            if item.data.len() != 4 {
                return Err(ReadError::UuidItemLength);
            }
            for (b, x) in uuid_bytes.chunks_mut(4).zip(item.data) {
                b.copy_from_slice(&x.to_be_bytes());
            }
            let uuid = Uuid::from_bytes(uuid_bytes);
            self.uuid_index.insert(item.id, uuid);
        }

        for item in snap.items().filter(|item| item.type_id != 0) {
            let type_id = if item.type_id < u16::MAX / 4 {
                TypeId::Ordinal(item.type_id)
            } else {
                let uuid = self
                    .uuid_index
                    .get(&item.type_id)
                    .ok_or(ReadError::UnregisteredUuidTypeId)?;
                TypeId::Uuid(*uuid)
            };
            let mut int_unpacker = packer::IntUnpacker::new(item.data);
            match snap_obj::SnapObj::decode_obj(wrap(warn), type_id, &mut int_unpacker) {
                Ok(obj) => self.objects.push((obj, item.id)),
                Err(err) => warn.warn(Warning::GamenetDdnet(err)),
            }
        }

        Ok(())
    }
}
