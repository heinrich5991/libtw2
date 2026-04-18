use crate::format;
use libtw2_common::digest::Sha256;
use libtw2_gamenet_common::traits::MessageExt as _;
use libtw2_gamenet_common::traits::Protocol;
use libtw2_gamenet_common::traits::SnapObj as _;
use libtw2_packer::with_packer;
use libtw2_snapshot::snap;
use libtw2_snapshot::Delta;
use libtw2_snapshot::Snap;
use std::io;
use std::marker::PhantomData;
use std::mem;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
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

/// DDNet demo writer.
///
/// Automatically writes snapshot deltas.
pub struct DemoWriter<'a, P: for<'p> Protocol<'p>> {
    inner: crate::Writer<'a>,
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
        file: T,
        net_version: &[u8],
        map_name: &[u8],
        map_sha256: Option<Sha256>,
        map_crc: u32,
        kind: crate::DemoKind,
        length: i32,
        timestamp: &[u8],
        map: &[u8],
    ) -> Result<Self, WriteError> {
        let raw = crate::Writer::new(
            file,
            net_version,
            map_name,
            map_sha256,
            map_crc,
            kind,
            length,
            timestamp,
            map,
        )?;

        Ok(Self {
            inner: raw,
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

        self.inner.write_tick(is_keyframe, tick)?;
        if is_keyframe {
            let keys = &mut self.i32_buf;
            with_packer(&mut self.buf, |p| new_snap.write(keys, p))
                .map_err(|_| WriteError::TooLargeSnap)?;
            self.inner.write_snapshot(&self.buf)?;
        } else {
            self.delta.create(&old_snap, &new_snap);
            let delta = &self.delta;
            with_packer(&mut self.buf, |p| delta.write(P::obj_size, p))
                .map_err(|_| WriteError::TooLargeSnap)?;
            self.inner.write_snapshot_delta(&self.buf)?;
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
        with_packer(&mut self.buf, |p| msg.encode(p)).map_err(|_| WriteError::TooLongNetMsg)?;
        self.inner.write_message(self.buf.as_slice())?;
        self.buf.clear();
        Ok(())
    }
}
