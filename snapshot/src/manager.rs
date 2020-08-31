use Delta;
use DeltaReader;
use DeltaReceiver;
use ReceivedDelta;
use Snap;
use Storage;
use format;
use gamenet::msg::system;
use packer::Unpacker;
use receiver;
use snap;
use storage;
use warn::Warn;
use warn::wrap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Receiver(receiver::Error),
    Snap(snap::Error),
    Storage(storage::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Warning {
    Receiver(receiver::Warning),
    Snap(format::Warning),
    Storage(storage::Warning),
}

impl From<receiver::Error> for Error {
    fn from(err: receiver::Error) -> Error {
        Error::Receiver(err)
    }
}

impl From<snap::Error> for Error {
    fn from(err: snap::Error) -> Error {
        Error::Snap(err)
    }
}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Error {
        Error::Storage(err)
    }
}

impl From<receiver::Warning> for Warning {
    fn from(w: receiver::Warning) -> Warning {
        Warning::Receiver(w)
    }
}

impl From<format::Warning> for Warning {
    fn from(w: format::Warning) -> Warning {
        Warning::Snap(w)
    }
}

impl From<storage::Warning> for Warning {
    fn from(w: storage::Warning) -> Warning {
        Warning::Storage(w)
    }
}

#[derive(Clone, Default)]
struct ManagerInner {
    temp_delta: Delta,
    reader: DeltaReader,
    storage: Storage,
}

#[derive(Clone, Default)]
pub struct Manager {
    inner: ManagerInner,
    receiver: DeltaReceiver,
}

impl Manager {
    pub fn new() -> Manager {
        Default::default()
    }
    pub fn reset(&mut self) {
        self.inner.storage.reset();
        self.receiver.reset();
    }
    pub fn ack_tick(&self) -> Option<i32> {
        self.inner.storage.ack_tick()
    }
    pub fn snap_empty<W, O>(&mut self, warn: &mut W, object_size: O, snap: system::SnapEmpty)
        -> Result<Option<&Snap>, Error>
        where W: Warn<Warning>,
              O: FnMut(u16) -> Option<u32>,
    {
        let res = self.receiver.snap_empty(wrap(warn), snap);
        self.inner.handle_msg(warn, object_size, res)
    }
    pub fn snap_single<W, O>(&mut self, warn: &mut W, object_size: O, snap: system::SnapSingle)
        -> Result<Option<&Snap>, Error>
        where W: Warn<Warning>,
              O: FnMut(u16) -> Option<u32>,
    {
        let res = self.receiver.snap_single(wrap(warn), snap);
        self.inner.handle_msg(warn, object_size, res)
    }
    pub fn snap<W, O>(&mut self, warn: &mut W, object_size: O, snap: system::Snap)
        -> Result<Option<&Snap>, Error>
        where W: Warn<Warning>,
              O: FnMut(u16) -> Option<u32>,
    {
        let res = self.receiver.snap(wrap(warn), snap);
        self.inner.handle_msg(warn, object_size, res)
    }
}

impl ManagerInner {
    fn handle_msg<W, O>(&mut self, warn: &mut W, object_size: O, res: Result<Option<ReceivedDelta>, receiver::Error>)
        -> Result<Option<&Snap>, Error>
        where W: Warn<Warning>,
              O: FnMut(u16) -> Option<u32>,
    {
        Ok(match res? {
            Some(delta) => Some(self.add_delta(warn, object_size, delta)?),
            None => None,
        })
    }
    fn add_delta<W, O>(&mut self, warn: &mut W, object_size: O, delta: ReceivedDelta)
        -> Result<&Snap, Error>
        where W: Warn<Warning>,
              O: FnMut(u16) -> Option<u32>,
    {
        let crc = delta.data_and_crc.map(|d| d.1);
        if let Some((data, _)) = delta.data_and_crc {
            self.reader.read(wrap(warn), &mut self.temp_delta, object_size, &mut Unpacker::new(data))?;
        } else {
            self.temp_delta.clear();
        }
        Ok(self.storage.add_delta(wrap(warn), crc, delta.delta_tick, delta.tick, &self.temp_delta)?)
    }
}
