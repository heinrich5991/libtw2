use common::MapIterator;
use num::ToPrimitive;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io;
use std::ops;

use format;
use raw::CallbackNew;
use raw::CallbackReadData;
use raw::DataCallback;
use raw::ItemView;
use raw::ResultExt;
use raw;

#[derive(Debug)]
pub enum Error {
    Df(format::Error),
    Io(io::Error),
}

impl From<format::Error> for Error {
    fn from(err: format::Error) -> Error {
        Error::Df(err)
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
            raw::Error::Df(e) => Error::Df(e),
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
    file: File,
    datafile_start: u64,
    seek_base: Option<u64>,
}

pub struct Reader {
    callback_data: CallbackData,
    raw: raw::Reader,
}

impl Reader {
    pub fn new(file: File) -> Result<Reader,Error> {
        let mut file = file;
        let datafile_start = try!(file.seek(SeekFrom::Current(0)).wrap());
        let mut callback_data = CallbackData {
            file: file,
            datafile_start: datafile_start,
            seek_base: None,
        };
        let raw = try!(raw::Reader::new(&mut callback_data));
        Ok(Reader { 
            callback_data: callback_data,
            raw: raw,
        })
    }
    pub fn debug_dump(&mut self) -> Result<(),Error> {
        Ok(try!(self.raw.debug_dump(&mut self.callback_data)))
    }
    pub fn version(&self) -> raw::Version {
        self.raw.version()
    }
    pub fn read_data(&mut self, index: usize) -> Result<Vec<u8>,Error> {
        Ok(try!(self.raw.read_data(&mut self.callback_data, index).map(|w| w.inner)))
    }
    pub fn item(&self, index: usize) -> ItemView {
        self.raw.item(index)
    }
    pub fn num_items(&self) -> usize {
        self.raw.num_items()
    }
    pub fn num_data(&self) -> usize {
        self.raw.num_data()
    }
    pub fn item_type_indices(&self, type_id: u16) -> ops::Range<usize> {
        self.raw.item_type_indices(type_id)
    }
    pub fn item_type(&self, index: usize) -> u16 {
        self.raw.item_type(index)
    }
    pub fn num_item_types(&self) -> usize {
        self.raw.num_item_types()
    }

    pub fn find_item(&self, type_id: u16, item_id: u16) -> Option<ItemView> {
        self.raw.find_item(type_id, item_id)
    }

    pub fn items(&self) -> raw::Items {
        self.raw.items()
    }
    pub fn item_types(&self) -> raw::ItemTypes {
        self.raw.item_types()
    }
    pub fn item_type_items(&self, type_id: u16) -> raw::ItemTypeItems {
        self.raw.item_type_items(type_id)
    }
    pub fn data_iter(&mut self) -> DataIter {
        fn map_fn(i: usize, self_: &mut &mut Reader) -> Result<Vec<u8>,Error> {
            self_.read_data(i)
        }
        let num_data = self.num_data();
        MapIterator::new(self, 0..num_data, map_fn)
    }
}

pub type DataIter<'a> = MapIterator<Result<Vec<u8>,Error>,&'a mut Reader,ops::Range<usize>>;

pub struct WrapVecU8 {
    inner: Vec<u8>,
}

impl DataCallback for WrapVecU8 {
    fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl CallbackNew for CallbackData {
    type Error = io::Error;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize,io::Error> {
        self.file.read(buffer)
    }
    fn set_seek_base(&mut self) -> Result<(),io::Error> {
        self.seek_base = Some(try!(self.file.seek(SeekFrom::Current(0))));
        Ok(())
    }
    fn ensure_filesize(&mut self, filesize: u32) -> Result<Result<(),()>,io::Error> {
        let actual = try!(self.file.seek(SeekFrom::End(0)));
        Ok(if actual.checked_sub(self.datafile_start).unwrap() >= filesize.to_u64().unwrap() {
            Ok(())
        } else {
            Err(())
        })
    }
}
impl CallbackReadData for CallbackData {
    type Error = io::Error;
    fn seek_read(&mut self, start: u32, buffer: &mut [u8]) -> Result<usize,io::Error> {
        try!(self.file.seek(SeekFrom::Start(self.seek_base.unwrap())));
        try!(self.file.seek(SeekFrom::Current(start.to_i64().unwrap())));
        self.file.read(buffer)
    }
    type Data = WrapVecU8;
    fn alloc_data(&mut self, length: usize) -> Result<WrapVecU8,io::Error> {
        let mut vec = Vec::with_capacity(length);
        unsafe { vec.set_len(length); }
        Ok(WrapVecU8 { inner: vec })
    }
}
