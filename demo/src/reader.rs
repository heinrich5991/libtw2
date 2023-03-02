use arrayvec::ArrayVec;
use binrw::BinRead;
use common::num::Cast;
use huffman;
use huffman::instances::TEEWORLDS as HUFFMAN;
use packer;
use std::io;
use thiserror::Error;
use warn::wrap;
use warn::Warn;

use crate::format;
use crate::format::TickMarker;
use crate::format::Warning;
use crate::format::MAX_SNAPSHOT_SIZE;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum ReadError {
    Io(#[from] io::Error),
    Binrw(#[from] binrw::Error),
    Huffman(#[from] huffman::DecompressionError),
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

impl ReadError {
    pub fn io_error(self) -> Result<io::Error, ReadError> {
        match self {
            ReadError::Io(io) => Ok(io),
            ReadError::Binrw(binrw::Error::Io(io)) => Ok(io),
            err => Err(err),
        }
    }
}

trait SeekableRead: io::Read + io::Seek {}
impl<T: io::Read + io::Seek> SeekableRead for T {}

pub struct Reader {
    data: Box<dyn SeekableRead>,
    start: format::HeaderStart,
    current_tick: Option<i32>,
    raw: [u8; MAX_SNAPSHOT_SIZE],
    huffman: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
}

impl Reader {
    pub fn new<W, R>(mut data: R, warn: &mut W) -> Result<Reader, ReadError>
    where
        W: Warn<Warning>,
        R: io::Read + io::Seek + 'static,
    {
        let start = format::HeaderStart::read(&mut data)?;
        start.header.check(warn);
        start.timeline_markers.check(warn);
        Ok(Self {
            data: Box::new(data),
            start: start,
            current_tick: None,
            raw: [0; MAX_SNAPSHOT_SIZE],
            huffman: ArrayVec::new(),
        })
    }
    pub fn version(&self) -> format::Version {
        self.start.version
    }
    pub fn net_version(&self) -> &[u8] {
        self.start.header.net_version.raw()
    }
    pub fn map_name(&self) -> &[u8] {
        self.start.header.map_name.raw()
    }
    pub fn map_size(&self) -> u32 {
        self.start.header.map_size.assert_u32()
    }
    pub fn map_crc(&self) -> u32 {
        self.start.header.map_crc
    }
    pub fn timestamp(&self) -> &[u8] {
        self.start.header.timestamp.raw()
    }
    pub fn timeline_markers(&self) -> &[i32] {
        self.start.timeline_markers.markers()
    }
    pub fn read_chunk<W>(&mut self, warn: &mut W) -> Result<Option<format::RawChunk>, ReadError>
    where
        W: Warn<Warning>,
    {
        use crate::format::ChunkHeader;
        use crate::format::DataKind;
        use crate::format::RawChunk;

        let chunk_header = match ChunkHeader::read(&mut self.data, self.start.version, warn)? {
            Some(ch) => ch,
            None => return Ok(None),
        };
        match chunk_header {
            ChunkHeader::Tick {
                marker: TickMarker::Absolute(t),
                keyframe,
            } => {
                if let Some(previous) = self.current_tick {
                    if previous >= t {
                        return Err(ReadError::NotIncreasingTick);
                    }
                }
                self.current_tick = Some(t);
                Ok(Some(RawChunk::Tick {
                    tick: t,
                    keyframe: keyframe,
                }))
            }
            ChunkHeader::Tick {
                marker: TickMarker::Delta(d),
                keyframe,
            } => match self.current_tick {
                None => Err(ReadError::StartingDeltaSnapshot),
                Some(t) => match t.checked_add(d.i32()) {
                    None => Err(ReadError::TickOverflow),
                    Some(new_t) => {
                        self.current_tick = Some(new_t);
                        Ok(Some(RawChunk::Tick {
                            tick: new_t,
                            keyframe: keyframe,
                        }))
                    }
                },
            },
            ChunkHeader::Data { kind, size } => {
                if kind == DataKind::Unknown {
                    return Ok(Some(RawChunk::Unknown));
                }
                let raw_data = &mut self.raw[..size.usize()];
                self.data.read_exact(raw_data)?;
                self.huffman.clear();
                HUFFMAN.decompress(raw_data, &mut self.huffman)?;
                Ok(Some(match kind {
                    DataKind::Unknown => RawChunk::Unknown,
                    DataKind::Snapshot => RawChunk::Snapshot(&self.huffman),
                    DataKind::SnapshotDelta => RawChunk::SnapshotDelta(&self.huffman),
                    DataKind::Message => {
                        let mut unpacker = packer::Unpacker::new(&self.huffman);
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
                        RawChunk::Message(&self.raw[..len])
                    }
                }))
            }
        }
    }
}
