use arrayvec::ArrayVec;
use buffer::Buffer;
use buffer::with_buffer;
use buffer;
use common::num::Cast;
use common::num::LeI32;
use huffman::instances::TEEWORLDS as HUFFMAN;
use huffman;
use packer::Unpacker;
use packer;
use warn::Warn;
use warn;

use bitmagic::CallbackExt;
use format::MAX_SNAPSHOT_SIZE;
use format::Warning;
use format;

fn huffman_error(e: huffman::DecompressionError) -> format::Error {
    use huffman::DecompressionError::*;
    match e {
        Capacity(_) => format::Error::HuffmanDecompressionTooLong,
        InvalidInput => format::Error::HuffmanDecompressionError,
    }
}

fn packer_warning(w: packer::Warning) -> format::Warning {
    use packer::Warning::*;
    match w {
        OverlongIntEncoding => Warning::IntDecompressionOverlongEncoding,
        NonZeroIntPadding => Warning::IntDecompressionNonZeroPadding,
        ExcessData => unreachable!(),
    }
}

fn packer_error(_: packer::UnexpectedEnd) -> format::Error {
    format::Error::IntDecompressionError
}

fn buffer_error(_: buffer::CapacityError) -> format::Error {
    format::Error::IntDecompressionTooLong
}

pub trait Callback {
    type Error;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
    fn skip(&mut self, num_bytes: u32) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum CallbackReadError<CE> {
    Cb(CE),
    EndOfFile,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Error<CE> {
    Demo(format::Error),
    Cb(CE),
}

impl<CE> From<format::Error> for Error<CE> {
    fn from(err: format::Error) -> Error<CE> {
        Error::Demo(err)
    }
}

pub struct WrapCallbackError<CE>(pub CE);
impl<CE> From<WrapCallbackError<CE>> for Error<CE> {
    fn from(err: WrapCallbackError<CE>) -> Error<CE> {
        let WrapCallbackError(err) = err;
        Error::Cb(err)
    }
}
impl<CE> From<WrapCallbackError<CE>> for CallbackReadError<CE> {
    fn from(err: WrapCallbackError<CE>) -> CallbackReadError<CE> {
        let WrapCallbackError(err) = err;
        CallbackReadError::Cb(err)
    }
}
pub trait ResultExt {
    type ResultWrapped;
    fn wrap(self) -> Self::ResultWrapped;
}
impl<T, CE> ResultExt for Result<T, CE> {
    type ResultWrapped = Result<T, WrapCallbackError<CE>>;
    fn wrap(self) -> Result<T, WrapCallbackError<CE>> {
        self.map_err(WrapCallbackError)
    }
}

pub trait CallbackReadResultExt {
    type Result;
    fn on_eof(self, demo_err: format::Error) -> Self::Result;
}
impl<T, CE> CallbackReadResultExt for Result<T, CallbackReadError<CE>> {
    type Result = Result<T, Error<CE>>;
    fn on_eof(self, demo_err: format::Error) -> Result<T, Error<CE>> {
        self.map_err(|e| match e {
            CallbackReadError::Cb(err) => Error::Cb(err),
            CallbackReadError::EndOfFile => From::from(demo_err),
        })
    }
}

struct Inner {
    version: format::Version,
    header: format::Header,
    timeline_markers: format::TimelineMarkers,
    current_tick: Option<format::Tick>,
    buffer1: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
    buffer2: ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>,
}

pub struct Reader {
    i: Inner,
    error_encountered: bool,
}

impl Reader {
    pub fn new<W, CB>(warn: &mut W, cb: &mut CB) -> Result<Reader, Error<CB::Error>>
        where W: Warn<Warning>,
              CB: Callback,
    {
        let header_version: format::HeaderVersionPacked =
            cb.read_raw().on_eof(format::Error::TooShortHeaderVersion)?;
        let header_version = header_version.unpack()?;
        let version = header_version.version;
        let version_byte = version.to_u8();
        match version {
            format::Version::V4 | format::Version::V5 | format::Version::V6Ddnet => {},
            _ => return Err(format::Error::UnknownVersion(version_byte).into()),
        }
        let header: format::HeaderPacked =
            cb.read_raw().on_eof(format::Error::TooShortHeader)?;
        let header = header.unpack(warn)?;
        let timeline_markers: format::TimelineMarkersPacked =
            cb.read_raw().on_eof(format::Error::TooShortTimelineMarkers)?;
        let timeline_markers = timeline_markers.unpack(warn)?;
        if version == format::Version::V6Ddnet {
            cb.skip(48).wrap()?;
        }
        cb.skip(header.map_size).wrap()?;

        Ok(Reader {
            i: Inner {
                version: version,
                header: header,
                timeline_markers: timeline_markers,
                current_tick: None,
                buffer1: ArrayVec::new(),
                buffer2: ArrayVec::new(),
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
    pub fn read_chunk<'a, W, CB>(&'a mut self, warn: &mut W, cb: &mut CB)
        -> Result<Option<format::Chunk<'a>>, Error<CB::Error>>
        where W: Warn<Warning>,
              CB: Callback,
    {
        assert!(!self.error_encountered, "reading new chunks isn't supported after errors");
        let result = self.i.read_chunk(warn, cb);
        if let Err(_) = result {
            self.error_encountered = true;
        }
        result
    }
}

impl Inner {
    pub fn read_chunk<'a, W, CB>(&'a mut self, warn: &mut W, cb: &mut CB)
        -> Result<Option<format::Chunk<'a>>, Error<CB::Error>>
        where W: Warn<Warning>,
              CB: Callback,
    {
        use format::Chunk;
        use format::ChunkHeader;
        use format::ChunkType;
        use format::Tickmarker;

        let chunk_header;
        if let Some(ch) = ChunkHeader::read(warn, cb, self.version)? {
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
                {
                    self.buffer1.clear();
                    let result = cb.read_buffer(self.buffer1.cap_at(size.usize())).wrap()?;
                    if result.len() != size.usize() {
                        return Err(format::Error::TooShort.into());
                    }
                }
                self.buffer2.clear();
                HUFFMAN.decompress(&self.buffer1, &mut self.buffer2)
                    .map_err(huffman_error)?;
                if !matches!(type_, ChunkType::Snapshot | ChunkType::SnapshotDelta) {
                    self.buffer1.clear();
                    let mut u = Unpacker::new(&self.buffer2);
                    with_buffer(&mut self.buffer1, |mut buf| -> Result<(), format::Error> {
                        while !u.is_empty() {
                            let i = u.read_int(&mut warn::rev_map(warn, packer_warning))
                                .map_err(packer_error)?;
                            let packed = LeI32::from_i32(i);
                            buf.write(packed.as_bytes()).map_err(buffer_error)?;
                        }
                        Ok(())
                    })?;
                }
                Ok(Some(match type_ {
                    ChunkType::Unknown => return self.read_chunk(warn, cb),
                    ChunkType::Snapshot => Chunk::Snapshot(&self.buffer2),
                    ChunkType::SnapshotDelta => Chunk::SnapshotDelta(&self.buffer2),
                    ChunkType::Message => Chunk::Message(&self.buffer1),
                }))
            }
        }
    }
}
