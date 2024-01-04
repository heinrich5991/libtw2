use common::io::seek_overflow;
use common::io::FileExt;
use common::io::ReadExt;
use common::num::Cast;
use common::MapIterator;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops;
use std::path::Path;

use format;
use format::ItemView;
use raw;
use raw::CallbackError;
use raw::CallbackNew;
use raw::CallbackReadData;

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

struct CallbackDataNew {
    file: BufReader<File>,
    datafile_start: u64,
    cur_datafile_offset: u64,
    seek_base: Option<u64>,
    error: Option<io::Error>,
}

struct CallbackData {
    file: File,
    seek_base: u64,
    buffer: Option<Vec<u8>>,
    error: Option<io::Error>,
}

pub struct Reader {
    callback_data: CallbackData,
    raw: raw::Reader,
}

trait ResultExt {
    type T;
    fn retrieve(self, error: &mut Option<io::Error>) -> Result<Self::T, Error>;
}

impl<T> ResultExt for Result<T, raw::Error> {
    type T = T;
    fn retrieve(self, error: &mut Option<io::Error>) -> Result<T, Error> {
        self.map_err(|e| match e {
            raw::Error::Df(e) => Error::Df(e),
            raw::Error::Callback => Error::Io(error.take().unwrap()),
        })
    }
}

impl Reader {
    fn new_impl(file: File, check_initial_offset: bool) -> Result<Reader, Error> {
        let mut file = file;
        let datafile_start = if check_initial_offset {
            file.seek(SeekFrom::Current(0))?
        } else {
            0
        };
        let mut callback_data_new = CallbackDataNew {
            file: BufReader::new(file),
            datafile_start: datafile_start,
            cur_datafile_offset: 0,
            seek_base: None,
            error: None,
        };
        let raw =
            raw::Reader::new(&mut callback_data_new).retrieve(&mut callback_data_new.error)?;
        let callback_data = CallbackData {
            file: callback_data_new.file.into_inner(),
            seek_base: callback_data_new.seek_base.unwrap(),
            buffer: None,
            error: None,
        };
        Ok(Reader {
            callback_data: callback_data,
            raw: raw,
        })
    }
    pub fn new(file: File) -> Result<Reader, Error> {
        Reader::new_impl(file, true)
    }
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Reader, Error> {
        fn inner(path: &Path) -> Result<Reader, Error> {
            Reader::new_impl(File::open(path)?, false)
        }
        inner(path.as_ref())
    }
    pub fn debug_dump(&mut self) -> Result<(), Error> {
        Ok(self
            .raw
            .debug_dump(&mut self.callback_data)
            .retrieve(&mut self.callback_data.error)?)
    }
    pub fn version(&self) -> raw::Version {
        self.raw.version()
    }
    pub fn read_data(&mut self, index: usize) -> Result<Vec<u8>, Error> {
        self.raw
            .read_data(&mut self.callback_data, index)
            .retrieve(&mut self.callback_data.error)?;
        Ok(self.callback_data.buffer.take().unwrap())
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
        fn map_fn(i: usize, self_: &mut &mut Reader) -> Result<Vec<u8>, Error> {
            self_.read_data(i)
        }
        let num_data = self.num_data();
        MapIterator::new(self, 0..num_data, map_fn)
    }
}

pub type DataIter<'a> = MapIterator<Result<Vec<u8>, Error>, &'a mut Reader, ops::Range<usize>>;

// "SeekOverflow"
fn so(o: Option<u64>) -> io::Result<u64> {
    o.ok_or_else(seek_overflow)
}

impl CallbackNew for CallbackDataNew {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, CallbackError> {
        fn inner(self_: &mut CallbackDataNew, buffer: &mut [u8]) -> io::Result<usize> {
            let r = self_.file.read_retry(buffer)?;
            self_.cur_datafile_offset = so(self_.cur_datafile_offset.checked_add(r.u64()))?;
            Ok(r)
        }
        inner(self, buffer).map_err(|e| {
            self.error = Some(e);
            CallbackError
        })
    }
    fn set_seek_base(&mut self) -> Result<(), CallbackError> {
        self.seek_base = Some(self.cur_datafile_offset);
        Ok(())
    }
    fn ensure_filesize(&mut self, filesize: u32) -> Result<Result<(), ()>, CallbackError> {
        fn inner(self_: &mut CallbackDataNew, filesize: u32) -> io::Result<Result<(), ()>> {
            let actual = self_.file.get_ref().metadata()?.len();
            Ok(
                if actual.checked_sub(self_.datafile_start).unwrap() >= filesize.u64() {
                    Ok(())
                } else {
                    Err(())
                },
            )
        }
        inner(self, filesize).map_err(|e| {
            self.error = Some(e);
            CallbackError
        })
    }
}
impl CallbackReadData for CallbackData {
    fn seek_read(&mut self, start: u32, buffer: &mut [u8]) -> Result<usize, CallbackError> {
        fn inner(self_: &mut CallbackData, start: u32, buffer: &mut [u8]) -> io::Result<usize> {
            let offset = so(self_.seek_base.checked_add(start.u64()))?;
            self_.file.read_offset_retry(buffer, offset)
        }
        inner(self, start, buffer).map_err(|e| {
            self.error = Some(e);
            CallbackError
        })
    }
    fn alloc_data_buffer(&mut self, length: usize) -> Result<(), CallbackError> {
        let mut vec = Vec::with_capacity(length);
        unsafe {
            vec.set_len(length);
        }
        self.buffer = Some(vec);
        Ok(())
    }
    fn data_buffer(&mut self) -> &mut [u8] {
        self.buffer.as_mut().unwrap()
    }
}
