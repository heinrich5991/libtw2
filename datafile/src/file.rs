use num::ToPrimitive;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io;

use raw::Callback;
use raw::CallbackError;
use raw::DataCallback;
use raw::DatafileError;
use raw;

#[derive(Debug)]
pub enum Error {
    Df(DatafileError),
    Io(io::Error),
}

struct CallbackData {
    file: File,
    datafile_start: u64,
    seek_base: Option<u64>,
    last_error: Option<io::Error>,
}

pub struct DatafileReaderFile {
    callback_data: CallbackData,
    raw: raw::Reader,
}

impl From<DatafileError> for Error {
    fn from(err: DatafileError) -> Error {
        Error::Df(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl DatafileReaderFile {
    pub fn new(file: File) -> Result<DatafileReaderFile,Error> {
        let mut file = file;
        let datafile_start = try!(file.seek(SeekFrom::Current(0)));
        let mut callback_data = CallbackData {
            file: file,
            datafile_start: datafile_start,
            seek_base: None,
            last_error: None,
        };
        let result = raw::Reader::new(&mut callback_data);
        let raw = try!(callback_data.retrieve_error(result));
        Ok(DatafileReaderFile {
            callback_data: callback_data,
            raw: raw,
        })
    }
    pub fn debug_dump(&mut self) -> Result<(),Error> {
        let result = self.raw.debug_dump(&mut self.callback_data);
        self.callback_data.retrieve_error(result)
    }
}

impl CallbackData {
    fn store_error<T>(&mut self, result: Result<T,io::Error>) -> Result<T,CallbackError> {
        assert!(self.last_error.is_none());
        result.map_err(|e| {
            self.last_error = Some(e);
            CallbackError
        })
    }
    fn retrieve_error<T>(&mut self, result: Result<T,raw::Error>) -> Result<T,Error> {
        result.map_err(|e| {
            match e {
                raw::Error::Df(err) => Error::Df(err),
                raw::Error::Cb(CallbackError) => {
                    Error::Io(self.last_error.take().unwrap())
                }
            }
        })
    }
}

pub struct WrapVecU8 {
    inner: Vec<u8>,
}

impl DataCallback for WrapVecU8 {
    fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }
}

impl Callback for CallbackData {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize,CallbackError> {
        let result = self.file.read(buffer);
        self.store_error(result)
    }
    fn seek_read(&mut self, start: u32, buffer: &mut [u8]) -> Result<usize,CallbackError> {
        let result = self.file.seek(SeekFrom::Start(self.seek_base.unwrap()));
        try!(self.store_error(result));
        let result = self.file.seek(SeekFrom::Current(start.to_i64().unwrap()));
        try!(self.store_error(result));
        let result = self.file.read(buffer);
        Ok(try!(self.store_error(result)))
    }
    fn set_seek_base(&mut self) -> Result<(),CallbackError> {
        let result = self.file.seek(SeekFrom::Current(0));
        self.seek_base = Some(try!(self.store_error(result)));
        Ok(())
    }
    fn ensure_filesize(&mut self, filesize: u32) -> Result<Result<(),()>,CallbackError> {
        let result = self.file.seek(SeekFrom::End(0));
        let actual = try!(self.store_error(result));
        Ok(if actual.checked_sub(self.datafile_start).unwrap() > filesize.to_u64().unwrap() {
            Ok(())
        } else {
            Err(())
        })
    }
    type Data = WrapVecU8;
    fn alloc_data(&mut self, length: usize) -> Result<WrapVecU8,CallbackError> {
        let mut vec = Vec::with_capacity(length);
        unsafe { vec.set_len(length); }
        Ok(WrapVecU8 { inner: vec })
    }
}
