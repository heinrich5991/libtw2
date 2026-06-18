use arrayvec::ArrayVec;
use binrw::BinRead;
use libtw2_common::num::Cast;
use libtw2_gamenet_common::traits;
use libtw2_gamenet_common::traits::MessageExt;
use libtw2_gamenet_common::traits::Protocol;
use libtw2_gamenet_common::traits::ProtocolStatic;
use libtw2_huffman::instances::TEEWORLDS as HUFFMAN;
use libtw2_packer::ExcessData;
use libtw2_packer::IntUnpacker;
use libtw2_packer::Unpacker;
use libtw2_snapshot::format::Item as SnapItem;
use libtw2_snapshot::snap;
use libtw2_snapshot::Delta;
use libtw2_snapshot::Snap;
use libtw2_warn::wrap;
use libtw2_warn::Warn;
use std::io;
use std::marker::PhantomData;
use std::mem;
use std::slice;
use thiserror::Error;

use crate::format;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Binrw(#[from] binrw::Error),
    #[error(transparent)]
    Huffman(#[from] libtw2_huffman::DecompressionError),
    #[error("Snap parsing - {0:?}")]
    Snap(snap::Error),
    #[error("Unexpected data end during secondary decompression of message")]
    MessageVarIntUnexpectedEnd,
    #[error("Too big decompressed size during secondary decompression of message")]
    MessageVarIntTooLong,
    #[error("Tick number did not increase")]
    NotIncreasingTick,
    #[error("The first snapshot is only a snapshot-delta")]
    StartingDeltaSnapshot,
    #[error("The tick number overflowed")]
    TickOverflow,
}

trait SeekableRead: io::Read + io::Seek {}
impl<T: io::Read + io::Seek> SeekableRead for T {}

pub struct DemoReader<'a, P: for<'p> Protocol<'p>> {
    data: Box<dyn SeekableRead + 'a>,
    start: format::HeaderStart,
    current_tick: Option<i32>,
    raw: [u8; format::MAX_SNAPSHOT_SIZE],
    huffman: ArrayVec<[u8; format::MAX_SNAPSHOT_SIZE]>,
    delta: Delta,
    snap: Snap,
    old_snap: Snap,
    snap_read_buf: Vec<i32>,
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
impl From<ExcessData> for Warning {
    fn from(ExcessData: ExcessData) -> Self {
        Warning::ExcessItemData
    }
}

pub enum Chunk<'a, P: Protocol<'a>> {
    Message(P::Game),
    Snapshot(slice::Iter<'a, (P::SnapObj, u16)>),
    Tick { tick: i32, keyframe: bool },
    Invalid,
}

impl<'a, P: for<'p> Protocol<'p>> DemoReader<'a, P> {
    pub fn new<R, W>(mut data: R, warn: &mut W) -> Result<Self, ReadError>
    where
        R: io::Read + io::Seek + 'a,
        W: Warn<Warning>,
    {
        let start = format::HeaderStart::read(&mut data)?;
        start.header.check(wrap(warn));
        start.timeline_markers.check(wrap(warn));
        Ok(DemoReader {
            data: Box::new(data),
            start: start,
            current_tick: None,
            raw: [0; format::MAX_SNAPSHOT_SIZE],
            huffman: ArrayVec::new(),
            delta: Delta::new(),
            snap: Snap::empty(),
            old_snap: Snap::empty(),
            snap_read_buf: Vec::new(),
            snapshot: Snapshot::default(),
            protocol: PhantomData,
        })
    }

    pub fn header(&self) -> &format::HeaderStart {
        &self.start
    }

    pub fn next_chunk<W: Warn<Warning>>(
        &mut self,
        warn: &mut W,
    ) -> Result<Option<Chunk<'_, P>>, ReadError> {
        let Some(chunk_header) =
            format::ChunkHeader::read(&mut self.data, self.start.version, wrap(warn))?
        else {
            return Ok(None);
        };
        match chunk_header {
            format::ChunkHeader::Tick {
                marker: format::TickMarker::Absolute(t),
                keyframe,
            } => {
                if let Some(previous) = self.current_tick {
                    if previous >= t {
                        return Err(ReadError::NotIncreasingTick);
                    }
                }
                self.current_tick = Some(t);
                Ok(Some(Chunk::Tick {
                    tick: t,
                    keyframe: keyframe,
                }))
            }
            format::ChunkHeader::Tick {
                marker: format::TickMarker::Delta(d),
                keyframe,
            } => match self.current_tick {
                None => Err(ReadError::StartingDeltaSnapshot),
                Some(t) => match t.checked_add(d.i32()) {
                    None => Err(ReadError::TickOverflow),
                    Some(new_t) => {
                        self.current_tick = Some(new_t);
                        Ok(Some(Chunk::Tick {
                            tick: new_t,
                            keyframe: keyframe,
                        }))
                    }
                },
            },
            format::ChunkHeader::Data { kind, size } => {
                let raw_data = &mut self.raw[..size.usize()];
                self.data.read_exact(raw_data)?;
                self.huffman.clear();
                HUFFMAN.decompress(raw_data, &mut self.huffman)?;
                match kind {
                    format::DataKind::Unknown => Ok(Some(Chunk::Invalid)),
                    format::DataKind::Snapshot => {
                        self.snap
                            .read(wrap(warn), &mut self.snap_read_buf, &self.huffman)
                            .map_err(ReadError::Snap)?;
                        self.snapshot.build::<P, _>(warn, &self.snap)?;
                        Ok(Some(Chunk::Snapshot(self.snapshot.objects.iter())))
                    }
                    format::DataKind::SnapshotDelta => {
                        let mut unpacker = Unpacker::new(&self.huffman);
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
                    format::DataKind::Message => {
                        let mut unpacker = Unpacker::new(&self.huffman);
                        let mut len = 0;
                        let mut buffer = self.raw.chunks_mut(4);
                        while !unpacker.is_empty() {
                            let n: i32 = unpacker
                                .read_int(wrap(warn))
                                .map_err(|_| ReadError::MessageVarIntUnexpectedEnd)?;
                            buffer
                                .next()
                                .ok_or(ReadError::MessageVarIntTooLong)?
                                .copy_from_slice(&n.to_le_bytes());
                            len += 4;
                        }
                        let mut unpacker = Unpacker::new_from_demo(&self.raw[..len]);
                        match P::Game::decode(wrap(warn), &mut unpacker) {
                            Ok(msg) => Ok(Some(Chunk::Message(msg))),
                            Err(err) => {
                                warn.warn(Warning::Gamenet(err));
                                Ok(Some(Chunk::Invalid))
                            }
                        }
                    }
                }
            }
        }
    }
}

struct Snapshot<T> {
    pub objects: Vec<(T, u16)>,
}

impl<T> Default for Snapshot<T> {
    fn default() -> Snapshot<T> {
        Snapshot {
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
        self.objects.clear();

        for SnapItem { type_id, id, data } in snap.items() {
            let mut int_unpacker = IntUnpacker::new(data);
            match P::SnapObj::decode_obj(wrap(warn), type_id, &mut int_unpacker) {
                Ok(obj) => self.objects.push((obj, id)),
                Err(err) => warn.warn(Warning::Gamenet(err)),
            }
        }

        Ok(())
    }
}
