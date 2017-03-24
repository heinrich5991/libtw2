use bitmagic::CallbackNewExt;
use format;
use warn::Warn;

pub trait CallbackNew {
    type Error;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
    fn ensure_filesize(&mut self, filesize: u32) -> Result<Result<(), ()>, Self::Error>;
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

pub struct Reader {
    header_version: format::HeaderVersion,
    header: format::Header,
    timeline_markers: format::TimelineMarkers,
}

impl Reader {
    pub fn new<W, CB>(warn: &mut W, cb: &mut CB) -> Result<Reader, Error<CB::Error>>
        where W: Warn<format::Warning>,
              CB: CallbackNew,
    {
        let header_version: format::HeaderVersionPacked =
            cb.read_raw().on_eof(format::Error::TooShortHeaderVersion)?;
        let header_version = header_version.unpack()?;
        let version_byte = header_version.version.to_u8();
        match header_version.version {
            format::Version::V4 | format::Version::V5 => {},
            _ => return Err(format::Error::UnknownVersion(version_byte).into()),
        }
        let header: format::HeaderPacked =
            cb.read_raw().on_eof(format::Error::TooShortHeader)?;
        let header = header.unpack(warn)?;
        let timeline_markers: format::TimelineMarkersPacked =
            cb.read_raw().on_eof(format::Error::TooShortTimelineMarkers)?;
        let timeline_markers = timeline_markers.unpack(warn)?;
        Ok(Reader {
            header_version: header_version,
            header: header,
            timeline_markers: timeline_markers,
        })
    }
    pub fn version(&self) -> format::Version {
        self.header_version.version
    }
    pub fn net_version(&self) -> &[u8] {
        &self.header.net_version
    }
    pub fn map_name(&self) -> &[u8] {
        &self.header.map_name
    }
    pub fn map_size(&self) -> u32 {
        self.header.map_size
    }
    pub fn map_crc(&self) -> u32 {
        self.header.map_crc
    }
    pub fn timestamp(&self) -> &[u8] {
        &self.header.timestamp
    }
    pub fn timeline_markers(&self) -> &[format::Tick] {
        &self.timeline_markers.timeline_markers
    }
}
