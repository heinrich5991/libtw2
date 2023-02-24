use common::num::Cast;
use huffman;
use huffman::instances::TEEWORLDS as HUFFMAN;
use packer;
use packer::Unpacker;
use std::io;
use warn::Warn;

use bitmagic::ReadExt;
use format;
use format::Warning;
use format::MAX_SNAPSHOT_SIZE;

fn huffman_error(e: huffman::DecompressionError) -> format::Error {
    use huffman::DecompressionError::*;
    match e {
        Capacity(_) => format::Error::HuffmanDecompressionTooLong,
        InvalidInput => format::Error::HuffmanDecompressionError,
    }
}

fn _packer_warning(w: packer::Warning) -> format::Warning {
    use packer::Warning::*;
    match w {
        OverlongIntEncoding => Warning::IntDecompressionOverlongEncoding,
        NonZeroIntPadding => Warning::IntDecompressionNonZeroPadding,
        ExcessData => unreachable!(),
    }
}

fn _packer_error(_: packer::UnexpectedEnd) -> format::Error {
    format::Error::IntDecompressionError
}

fn _buffer_error(_: buffer::CapacityError) -> format::Error {
    format::Error::IntDecompressionTooLong
}

#[derive(Debug)]
pub enum Error {
    Demo(format::Error),
    Io(io::Error),
}

impl From<format::Error> for Error {
    fn from(err: format::Error) -> Error {
        Error::Demo(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

pub(crate) trait IoResultExt {
    type Ok;
    fn on_eof(self, demo_err: format::Error) -> Result<Self::Ok, Error>;
}

impl<T> IoResultExt for Result<T, io::Error> {
    type Ok = T;

    fn on_eof(self, demo_err: format::Error) -> Result<T, Error> {
        self.map_err(|err| {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                Error::Demo(demo_err)
            } else {
                Error::Io(err)
            }
        })
    }
}

struct Inner {
    version: format::Version,
    header: format::Header,
    timeline_markers: format::TimelineMarkers,
    _map_sha256: Option<[u8; 32]>,
    _map: Vec<u8>,
    current_tick: Option<format::Tick>,
    buffer1: [u8; MAX_SNAPSHOT_SIZE],
    buffer2: [u8; MAX_SNAPSHOT_SIZE],
}

pub struct Reader {
    i: Inner,
    error_encountered: bool,
}

const SHA_256_EXTENSION: [u8; 16] = [
    0x6b, 0xe6, 0xda, 0x4a, 0xce, 0xbd, 0x38, 0x0c, 0x9b, 0x5b, 0x12, 0x89, 0xc8, 0x42, 0xd7, 0x80,
];

impl Reader {
    pub fn new<W, R>(warn: &mut W, data: &mut R) -> Result<Reader, Error>
    where
        W: Warn<Warning>,
        R: io::Read,
    {
        let header_version: format::HeaderVersionPacked = data
            .read_packed()
            .on_eof(format::Error::TooShortHeaderVersion)?;
        let header_version = header_version.unpack()?;
        let version = header_version.version;
        let version_byte = version.to_u8();
        match version {
            format::Version::V4 | format::Version::V5 | format::Version::V6Ddnet => {}
            _ => return Err(format::Error::UnknownVersion(version_byte).into()),
        }
        let header: format::HeaderPacked =
            data.read_packed().on_eof(format::Error::TooShortHeader)?;
        let header = header.unpack(warn)?;
        let timeline_markers: format::TimelineMarkersPacked = data
            .read_packed()
            .on_eof(format::Error::TooShortTimelineMarkers)?;
        let timeline_markers = timeline_markers.unpack(warn)?;
        let mut map_sha256: Option<[u8; 32]> = None;
        if version == format::Version::V6Ddnet {
            let map_sha256_uuid: [u8; 16] = data
                .read_packed()
                .on_eof(format::Error::TooShortMapSha256)?;
            if map_sha256_uuid != SHA_256_EXTENSION {
                return Err(format::Error::NotMapSha256Extension.into());
            }
            map_sha256 = Some(
                data.read_packed()
                    .on_eof(format::Error::TooShortMapSha256)?,
            );
        }
        let mut map = vec![0; header.map_size.usize()];
        data.read_exact(&mut map)?;

        Ok(Reader {
            i: Inner {
                version: version,
                header: header,
                timeline_markers: timeline_markers,
                _map_sha256: map_sha256,
                _map: map,
                current_tick: None,
                buffer1: [0; MAX_SNAPSHOT_SIZE],
                buffer2: [0; MAX_SNAPSHOT_SIZE],
            },
            error_encountered: false,
        })
    }
    pub fn version(&self) -> format::Version {
        self.i.version
    }
    pub fn net_version(&self) -> &[u8] {
        &self.i.header.net_version
    }
    pub fn map_name(&self) -> &[u8] {
        &self.i.header.map_name
    }
    pub fn map_size(&self) -> u32 {
        self.i.header.map_size
    }
    pub fn map_crc(&self) -> u32 {
        self.i.header.map_crc
    }
    pub fn timestamp(&self) -> &[u8] {
        &self.i.header.timestamp
    }
    pub fn timeline_markers(&self) -> &[format::Tick] {
        &self.i.timeline_markers.timeline_markers
    }
    pub fn read_chunk<'a, W, R>(
        &'a mut self,
        warn: &mut W,
        cb: &mut R,
    ) -> Result<Option<format::Chunk<'a>>, Error>
    where
        W: Warn<Warning>,
        R: io::Read,
    {
        assert!(
            !self.error_encountered,
            "reading new chunks isn't supported after errors"
        );
        let result = self.i.read_chunk(warn, cb);
        if let Err(_) = result {
            self.error_encountered = true;
        }
        result
    }
}

impl Inner {
    pub fn read_chunk<'a, W, R>(
        &'a mut self,
        warn: &mut W,
        data: &mut R,
    ) -> Result<Option<format::Chunk<'a>>, Error>
    where
        W: Warn<Warning>,
        R: io::Read,
    {
        use format::Chunk;
        use format::ChunkHeader;
        use format::ChunkType;
        use format::Tickmarker;

        let chunk_header;
        if let Some(ch) = ChunkHeader::read(warn, data, self.version)? {
            chunk_header = ch;
        } else {
            return Ok(None);
        }
        match chunk_header {
            ChunkHeader::Tickmarker(keyframe, Tickmarker::Absolute(t)) => {
                if let Some(previous) = self.current_tick {
                    if previous >= t {
                        warn.warn(Warning::NonIncreasingTick);
                    }
                }
                self.current_tick = Some(t);
                Ok(Some(Chunk::Tick(keyframe, t)))
            }
            ChunkHeader::Tickmarker(keyframe, Tickmarker::Delta(d)) => {
                let cur = self.current_tick.unwrap_or_else(|| {
                    warn.warn(Warning::StartingDeltaTick);
                    format::Tick(0)
                });
                let result = format::Tick(cur.0.wrapping_add(d.i32()));
                if result < cur {
                    warn.warn(Warning::TickOverflow);
                }
                self.current_tick = Some(result);
                Ok(Some(Chunk::Tick(keyframe, result)))
            }
            ChunkHeader::Chunk(type_, size) => {
                let raw_data = &mut self.buffer1[..size.usize()];
                data.read_exact(raw_data).on_eof(format::Error::TooShort)?;
                let huff_data = HUFFMAN
                    .decompress(raw_data, self.buffer2.as_mut_slice())
                    .map_err(huffman_error)?;
                if !matches!(type_, ChunkType::Snapshot | ChunkType::SnapshotDelta) {
                    let _u = Unpacker::new(huff_data);
                    // No need for manual var-int unpacking
                }
                Ok(Some(match type_ {
                    ChunkType::Unknown => return self.read_chunk(warn, data),
                    ChunkType::Snapshot => Chunk::Snapshot(&self.buffer2),
                    ChunkType::SnapshotDelta => Chunk::SnapshotDelta(&self.buffer2),
                    ChunkType::Message => Chunk::Message(&self.buffer1),
                }))
            }
        }
    }
}
