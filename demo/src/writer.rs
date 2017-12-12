use arrayvec::Array;
use arrayvec::ArrayVec;
use buffer;
use common::num::Cast;
use common::num::LeI32;
use huffman::instances::TEEWORLDS as HUFFMAN;
use packer::with_packer;
use std::mem;
use std::slice;

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
    assert!(A::capacity() - 1 >= data.len());
    data.iter().cloned().collect()
}

const WRITER_VERSION: Version = Version::V5;

struct BytesToInts<'a>(slice::Chunks<'a, u8>);

impl<'a> BytesToInts<'a> {
    fn new(bytes: &'a [u8]) -> BytesToInts<'a> {
        BytesToInts(bytes.chunks(mem::size_of::<LeI32>()))
    }
}

impl<'a> Iterator for BytesToInts<'a> {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        fn g(bytes: &[u8], idx: usize) -> u8 {
            bytes.get(idx).cloned().unwrap_or(0)
        }
        self.0.next().map(|b| {
            LeI32::from_bytes(&[g(b, 0), g(b, 1), g(b, 2), g(b, 3)]).to_i32()
        })
    }
}


impl Writer {
    pub fn new<CB: Callback>(cb: &mut CB, net_version: &[u8], map_name: &[u8], map_crc: u32, type_: &[u8], timestamp: &[u8])
        -> Result<Writer, CB::Error>
    {
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
        writer.write_header(cb)?;
        cb.write_raw(&TimelineMarkers { timeline_markers: ArrayVec::new() }.pack())?;
        Ok(writer)
    }
    fn write_header<CB: Callback>(&mut self, cb: &mut CB) -> Result<(), CB::Error> {
        cb.write_raw(&HeaderVersion { version: WRITER_VERSION }.pack())?;
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
    fn write_chunk_impl<CB>(cb: &mut CB, type_: ChunkType, data: &[u8])
        -> Result<(), CB::Error>
        where CB: Callback,
    {
        ChunkHeader::Chunk(type_, data.len().assert_u32())
            .write(cb, WRITER_VERSION)?;
        cb.write(data)?;
        Ok(())
    }
    fn write_chunk_bytes<CB>(&mut self, cb: &mut CB, type_: ChunkType, data: &[u8])
        -> Result<(), CB::Error>
        where CB: Callback,
    {
        self.write_chunk_ints(cb, type_, BytesToInts::new(data))
    }
    fn write_chunk_ints<CB, I>(&mut self, cb: &mut CB, type_: ChunkType, data: I)
        -> Result<(), CB::Error>
        where CB: Callback,
              I: IntoIterator<Item=i32>,
    {
        self.buffer2.clear();
        with_packer(&mut self.buffer2, |mut p| -> Result<(), buffer::CapacityError> {
            for i in data {
                p.write_int(i)?;
            }
            Ok(())
        }).expect("overlong ints chunk");
        HUFFMAN.compress(&self.buffer2, &mut self.buffer1).expect("too long compression");
        Self::write_chunk_impl(cb, type_, &self.buffer1)
    }
    pub fn write_snapshot<CB: Callback>(&mut self, cb: &mut CB, snapshot: &[i32])
        -> Result<(), CB::Error>
    {
        self.write_chunk_ints(cb, ChunkType::Snapshot, snapshot.iter().cloned())
    }
    pub fn write_snapshot_delta<CB: Callback>(&mut self, cb: &mut CB, delta: &[i32])
        -> Result<(), CB::Error>
    {
        self.write_chunk_ints(cb, ChunkType::SnapshotDelta, delta.iter().cloned())
    }
    pub fn write_message<CB: Callback>(&mut self, cb: &mut CB, msg: &[u8])
        -> Result<(), CB::Error>
    {
        self.write_chunk_bytes(cb, ChunkType::Message, msg)
    }
    // TODO: Add a `finalize` function that writes the demo length into the
    // original header.
}
