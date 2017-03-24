use common::io::ReadExt;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::path::Path;
use warn::Warn;

use format;
use raw::CallbackNew;
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

struct CallbackDataNew {
    file: BufReader<File>,
}

pub struct Reader {
    raw: raw::Reader,
}

impl Reader {
    fn new_impl<W: Warn<format::Warning>>(warn: &mut W, file: File)
        -> Result<Reader, Error>
    {
        let mut callback_data_new = CallbackDataNew {
            file: BufReader::new(file),
        };
        let raw = raw::Reader::new(warn, &mut callback_data_new)?;
        Ok(Reader {
            raw: raw,
        })
    }
    pub fn new<W: Warn<format::Warning>>(warn: &mut W, file: File)
        -> Result<Reader, Error>
    {
        Reader::new_impl(warn, file)
    }
    pub fn open<W, P>(warn: &mut W, path: P) -> Result<Reader, Error>
        where W: Warn<format::Warning>,
              P: AsRef<Path>,
    {
        fn inner<W>(warn: &mut W, path: &Path) -> Result<Reader, Error>
            where W: Warn<format::Warning>,
        {
            Reader::new_impl(warn, File::open(path)?)
        }
        inner(warn, path.as_ref())
    }
    pub fn timestamp(&self) -> &[u8] {
        self.raw.timestamp()
    }
    pub fn timeline_markers(&self) -> &[format::Tick] {
        self.raw.timeline_markers()
    }
}

impl CallbackNew for CallbackDataNew {
    type Error = io::Error;
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.file.read_retry(buffer)
    }
    fn ensure_filesize(&mut self, filesize: u32) -> io::Result<Result<(), ()>> {
        unimplemented!();
    }
}
