use common::io::ReadExt;
use common::num::Cast;
use std::fs::File;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::io;
use std::path::Path;
use warn::Warn;

use format::Warning;
use format;
use raw::Callback;
use raw;

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
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<raw::Error<io::Error>> for Error {
    fn from(err: raw::Error<io::Error>) -> Error {
        match err {
            raw::Error::Demo(e) => Error::Demo(e),
            raw::Error::Cb(e) => Error::Io(e),
        }
    }
}

impl From<raw::WrapCallbackError<io::Error>> for Error {
    fn from(err: raw::WrapCallbackError<io::Error>) -> Error {
        let raw::WrapCallbackError(e) = err;
        Error::Io(e)
    }
}

struct CallbackData {
    file: BufReader<File>,
}

pub struct Reader {
    callback_data: CallbackData,
    raw: raw::Reader,
}

impl Reader {
    fn new_impl<W: Warn<Warning>>(warn: &mut W, file: File)
        -> Result<Reader, Error>
    {
        let mut callback_data = CallbackData {
            file: BufReader::new(file),
        };
        let raw = raw::Reader::new(warn, &mut callback_data)?;
        Ok(Reader {
            callback_data: callback_data,
            raw: raw,
        })
    }
    pub fn new<W: Warn<Warning>>(warn: &mut W, file: File)
        -> Result<Reader, Error>
    {
        Reader::new_impl(warn, file)
    }
    pub fn open<W, P>(warn: &mut W, path: P) -> Result<Reader, Error>
        where W: Warn<Warning>,
              P: AsRef<Path>,
    {
        fn inner<W>(warn: &mut W, path: &Path) -> Result<Reader, Error>
            where W: Warn<Warning>,
        {
            Reader::new_impl(warn, File::open(path)?)
        }
        inner(warn, path.as_ref())
    }
    pub fn version(&self) -> format::Version {
        self.raw.version()
    }
    pub fn net_version(&self) -> &[u8] {
        self.raw.net_version()
    }
    pub fn map_name(&self) -> &[u8] {
        self.raw.map_name()
    }
    pub fn map_size(&self) -> u32 {
        self.raw.map_size()
    }
    pub fn map_crc(&self) -> u32 {
        self.raw.map_crc()
    }
    pub fn timestamp(&self) -> &[u8] {
        self.raw.timestamp()
    }
    pub fn timeline_markers(&self) -> &[format::Tick] {
        self.raw.timeline_markers()
    }
    pub fn read_chunk<'a, W>(&'a mut self, warn: &mut W)
        -> Result<Option<format::Chunk<'a>>, Error>
        where W: Warn<Warning>,
    {
        Ok(self.raw.read_chunk(warn, &mut self.callback_data)?)
    }
}

impl Callback for CallbackData {
    type Error = io::Error;
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.file.read_retry(buffer)
    }
    fn skip(&mut self, num_bytes: u32) -> io::Result<()> {
        self.file.seek(SeekFrom::Current(num_bytes.i64())).map(|_| ())
    }
}
