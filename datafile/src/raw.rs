use common::MapIterator;
use common::num::Cast;
use hexdump::hexdump_iter;
use hexdump::sanitize_byte;
use itertools::Itertools;
use log;
use std::mem;
use std::ops;
use zlib;

use bitmagic::CallbackNewExt;
use bitmagic::CallbackReadDataExt;
use bitmagic::relative_size_of;
use bitmagic::relative_size_of_mult;
use bitmagic::transmute_slice;
use format::ItemView;
use format::OnlyI32;
use format;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Version {
    V3,
    V4Crude,
    V4,
}

impl Version {
    fn has_compressed_data(&self) -> bool {
        match *self {
            Version::V3 => false,
            Version::V4Crude | Version::V4 => true,
        }
    }
}

pub struct CallbackError;

pub trait CallbackNew {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, CallbackError>;
    fn set_seek_base(&mut self) -> Result<(), CallbackError>;
    fn ensure_filesize(&mut self, filesize: u32) -> Result<Result<(), ()>, CallbackError>;
}

pub trait CallbackReadData {
    fn seek_read(&mut self, start: u32, buffer: &mut [u8]) -> Result<usize, CallbackError>;
    fn alloc_data_buffer(&mut self, length: usize) -> Result<(), CallbackError>;
    fn data_buffer(&mut self) -> &mut [u8];
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum CallbackReadError {
    Callback,
    EndOfFile,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum Error {
    Df(format::Error),
    Callback,
}

impl From<format::Error> for Error {
    fn from(err: format::Error) -> Error {
        Error::Df(err)
    }
}

impl From<CallbackError> for Error {
    fn from(CallbackError: CallbackError) -> Error {
        Error::Callback
    }
}
impl From<CallbackError> for CallbackReadError {
    fn from(CallbackError: CallbackError) -> CallbackReadError {
        CallbackReadError::Callback
    }
}

impl CallbackReadError {
    pub fn on_eof(self, df_err: format::Error) -> Error {
        match self {
            CallbackReadError::Callback => Error::Callback,
            CallbackReadError::EndOfFile => From::from(df_err),
        }
    }
}

pub struct Reader {
    header: format::Header,
    item_types: Vec<format::ItemType>,
    item_offsets: Vec<i32>,
    data_offsets: Vec<i32>,
    uncomp_data_sizes: Option<Vec<i32>>,
    items_raw: Vec<i32>,
    version: Version,
}

impl Reader {
    pub fn new(cb: &mut dyn CallbackNew) -> Result<Reader, Error> {
        fn read_i32s<T: OnlyI32>(mut cb: &mut dyn CallbackNew, len: usize) -> Result<Vec<T>,Error> {
            cb.read_exact_le_i32s_owned::<T>(len).map_err(|e| e.on_eof(format::Error::TooShort))
        }

        let header = format::Header::read(cb)?;
        let header_check = header.check_size_and_swaplen()?;
        let version = match header.hv.version {
            3 => Version::V3,
            4 => if !header_check.crude_version { Version::V4 } else { Version::V4Crude },
            _ => unreachable!(), // Should have been caught earlier, in Header::read().
        };
        let item_types_raw = read_i32s(cb, header.hr.num_item_types as usize)?;
        let item_offsets = read_i32s(cb, header.hr.num_items as usize)?;
        let data_offsets = read_i32s(cb, header.hr.num_data as usize)?;
        let uncomp_data_sizes = if !version.has_compressed_data() {
            None
        } else {
            Some(read_i32s(cb, header.hr.num_data as usize)?)
        };

        // Possible failure of relative_size_of_mult should have been caught in Header::read().
        let items_raw = read_i32s(cb, relative_size_of_mult::<u8,i32>(header.hr.size_items as usize))?;

        cb.set_seek_base()?;

        cb.ensure_filesize(header_check.expected_size)?.map_err(|()| {
            error!("file is not long enough, wanted {}", header_check.expected_size);
            format::Error::TooShort
        })?;

        let result = Reader {
            header: header,
            item_types: item_types_raw,
            item_offsets: item_offsets,
            data_offsets: data_offsets,
            uncomp_data_sizes: uncomp_data_sizes,
            items_raw: items_raw,
            version: version,
        };
        result.check()?;
        Ok(result)
    }
    pub fn check(&self) -> Result<(), format::Error> {
        {
            let mut expected_start = 0;
            let mut previous = None;
            for (i, t) in self.item_types.iter().enumerate() {
                if !(0 <= t.type_id && t.type_id < format::ITEMTYPE_ID_RANGE) {
                    error!("invalid item_type type_id: must be in range 0 to {:x}, item_type={} type_id={}", format::ITEMTYPE_ID_RANGE, i, t.type_id);
                    return Err(format::Error::Malformed);
                }
                if let Some((previous_index, previous_type_id)) = previous {
                    if !(t.type_id > previous_type_id) {
                        error!("item_type type_id: must be larger than previous type_id, item_type1={} type_id1={} item_type2={} type_id2={}", previous_index, previous_type_id, i, t.type_id);
                        return Err(format::Error::Malformed);
                    }
                }
                if !(0 <= t.num && t.num <= self.header.hr.num_items - t.start) {
                    error!("invalid item_type num: must be in range 0 to num_items - start + 1, item_type={} type_id={} start={} num={}", i, t.type_id, t.start, t.num);
                    return Err(format::Error::Malformed);
                }
                if t.start != expected_start {
                    error!("item_types are not sequential, item_type={} type_id={} start={} expected={}", i, t.type_id, t.start, expected_start);
                    return Err(format::Error::Malformed);
                }
                expected_start += t.num;
                for (k, t2) in self.item_types[..i].iter().enumerate() {
                    if t.type_id == t2.type_id {
                        error!("item_type type_id occurs twice, type_id={} item_type1={} item_type2={}", t.type_id, i, k);
                        return Err(format::Error::Malformed);
                    }
                }
                previous = Some((i, t.type_id));
            }
            if expected_start != self.header.hr.num_items {
                error!("last item_type does not contain last item, item_type={}", self.header.hr.num_item_types - 1);
                return Err(format::Error::Malformed);
            }
        }
        {
            let mut offset = 0;
            for i in 0..self.header.hr.num_items as usize {
                if self.item_offsets[i] < 0 {
                    error!("invalid item offset (negative), item={} offset={}", i, self.item_offsets[i]);
                    return Err(format::Error::Malformed);
                }
                if offset != self.item_offsets[i] as usize {
                    error!("invalid item offset, item={} offset={} wanted={}", i, self.item_offsets[i], offset);
                    return Err(format::Error::Malformed);
                }
                offset += mem::size_of::<format::ItemHeader>();
                if offset > self.header.hr.size_items as usize {
                    error!("item header out of bounds, item={} offset={} size_items={}", i, offset, self.header.hr.size_items);
                    return Err(format::Error::Malformed);
                }
                let item_header = self.item_header(i);
                if item_header.size < 0 {
                    error!("item has negative size, item={} size={}", i, item_header.size);
                    return Err(format::Error::Malformed);
                }
                offset += item_header.size as usize;
                if offset > self.header.hr.size_items as usize {
                    error!("item out of bounds, item={} size={} size_items={}", i, item_header.size, self.header.hr.size_items);
                    return Err(format::Error::Malformed);
                }
            }
            if offset != self.header.hr.size_items as usize {
                error!("last item not large enough, item={} offset={} size_items={}", self.header.hr.num_items - 1, offset, self.header.hr.size_items);
                return Err(format::Error::Malformed);
            }
        }
        {
            let mut previous = 0;
            for i in 0..self.header.hr.num_data as usize {
                if let Some(ref uds) = self.uncomp_data_sizes {
                    if uds[i] < 0 {
                        error!("invalid data's uncompressed size, data={} uncomp_data_size={}", i, uds[i]);
                        return Err(format::Error::Malformed);
                    }
                }
                let offset = self.data_offsets[i];
                if offset < 0 || offset > self.header.hr.size_data {
                    error!("invalid data offset, data={} offset={}", i, offset);
                    return Err(format::Error::Malformed);
                }
                if previous > offset {
                    // TODO: fix overflow issue
                    error!("data overlaps, data1={} data2={}", i - 1, i);
                    return Err(format::Error::Malformed);
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
                        return Err(format::Error::Malformed);
                    }
                }
            }
        }
        Ok(())
    }
    fn item_header(&self, index: usize) -> &format::ItemHeader {
        let slice = &self.items_raw
            [relative_size_of_mult::<u8,i32>(self.item_offsets[index].assert_usize())..]
            [..relative_size_of::<format::ItemHeader,i32>()];
        // TODO: Find out why paranthesis are necessary.
        //
        // This operation is safe because both `i32` and `format::ItemHeader`
        // are POD.
        &(unsafe { transmute_slice::<i32,format::ItemHeader>(slice) })[0]
    }
    fn data_size_file(&self, index: usize) -> usize {
        let start = self.data_offsets[index] as usize;
        let end = if index < self.data_offsets.len() - 1 {
            self.data_offsets[index + 1] as usize
        } else {
            self.header.hr.size_data as usize
        };
        assert!(start <= end);
        end - start
    }
    pub fn version(&self) -> Version {
        self.version
    }
    pub fn read_data<'a>(&self, mut cb: &'a mut dyn CallbackReadData, index: usize) -> Result<(), Error> {
        let raw_data_len = self.data_size_file(index);
        let raw_data = cb.seek_read_exact_owned(self.data_offsets[index] as u32, raw_data_len).map_err(|e| e.on_eof(format::Error::TooShort))?;

        if let Some(ref uds) = self.uncomp_data_sizes {
            let data_len = uds[index] as usize;
            cb.alloc_data_buffer(data_len)?;
            let data = cb.data_buffer();

            match zlib::uncompress(data, &raw_data) {
                Ok(len) if len == data_len => {
                    Ok(())
                }
                Ok(len) => {
                    error!("decompression error: wrong size, data={} size={} wanted={}", index, data_len, len);
                    Err(format::Error::CompressionWrongSize.into())
                }
                Err(e) => {
                    error!("decompression error: {:?}", e);
                    Err(format::Error::CompressionError(e).into())
                }
            }
        } else {
            let data_len = raw_data_len;
            cb.alloc_data_buffer(data_len)?;
            let data = cb.data_buffer();
            data.iter_mut().set_from(raw_data.iter().cloned());
            Ok(())
        }
    }
    pub fn item(&self, index: usize) -> ItemView {
        let item_header = self.item_header(index);
        let data = &self.items_raw
            [relative_size_of_mult::<u8,i32>(self.item_offsets[index].assert_usize())..]
            [relative_size_of::<format::ItemHeader,i32>()..]
            [..relative_size_of_mult::<u8,i32>(item_header.size.assert_usize())];
        ItemView {
            type_id: item_header.type_id(),
            id: item_header.id(),
            data: data,
        }
    }
    pub fn num_items(&self) -> usize {
        self.header.hr.num_items.assert_usize()
    }
    pub fn num_data(&self) -> usize {
        self.header.hr.num_data.assert_usize()
    }
    pub fn item_type_indices(&self, type_id: u16) -> ops::Range<usize> {
        for t in &self.item_types {
            if t.type_id as u16 == type_id {
                let start = t.start.assert_usize();
                let num = t.num.assert_usize();
                // Overflow check was in Reader::check().
                return start..start+num;
            }
        }
        0..0
    }
    pub fn item_type(&self, index: usize) -> u16 {
        self.item_types[index].type_id.assert_u16()
    }
    pub fn num_item_types(&self) -> usize {
        self.header.hr.num_item_types.assert_usize()
    }

    pub fn find_item(&self, type_id: u16, item_id: u16) -> Option<ItemView> {
        for item in self.item_type_items(type_id) {
            if item.id == item_id {
                return Some(item);
            }
        }
        None
    }

    pub fn debug_dump(&self, cb: &mut dyn CallbackReadData) -> Result<(), Error> {
        if !log_enabled!(log::LogLevel::Debug) {
            return Ok(())
        }
        debug!("DATAFILE");
        debug!("header: {:?}", self.header);

        for type_id in self.item_types() {
            debug!("item_type type_id={}", type_id);
            for item in self.item_type_items(type_id) {
                debug!("  item id={}", item.id);
                for &data in item.data {
                    #[cfg_attr(feature = "cargo-clippy", allow(identity_op))]
                    fn i32_to_bytes(input: i32) -> [u8; 4] { [
                        (((input >> 24) & 0xff) - 0x80) as u8,
                        (((input >> 16) & 0xff) - 0x80) as u8,
                        (((input >>  8) & 0xff) - 0x80) as u8,
                        (((input >>  0) & 0xff) - 0x80) as u8,
                    ] }
                    let bytes = i32_to_bytes(data);
                    debug!("    {:08x} {:11} {}{}{}{}", data, data,
                        sanitize_byte(bytes[0]),
                        sanitize_byte(bytes[1]),
                        sanitize_byte(bytes[2]),
                        sanitize_byte(bytes[3]),
                    )
                }
            }
        }
        for i in 0..self.num_data() {
            self.read_data(cb, i)?;
            let data = cb.data_buffer();
            let len = data.len();
            debug!("data id={} size={}", i, len);
            if len < 256 {
                for line in hexdump_iter(data) {
                    debug!("  {}", line);
                }
            }
        }
        Ok(())
    }

    pub fn items(&self) -> Items {
        fn map_fn<'a>(i: usize, self_: &mut &'a Reader) -> ItemView<'a> {
            self_.item(i)
        }
        MapIterator::new(self, 0..self.num_items(), map_fn)
    }
    pub fn item_types(&self) -> ItemTypes {
        fn map_fn<'a>(i: usize, self_: &mut &'a Reader) -> u16 {
            self_.item_type(i)
        }
        MapIterator::new(self, 0..self.num_item_types(), map_fn)
    }
    pub fn item_type_items(&self, type_id: u16) -> ItemTypeItems {
        fn map_fn<'a>(i: usize, self_: &mut &'a Reader) -> ItemView<'a> {
            self_.item(i)
        }
        MapIterator::new(self, self.item_type_indices(type_id), map_fn)
    }
}

pub type Items<'a> = MapIterator<ItemView<'a>, &'a Reader, ops::Range<usize>>;
pub type ItemTypes<'a> = MapIterator<u16, &'a Reader, ops::Range<usize>>;
pub type ItemTypeItems<'a> = MapIterator<ItemView<'a>,&'a Reader,ops::Range<usize>>;
