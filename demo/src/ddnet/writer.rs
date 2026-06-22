use crate::format;
use arrayvec::ArrayVec;
use binrw::BinWrite;
use libtw2_buffer as buffer;
use libtw2_common::digest::Sha256;
use libtw2_common::num::Cast;
use libtw2_gamenet_common::traits::MessageExt as _;
use libtw2_gamenet_common::traits::Protocol;
use libtw2_gamenet_common::traits::SnapObj as _;
use libtw2_huffman::instances::TEEWORLDS as HUFFMAN;
use libtw2_packer::with_packer;
use libtw2_snapshot::snap;
use libtw2_snapshot::Delta;
use libtw2_snapshot::Snap;
use std::io;
use std::marker::PhantomData;
use std::mem;
use thiserror::Error;

const WRITER_VERSION: format::Version = format::Version::V5;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error(transparent)]
    BinRw(#[from] binrw::Error),
    #[error(transparent)]
    Inner(crate::WriteError),
    #[error("Snap creation - {0:?}")]
    SnapBuilder(snap::BuilderError),
    #[error("Tick decreased or is negative")]
    TooLowTickNumber,
    #[error("Snap data does not fit into buffer")]
    TooLargeSnap,
    #[error("Net message data does not fit into buffer")]
    TooLongNetMsg,
}

impl From<crate::WriteError> for WriteError {
    fn from(value: crate::WriteError) -> Self {
        Self::Inner(value)
    }
}

impl From<snap::BuilderError> for WriteError {
    fn from(value: snap::BuilderError) -> Self {
        Self::SnapBuilder(value)
    }
}

pub(crate) trait SeekableWrite: io::Write + io::Seek {}
impl<T: io::Write + io::Seek> SeekableWrite for T {}

/// DDNet demo writer.
///
/// Automatically writes snapshot deltas.
pub struct DemoWriter<'a, P: for<'p> Protocol<'p>> {
    file: Box<dyn SeekableWrite + 'a>,
    prev_tick: Option<i32>,
    huffman: ArrayVec<[u8; format::MAX_SNAPSHOT_SIZE]>,
    // To verify the monotonic increase
    last_tick: i32,
    // Stores the last tick, in which a snapshot was written.
    last_keyframe: Option<i32>,
    snap: Snap,
    builder: snap::Builder,
    delta: Delta,
    buf: arrayvec::ArrayVec<[u8; format::MAX_SNAPSHOT_SIZE]>,
    i32_buf: Vec<i32>,
    protocol: PhantomData<P>,
}

impl<'a, P: for<'p> Protocol<'p>> DemoWriter<'a, P> {
    pub fn new<T: io::Write + io::Seek + 'a>(
        mut file: T,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Option<Sha256>,
        map_crc: u32,
        kind: crate::DemoKind,
        length: i32,
        timestamp: &[u8],
        map: &[u8],
    ) -> Result<Self, WriteError> {
        let version = if map_sha256.is_some() {
            format::Version::V6Ddnet
        } else {
            format::Version::V5
        };
        version.write(&mut file)?;
        let header = format::Header {
            net_version: format::CappedString::from_raw(net_version),
            map_name: format::CappedString::from_raw(map_name),
            map_size: map.len().assert_i32(),
            map_crc: map_crc,
            kind: kind,
            length,
            timestamp: format::CappedString::from_raw(timestamp),
        };
        header.write(&mut file)?;
        format::TimelineMarkers {
            amount: 0,
            markers: [0; 64],
        }
        .write(&mut file)?;
        if let Some(sha256) = map_sha256 {
            format::MapSha256::new(sha256).write_le(&mut file)?;
        }
        map.write(&mut file)?;

        Ok(Self {
            file: Box::new(file),
            prev_tick: None,
            huffman: ArrayVec::new(),
            last_tick: -1,
            last_keyframe: None,
            snap: Snap::default(),
            delta: Delta::default(),
            builder: snap::Builder::default(),
            buf: arrayvec::ArrayVec::new(),
            i32_buf: Vec::new(),
            protocol: PhantomData,
        })
    }

    pub fn write_snap<'b, T: Iterator<Item = (&'b P::SnapObj, u16)>>(
        &mut self,
        tick: i32,
        items: T,
    ) -> Result<(), WriteError> {
        // Verify that the tick number is strictly increasing.
        if tick < self.last_tick {
            return Err(WriteError::TooLowTickNumber);
        }
        // We write a keyframe at the start, and another keyframe every 5 seconds.
        // We assume a tickrate of 50 ticks per second.
        let is_keyframe = match self.last_keyframe {
            None => true,
            Some(last_keyframe) => tick - last_keyframe > 250,
        };

        // Build snap.
        for (item, id) in items {
            self.builder
                .add_item(item.obj_type_id(), id, item.encode())?;
        }

        let old_snap = mem::take(&mut self.snap);
        let new_snap = mem::take(&mut self.builder).finish();

        let tm = format::TickMarker::new(tick, self.prev_tick, is_keyframe, WRITER_VERSION);
        format::ChunkHeader::Tick {
            marker: tm,
            keyframe: is_keyframe,
        }
        .write(&mut self.file, WRITER_VERSION)?;
        self.prev_tick = Some(tick);

        if is_keyframe {
            let keys = &mut self.i32_buf;
            self.buf.clear();
            with_packer(&mut self.buf, |p| new_snap.write(keys, p))
                .map_err(|_| WriteError::TooLargeSnap)?;
            self.write_chunk_impl(format::DataKind::Snapshot)?;
        } else {
            self.delta.create(&old_snap, &new_snap);
            let delta = &self.delta;
            self.buf.clear();
            with_packer(&mut self.buf, |p| delta.write(P::obj_size, p))
                .map_err(|_| WriteError::TooLargeSnap)?;
            self.write_chunk_impl(format::DataKind::SnapshotDelta)?;
        }

        // Snap deltas always rely on the snap of the last tick in the demo.
        // They don't rely on the last keyframe.
        // For that, we always need to store the newest snap.
        self.snap = new_snap;
        self.builder = old_snap.recycle();
        self.buf.clear();
        self.last_tick = tick;
        if is_keyframe {
            self.last_keyframe = Some(tick);
        }
        Ok(())
    }
    pub fn write_msg(&mut self, msg: &<P as Protocol<'_>>::Game) -> Result<(), WriteError> {
        // We reuse the huffman buffer as we need to do twint decoding twice.
        self.huffman.clear();
        with_packer(&mut self.huffman, |p| msg.encode(p)).map_err(|_| WriteError::TooLongNetMsg)?;
        self.buf.clear();
        with_packer(
            &mut self.buf,
            |mut p| -> Result<(), buffer::CapacityError> {
                for b in self.huffman.chunks(4) {
                    // Get or return 0.
                    fn g(bytes: &[u8], idx: usize) -> u8 {
                        bytes.get(idx).cloned().unwrap_or(0)
                    }
                    p.write_int(i32::from_le_bytes([g(b, 0), g(b, 1), g(b, 2), g(b, 3)]))?;
                }
                Ok(())
            },
        )
        .expect("overlong message");
        self.write_chunk_impl(format::DataKind::Message)?;
        Ok(())
    }
    fn write_chunk_impl(&mut self, kind: format::DataKind) -> Result<(), WriteError> {
        let data = &self.buf;
        self.huffman.clear();
        HUFFMAN
            .compress(data, &mut self.huffman)
            .expect("too long compression");
        format::ChunkHeader::Data {
            kind,
            size: self.huffman.len().assert_u16(),
        }
        .write(&mut self.file, format::Version::V5)?;
        self.file
            .write_all(&self.huffman)
            .map_err(binrw::Error::Io)?;
        Ok(())
    }
}
