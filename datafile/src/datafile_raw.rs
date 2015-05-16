pub trait DatafileCallback {
    fn read(start: Option<u32>, buffer: &mut [u8]) -> Result<u32,()>;
    fn ensure_filesize(filesize: u32) -> Result<(),()>;
}

use std::cell::RefCell;
use std::io::{IoResult, SeekSet};
use std::iter;
use std::mem;
use std::slice::mut_ref_slice;
use std::str::from_utf8;

use bitmagic::{
    read_exact_le_ints,
    read_exact_le_ints_owned,
    relative_size_of,
    relative_size_of_mult,
    to_little_endian,
    transmute_mut_slice,
    transmute_slice,
};

pub trait DatafileCallback {
    fn read(start: u32, buffer: &mut [u8]) -> Result<u32,()>;
    fn ensure_filesize(filesize: u32) -> Result<(),()>;
}

#[deriving(Clone, Copy, Show)]
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

fn read_as_le_i32s<T:UnsafeOnlyI32>(reader: &mut Read) -> IoResult<T> {
    // this is safe as T is guaranteed by UnsafeOnlyI32 to be POD, which
    // means there won't be a destructor running over uninitialized
    // elements, even when returning early from the try!()
    let mut result = unsafe { mem::uninitialized() };
    try!(read_exact_le_ints(reader, as_mut_i32_slice(mut_ref_slice(&mut result))));
    Ok(result)
}

fn read_owned_vec_as_le_i32s<T:UnsafeOnlyI32>(reader: &mut Read, count: usize) -> IoResult<Vec<T>> {
    let mut result = Vec::with_capacity(count);
    // this operation is safe by the same reasoning for the unsafe block in
    // `read_as_le_i32s`.
    unsafe { result.set_len(count); }
    try!(read_exact_le_ints(reader, as_mut_i32_slice(result.as_mut_slice())));
    Ok(result)
}

#[deriving(Clone, Copy, Eq, Hash, PartialEq, Show)]
pub enum DatafileErr {
    WrongMagic,
    UnsupportedVersion,
    MalformedHeader,
    Malformed,
    CompressionError,
}

pub static DATAFILE_MAGIC: [u8, ..4] = [b'D', b'A', b'T', b'A'];
pub static DATAFILE_MAGIC_BIGENDIAN: [u8, ..4] = [b'A', b'T', b'A', b'D'];
pub static DATAFILE_VERSION3: i32 = 3;
pub static DATAFILE_VERSION4: i32 = 4;

pub static DATAFILE_ITEMTYPE_ID_RANGE: i32 = 0x10000;

pub type DfResult<T> = Result<T,DatafileErr>;

impl DatafileHeaderVersion {
    pub fn read_raw(reader: &mut Read) -> IoResult<DatafileHeaderVersion> {
        let mut result: DatafileHeaderVersion = try!(read_as_le_i32s(reader));
        {
            // this operation is safe because result.magic is POD
            let magic_view: &mut [i32] = unsafe { transmute_mut_slice(result.magic.as_mut_slice()) };
            unsafe { to_little_endian(magic_view) };
        }
        Ok(result)
    }
    pub fn read(reader: &mut Read) -> IoResult<DfResult<DatafileHeaderVersion>> {
        let result = try!(DatafileHeaderVersion::read_raw(reader));
        debug!("read header_ver={}", result);
        tryi!(result.check());
        Ok(Ok(result))
    }
    pub fn check(&self) -> DfResult<()> {
        Err(
            if self.magic != DATAFILE_MAGIC && self.magic != DATAFILE_MAGIC_BIGENDIAN {
                error!("wrong datafile signature, magic={:08x}",
                    (self.magic[0] as u32 << 24)
                    | (self.magic[1] as u32 << 16)
                    | (self.magic[2] as u32 << 8)
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
    pub fn write(self, _writer: &mut Writer) -> IoResult<()> {
        unimplemented!();
    }
}

impl DatafileHeader {
    pub fn read_raw(reader: &mut Read) -> IoResult<DatafileHeader> {
        Ok(try!(read_as_le_i32s(reader)))
    }
    pub fn read(reader: &mut Read) -> IoResult<DfResult<DatafileHeader>> {
        let result = try!(DatafileHeader::read_raw(reader));
        debug!("read header={}", result);
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
    pub fn write(self, _writer: &mut Writer) -> IoResult<()> {
        unimplemented!();
    }
}

impl DatafileItemType {
    pub fn write(self, _writer: &mut Writer) -> IoResult<()> {
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

pub struct MapIterator<T,U,D,I> {
    data: D,
    iterator: I,
    // `map` is already an function of an iterator, so we can't use `map` as a name here
    map_fn: fn (T, &D) -> U,
}

pub type DfItemIter<'a,T> = MapIterator<usize,DatafileItem<'a>,&'a T,iter::Range<usize>>;
pub type DfItemTypeIter<'a,T> = MapIterator<usize,u16,&'a T,iter::Range<usize>>;
pub type DfDataIter<'a,T> = MapIterator<usize,Result<Vec<u8>,()>,&'a T,iter::Range<usize>>;

impl<T,U,D,I:Iterator<T>> Iterator<U> for MapIterator<T,U,D,I> {
    fn next(&mut self) -> Option<U> {
        self.iterator.next().map(|x| (self.map_fn)(x, &self.data))
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


    fn items<'a>(&'a self) -> DfItemIter<'a,Self> {
        MapIterator { data: self, iterator: range(0, self.num_items()), map_fn: datafile_item_map_fn }
    }
    fn item_types<'a>(&'a self) -> DfItemTypeIter<'a,Self> {
        MapIterator { data: self, iterator: range(0, self.num_item_types()), map_fn: datafile_item_type_map_fn }
    }
    fn item_type_items<'a>(&'a self, type_id: u16) -> DfItemIter<'a,Self> {
        let (start, num) = self.item_type_indexes_start_num(type_id);
        MapIterator { data: self, iterator: range(start, start + num), map_fn: datafile_item_map_fn }
    }
    fn item_find<'a>(&'a self, type_id: u16, id: u16) -> Option<DatafileItem<'a>> {
        for item in self.item_type_items(type_id) {
            if item.id == id {
                return Some(item);
            }
        }
        None
    }
    fn data_iter<'a>(&'a self) -> DfDataIter<'a,Self> {
        MapIterator { data: self, iterator: range(0, self.num_data()), map_fn: datafile_data_map_fn }
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
    pub fn read(mut file: T) -> IoResult<DfResult<DatafileReader<T>>> {
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
                4 => Some(try!(read_exact_le_ints_owned(reader, header.num_data as usize))),
                _ => unreachable!(), // should have been caught in header_ver.check()
            };
            // possible failure of relative_size_of_mult should have been caught in header.check()
            items_raw = try!(read_owned_vec_as_le_i32s(reader, relative_size_of_mult::<u8,i32>(header.size_items as usize)));

        }
        // TODO: FIXME: check for u64 -> i64 overflow
        let data_offset = try!(file.as_seek_mut().tell());

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
        tryi!(result.check())
        Ok(Ok(result))
    }
    pub fn check(&self) -> DfResult<()> {
        {
            let mut expected_start = 0;
            for (i, t) in self.item_types.iter().enumerate() {
                if !(0 <= t.type_id && t.type_id < DATAFILE_ITEMTYPE_ID_RANGE) {
                    error!("invalid item_type type_id: must be in range 0 to {:x}, item_type={} type_id={}", DATAFILE_ITEMTYPE_ID_RANGE, i, t.type_id)
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
                for (k, t2) in self.item_types.slice_to(i).iter().enumerate() {
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
            for i in range(0, self.header.num_items as usize) {
                if self.item_offsets.as_slice()[i] < 0 {
                    error!("invalid item offset (negative), item={} offset={}", i, self.item_offsets.as_slice()[i]);
                    return Err(DatafileErr::Malformed);
                }
                if offset != self.item_offsets.as_slice()[i] as usize {
                    error!("invalid item offset, item={} offset={} wanted={}", i, self.item_offsets.as_slice()[i], offset);
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
            for i in range(0, self.header.num_data as usize) {
                match self.uncomp_data_sizes {
                    Some(ref uds) => {
                        if uds.as_slice()[i] < 0 {
                            error!("invalid data's uncompressed size, data={} uncomp_data_size={}", i, uds.as_slice()[i]);
                            return Err(DatafileErr::Malformed);
                        }
                    }
                    None => (),
                }
                let offset = self.data_offsets.as_slice()[i];
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
                for k in range(t.start as usize, (t.start + t.num) as usize) {
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
        let slice = self.items_raw
            .slice_from(relative_size_of_mult::<u8,i32>(self.item_offsets.as_slice()[index] as usize))
            .slice_to(relative_size_of::<DatafileItemHeader,i32>());
        // TODO: find out why paranthesis are necessary
        // this operation is safe because both `i32` and
        // `DatafileItemHeader` are POD
        &(unsafe { transmute_slice::<i32,DatafileItemHeader>(slice) })[0]
    }
    fn data_size_file(&self, index: usize) -> usize {
        let start = self.data_offsets.as_slice()[index] as usize;
        let end = if index < self.data_offsets.len() - 1 {
            self.data_offsets.as_slice()[index + 1] as usize
        } else {
            self.header.size_data as usize
        };
        assert!(start <= end);
        end - start
    }
    fn uncomp_data_impl(&self, index: usize) -> IoResult<DfResult<Vec<u8>>> {
        let mut file = self.file.borrow_mut();
        try!(file.as_seek_mut().seek(self.data_offset as i64 + self.data_offsets.as_slice()[index] as i64, SeekSet));

        let raw_data_len = self.data_size_file(index);
        let mut raw_data = Vec::with_capacity(raw_data_len);
        unsafe { raw_data.set_len(raw_data_len); }
        {
            let raw_data_slice = raw_data.as_mut_slice();
            try!(file.as_reader_mut().read_at_least(raw_data_slice.len(), raw_data_slice));
        }

        match self.uncomp_data_sizes {
            Some(ref uds) => {
                let data_len = uds.as_slice()[index] as usize;
                let mut data = Vec::with_capacity(data_len);
                unsafe { data.set_len(data_len); }

                match zlib::uncompress(data.as_mut_slice(), raw_data.as_slice()) {
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
        if !log_enabled!(log::DEBUG) {
            return;
        }
        debug!("DATAFILE");
        debug!("header_ver: {}", self.header_ver);
        debug!("header: {}", self.header);
        for type_id in self.item_types() {
            debug!("item_type type_id={}", type_id);
            for item in self.item_type_items(type_id) {
                debug!("\titem id={} data={}", item.id, item.data);
            }
        }
        for (i, data) in self.data_iter().enumerate() {
            let data = data.unwrap();
            debug!("data id={} size={}", i, data.len());
            if data.len() < 256 {
                match from_utf8(data.as_slice()) {
                    Some(s) => debug!("\tstr={}", s),
                    None => {},
                }
            }
        }
    }
}

impl<T:Read+Seek> Datafile for DatafileReader<T> {
    fn item_type(&self, index: usize) -> u16 {
        self.item_types.as_slice()[index].type_id as u16
    }
    fn num_item_types(&self) -> usize {
        self.header.num_item_types as usize
    }

    fn item<'a>(&'a self, index: usize) -> DatafileItem<'a> {
        let item_header = self.item_header(index);
        let data = self.items_raw
            .slice_from(relative_size_of_mult::<u8,i32>(self.item_offsets.as_slice()[index] as usize))
            .slice_from(relative_size_of::<DatafileItemHeader,i32>())
            .slice_to(relative_size_of_mult::<u8,i32>(item_header.size as usize));
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
                error!("datafile uncompression error {}", x);
                Err(())
            },
            Err(x) => {
                error!("IO error while uncompressing {}", x);
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

#[deriving(Clone, Copy, Show)]
struct DfBufItemType {
    type_id: u16,
    start: usize,
    num: usize,
}

#[deriving(Clone, Show)]
struct DfBufItem {
    type_id: u16,
    id: u16,
    data: Vec<i32>,
}

pub type DfDataNoerrIter<'a,T> = MapIterator<usize,&'a [u8],&'a T,iter::Range<usize>>;

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
                (self.item_types.as_slice()[item_type_index].start, false)
            } else {
                (self.items.len(), false)
            }
        } else {
            let DfBufItemType { start, num, .. } = self.item_types.as_slice()[item_type_index];

            for (i, &DfBufItem { id: other_id, .. })
                in self.items.slice_from(start).slice_to(num).iter().enumerate().map(|(i, x)| (start+i, x)) {

                if id <= other_id {
                    return (i, id == other_id)
                }
            }

            (start + num, false)
        }
    }

    pub fn data_noerr<'a>(&'a self, index: usize) -> &'a [u8] {
        self.data.as_slice()[index].as_slice()
    }

    pub fn data_noerr_iter<'a>(&'a self) -> DfDataNoerrIter<'a,DatafileBuffer> {
        MapIterator { data: self, iterator: range(0, self.num_data()), map_fn: datafile_data_noerr_map_fn }
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
        self.item_types.as_mut_slice()[type_index].num += 1;

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
        let DfBufItem { type_id, id, ref data } = self.items.as_slice()[index];
        DatafileItem {
            type_id: type_id,
            id: id,
            data: data.as_slice(),
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
        let item_type = self.item_types.as_slice()[type_index];
        (item_type.start, item_type.num)
    }
}

/*
pub fn write_datafile<T:Datafile,W:Writer>(df: &T, writer: &mut W) -> Result<IoResult<()>,()> {
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
