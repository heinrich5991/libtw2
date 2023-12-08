use arrayvec::ArrayVec;
use binrw::BinWrite;
use buffer;
use common::digest::Sha256;
use common::num::Cast;
use common::num::LeI32;
use huffman::instances::TEEWORLDS as HUFFMAN;
use packer::with_packer;
use std::io;
use std::mem;
use thiserror::Error;

use crate::format::CappedString;
use crate::format::ChunkHeader;
use crate::format::DataKind;
use crate::format::DemoKind;
use crate::format::Header;
use crate::format::MapSha256;
use crate::format::RawChunk;
use crate::format::TickMarker;
use crate::format::TimelineMarkers;
use crate::format::Version;
use crate::format::MAX_SNAPSHOT_SIZE;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct WriteError(#[from] binrw::Error);

impl WriteError {
    pub fn io_error(self) -> Result<io::Error, WriteError> {
        match self.0 {
            binrw::Error::Io(io) => Ok(io),
            err => Err(WriteError(err)),
        }
    }
}

pub struct Writer {
    file: Box<dyn SeekableWrite>,
    header: Header,
    prev_tick: Option<i32>,
    huffman: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
    buffer2: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
}

const WRITER_VERSION: Version = Version::V5;
const WRITER_VERSION_DDNET: Version = Version::V6Ddnet;

pub(crate) trait SeekableWrite: io::Write + io::Seek {}
impl<T: io::Write + io::Seek> SeekableWrite for T {}

impl Writer {
    pub fn new<W: io::Write + io::Seek + 'static>(
        file: W,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Option<Sha256>,
        map_crc: u32,
        kind: DemoKind,
        length: i32,
        timestamp: &[u8],
        map: &[u8],
    ) -> Result<Writer, WriteError> {
        let mut writer = Writer {
            file: Box::new(file),
            header: Header {
                net_version: CappedString::from_raw(net_version),
                map_name: CappedString::from_raw(map_name),
                map_size: map.len().assert_i32(),
                map_crc: map_crc,
                kind: kind,
                length,
                timestamp: CappedString::from_raw(timestamp),
            },
            prev_tick: None,
            huffman: ArrayVec::new(),
            buffer2: ArrayVec::new(),
        };
        writer.write_header(map_sha256.is_some())?;
        TimelineMarkers {
            amount: 0,
            markers: [0; 64],
        }
        .write(&mut writer.file)?;
        if let Some(sha256) = map_sha256 {
            MapSha256::new(sha256).write_le(&mut writer.file)?;
        }
        map.write(&mut writer.file)?;
        Ok(writer)
    }
    fn write_header(&mut self, ddnet: bool) -> Result<(), WriteError> {
        let version = if ddnet {
            WRITER_VERSION_DDNET
        } else {
            WRITER_VERSION
        };
        version.write(&mut self.file)?;
        self.header.write(&mut self.file)?;
        Ok(())
    }
    pub fn write_chunk(&mut self, chunk: RawChunk) -> Result<(), WriteError> {
        match chunk {
            RawChunk::Tick { tick, keyframe } => self.write_tick(keyframe, tick),
            RawChunk::Snapshot(snapshot) => self.write_snapshot(snapshot),
            RawChunk::SnapshotDelta(delta) => self.write_snapshot_delta(delta),
            RawChunk::Message(msg) => self.write_message(msg),
            RawChunk::Unknown => panic!(),
        }
    }
    pub fn write_tick(&mut self, keyframe: bool, tick: i32) -> Result<(), WriteError> {
        let tm = TickMarker::new(tick, self.prev_tick, keyframe, WRITER_VERSION);
        ChunkHeader::Tick {
            marker: tm,
            keyframe: keyframe,
        }
        .write(&mut self.file, WRITER_VERSION)?;
        self.prev_tick = Some(tick);
        Ok(())
    }
    fn write_chunk_impl(&mut self, kind: DataKind, data: Option<&[u8]>) -> Result<(), WriteError> {
        let data = data.unwrap_or(&self.buffer2);
        self.huffman.clear();
        HUFFMAN
            .compress(data, &mut self.huffman)
            .expect("too long compression");
        ChunkHeader::Data {
            kind,
            size: self.huffman.len().assert_u16(),
        }
        .write(&mut self.file, WRITER_VERSION)?;
        self.file
            .write_all(&self.huffman)
            .map_err(binrw::Error::Io)?;
        Ok(())
    }
    pub fn write_snapshot(&mut self, snapshot: &[u8]) -> Result<(), WriteError> {
        self.write_chunk_impl(DataKind::Snapshot, Some(snapshot))
    }
    pub fn write_snapshot_delta(&mut self, delta: &[u8]) -> Result<(), WriteError> {
        self.write_chunk_impl(DataKind::SnapshotDelta, Some(delta))
    }
    pub fn write_message(&mut self, msg: &[u8]) -> Result<(), WriteError> {
        self.buffer2.clear();
        with_packer(
            &mut self.buffer2,
            |mut p| -> Result<(), buffer::CapacityError> {
                for b in msg.chunks(mem::size_of::<LeI32>()) {
                    // Get or return 0.
                    fn g(bytes: &[u8], idx: usize) -> u8 {
                        bytes.get(idx).cloned().unwrap_or(0)
                    }
                    let i = LeI32::from_bytes(&[g(b, 0), g(b, 1), g(b, 2), g(b, 3)]).to_i32();
                    p.write_int(i)?;
                }
                Ok(())
            },
        )
        .expect("overlong message");
        self.write_chunk_impl(DataKind::Message, None)
    }
    // TODO: Add a `finalize` function that writes the demo length into the
    // original header.
}
