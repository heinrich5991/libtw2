use arrayvec::ArrayVec;
use common::num::BeI32;
use common::num::BeU32;
use common::num::Cast;
use common::num::LeU16;
use packer::bytes_to_string;
use std::iter::FromIterator;
use std::u8;
use warn::Warn;
use warn;

use bitmagic::CallbackExt;
use bitmagic::Packed;
use raw::Callback;
use raw::CallbackReadError;
use raw::CallbackReadResultExt;
use raw;

pub const MAGIC: &'static [u8; 7] = b"TWDEMO\0";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tick(pub i32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Version {
    V3,
    V4,
    V5,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Warning {
    NonAbsoluteTickMarkerTick,
    NonIncreasingTick,
    NonIncreasingTimelineMarkers,
    NonZeroTickmarkerPadding,
    IntDecompressionOverlongEncoding,
    IntDecompressionNonZeroPadding,
    OverlongChunkSizeEncoding,
    StartingDeltaTick,
    TickOverflow,
    UnknownChunkType,
    WeirdMapName,
    WeirdNetVersion,
    WeirdTimelineMarkerPadding,
    WeirdTimestamp,
    WeirdType,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    HuffmanDecompressionError,
    HuffmanDecompressionTooLong,
    IntDecompressionError,
    IntDecompressionTooLong,
    InvalidNumTimelineMarkers,
    NegativeLength,
    NegativeMapSize,
    TooShort,
    TooShortHeader,
    TooShortHeaderVersion,
    TooShortTimelineMarkers,
    UnknownMagic([u8; 7]),
    UnknownVersion(u8),
}

impl Version {
    pub fn from_u8(v: u8) -> Result<Version, Error> {
        Ok(match v {
            3 => Version::V3,
            4 => Version::V4,
            5 => Version::V5,
            _ => return Err(Error::UnknownVersion(v)),
        })
    }
    pub fn to_u8(self) -> u8 {
        match self {
            Version::V3 => 3,
            Version::V4 => 4,
            Version::V5 => 5,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Chunk<'a> {
    Tick(Tick),
    Snapshot(&'a [u8]),
    SnapshotDelta(&'a [u8]),
    Message(&'a [u8]),
}

#[derive(Clone, Copy, Debug)]
pub struct HeaderVersion {
    pub version: Version,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HeaderVersionPacked {
    pub magic: [u8; 7],
    pub version: u8,
}
unsafe impl Packed for HeaderVersionPacked { }

impl HeaderVersionPacked {
    pub fn unpack(&self) -> Result<HeaderVersion, Error> {
        if self.magic != *MAGIC {
            return Err(Error::UnknownMagic(self.magic));
        }
        Ok(HeaderVersion {
            version: Version::from_u8(self.version)?,
        })
    }
}

#[derive(Clone, /*Copy*/)]
pub struct Header {
    pub net_version: ArrayVec<[u8; 64]>,
    pub map_name: ArrayVec<[u8; 64]>,
    pub map_size: u32,
    pub map_crc: u32,
    pub type_: ArrayVec<[u8; 8]>,
    pub length: u32,
    pub timestamp: ArrayVec<[u8; 20]>,
}

#[derive(Copy)]
#[repr(C)]
pub struct HeaderPacked {
    pub net_version: [u8; 64],
    pub map_name: [u8; 64],
    pub map_size: BeI32,
    pub map_crc: BeU32,
    pub type_: [u8; 8],
    pub length: BeI32,
    pub timestamp: [u8; 20],
}
unsafe impl Packed for HeaderPacked { }

impl Clone for HeaderPacked {
    fn clone(&self) -> HeaderPacked {
        *self
    }
}

impl HeaderPacked {
    pub fn unpack<W: Warn<Warning>>(&self, warn: &mut W) -> Result<Header, Error> {
        fn b2sw<'a, W, FI>(warn: &mut W, warning: Warning, bytes: &'a [u8]) -> FI
            where W: Warn<Warning>,
                  FI: FromIterator<u8>,
        {
            bytes_to_string(&mut warn::rev_map(warn, |_| warning), bytes)
                .iter().cloned().collect()
        }
        Ok(Header {
            net_version: b2sw(warn, Warning::WeirdNetVersion, &self.net_version),
            map_name: b2sw(warn, Warning::WeirdMapName, &self.map_name),
            map_size: self.map_size.to_i32().try_u32().ok_or(Error::NegativeMapSize)?,
            map_crc: self.map_crc.to_u32(),
            type_: b2sw(warn, Warning::WeirdType, &self.type_),
            length: self.length.to_i32().try_u32().ok_or(Error::NegativeLength)?,
            timestamp: b2sw(warn, Warning::WeirdTimestamp, &self.timestamp),
        })
    }
}

#[derive(Clone, /*Copy,*/ Debug)]
pub struct TimelineMarkers {
    pub timeline_markers: ArrayVec<[Tick; 64]>,
}

#[derive(Copy)]
#[repr(C)]
pub struct TimelineMarkersPacked {
    pub num_timeline_markers: BeI32,
    pub timeline_markers: [BeI32; 64],
}
unsafe impl Packed for TimelineMarkersPacked { }

impl Clone for TimelineMarkersPacked {
    fn clone(&self) -> TimelineMarkersPacked {
        *self
    }
}

impl TimelineMarkersPacked {
    pub fn unpack<W: Warn<Warning>>(&self, warn: &mut W)
        -> Result<TimelineMarkers, Error>
    {
        let num = self.num_timeline_markers.to_i32()
            .try_u32().ok_or(Error::InvalidNumTimelineMarkers)?.usize();
        if num > self.timeline_markers.len() {
            return Err(Error::InvalidNumTimelineMarkers);
        }
        let mut previous = None;
        let mut result = ArrayVec::new();
        let mut weird_padding = false;
        let mut nonincreasing = false;
        for (i, tm) in self.timeline_markers.iter().enumerate() {
            if i < num {
                let tick = Tick(tm.to_i32());
                if let Some(p) = previous {
                    if !nonincreasing && p >= tick {
                        nonincreasing = true;
                        warn.warn(Warning::NonIncreasingTimelineMarkers);
                    }
                }
                previous = Some(tick);
                assert!(result.push(tick).is_none());
            } else if !weird_padding && tm.to_i32() != 0 {
                weird_padding = true;
                warn.warn(Warning::WeirdTimelineMarkerPadding);
            }
        }
        Ok(TimelineMarkers {
            timeline_markers: result,
        })
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ChunkHeaderStartPacked {
    pub byte: u8,
}
unsafe impl Packed for ChunkHeaderStartPacked { }

pub const CHUNKTYPEFLAG_TICKMARKER: u8 = 0b1000_0000;

pub const CHUNKTICKFLAG_KEYFRAME: u8 = 0b0100_0000;
pub const CHUNKTICKFLAG_INLINETICK: u8 = 0b0010_0000; // Only in V5.

pub const CHUNKTICKMASK_TICK_V3: u8 = 0b0011_1111;
pub const CHUNKTICKMASK_TICK_V5: u8 = 0b0001_1111;

pub const CHUNKMASK_TYPE: u8 = 0b0110_0000;
pub const CHUNKMASK_SIZE: u8 = 0b0001_1111;
pub const CHUNKTYPE_UNKNOWN: u8 = 0b0000_0000;
pub const CHUNKTYPE_SNAPSHOT: u8 = 0b0010_0000;
pub const CHUNKTYPE_MESSAGE: u8 = 0b0100_0000;
pub const CHUNKTYPE_SNAPSHOTDELTA: u8 = 0b0110_0000;

pub const CHUNKSIZE_ONEBYTEFOLLOWS: u8 = 0b0001_1110;
pub const CHUNKSIZE_TWOBYTESFOLLOW: u8 = 0b0001_1111;

#[derive(Clone, Copy, Debug)]
pub enum TickmarkerStart {
    Delta(u8),
    TickFollows
}

#[derive(Clone, Copy, Debug)]
pub enum ChunkType {
    Unknown,
    Snapshot,
    SnapshotDelta,
    Message,
}

#[derive(Clone, Copy, Debug)]
pub enum ChunkSize {
    Size(u8),
    OneSizeByteFollows,
    TwoSizeBytesFollow,
}

#[derive(Clone, Copy, Debug)]
pub enum ChunkHeaderStart {
    /// Tickmarker(keyframe, tickmarker)
    Tickmarker(bool, TickmarkerStart),
    Chunk(ChunkType, ChunkSize),
}

impl ChunkHeaderStartPacked {
    pub fn unpack<W: Warn<Warning>>(self, warn: &mut W, version: Version)
        -> ChunkHeaderStart
    {
        self.unpack_impl(warn, version >= Version::V5)
    }
    fn unpack_impl<W: Warn<Warning>>(self, warn: &mut W, v5: bool) -> ChunkHeaderStart {
        if self.byte & CHUNKTYPEFLAG_TICKMARKER != 0 {
            let keyframe = self.byte & CHUNKTICKFLAG_KEYFRAME != 0;
            let tickmarker = if v5 {
                if self.byte & CHUNKTICKFLAG_INLINETICK != 0 {
                    TickmarkerStart::Delta(self.byte & CHUNKTICKMASK_TICK_V5)
                } else {
                    if self.byte & CHUNKTICKMASK_TICK_V5 != 0 {
                        warn.warn(Warning::NonZeroTickmarkerPadding);
                    }
                    TickmarkerStart::TickFollows
                }
            } else {
                if self.byte & CHUNKTICKMASK_TICK_V3 != 0 {
                    TickmarkerStart::Delta(self.byte & CHUNKTICKMASK_TICK_V3)
                } else {
                    // TODO: Deviating from the reference implementation here.
                    // The reference implementation differentiates the same
                    // cases, but ends up doing the same thing in the first
                    // block as in the else block. Probably intended to do this
                    // instead:
                    TickmarkerStart::TickFollows
                }
            };
            ChunkHeaderStart::Tickmarker(keyframe, tickmarker)
        } else {
            let type_ = match self.byte & CHUNKMASK_TYPE {
                CHUNKTYPE_UNKNOWN => {
                    warn.warn(Warning::UnknownChunkType);
                    ChunkType::Unknown
                },
                CHUNKTYPE_SNAPSHOT => ChunkType::Snapshot,
                CHUNKTYPE_MESSAGE => ChunkType::Message,
                CHUNKTYPE_SNAPSHOTDELTA => ChunkType::SnapshotDelta,
                _ => unreachable!(),
            };
            let size = match self.byte & CHUNKMASK_SIZE {
                CHUNKSIZE_ONEBYTEFOLLOWS => ChunkSize::OneSizeByteFollows,
                CHUNKSIZE_TWOBYTESFOLLOW => ChunkSize::TwoSizeBytesFollow,
                s => ChunkSize::Size(s),
            };
            ChunkHeaderStart::Chunk(type_, size)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Tickmarker {
    Delta(u8),
    Absolute(Tick),
}

#[derive(Clone, Copy, Debug)]
pub enum ChunkHeader {
    /// Tickmarker(keyframe, tickmarker)
    Tickmarker(bool, Tickmarker),
    /// Chunk(type_, size)
    Chunk(ChunkType, u32),
}

impl ChunkHeader {
    pub fn read<W, CB>(warn: &mut W, cb: &mut CB, version: Version)
        -> Result<Option<ChunkHeader>, raw::Error<CB::Error>>
        where W: Warn<Warning>,
              CB: Callback,
    {
        let chunk_header_start: ChunkHeaderStartPacked = match cb.read_raw() {
            Err(CallbackReadError::EndOfFile) => return Ok(None),
            Err(CallbackReadError::Cb(e)) => return Err(raw::Error::Cb(e)),
            Ok(chsp) => chsp,
        };
        let chunk_header_start = chunk_header_start.unpack(warn, version);
        ChunkHeader::read_rest(warn, cb, chunk_header_start).map(Some)
    }
    fn read_rest<W, CB>(warn: &mut W, cb: &mut CB, chs: ChunkHeaderStart)
        -> Result<ChunkHeader, raw::Error<CB::Error>>
        where W: Warn<Warning>,
              CB: Callback,
    {
        use self::ChunkHeaderStart as Chs;
        use self::ChunkSize as Cs;
        use self::TickmarkerStart as Ts;
        use self::ChunkHeader::*;
        use self::Tickmarker::*;
        Ok(match chs {
            Chs::Tickmarker(keyframe, Ts::Delta(d)) => {
                if keyframe {
                    warn.warn(Warning::NonAbsoluteTickMarkerTick);
                }
                Tickmarker(keyframe, Delta(d))
            },
            Chs::Tickmarker(keyframe, Ts::TickFollows) => {
                let tick_packed: BeI32 = cb.read_raw().on_eof(Error::TooShort)?;
                Tickmarker(keyframe, Absolute(Tick(tick_packed.to_i32())))
            },
            Chs::Chunk(type_, Cs::Size(s)) => Chunk(type_, s.u32()),
            Chs::Chunk(type_, Cs::OneSizeByteFollows) => {
                let size: u8 = cb.read_raw().on_eof(Error::TooShort)?;
                if size < 30 {
                    warn.warn(Warning::OverlongChunkSizeEncoding);
                }
                Chunk(type_, size.u32())
            },
            Chs::Chunk(type_, Cs::TwoSizeBytesFollow) => {
                let size_packed: LeU16 = cb.read_raw().on_eof(Error::TooShort)?;
                if size_packed.to_u16() <= u8::max_value().u16() {
                    warn.warn(Warning::OverlongChunkSizeEncoding);
                }
                Chunk(type_, size_packed.to_u16().u32())
            },
        })
    }
}
