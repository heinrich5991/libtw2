use arrayvec::Array;
use arrayvec::ArrayVec;
use buffer;
use common::digest::Sha256;
use common::num::Cast;
use common::num::LeI32;
use huffman::instances::TEEWORLDS as HUFFMAN;
use packer::with_packer;
use std::mem;
use uuid::Uuid;

use bitmagic::WriteCallbackExt;
use format::Chunk;
use format::ChunkHeader;
use format::ChunkType;
use format::Header;
use format::HeaderVersion;
use format::MAX_SNAPSHOT_SIZE;
use format::Tick;
use format::Tickmarker;
use format::TimelineMarkers;
use format::Version;

pub trait Callback {
    type Error;
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error>;
}

pub struct Writer {
    header: Header,
    prev_tick: Option<Tick>,
    buffer1: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
    buffer2: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
}

fn nullterminated_arrayvec_from_slice<A: Array>(data: &[A::Item]) -> ArrayVec<A>
    where A::Item: Clone,
{
    // `- 1` for null termination.
    assert!(A::CAPACITY - 1 >= data.len());
    data.iter().cloned().collect()
}

const WRITER_VERSION: Version = Version::V5;
const WRITER_VERSION_DDNET: Version = Version::V6Ddnet;

const DDNET_SHA256_EXTENSION: Uuid =
    Uuid::from_u128(0x6be6da4a_cebd_380c_9b5b_1289c842d780);

impl Writer {
    pub fn new<CB: Callback>(
        cb: &mut CB,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Option<Sha256>,
        map_crc: u32,
        type_: &[u8],
        timestamp: &[u8],
    ) -> Result<Writer, CB::Error> {
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
        writer.write_header(cb, map_sha256.is_some())?;
        cb.write_raw(&TimelineMarkers { timeline_markers: ArrayVec::new() }.pack())?;
        if let Some(sha256) = map_sha256 {
            cb.write_raw(DDNET_SHA256_EXTENSION.as_bytes())?;
            cb.write_raw(&sha256.0)?;
        }
        Ok(writer)
    }
    fn write_header<CB: Callback>(&mut self, cb: &mut CB, ddnet: bool)
        -> Result<(), CB::Error>
    {
        let version = if ddnet { WRITER_VERSION_DDNET } else { WRITER_VERSION };
        cb.write_raw(&HeaderVersion { version: version }.pack())?;
        cb.write_raw(&self.header.pack())?;
        Ok(())
    }
    pub fn write_chunk<CB: Callback>(&mut self, cb: &mut CB, chunk: Chunk)
        -> Result<(), CB::Error>
    {
        match chunk {
            Chunk::Tick(keyframe, tick) => self.write_tick(cb, keyframe, tick),
            Chunk::Snapshot(snapshot) => self.write_snapshot(cb, snapshot),
            Chunk::SnapshotDelta(delta) => self.write_snapshot_delta(cb, delta),
            Chunk::Message(msg) => self.write_message(cb, msg),
        }
    }
    pub fn write_tick<CB: Callback>(&mut self, cb: &mut CB, keyframe: bool, tick: Tick)
        -> Result<(), CB::Error>
    {
        let tm = Tickmarker::new(tick, self.prev_tick, keyframe, WRITER_VERSION);
        ChunkHeader::Tickmarker(keyframe, tm).write(cb, WRITER_VERSION)?;
        self.prev_tick = Some(tick);
        Ok(())
    }
    fn write_chunk_impl<CB>(
        cb: &mut CB,
        buffer: &mut ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
        type_: ChunkType,
        data: &[u8]
    ) -> Result<(), CB::Error>
        where CB: Callback,
    {
        buffer.clear();
        HUFFMAN.compress(&data, &mut *buffer).expect("too long compression");
        ChunkHeader::Chunk(type_, buffer.len().assert_u32())
            .write(cb, WRITER_VERSION)?;
        cb.write(buffer)?;
        Ok(())
    }
    pub fn write_snapshot<CB: Callback>(&mut self, cb: &mut CB, snapshot: &[u8])
        -> Result<(), CB::Error>
    {
        Self::write_chunk_impl(cb, &mut self.buffer1, ChunkType::Snapshot, snapshot)
    }
    pub fn write_snapshot_delta<CB: Callback>(&mut self, cb: &mut CB, delta: &[u8])
        -> Result<(), CB::Error>
    {
        Self::write_chunk_impl(cb, &mut self.buffer1, ChunkType::SnapshotDelta, delta)
    }
    pub fn write_message<CB: Callback>(&mut self, cb: &mut CB, msg: &[u8])
        -> Result<(), CB::Error>
    {
        self.buffer2.clear();
        with_packer(&mut self.buffer2, |mut p| -> Result<(), buffer::CapacityError> {
            for b in msg.chunks(mem::size_of::<LeI32>()) {
                // Get or return 0.
                fn g(bytes: &[u8], idx: usize) -> u8 {
                    bytes.get(idx).cloned().unwrap_or(0)
                }
                let i = LeI32::from_bytes(&[g(b, 0), g(b, 1), g(b, 2), g(b, 3)]).to_i32();
                p.write_int(i)?;
            }
            Ok(())
        }).expect("overlong message");
        Self::write_chunk_impl(cb, &mut self.buffer1, ChunkType::Message, &self.buffer2)
    }
    // TODO: Add a `finalize` function that writes the demo length into the
    // original header.
}
