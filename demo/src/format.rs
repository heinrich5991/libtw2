use arrayvec::ArrayVec;
use binrw::BinRead;
use binrw::BinWrite;
use common::digest::Sha256;
use common::num::Cast;
use std::convert::TryFrom;
use std::io;
use warn;
use warn::Warn;

pub const MAX_SNAPSHOT_SIZE: usize = 65536;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Warning {
    NonAbsoluteTickmarkerTick,
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

#[derive(BinRead, BinWrite, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[brw(repr(u8), magic = b"TWDEMO\0")]
pub enum Version {
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6Ddnet = 6,
}

impl Version {
    fn max_tick_delta(self) -> u8 {
        match self {
            Version::V3 | Version::V4 => CHUNKTICKMASK_TICK_V3,
            Version::V5 | Version::V6Ddnet => CHUNKTICKMASK_TICK_V5,
        }
    }
}

pub enum RawChunk<'a> {
    /// Tick(keyframe, tick)
    Tick {
        tick: i32,
        keyframe: bool,
    },
    Snapshot(&'a ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>),
    SnapshotDelta(&'a ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>),
    Message(&'a ArrayVec<[u8; MAX_SNAPSHOT_SIZE]>),
    Unknown,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub(crate) struct HeaderStart {
    pub version: Version,
    pub header: Header,
    #[br(if(version >= Version::V4))]
    pub timeline_markers: TimelineMarkers,
    #[br(if(version == Version::V6Ddnet))]
    pub map_sha256: Option<MapSha256>,
    #[br(count = header.map_size)]
    pub map: Vec<u8>,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub(crate) struct Header {
    pub net_version: CappedString<64>,
    pub map_name: CappedString<64>,
    #[br(assert(map_size >= 0))]
    pub map_size: i32,
    pub map_crc: u32,
    pub kind: DemoKind,
    #[br(assert(length >= 0))]
    pub length: i32,
    pub timestamp: CappedString<20>,
}

impl Header {
    pub fn check<W: Warn<Warning>>(&self, warn: &mut W) {
        self.net_version
            .check_padding_warn(warn, Warning::WeirdNetVersion);
        self.map_name
            .check_padding_warn(warn, Warning::WeirdMapName);
        self.timestamp
            .check_padding_warn(warn, Warning::WeirdTimestamp);
    }
}

#[derive(BinRead, BinWrite, Debug)]
pub enum DemoKind {
    #[brw(magic = b"client\0\0")]
    Client,
    #[brw(magic = b"server\0\0")]
    Server,
}

#[derive(BinRead, BinWrite, Debug)]
pub(crate) struct CappedString<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> CappedString<N> {
    fn length(&self) -> usize {
        self.bytes.iter().position(|c| *c == 0).unwrap_or(N)
    }
    pub fn raw(&self) -> &[u8] {
        &self.bytes[..self.length()]
    }
    pub fn from_raw(raw: &[u8]) -> Self {
        assert!(raw.len() < N);
        let mut bytes = [0; N];
        bytes[..raw.len()].copy_from_slice(raw);
        Self { bytes: bytes }
    }
    fn check_padding_warn<W: Warn<Warning>>(&self, warn: &mut W, w: Warning) {
        if self.bytes[self.length()..].iter().any(|c| *c != 0) {
            warn.warn(w)
        }
    }
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(big)]
pub(crate) struct TimelineMarkers {
    #[br(assert(amount >= 0), assert(amount <= 64))]
    pub amount: i32,
    pub markers: [i32; 64],
}

impl Default for TimelineMarkers {
    fn default() -> Self {
        Self {
            amount: 0,
            markers: [0; 64],
        }
    }
}

impl TimelineMarkers {
    pub(crate) fn markers(&self) -> &[i32] {
        &self.markers[..self.amount.assert_usize()]
    }

    pub(crate) fn check<W: Warn<Warning>>(&self, warn: &mut W) {
        if self.markers[self.amount.assert_usize()..64]
            .iter()
            .any(|n| *n != 0)
        {
            warn.warn(Warning::WeirdTimelineMarkerPadding);
        }
        if self.markers().windows(2).any(|m| m[0] >= m[1]) {
            warn.warn(Warning::NonAbsoluteTickmarkerTick)
        }
    }
}

const SHA_256_EXTENSION: [u8; 16] = [
    0x6b, 0xe6, 0xda, 0x4a, 0xce, 0xbd, 0x38, 0x0c, 0x9b, 0x5b, 0x12, 0x89, 0xc8, 0x42, 0xd7, 0x80,
];

#[derive(Debug, Default, BinRead, BinWrite)]
pub(crate) struct MapSha256 {
    #[br(assert(_uuid == SHA_256_EXTENSION))]
    _uuid: [u8; 16],
    pub sha_256: [u8; 32],
}

impl MapSha256 {
    pub(crate) fn new(sha: Sha256) -> Self {
        Self {
            _uuid: SHA_256_EXTENSION,
            sha_256: sha.0,
        }
    }
}

const CHUNKTYPEFLAG_TICKMARKER: u8 = 0b1000_0000;

const CHUNKTICKFLAG_KEYFRAME: u8 = 0b0100_0000;
const CHUNKTICKFLAG_INLINETICK: u8 = 0b0010_0000; // Only in V5.

const CHUNKTICKMASK_TICK_V3: u8 = 0b0011_1111;
const CHUNKTICKMASK_TICK_V5: u8 = 0b0001_1111;

const CHUNKMASK_TYPE: u8 = 0b0110_0000;
const CHUNKMASK_SIZE: u8 = 0b0001_1111;
const CHUNKTYPE_UNKNOWN: u8 = 0b0000_0000;
const CHUNKTYPE_SNAPSHOT: u8 = 0b0010_0000;
const CHUNKTYPE_MESSAGE: u8 = 0b0100_0000;
const CHUNKTYPE_SNAPSHOTDELTA: u8 = 0b0110_0000;

const CHUNKSIZE_ONEBYTEFOLLOWS: u8 = 0b0001_1110;
const CHUNKSIZE_TWOBYTESFOLLOW: u8 = 0b0001_1111;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DataKind {
    Snapshot,
    Message,
    SnapshotDelta,
    Unknown,
}

#[derive(Debug, Copy, Clone)]
pub enum TickMarker {
    Delta(u8),
    Absolute(i32),
}

#[derive(Debug, Copy, Clone)]
pub enum ChunkHeader {
    Tick { marker: TickMarker, keyframe: bool },
    Data { kind: DataKind, size: u16 },
}

impl ChunkHeader {
    pub(crate) fn read<R, W>(
        data: &mut R,
        version: Version,
        warn: &mut W,
    ) -> binrw::BinResult<Option<ChunkHeader>>
    where
        W: Warn<Warning>,
        R: io::Read + io::Seek,
    {
        let flags = match u8::read(data) {
            Ok(flags) => flags,
            Err(err) => return if err.is_eof() { Ok(None) } else { Err(err) },
        };
        if flags & CHUNKTYPEFLAG_TICKMARKER != 0 {
            let keyframe = flags & CHUNKTICKFLAG_KEYFRAME != 0;
            let marker = if version >= Version::V5 {
                if flags & CHUNKTICKFLAG_INLINETICK != 0 {
                    TickMarker::Delta(flags & CHUNKTICKMASK_TICK_V5)
                } else {
                    if flags & CHUNKTICKMASK_TICK_V5 != 0 {
                        warn.warn(Warning::NonZeroTickmarkerPadding);
                    }
                    TickMarker::Absolute(i32::read_be(data)?)
                }
            } else {
                let legacy_delta = flags & CHUNKTICKMASK_TICK_V3;
                if legacy_delta == 0 {
                    // TODO: Deviating from the reference implementation here.
                    // The reference implementation differentiates the same
                    // cases, but ends up doing the same thing in the first
                    // block as in the else block. Probably intended to do this
                    // instead:
                    TickMarker::Absolute(i32::read_be(data)?)
                } else {
                    TickMarker::Delta(legacy_delta)
                }
            };
            if keyframe && matches!(marker, TickMarker::Delta(_)) {
                warn.warn(Warning::NonAbsoluteTickmarkerTick);
            }
            Ok(Some(ChunkHeader::Tick {
                marker: marker,
                keyframe: keyframe,
            }))
        } else {
            let kind = match flags & CHUNKMASK_TYPE {
                CHUNKTYPE_UNKNOWN => {
                    warn.warn(Warning::UnknownChunkType);
                    DataKind::Unknown
                }
                CHUNKTYPE_SNAPSHOT => DataKind::Snapshot,
                CHUNKTYPE_MESSAGE => DataKind::Message,
                CHUNKTYPE_SNAPSHOTDELTA => DataKind::SnapshotDelta,
                _ => unreachable!(),
            };
            let size = match flags & CHUNKMASK_SIZE {
                CHUNKSIZE_ONEBYTEFOLLOWS => {
                    let size = u8::read(data)?.u16();
                    if size < 30 {
                        warn.warn(Warning::OverlongChunkSizeEncoding);
                    }
                    size
                }
                CHUNKSIZE_TWOBYTESFOLLOW => {
                    let size = u16::read_le(data)?;
                    if size < u8::MAX.u16() {
                        warn.warn(Warning::OverlongChunkSizeEncoding);
                    }
                    size
                }
                s => u16::from(s),
            };
            Ok(Some(ChunkHeader::Data {
                kind: kind,
                size: size,
            }))
        }
    }

    pub fn write<W>(&self, file: &mut W, version: Version) -> binrw::BinResult<()>
    where
        W: io::Write + io::Seek,
    {
        assert!(version >= Version::V5);
        match *self {
            ChunkHeader::Tick {
                marker: TickMarker::Delta(dt),
                keyframe,
            } => {
                assert!(dt <= version.max_tick_delta());
                assert!(!keyframe);
                let flags: u8 = CHUNKTYPEFLAG_TICKMARKER | CHUNKTICKFLAG_INLINETICK | dt;
                flags.write(file)?;
            }
            ChunkHeader::Tick {
                marker: TickMarker::Absolute(t),
                keyframe,
            } => {
                let keyframe_flag = if keyframe { CHUNKTICKFLAG_KEYFRAME } else { 0 };
                let flags = CHUNKTYPEFLAG_TICKMARKER | keyframe_flag;
                flags.write(file)?;
                t.write_be(file)?;
            }
            ChunkHeader::Data { kind, size } => {
                let kind_flag = match kind {
                    DataKind::Snapshot => CHUNKTYPE_SNAPSHOT,
                    DataKind::Message => CHUNKTYPE_MESSAGE,
                    DataKind::SnapshotDelta => CHUNKTYPE_SNAPSHOTDELTA,
                    DataKind::Unknown => CHUNKTYPE_UNKNOWN,
                };
                if size < CHUNKSIZE_ONEBYTEFOLLOWS.u16() {
                    let flags = kind_flag | size.assert_u8();
                    flags.write(file)?;
                } else if size <= u8::MAX.u16() {
                    let flags = kind_flag | CHUNKSIZE_ONEBYTEFOLLOWS;
                    flags.write(file)?;
                    size.assert_u8().write(file)?;
                } else {
                    let flags = kind_flag | CHUNKSIZE_TWOBYTESFOLLOW;
                    flags.write(file)?;
                    size.write_le(file)?;
                }
            }
        }
        Ok(())
    }
}

impl TickMarker {
    pub fn new(tick: i32, prev_tick: Option<i32>, keyframe: bool, version: Version) -> TickMarker {
        if let Some(p) = prev_tick {
            assert!(tick > p);
            if let Some(d) = tick.checked_sub(p) {
                if !keyframe && d <= version.max_tick_delta().i32() {
                    return TickMarker::Delta((tick - p).assert_u8());
                }
            }
        }
        TickMarker::Absolute(tick)
    }
}
