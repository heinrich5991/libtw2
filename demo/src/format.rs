use arrayvec::ArrayVec;
use bitmagic::Packed;
use common::num::BeI32;
use common::num::BeU32;
use common::num::Cast;
use packer::bytes_to_string;
use std::iter::FromIterator;
use warn::Warn;
use warn;

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
    NonIncreasingTimelineMarkers,
    WeirdMapName,
    WeirdNetVersion,
    WeirdTimelineMarkerPadding,
    WeirdTimestamp,
    WeirdType,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    InvalidNumTimelineMarkers,
    NegativeLength,
    NegativeMapSize,
    TooShortHeaderVersion,
    TooShortHeader,
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
pub struct ChunkHeaderPacked {
    pub type_: u8,
}
