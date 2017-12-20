use std::fs::File;
use std::io::Read;
use std::io;
use std::ops;
use std::path::Path;

use format::Header;
use format::item::INPUT_LEN;
use format;
use raw::Callback;
use raw;

pub use raw::Buffer;
pub use raw::Item;
pub use raw::Pos;

#[derive(Debug)]
pub enum Error {
    Teehistorian(format::Error),
    Io(io::Error),
}

impl From<format::Error> for Error {
    fn from(err: format::Error) -> Error {
        Error::Teehistorian(err)
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
            raw::Error::Teehistorian(e) => Error::Teehistorian(e),
            raw::Error::Cb(e) => Error::Io(e),
        }
    }
}

struct CallbackData {
    file: File,
}

pub struct Reader {
    callback_data: CallbackData,
    raw: raw::Reader,
}

impl Reader {
    fn new_impl<'a>(file: File, buffer: &'a mut Buffer)
        -> Result<(Header<'a>, Reader), Error>
    {
        let mut callback_data = CallbackData {
            file: file,
        };
        let (header, raw) = raw::Reader::new(&mut callback_data, buffer)?;
        Ok((header, Reader {
            callback_data: callback_data,
            raw: raw,
        }))
    }
    pub fn new<'a>(file: File, buffer: &'a mut Buffer)
        -> Result<(Header<'a>, Reader), Error>
    {
        Reader::new_impl(file, buffer)
    }
    pub fn open<'a, P: AsRef<Path>>(path: P, buffer: &'a mut Buffer)
        -> Result<(Header, Reader), Error>
    {
        fn inner<'a>(path: &Path, buffer: &'a mut Buffer)
            -> Result<(Header<'a>, Reader), Error>
        {
            Reader::new_impl(File::open(path)?, buffer)
        }
        inner(path.as_ref(), buffer)
    }
    pub fn read<'a>(&mut self, buffer: &'a mut Buffer)
        -> Result<Option<Item<'a>>, Error>
    {
        Ok(self.raw.read(&mut self.callback_data, buffer)?)
    }
    pub fn player_pos(&self, cid: i32) -> Option<Pos> {
        self.raw.player_pos(cid)
    }
    pub fn input(&self, cid: i32) -> Option<[i32; INPUT_LEN]> {
        self.raw.input(cid)
    }
    pub fn cids(&self) -> ops::Range<i32> {
        self.raw.cids()
    }
}

impl Callback for CallbackData {
    type Error = io::Error;
    fn read_at_most(&mut self, buffer: &mut [u8]) -> io::Result<Option<usize>> {
        match self.file.read(buffer) {
            Ok(0) => Ok(None),
            Ok(read) => Ok(Some(read)),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => Ok(Some(0)),
            Err(e) => Err(e),
        }
    }
}
