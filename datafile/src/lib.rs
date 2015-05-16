#[macro_use]
extern crate log;

extern crate common;
extern crate zlib_minimal as zlib;

use std::cell::RefCell;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io;
use std::mem;
use std::ops;
use std::str::from_utf8;

use bitmagic::read_exact_le_ints;
use bitmagic::read_exact_le_ints_owned;
use bitmagic::relative_size_of;
use bitmagic::relative_size_of_mult;
use bitmagic::to_little_endian;
use bitmagic::transmute_mut_slice;
use bitmagic::transmute_slice;
use common::slice::mut_ref_slice;
use ext::ReadComplete;

mod bitmagic;
mod ext;

// TODO: export these into a separate module
/// `try` for nested results
macro_rules! try2 {
    ($e:expr) => (match $e { Ok(Ok(e)) => e, Ok(Err(e)) => return Ok(Err(e)), Err(e) => return Err(e) })
}

/// `try` for the inner nested result
macro_rules! tryi {
    ($e:expr) => (match $e { Ok(e) => e, Err(e) => return Ok(Err(e)) })
}

pub trait DatafileCallback {
    fn read(start: u32, buffer: &mut [u8]) -> Result<usize,()>;
    fn ensure_filesize(filesize: u32) -> Result<(),()>;
}

pub trait SeekReaderCast: Seek + Read {
    fn as_seek_ref(&self) -> &Seek;
    fn as_reader_ref(&self) -> &Read;
    fn as_seek_mut(&mut self) -> &mut Seek;
    fn as_reader_mut(&mut self) -> &mut Read;
}

impl<T:Seek+Read> SeekReaderCast for T {
    fn as_seek_ref(&self) -> &Seek { self as &Seek }
    fn as_reader_ref(&self) -> &Read { self as &Read }
    fn as_seek_mut(&mut self) -> &mut Seek { self as &mut Seek }
    fn as_reader_mut(&mut self) -> &mut Read { self as &mut Read }
}

// --------------
// DATAFILE STUFF
// --------------

#[derive(Clone, Copy, Debug)]
#[packed]
pub struct DatafileHeaderVersion {
    pub magic: [u8; 4],
    pub version: i32,
}

#[derive(Clone, Copy, Debug)]
#[packed]
pub struct DatafileHeader {
    pub _size: i32,
    pub _swaplen: i32,
    pub num_item_types: i32,
    pub num_items: i32,
    pub num_data: i32,
    pub size_items: i32,
    pub size_data: i32,
}

#[derive(Clone, Copy, Debug)]
#[packed]
pub struct DatafileItemType {
    pub type_id: i32,
    pub start: i32,
    pub num: i32,
}

#[derive(Clone, Copy, Debug)]
#[packed]
pub struct DatafileItemHeader {
    pub type_id_and_id: i32,
    pub size: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct DatafileItem<'a> {
    pub type_id: u16,
    pub id: u16,
    pub data: &'a [i32],
}

// A struct may only implement UnsafeOnlyI32 if it consists entirely of
// tightly packed i32 and does not have a destructor.
pub trait UnsafeOnlyI32: Copy { }
impl UnsafeOnlyI32 for i32 { }
impl UnsafeOnlyI32 for DatafileHeaderVersion { }
impl UnsafeOnlyI32 for DatafileHeader { }
impl UnsafeOnlyI32 for DatafileItemType { }
impl UnsafeOnlyI32 for DatafileItemHeader { }


fn as_mut_i32_slice<'a, T:UnsafeOnlyI32>(x: &'a mut [T]) -> &'a mut [i32] {
    unsafe { transmute_mut_slice(x) }
}

fn read_as_le_i32s<T:UnsafeOnlyI32>(reader: &mut Read) -> io::Result<T> {
    // this is safe as T is guaranteed by UnsafeOnlyI32 to be POD, which
    // means there won't be a destructor running over uninitialized
    // elements, even when returning early from the try!()
    let mut result = unsafe { mem::uninitialized() };
    try!(unsafe { read_exact_le_ints(reader, as_mut_i32_slice(mut_ref_slice(&mut result)))});
    Ok(result)
}

fn read_owned_vec_as_le_i32s<T:UnsafeOnlyI32>(reader: &mut Read, count: usize) -> io::Result<Vec<T>> {
    let mut result = Vec::with_capacity(count);
    // this operation is safe by the same reasoning for the unsafe block in
    // `read_as_le_i32s`.
    unsafe { result.set_len(count); }
    try!(unsafe { read_exact_le_ints(reader, as_mut_i32_slice(&mut result))});
    Ok(result)
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum DatafileErr {
    WrongMagic,
    UnsupportedVersion,
    MalformedHeader,
    Malformed,
    CompressionError,
}

pub static DATAFILE_MAGIC: [u8; 4] = [b'D', b'A', b'T', b'A'];
pub static DATAFILE_MAGIC_BIGENDIAN: [u8; 4] = [b'A', b'T', b'A', b'D'];
pub static DATAFILE_VERSION3: i32 = 3;
pub static DATAFILE_VERSION4: i32 = 4;

pub static DATAFILE_ITEMTYPE_ID_RANGE: i32 = 0x10000;

pub type DfResult<T> = Result<T,DatafileErr>;

impl DatafileHeaderVersion {
    pub fn read_raw(reader: &mut Read) -> io::Result<DatafileHeaderVersion> {
        let mut result: DatafileHeaderVersion = try!(read_as_le_i32s(reader));
        {
            // this operation is safe because result.magic is POD
            let magic_view: &mut [i32] = unsafe { transmute_mut_slice(&mut result.magic) };
            unsafe { to_little_endian(magic_view) };
        }
        Ok(result)
    }
    pub fn read(reader: &mut Read) -> io::Result<DfResult<DatafileHeaderVersion>> {
        let result = try!(DatafileHeaderVersion::read_raw(reader));
        debug!("read header_ver={:?}", result);
        tryi!(result.check());
        Ok(Ok(result))
    }
    pub fn check(&self) -> DfResult<()> {
        Err(
            if self.magic != DATAFILE_MAGIC && self.magic != DATAFILE_MAGIC_BIGENDIAN {
                error!("wrong datafile signature, magic={:08x}",
                    ((self.magic[0] as u32) << 24)
                    | ((self.magic[1] as u32) << 16)
                    | ((self.magic[2] as u32) << 8)
                    | (self.magic[3] as u32));
                DatafileErr::WrongMagic
            } else if self.version != DATAFILE_VERSION3 && self.version != DATAFILE_VERSION4 {
                error!("unsupported datafile version, version={}", self.version);
                DatafileErr::UnsupportedVersion
            } else {
                return Ok(());
            }
        )
    }
    pub fn write(self, _writer: &mut Write) -> io::Result<()> {
        unimplemented!();
    }
}

impl DatafileHeader {
    pub fn read_raw(reader: &mut Read) -> io::Result<DatafileHeader> {
        Ok(try!(read_as_le_i32s(reader)))
    }
    pub fn read(reader: &mut Read) -> io::Result<DfResult<DatafileHeader>> {
        let result = try!(DatafileHeader::read_raw(reader));
        debug!("read header={:?}", result);
        tryi!(result.check());
        Ok(Ok(result))
    }
    pub fn check(&self) -> DfResult<()> {
        Err(
            if self._size < 0 {
                error!("_size is negative, _size={}", self._size);
                DatafileErr::MalformedHeader
            } else if self._swaplen < 0 {
                error!("_swaplen is negative, _swaplen={}", self._swaplen);
                DatafileErr::MalformedHeader
            } else if self.num_item_types < 0 {
                error!("num_item_types is negative, num_item_types={}", self.num_item_types);
                DatafileErr::MalformedHeader
            } else if self.num_items < 0 {
                error!("num_items is negative, num_items={}", self.num_items);
                DatafileErr::MalformedHeader
            } else if self.num_data < 0 {
                error!("num_data is negative, num_data={}", self.num_data);
                DatafileErr::MalformedHeader
            } else if self.size_items < 0 {
                error!("size_items is negative, size_items={}", self.size_items);
                DatafileErr::MalformedHeader
            } else if self.size_data < 0 {
                error!("size_data is negative, size_data={}", self.size_data);
                DatafileErr::MalformedHeader
            } else if self.size_items as u32 % mem::size_of::<i32>() as u32 != 0 {
                error!("size_items not divisible by 4, size_items={}", self.size_items);
                DatafileErr::MalformedHeader
            // TODO: make various check about size, swaplen (non-critical)
            } else {
                return Ok(())
            }
        )
    }
    pub fn write(self, _writer: &mut Write) -> io::Result<()> {
        unimplemented!();
    }
}

impl DatafileItemType {
    pub fn write(self, _writer: &mut Write) -> io::Result<()> {
        unimplemented!();
    }
}

impl DatafileItemHeader {
    pub fn new(type_id: u16, id: u16, size: i32) -> DatafileItemHeader {
        let mut result = DatafileItemHeader { type_id_and_id: 0, size: size };
        result.set_type_id_and_id(type_id, id);
        result
    }
    pub fn type_id(&self) -> u16 {
        (((self.type_id_and_id as u32) >> 16) & 0xffff) as u16
    }
    pub fn id(&self) -> u16 {
        ((self.type_id_and_id as u32) & 0xffff) as u16
    }
    pub fn set_type_id_and_id(&mut self, type_id: u16, id: u16) {
        self.type_id_and_id = (((type_id as u32) << 16) | (id as u32)) as i32;
    }
}

pub struct MapIterator<T,D,I:Iterator> {
    data: D,
    iterator: I,
    // `map` is already an function of an iterator, so we can't use `map` as a name here
    map_fn: fn (I::Item, &D) -> T,
}

pub type DfItemIter<'a,T> = MapIterator<DatafileItem<'a>,&'a T,ops::Range<usize>>;
pub type DfItemTypeIter<'a,T> = MapIterator<u16,&'a T,ops::Range<usize>>;
pub type DfDataIter<'a,T> = MapIterator<Result<Vec<u8>,()>,&'a T,ops::Range<usize>>;

impl<T,D,I:Iterator> Iterator for MapIterator<T,D,I> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iterator.next().map(|x| (self.map_fn)(x, &self.data))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

fn datafile_item_map_fn<'a,T:Datafile>(index: usize, df: &&'a T) -> DatafileItem<'a> {
    df.item(index)
}

fn datafile_item_type_map_fn<'a,T:Datafile>(index: usize, df: &&'a T) -> u16 {
    df.item_type(index)
}

fn datafile_data_map_fn<'a,T:Datafile>(index: usize, df: &&'a T) -> Result<Vec<u8>,()> {
    df.data(index)
}

pub trait Datafile {
    // TODO: doc
    fn item_type(&self, index: usize) -> u16;
    fn num_item_types(&self) -> usize;

    fn item<'a>(&'a self, index: usize) -> DatafileItem<'a>;
    fn num_items(&self) -> usize;

    fn data<'a>(&'a self, index: usize) -> Result<Vec<u8>,()>;
    fn num_data(&self) -> usize;

    fn item_type_indexes_start_num(&self, type_id: u16) -> (usize, usize);


    fn items<'a>(&'a self) -> DfItemIter<'a,Self> where Self: Sized {
        MapIterator { data: self, iterator: 0..self.num_items(), map_fn: datafile_item_map_fn }
    }
    fn item_types<'a>(&'a self) -> DfItemTypeIter<'a,Self> where Self: Sized {
        MapIterator { data: self, iterator: 0..self.num_item_types(), map_fn: datafile_item_type_map_fn }
    }
    fn item_type_items<'a>(&'a self, type_id: u16) -> DfItemIter<'a,Self> where Self: Sized {
        let (start, num) = self.item_type_indexes_start_num(type_id);
        MapIterator { data: self, iterator: start..start+num, map_fn: datafile_item_map_fn }
    }
    fn item_find<'a>(&'a self, type_id: u16, id: u16) -> Option<DatafileItem<'a>> where Self: Sized {
        for item in self.item_type_items(type_id) {
            if item.id == id {
                return Some(item);
            }
        }
        None
    }
    fn data_iter<'a>(&'a self) -> DfDataIter<'a,Self> where Self: Sized {
        MapIterator { data: self, iterator: 0..self.num_data(), map_fn: datafile_data_map_fn }
    }
}

pub struct DatafileReader<T> {
    header_ver: DatafileHeaderVersion,
    header: DatafileHeader,

    item_types: Vec<DatafileItemType>,
    item_offsets: Vec<i32>,
    data_offsets: Vec<i32>,
    uncomp_data_sizes: Option<Vec<i32>>,
    items_raw: Vec<i32>,

    data_offset: u64,

    file: RefCell<T>,
}

impl<T:Seek+Read> DatafileReader<T> {
    pub fn read(mut file: T) -> io::Result<DfResult<DatafileReader<T>>> {
        let header_ver;
        let header;
        let item_types_raw;
        let item_offsets;
        let data_offsets;
        let uncomp_data_sizes;
        let items_raw;
        {
            let mut reader = file.as_reader_mut();
            header_ver = try2!(DatafileHeaderVersion::read(reader));
            header = try2!(DatafileHeader::read(reader));
            item_types_raw = try!(read_owned_vec_as_le_i32s(reader, header.num_item_types as usize));
            item_offsets = try!(read_owned_vec_as_le_i32s(reader, header.num_items as usize));
            data_offsets = try!(read_owned_vec_as_le_i32s(reader, header.num_data as usize));
            uncomp_data_sizes = match header_ver.version {
                3 => None,
                4 => Some(try!(unsafe { read_exact_le_ints_owned(reader, header.num_data as usize)})),
                _ => unreachable!(), // should have been caught in header_ver.check()
            };
            // possible failure of relative_size_of_mult should have been caught in header.check()
            items_raw = try!(read_owned_vec_as_le_i32s(reader, relative_size_of_mult::<u8,i32>(header.size_items as usize)));

        }
        // TODO: FIXME: check for u64 -> i64 overflow
        let data_offset = try!(file.as_seek_mut().seek(SeekFrom::Current(0)));

        let result = DatafileReader {
            header_ver: header_ver,
            header: header,
            item_types: item_types_raw,
            item_offsets: item_offsets,
            data_offsets: data_offsets,
            uncomp_data_sizes: uncomp_data_sizes,
            items_raw: items_raw,
            data_offset: data_offset,
            file: RefCell::new(file),
        };
        tryi!(result.check());
        Ok(Ok(result))
    }
    pub fn check(&self) -> DfResult<()> {
        {
            let mut expected_start = 0;
            for (i, t) in self.item_types.iter().enumerate() {
                if !(0 <= t.type_id && t.type_id < DATAFILE_ITEMTYPE_ID_RANGE) {
                    error!("invalid item_type type_id: must be in range 0 to {:x}, item_type={} type_id={}", DATAFILE_ITEMTYPE_ID_RANGE, i, t.type_id);
                    return Err(DatafileErr::Malformed);
                }
                if !(0 <= t.num && t.num <= self.header.num_items - t.start) {
                    error!("invalid item_type num: must be in range 0 to num_items - start + 1, item_type={} type_id={} start={} num={}", i, t.type_id, t.start, t.num);
                    return Err(DatafileErr::Malformed);
                }
                if t.start != expected_start {
                    error!("item_types are not sequential, item_type={} type_id={} start={} expected={}", i, t.type_id, t.start, expected_start);
                    return Err(DatafileErr::Malformed);
                }
                expected_start += t.num;
                for (k, t2) in self.item_types[..i].iter().enumerate() {
                    if t.type_id == t2.type_id {
                        error!("item_type type_id occurs twice, type_id={} item_type1={} item_type2={}", t.type_id, i, k);
                        return Err(DatafileErr::Malformed);
                    }
                }
            }
            if expected_start != self.header.num_items {
                error!("last item_type does not contain last item, item_type={}", self.header.num_item_types - 1);
                return Err(DatafileErr::Malformed);
            }
        }
        {
            let mut offset = 0;
            for i in 0..self.header.num_items as usize {
                if self.item_offsets[i] < 0 {
                    error!("invalid item offset (negative), item={} offset={}", i, self.item_offsets[i]);
                    return Err(DatafileErr::Malformed);
                }
                if offset != self.item_offsets[i] as usize {
                    error!("invalid item offset, item={} offset={} wanted={}", i, self.item_offsets[i], offset);
                    return Err(DatafileErr::Malformed);
                }
                offset += mem::size_of::<DatafileItemHeader>();
                if offset > self.header.size_items as usize {
                    error!("item header out of bounds, item={} offset={} size_items={}", i, offset, self.header.size_items);
                    return Err(DatafileErr::Malformed);
                }
                let item_header = self.item_header(i);
                if item_header.size < 0 {
                    error!("item has negative size, item={} size={}", i, item_header.size);
                    return Err(DatafileErr::Malformed);
                }
                offset += item_header.size as usize;
                if offset > self.header.size_items as usize {
                    error!("item out of bounds, item={} size={} size_items={}", i, item_header.size, self.header.size_items);
                    return Err(DatafileErr::Malformed);
                }
            }
            if offset != self.header.size_items as usize {
                error!("last item not large enough, item={} offset={} size_items={}", self.header.num_items - 1, offset, self.header.size_items);
                return Err(DatafileErr::Malformed);
            }
        }
        {
            let mut previous = 0;
            for i in 0..self.header.num_data as usize {
                match self.uncomp_data_sizes {
                    Some(ref uds) => {
                        if uds[i] < 0 {
                            error!("invalid data's uncompressed size, data={} uncomp_data_size={}", i, uds[i]);
                            return Err(DatafileErr::Malformed);
                        }
                    }
                    None => (),
                }
                let offset = self.data_offsets[i];
                if offset < 0 || offset > self.header.size_data {
                    error!("invalid data offset, data={} offset={}", i, offset);
                    return Err(DatafileErr::Malformed);
                }
                if previous > offset {
                    error!("data overlaps, data1={} data2={}", i - 1, i);
                    return Err(DatafileErr::Malformed);
                }
                previous = offset;
            }
        }
        {
            for (i, t) in self.item_types.iter().enumerate() {
                for k in t.start as usize..(t.start + t.num) as usize {
                    let item_header = self.item_header(k);
                    if item_header.type_id() != t.type_id as u16 {
                        error!("item does not have right type_id, type={} type_id1={} item={} type_id2={}", i, t.type_id, k, item_header.type_id());
                        return Err(DatafileErr::Malformed);
                    }
                }
            }
        }
        Ok(())
    }
    fn item_header<'a>(&'a self, index: usize) -> &'a DatafileItemHeader {
        let slice = &self.items_raw
            [relative_size_of_mult::<u8,i32>(self.item_offsets[index] as usize)..]
            [..relative_size_of::<DatafileItemHeader,i32>()];
        // TODO: find out why paranthesis are necessary
        // this operation is safe because both `i32` and
        // `DatafileItemHeader` are POD
        &(unsafe { transmute_slice::<i32,DatafileItemHeader>(slice) })[0]
    }
    fn data_size_file(&self, index: usize) -> usize {
        let start = self.data_offsets[index] as usize;
        let end = if index < self.data_offsets.len() - 1 {
            self.data_offsets[index + 1] as usize
        } else {
            self.header.size_data as usize
        };
        assert!(start <= end);
        end - start
    }
    fn uncomp_data_impl(&self, index: usize) -> io::Result<DfResult<Vec<u8>>> {
        let mut file = self.file.borrow_mut();
        try!(file.as_seek_mut().seek(SeekFrom::Start(self.data_offset + self.data_offsets[index] as u64)));

        let raw_data_len = self.data_size_file(index);
        let mut raw_data = Vec::with_capacity(raw_data_len);
        unsafe { raw_data.set_len(raw_data_len); }
        try!(file.as_reader_mut().read_complete(&mut raw_data));

        match self.uncomp_data_sizes {
            Some(ref uds) => {
                let data_len = uds[index] as usize;
                let mut data = Vec::with_capacity(data_len);
                unsafe { data.set_len(data_len); }

                match zlib::uncompress(&mut data, &raw_data) {
                    Ok(len) if len == data.len() => {
                        Ok(Ok(data))
                    }
                    Ok(len) => {
                        error!("decompression error: wrong size, data={} size={} wanted={}", index, data.len(), len);
                        Ok(Err(DatafileErr::CompressionError))
                    }
                    _ => {
                        error!("decompression error: zlib error");
                        Ok(Err(DatafileErr::CompressionError))
                    }
                }
            },
            None => {
                Ok(Ok(raw_data))
            },
        }
    }
    pub fn debug_dump(&self) {
        if !log_enabled!(log::LogLevel::Debug) {
            return;
        }
        debug!("DATAFILE");
        debug!("header_ver: {:?}", self.header_ver);
        debug!("header: {:?}", self.header);
        for type_id in self.item_types() {
            debug!("item_type type_id={}", type_id);
            for item in self.item_type_items(type_id) {
                debug!("\titem id={} data={:?}", item.id, item.data);
            }
        }
        for (i, data) in self.data_iter().enumerate() {
            let data = data.unwrap();
            debug!("data id={} size={}", i, data.len());
            if data.len() < 256 {
                match from_utf8(&data).ok() {
                    Some(s) => debug!("\tstr={}", s),
                    None => {},
                }
            }
        }
    }
}

impl<T:Read+Seek> Datafile for DatafileReader<T> {
    fn item_type(&self, index: usize) -> u16 {
        self.item_types[index].type_id as u16
    }
    fn num_item_types(&self) -> usize {
        self.header.num_item_types as usize
    }

    fn item<'a>(&'a self, index: usize) -> DatafileItem<'a> {
        let item_header = self.item_header(index);
        let data = &self.items_raw
            [relative_size_of_mult::<u8,i32>(self.item_offsets[index] as usize)..]
            [relative_size_of::<DatafileItemHeader,i32>()..]
            [..relative_size_of_mult::<u8,i32>(item_header.size as usize)];
        DatafileItem {
            type_id: item_header.type_id(),
            id: item_header.id(),
            data: data,
        }
    }
    fn num_items(&self) -> usize {
        self.header.num_items as usize
    }

    fn data<'a>(&'a self, index: usize) -> Result<Vec<u8>,()> {
        let result: Result<Vec<u8>,()> = match self.uncomp_data_impl(index) {
            Ok(Ok(x)) => Ok(x),
            Ok(Err(x)) => {
                error!("datafile uncompression error {:?}", x);
                Err(())
            },
            Err(x) => {
                error!("IO error while uncompressing {:?}", x);
                Err(())
            },
        };
        result
    }
    fn num_data(&self) -> usize {
        self.header.num_data as usize
    }

    fn item_type_indexes_start_num(&self, type_id: u16) -> (usize, usize) {
        for t in self.item_types.iter() {
            if t.type_id as u16 == type_id {
                return (t.start as usize, t.num as usize);
            }
        }
        (0, 0)
    }
}

#[derive(Clone, Copy, Debug)]
struct DfBufItemType {
    type_id: u16,
    start: usize,
    num: usize,
}

#[derive(Clone, Debug)]
struct DfBufItem {
    type_id: u16,
    id: u16,
    data: Vec<i32>,
}

pub type DfDataNoerrIter<'a,T> = MapIterator<&'a [u8],&'a T,ops::Range<usize>>;

fn datafile_data_noerr_map_fn<'a>(index: usize, df: &&'a DatafileBuffer) -> &'a [u8] {
    df.data_noerr(index)
}

pub struct DatafileBuffer {
    item_types: Vec<DfBufItemType>,
    items: Vec<DfBufItem>,
    data: Vec<Vec<u8>>,
}

impl DatafileBuffer {
    pub fn new() -> DatafileBuffer {
        DatafileBuffer {
            item_types: Vec::new(),
            items: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn from_datafile<T:Datafile>(df: &T) -> Option<DatafileBuffer> {
        let mut result = DatafileBuffer::new();
        for (i, maybe_data) in df.data_iter().enumerate() {
            match maybe_data {
                Ok(x) => {
                    let index = result.add_data(x);
                    assert!(index == i);
                },
                Err(()) => return None,
            }
        }
        for DatafileItem { type_id, id, data } in df.items() {
            result.add_item(type_id, id, data).unwrap();
        }
        Some(result)
    }

    fn get_item_type_index(&self, type_id: u16) -> (usize, bool) {
        for (i, &DfBufItemType { type_id: other_type_id, .. }) in self.item_types.iter().enumerate() {
            if type_id <= other_type_id {
                return (i, type_id == other_type_id);
            }
        }
        (self.item_types.len(), false)
    }

    fn get_item_index(&self, item_type_index: usize, item_type_found: bool, id: u16) -> (usize, bool) {
        if !item_type_found {
            if item_type_index != self.item_types.len() {
                (self.item_types[item_type_index].start, false)
            } else {
                (self.items.len(), false)
            }
        } else {
            let DfBufItemType { start, num, .. } = self.item_types[item_type_index];

            for (i, &DfBufItem { id: other_id, .. })
                in self.items[start..][..num].iter().enumerate().map(|(i, x)| (start+i, x)) {

                if id <= other_id {
                    return (i, id == other_id)
                }
            }

            (start + num, false)
        }
    }

    pub fn data_noerr<'a>(&'a self, index: usize) -> &'a [u8] {
        &self.data[index]
    }

    pub fn data_noerr_iter<'a>(&'a self) -> DfDataNoerrIter<'a,DatafileBuffer> {
        MapIterator { data: self, iterator: 0..self.num_data(), map_fn: datafile_data_noerr_map_fn }
    }

    pub fn add_item(&mut self, type_id: u16, id: u16, data: &[i32]) -> Result<(),()> {
        let (type_index, type_found) = self.get_item_type_index(type_id);
        let (item_index, item_found) = self.get_item_index(type_index, type_found, id);

        // if we already have an item of the given type and id,
        // return an error
        if item_found {
            return Err(());
        }

        // if there isn't a type with such an id yet, insert it
        if !type_found {
            self.item_types.insert(type_index, DfBufItemType {
                type_id: type_id,
                start: item_index,
                num: 0,
            });
        }

        // we're going to insert an item, increase the count by one
        self.item_types[type_index].num += 1;

        // increase the starts of the following item types by one
        for t in self.item_types.iter_mut().skip(type_index + 1) {
            t.start += 1;
        }

        // actually insert the item
        self.items.insert(item_index, DfBufItem {
            type_id: type_id,
            id: id,
            data: data.to_vec(),
        });

        Ok(())
    }

    pub fn add_data(&mut self, data: Vec<u8>) -> usize {
        // add the data
        self.data.push(data);
        // return the index
        self.data.len() - 1
    }
}

impl Datafile for DatafileBuffer {
    fn item_type(&self, index: usize) -> u16 {
        let &DfBufItemType { type_id, .. } = self.item_types.iter().nth(index).expect("Invalid type index");
        type_id
    }
    fn num_item_types(&self) -> usize {
        self.item_types.len()
    }

    fn item<'a>(&'a self, index: usize) -> DatafileItem<'a> {
        let DfBufItem { type_id, id, ref data } = self.items[index];
        DatafileItem {
            type_id: type_id,
            id: id,
            data: &data,
        }
    }
    fn num_items(&self) -> usize {
        self.items.len()
    }

    fn data<'a>(&'a self, index: usize) -> Result<Vec<u8>,()> {
        Ok(self.data_noerr(index).to_vec())
    }
    fn num_data(&self) -> usize {
        self.data.len()
    }

    fn item_type_indexes_start_num(&self, type_id: u16) -> (usize, usize) {
        let (type_index, type_found) = self.get_item_type_index(type_id);
        if !type_found {
            return (0, 0);
        }
        let item_type = self.item_types[type_index];
        (item_type.start, item_type.num)
    }
}

/*
pub fn write_datafile<T:Datafile,W:Write>(df: &T, writer: &mut W) -> Result<io::Result<()>,()> {
    let compressed_data: Vec<Vec<u8>> = try!(result::collect(df.data_iter().map(|maybe_x| maybe_x.map(|x| {
        zlib::compress_vec(x).unwrap()
    }))));

    let size_items = df.items().fold(0, |s, i| {
        s + i.data.len() * mem::size_of::<i32>() + mem::size_of::<DatafileItemHeader>()
    });

    let size_data = compressed_data.iter().fold(0, |s, d| s + d.len());

    try!(DatafileHeaderVersion {
        magic: DATAFILE_MAGIC,
        version: 4,
    }.write(writer));

    try!(DatafileHeader {
        _size: unimplemented!(),
        _swaplen: unimplemented!(),
        num_item_types: df.item_types().len(),
        num_items: df.items().len(),
        num_data: df.data_iter().len(),
        size_items: size_items,
        size_data: size_data,
    }.write(writer));

    for &type_id in df.item_types() {
        let (start, num) = df.item_type_indexes_start_num(type_id);
        try!(DatafileItemType {
            type_id: type_id.as_i32().unwrap(),
            start: start.as_i32().unwrap(),
            num: num.as_i32().unwrap(),
        }.write(writer));
    }

    for DatafileItem { type_id, id, data } in df.items() {
        try!(DatafileItemHeader::new(type_id, id, data.len()).write(writer));
    }
    unimplemented!();
    Ok(Ok(()))
}
*/
