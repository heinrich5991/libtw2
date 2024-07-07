use libtw2_common::digest::Sha256;
use libtw2_gamenet_common::snap_obj::TypeId;
use libtw2_gamenet_common::traits;
use libtw2_gamenet_common::traits::MessageExt as _;
use libtw2_gamenet_common::traits::Protocol;
use libtw2_gamenet_common::traits::ProtocolStatic;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Unpacker;
use libtw2_snapshot::snap;
use libtw2_snapshot::Delta;
use libtw2_snapshot::Snap;
use libtw2_snapshot::SnapReader;
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::slice;
use thiserror::Error;
use uuid::Uuid;
use warn::wrap;
use warn::Warn;

use crate::format;
use crate::reader;
use crate::DemoKind;
use crate::RawChunk;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error(transparent)]
    Inner(#[from] reader::ReadError),
    #[error("Snap parsing - {0:?}")]
    Snap(snap::Error),
    #[error("UUID item has an incorrect size")]
    UuidItemLength,
    #[error("Extended type ID that is not registered")]
    UnregisteredUuidTypeId,
}

pub struct DemoReader<'a, P: for<'p> Protocol<'p>> {
    raw: reader::Reader<'a>,
    delta: Delta,
    snap: Snap,
    old_snap: Snap,
    snap_reader: SnapReader,
    snapshot: Snapshot<P::SnapObj>,
    protocol: PhantomData<P>,
}

#[derive(Debug)]
pub enum Warning {
    Demo(format::Warning),
    Snapshot(libtw2_snapshot::format::Warning),
    Packer(libtw2_packer::Warning),
    ExcessItemData,
    Gamenet(libtw2_gamenet_common::error::Error),
}

impl From<format::Warning> for Warning {
    fn from(w: format::Warning) -> Self {
        Warning::Demo(w)
    }
}
impl From<libtw2_snapshot::format::Warning> for Warning {
    fn from(w: libtw2_snapshot::format::Warning) -> Self {
        Warning::Snapshot(w)
    }
}
impl From<libtw2_packer::Warning> for Warning {
    fn from(w: libtw2_packer::Warning) -> Self {
        Warning::Packer(w)
    }
}
impl From<libtw2_packer::ExcessData> for Warning {
    fn from(_: libtw2_packer::ExcessData) -> Self {
        Warning::ExcessItemData
    }
}

pub enum Chunk<'a, P: Protocol<'a>> {
    Message(P::Game),
    Snapshot(slice::Iter<'a, (P::SnapObj, u16)>),
    Tick(i32),
    Invalid,
}

impl<'a, P: for<'p> Protocol<'p>> DemoReader<'a, P> {
    pub fn new<R, W>(data: R, warn: &mut W) -> Result<Self, ReadError>
    where
        R: io::Read + io::Seek + 'a,
        W: Warn<Warning>,
    {
        let reader = reader::Reader::new(data, wrap(warn))?;
        Ok(DemoReader {
            raw: reader,
            delta: Delta::new(),
            snap: Snap::empty(),
            old_snap: Snap::empty(),
            snap_reader: SnapReader::new(),
            snapshot: Snapshot::default(),
            protocol: PhantomData,
        })
    }

    pub fn version(&self) -> format::Version {
        self.raw.version()
    }
    pub fn net_version(&self) -> &[u8] {
        self.raw.net_version()
    }
    pub fn map_name(&self) -> &[u8] {
        self.raw.map_name()
    }
    pub fn map_size(&self) -> u32 {
        self.raw.map_size()
    }
    pub fn map_data(&self) -> &[u8] {
        self.raw.map_data()
    }
    pub fn map_crc(&self) -> u32 {
        self.raw.map_crc()
    }
    pub fn kind(&self) -> DemoKind {
        self.raw.kind()
    }
    pub fn length(&self) -> i32 {
        self.raw.length()
    }
    pub fn timestamp(&self) -> &[u8] {
        self.raw.timestamp()
    }
    pub fn timeline_markers(&self) -> &[i32] {
        self.raw.timeline_markers()
    }
    pub fn map_sha256(&self) -> Option<Sha256> {
        self.raw.map_sha256()
    }

    pub fn next_chunk<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
    ) -> Result<Option<Chunk<P>>, ReadError> {
        match self.raw.read_chunk(wrap(warn))? {
            None => return Ok(None),
            Some(RawChunk::Unknown) => Ok(Some(Chunk::Invalid)),
            Some(RawChunk::Tick { tick, .. }) => Ok(Some(Chunk::Tick(tick))),
            Some(RawChunk::Message(msg)) => {
                let mut unpacker = Unpacker::new_from_demo(msg);
                match P::Game::decode(wrap(warn), &mut unpacker) {
                    Ok(msg) => Ok(Some(Chunk::Message(msg))),
                    Err(err) => {
                        warn.warn(Warning::Gamenet(err));
                        Ok(Some(Chunk::Invalid))
                    }
                }
            }
            Some(RawChunk::Snapshot(snap)) => {
                let mut unpacker = Unpacker::new(snap);
                let mut swap = Snap::empty();
                mem::swap(&mut self.snap, &mut swap);
                self.snap = self
                    .snap_reader
                    .read(wrap(warn), swap, &mut unpacker)
                    .unwrap();
                self.snapshot.build::<P, _>(warn, &self.snap)?;
                Ok(Some(Chunk::Snapshot(self.snapshot.objects.iter())))
            }
            Some(RawChunk::SnapshotDelta(dt)) => {
                let mut unpacker = Unpacker::new(dt);
                self.delta
                    .read(wrap(warn), P::obj_size, &mut unpacker)
                    .map_err(ReadError::Snap)?;
                self.old_snap
                    .read_with_delta(wrap(warn), &self.snap, &self.delta)
                    .map_err(ReadError::Snap)?;
                mem::swap(&mut self.old_snap, &mut self.snap);
                self.snapshot.build::<P, _>(warn, &self.snap)?;
                Ok(Some(Chunk::Snapshot(self.snapshot.objects.iter())))
            }
        }
    }
}

struct Snapshot<T> {
    uuid_index: HashMap<u16, Uuid>,
    pub objects: Vec<(T, u16)>,
}

impl<T> Default for Snapshot<T> {
    fn default() -> Snapshot<T> {
        Snapshot {
            uuid_index: Default::default(),
            objects: Default::default(),
        }
    }
}

impl<T> Snapshot<T> {
    fn build<P, W>(&mut self, warn: &mut W, snap: &Snap) -> Result<(), ReadError>
    where
        P: ProtocolStatic<SnapObj = T>,
        T: traits::SnapObj,
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
            let mut int_unpacker = IntUnpacker::new(item.data);
            match P::SnapObj::decode_obj(wrap(warn), type_id, &mut int_unpacker) {
                Ok(obj) => self.objects.push((obj, item.id)),
                Err(err) => warn.warn(Warning::Gamenet(err)),
            }
        }

        Ok(())
    }
}
