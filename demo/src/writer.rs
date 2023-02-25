use arrayvec::Array;
use arrayvec::ArrayVec;
use buffer;
use common::digest::Sha256;
use common::num::Cast;
use common::num::LeI32;
use huffman::instances::TEEWORLDS as HUFFMAN;
use packer::with_packer;
use std::{io, mem};
use uuid::Uuid;

use crate::bitmagic::WriteExt;
use crate::format::Chunk;
use crate::format::ChunkHeader;
use crate::format::ChunkType;
use crate::format::Header;
use crate::format::HeaderVersion;
use crate::format::Tick;
use crate::format::Tickmarker;
use crate::format::TimelineMarkers;
use crate::format::Version;
use crate::format::MAX_SNAPSHOT_SIZE;

pub struct Writer {
    header: Header,
    prev_tick: Option<Tick>,
    buffer1: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
    buffer2: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
}

fn nullterminated_arrayvec_from_slice<A: Array>(data: &[A::Item]) -> ArrayVec<A>
where
    A::Item: Clone,
{
    // `- 1` for null termination.
    assert!(A::CAPACITY - 1 >= data.len());
    data.iter().cloned().collect()
}

const WRITER_VERSION: Version = Version::V5;
const WRITER_VERSION_DDNET: Version = Version::V6Ddnet;

const DDNET_SHA256_EXTENSION: Uuid = Uuid::from_u128(0x6be6da4a_cebd_380c_9b5b_1289c842d780);

pub(crate) trait SeekableWrite: io::Write + io::Seek {}
impl<T: io::Write + io::Seek> SeekableWrite for T {}

impl Writer {
    pub fn new<W: io::Write + io::Seek>(
        file: &mut W,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Option<Sha256>,
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> Result<Writer, io::Error> {
        use self::nullterminated_arrayvec_from_slice as nafs;

        let mut writer = Writer {
            header: Header {
                net_version: nafs(net_version),
                map_name: nafs(map_name),
                map_size: 0,
                map_crc: map_crc,
                type_: nafs(type_),
                length: Default::default(),
                timestamp: nafs(timestamp),
            },
            prev_tick: None,
            buffer1: ArrayVec::new(),
            buffer2: ArrayVec::new(),
        };
        writer.write_header(file, map_sha256.is_some())?;
        file.write_packed(
            &TimelineMarkers {
                timeline_markers: ArrayVec::new(),
            }
            .pack(),
        )?;
        if let Some(sha256) = map_sha256 {
            file.write_packed(DDNET_SHA256_EXTENSION.as_bytes())?;
            file.write_packed(&sha256.0)?;
        }
        Ok(writer)
    }
    fn write_header<W: SeekableWrite>(
        &mut self,
        file: &mut W,
        ddnet: bool,
    ) -> Result<(), io::Error> {
        let version = if ddnet {
            WRITER_VERSION_DDNET
        } else {
            WRITER_VERSION
        };
        file.write_packed(&HeaderVersion { version: version }.pack())?;
        file.write_packed(&self.header.pack())?;
        Ok(())
    }
    pub fn write_chunk<W: io::Write + io::Seek>(
        &mut self,
        file: &mut W,
        chunk: Chunk,
    ) -> Result<(), io::Error> {
        match chunk {
            Chunk::Tick(keyframe, tick) => self.write_tick(file, keyframe, tick),
            Chunk::Snapshot(snapshot) => self.write_snapshot(file, snapshot),
            Chunk::SnapshotDelta(delta) => self.write_snapshot_delta(file, delta),
            Chunk::Message(msg) => self.write_message(file, msg),
        }
    }
    pub fn write_tick<W: io::Write + io::Seek>(
        &mut self,
        file: &mut W,
        keyframe: bool,
        tick: Tick,
    ) -> Result<(), io::Error> {
        let tm = Tickmarker::new(tick, self.prev_tick, keyframe, WRITER_VERSION);
        ChunkHeader::Tickmarker(keyframe, tm).write(file, WRITER_VERSION)?;
        self.prev_tick = Some(tick);
        Ok(())
    }
    fn write_chunk_impl<W: SeekableWrite>(
        file: &mut W,
        buffer: &mut ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
        type_: ChunkType,
        data: &[u8],
    ) -> Result<(), io::Error> {
        buffer.clear();
        HUFFMAN
            .compress(&data, &mut *buffer)
            .expect("too long compression");
        ChunkHeader::Chunk(type_, buffer.len().assert_u32()).write(file, WRITER_VERSION)?;
        file.write(buffer)?;
        Ok(())
    }
    pub fn write_snapshot<W: io::Write + io::Seek>(
        &mut self,
        file: &mut W,
        snapshot: &[u8],
    ) -> Result<(), io::Error> {
        Self::write_chunk_impl(file, &mut self.buffer1, ChunkType::Snapshot, snapshot)
    }
    pub fn write_snapshot_delta<W: io::Write + io::Seek>(
        &mut self,
        file: &mut W,
        delta: &[u8],
    ) -> Result<(), io::Error> {
        Self::write_chunk_impl(file, &mut self.buffer1, ChunkType::SnapshotDelta, delta)
    }
    pub fn write_message<W: io::Write + io::Seek>(
        &mut self,
        file: &mut W,
        msg: &[u8],
    ) -> Result<(), io::Error> {
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
        Self::write_chunk_impl(file, &mut self.buffer1, ChunkType::Message, &self.buffer2)
    }
    // TODO: Add a `finalize` function that writes the demo length into the
    // original header.
}
