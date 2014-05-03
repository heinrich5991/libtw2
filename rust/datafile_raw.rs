#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![feature(macro_rules)]
#![feature(phase)]

#[phase(syntax, link)]
extern crate log;

extern crate oncecell;
extern crate zlib = "zlib_minimal";

use std::cell::RefCell;
use std::io::{File, IoResult, SeekSet};
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
	transmute_slice,
	transmute_mut_slice,
};

use oncecell::OnceCell;

mod bitmagic;

/// `try` for nested results
macro_rules! try2(
	($e:expr) => (match $e { Ok(Ok(e)) => e, Ok(Err(e)) => return Ok(Err(e)), Err(e) => return Err(e) })
)

/// `try` for the inner nested result
macro_rules! tryi(
	($e:expr) => (match $e { Ok(e) => e, Err(e) => return Ok(Err(e)) })
)

pub trait SeekReader {
	fn seek<'a>(&'a mut self) -> &'a mut Seek;
	fn reader<'a>(&'a mut self) -> &'a mut Reader;
}

impl SeekReader for File {
	fn seek<'a>(&'a mut self) -> &'a mut Seek {
		self as &'a mut Seek
	}
	fn reader<'a>(&'a mut self) -> &'a mut Reader {
		self as &'a mut Reader
	}
}

// --------------
// DATAFILE STUFF
// --------------

// FIXME: use #[deriving(Clone)]
//#[deriving(Clone)]
pub struct DatafileHeaderVersion {
	magic: [u8, ..4],
	version: i32,
}

#[deriving(Clone)]
pub struct DatafileHeader {
	_size: i32,
	_swaplen: i32,
	num_item_types: i32,
	num_items: i32,
	num_data: i32,
	size_items: i32,
	size_data: i32,
}

#[deriving(Clone)]
pub struct DatafileItemType {
	type_id: i32,
	start: i32,
	num: i32,
}

#[deriving(Clone)]
pub struct DatafileItemHeader {
	type_id__id: i32,
	size: i32,
}

#[deriving(Clone)]
pub struct DatafileItem<'a> {
	type_id: u16,
	id: u16,
	data: &'a [i32],
}

// A struct may only implement UnsafeDfOnlyI32 if it consists entirely of
// tightly packed i32 and does not have a destructor.
trait UnsafeDfOnlyI32 { }
impl UnsafeDfOnlyI32 for i32 { }
impl UnsafeDfOnlyI32 for DatafileHeaderVersion { }
impl UnsafeDfOnlyI32 for DatafileHeader { }
impl UnsafeDfOnlyI32 for DatafileItemType { }
impl UnsafeDfOnlyI32 for DatafileItemHeader { }


fn as_mut_i32_slice<'a, T:UnsafeDfOnlyI32>(x: &'a mut [T]) -> &'a mut [i32] {
	unsafe { transmute_mut_slice(x) }
}

fn read_as_le_i32s<T:UnsafeDfOnlyI32>(reader: &mut Reader) -> IoResult<T> {
	// this is safe as T is guaranteed by UnsafeDfOnlyI32 to be POD, which
	// means there won't be a destructor running over uninitialized
	// elements, even when returning early from the try!()
	let mut result = unsafe { mem::uninit() };
	try!(read_exact_le_ints(reader, as_mut_i32_slice(mut_ref_slice(&mut result))));
	Ok(result)
}

fn read_owned_vec_as_le_i32s<T:UnsafeDfOnlyI32>(reader: &mut Reader, count: uint) -> IoResult<Vec<T>> {
	let mut result = Vec::with_capacity(count);
	// this operation is safe by the same reasoning for the unsafe block in
	// `read_as_le_i32s`.
	unsafe { result.set_len(count); }
	try!(read_exact_le_ints(reader, as_mut_i32_slice(result.as_mut_slice())));
	Ok(result)
}

#[deriving(Eq, TotalEq, Show)]
pub enum DatafileErr {
	WrongMagic,
	UnsupportedVersion,
	MalformedHeader,
	Malformed,
	CompressionError,
}

pub static DATAFILE_MAGIC: &'static [u8] = bytes!("DATA");
pub static DATAFILE_MAGIC_BIGENDIAN: &'static [u8] = bytes!("ATAD");
pub static DATAFILE_VERSION3: i32 = 3;
pub static DATAFILE_VERSION4: i32 = 4;

pub static DATAFILE_ITEMTYPE_ID_RANGE: i32 = 0x10000;

pub type DfResult<T> = Result<T,DatafileErr>;

impl DatafileHeaderVersion {
	pub fn read_raw(reader: &mut Reader) -> IoResult<DatafileHeaderVersion> {
		let mut result: DatafileHeaderVersion = try!(read_as_le_i32s(reader));
		{
			let magic_view: &mut [i32] = unsafe { transmute_mut_slice(result.magic) };
			unsafe { to_little_endian(magic_view) };
		}
		Ok(result)
	}
	pub fn read(reader: &mut Reader) -> IoResult<DfResult<DatafileHeaderVersion>> {
		let result = try!(DatafileHeaderVersion::read_raw(reader));
		debug!("read header_ver={:?}", result);
		tryi!(result.check());
		Ok(Ok(result))
	}
	pub fn check(&self) -> DfResult<()> {
		Err(
			if self.magic != DATAFILE_MAGIC && self.magic != DATAFILE_MAGIC_BIGENDIAN {
				error!("wrong datafile signature, magic={:08x}",
					(self.magic[0] << 24)
					| (self.magic[1] << 16)
					| (self.magic[2] << 8)
					| (self.magic[3]));
				WrongMagic
			} else if self.version != DATAFILE_VERSION3 && self.version != DATAFILE_VERSION4 {
				error!("unsupported datafile version, version={:d}", self.version);
				UnsupportedVersion
			} else {
				return Ok(());
			}
		)
	}
}

impl DatafileHeader {
	pub fn read_raw(reader: &mut Reader) -> IoResult<DatafileHeader> {
		Ok(try!(read_as_le_i32s(reader)))
	}
	pub fn read(reader: &mut Reader) -> IoResult<DfResult<DatafileHeader>> {
		let result = try!(DatafileHeader::read_raw(reader));
		debug!("read header={:?}", result);
		tryi!(result.check());
		Ok(Ok(result))
	}
	pub fn check(&self) -> DfResult<()> {
		Err(
			if self._size < 0 {
				error!("_size is negative, _size={:d}", self._size);
				MalformedHeader
			} else if self._swaplen < 0 {
				error!("_swaplen is negative, _swaplen={:d}", self._swaplen);
				MalformedHeader
			} else if self.num_item_types < 0 {
				error!("num_item_types is negative, num_item_types={:d}", self.num_item_types);
				MalformedHeader
			} else if self.num_items < 0 {
				error!("num_items is negative, num_items={:d}", self.num_items);
				MalformedHeader
			} else if self.num_data < 0 {
				error!("num_data is negative, num_data={:d}", self.num_data);
				MalformedHeader
			} else if self.size_items < 0 {
				error!("size_items is negative, size_items={:d}", self.size_items);
				MalformedHeader
			} else if self.size_data < 0 {
				error!("size_data is negative, size_data={:d}", self.size_data);
				MalformedHeader
			} else if self.size_items as u32 % mem::size_of::<i32>() as u32 != 0 {
				error!("size_items not divisible by 4, size_items={:d}", self.size_items);
				MalformedHeader
			// TODO: make various check about size, swaplen (non-critical)
			} else {
				return Ok(())
			}
		)
	}
}

impl DatafileItemHeader {
	pub fn type_id(&self) -> u16 {
		(((self.type_id__id as u32) >> 16) & 0xffff) as u16
	}
	pub fn id(&self) -> u16 {
		((self.type_id__id as u32) & 0xffff) as u16
	}
	pub fn set_type_id__id(&mut self, type_id: u16, id: u16) {
		self.type_id__id = (((type_id as u32) << 16) | (id as u32)) as i32;
	}
}

pub struct MapIterator<T,U,D,I> {
	data: D,
	iterator: I,
	// `map` is already an function of an iterator, so we can't use `map` as a name here
	map_fn: fn (T, &D) -> U,
}

pub type DfItemIter<'a,T> = MapIterator<uint,DatafileItem<'a>,&'a T,iter::Range<uint>>;
pub type DfItemTypeIter<'a,T> = MapIterator<uint,u16,&'a T,iter::Range<uint>>;
pub type DfDataIter<'a,T> = MapIterator<uint,Result<&'a [u8],()>,&'a T,iter::Range<uint>>;

impl<T,U,D,I:Iterator<T>> Iterator<U> for MapIterator<T,U,D,I> {
	fn next(&mut self) -> Option<U> {
		self.iterator.next().map(|x| (self.map_fn)(x, &self.data))
	}
}

fn datafile_item_map_fn<'a,T:Datafile>(index: uint, df: &&'a T) -> DatafileItem<'a> {
	df.item(index)
}

fn datafile_item_type_map_fn<'a,T:Datafile>(index: uint, df: &&'a T) -> u16 {
	df.item_type(index)
}

fn datafile_data_map_fn<'a,T:Datafile>(index: uint, df: &&'a T) -> Result<&'a [u8],()> {
	df.data(index)
}

pub trait Datafile {
	// TODO: doc
	fn item_type(&self, index: uint) -> u16;
	fn num_item_types(&self) -> uint;

	fn item<'a>(&'a self, index: uint) -> DatafileItem<'a>;
	fn num_items(&self) -> uint;

	fn data<'a>(&'a self, index: uint) -> Result<&'a [u8],()>;
	fn num_data(&self) -> uint;

	fn item_type_indexes_start_num(&self, type_id: u16) -> (uint, uint);


	fn items<'a>(&'a self) -> MapIterator<uint,DatafileItem<'a>,&'a Self,iter::Range<uint>> {
		MapIterator { data: self, iterator: range(0, self.num_items()), map_fn: datafile_item_map_fn }
	}
	fn item_types<'a>(&'a self) -> MapIterator<uint,u16,&'a Self,iter::Range<uint>> {
		MapIterator { data: self, iterator: range(0, self.num_item_types()), map_fn: datafile_item_type_map_fn }
	}
	fn item_type_items<'a>(&'a self, type_id: u16) -> MapIterator<uint,DatafileItem<'a>,&'a Self,iter::Range<uint>> {
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

pub struct DatafileReader {
	header_ver: DatafileHeaderVersion,
	header: DatafileHeader,

	item_types: Vec<DatafileItemType>,
	item_offsets: Vec<i32>,
	data_offsets: Vec<i32>,
	uncomp_data_sizes: Option<Vec<i32>>,
	items_raw: Vec<i32>,

	data_offset: u64,
	uncomp_data: Vec<OnceCell<Result<Vec<u8>,()>>>,

	file: RefCell<~SeekReader>,
	// TODO: implement data read
}

impl DatafileReader {
	pub fn read(mut file: ~SeekReader) -> IoResult<DfResult<DatafileReader>> {
		let header_ver = try2!(DatafileHeaderVersion::read(file.reader()));
		let header = try2!(DatafileHeader::read(file.reader()));
		let item_types_raw = try!(read_owned_vec_as_le_i32s(file.reader(), header.num_item_types as uint));
		let item_offsets = try!(read_owned_vec_as_le_i32s(file.reader(), header.num_items as uint));
		let data_offsets = try!(read_owned_vec_as_le_i32s(file.reader(), header.num_data as uint));
		let uncomp_data_sizes = match header_ver.version {
			3 => None,
			4 => Some(try!(read_exact_le_ints_owned(file.reader(), header.num_data as uint))),
			_ => unreachable!(), // should have been caught in header_ver.check()
		};
		// possible failure of relative_size_of_mult should have been caught in header.check()
		let items_raw = try!(read_owned_vec_as_le_i32s(file.reader(), relative_size_of_mult::<u8,i32>(header.size_items as uint)));

		// TODO: FIXME: check for u64 -> i64 overflow
		let data_offset = try!(file.seek().tell());
		let uncomp_data = Vec::from_elem(header.num_data as uint, OnceCell::new());

		let result = DatafileReader {
			header_ver: header_ver,
			header: header,
			item_types: item_types_raw,
			item_offsets: item_offsets,
			data_offsets: data_offsets,
			uncomp_data_sizes: uncomp_data_sizes,
			items_raw: items_raw,
			data_offset: data_offset,
			uncomp_data: uncomp_data,
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
					error!("invalid item_type type_id: must be in range 0 to {:x}, item_type={:u} type_id={:d}", DATAFILE_ITEMTYPE_ID_RANGE, i, t.type_id)
					return Err(Malformed);
				}
				if !(0 <= t.num && t.num <= self.header.num_items - t.start) {
					error!("invalid item_type num: must be in range 0 to num_items - start + 1, item_type={:u} type_id={:d} start={:d} num={:d}", i, t.type_id, t.start, t.num);
					return Err(Malformed);
				}
				if t.start != expected_start {
					error!("item_types are not sequential, item_type={:u} type_id={:d} start={:d} expected={:d}", i, t.type_id, t.start, expected_start);
					return Err(Malformed);
				}
				expected_start += t.num;
				for (k, t2) in self.item_types.slice_to(i).iter().enumerate() {
					if t.type_id == t2.type_id {
						error!("item_type type_id occurs twice, type_id={:d} item_type1={:u} item_type2={:u}", t.type_id, i, k);
						return Err(Malformed);
					}
				}
			}
			if expected_start != self.header.num_items {
				error!("last item_type does not contain last item, item_type={:d}", self.header.num_item_types - 1);
				return Err(Malformed);
			}
		}
		{
			let mut offset = 0;
			for i in range(0, self.header.num_items as uint) {
				if self.item_offsets.as_slice()[i] < 0 {
					error!("invalid item offset (negative), item={:u} offset={:d}", i, self.item_offsets.as_slice()[i]);
					return Err(Malformed);
				}
				if offset != self.item_offsets.as_slice()[i] as uint {
					error!("invalid item offset, item={:u} offset={:d} wanted={:u}", i, self.item_offsets.as_slice()[i], offset);
					return Err(Malformed);
				}
				offset += mem::size_of::<DatafileItemHeader>();
				if offset > self.header.size_items as uint {
					error!("item header out of bounds, item={:u} offset={:u} size_items={:d}", i, offset, self.header.size_items);
					return Err(Malformed);
				}
				let item_header = self.item_header(i);
				if item_header.size < 0 {
					error!("item has negative size, item={:u} size={:d}", i, item_header.size);
					return Err(Malformed);
				}
				offset += item_header.size as uint;
				if offset > self.header.size_items as uint {
					error!("item out of bounds, item={:u} size={:d} size_items={:d}", i, item_header.size, self.header.size_items);
					return Err(Malformed);
				}
			}
			if offset != self.header.size_items as uint {
				error!("last item not large enough, item={:d} offset={:u} size_items={:d}", self.header.num_items - 1, offset, self.header.size_items);
				return Err(Malformed);
			}
		}
		{
			let mut previous = 0;
			for i in range(0, self.header.num_data as uint) {
				match self.uncomp_data_sizes {
					Some(ref uds) => {
						if uds.as_slice()[i] < 0 {
							error!("invalid data's uncompressed size, data={:u} uncomp_data_size={:d}", i, uds.as_slice()[i]);
							return Err(Malformed);
						}
					}
					None => (),
				}
				let offset = self.data_offsets.as_slice()[i];
				if offset < 0 || offset > self.header.size_data {
					error!("invalid data offset, data={:u} offset={:d}", i, offset);
					return Err(Malformed);
				}
				if previous > offset {
					error!("data overlaps, data1={:u} data2={:u}", i - 1, i);
					return Err(Malformed);
				}
				previous = offset;
			}
		}
		{
			for (i, t) in self.item_types.iter().enumerate() {
				for k in range(t.start as uint, (t.start + t.num) as uint) {
					let item_header = self.item_header(k);
					if item_header.type_id() != t.type_id as u16 {
						error!("item does not have right type_id, type={:u} type_id1={:d} item={:u} type_id2={:u}", i, t.type_id, k, item_header.type_id());
						return Err(Malformed);
					}
				}
			}
		}
		Ok(())
	}
	fn item_header<'a>(&'a self, index: uint) -> &'a DatafileItemHeader {
		let slice = self.items_raw
			.slice_from(relative_size_of_mult::<u8,i32>(self.item_offsets.as_slice()[index] as uint))
			.slice_to(relative_size_of::<DatafileItemHeader,i32>());
		// TODO: find out why paranthesis are necessary
		&(unsafe { transmute_slice::<i32,DatafileItemHeader>(slice) })[0]
	}
	fn data_size_file(&self, index: uint) -> uint {
		let start = self.data_offsets.as_slice()[index] as uint;
		let end = if index < self.data_offsets.len() - 1 {
			self.data_offsets.as_slice()[index + 1] as uint
		} else {
			self.header.size_data as uint
		};
		assert!(start <= end);
		end - start
	}
	fn uncomp_data_impl(&self, index: uint) -> IoResult<DfResult<Vec<u8>>> {
		let mut file = self.file.borrow_mut();
		try!(file.seek().seek(self.data_offset as i64 + self.data_offsets.as_slice()[index] as i64, SeekSet));

		let raw_data_len = self.data_size_file(index);
		let mut raw_data = Vec::with_capacity(raw_data_len);
		unsafe { raw_data.set_len(raw_data_len); }
		try!(file.reader().fill(raw_data.as_mut_slice()));

		match self.uncomp_data_sizes {
			Some(ref uds) => {
				let data_len = uds.as_slice()[index] as uint;
				let mut data = Vec::with_capacity(data_len);
				unsafe { data.set_len(data_len); }

				match zlib::uncompress(data.as_mut_slice(), raw_data.as_slice()) {
					Ok(len) if len == data.len() => {
						Ok(Ok(data))
					}
					Ok(len) => {
						error!("decompression error: wrong size, data={:u} size={:u} wanted={:u}", index, data.len(), len);
						Ok(Err(CompressionError))
					}
					_ => {
						error!("decompression error: zlib error");
						Ok(Err(CompressionError))
					}
				}
			},
			None => {
				Ok(Ok(raw_data))
			},
		}
	}
	pub fn debug_dump(&self) {
		debug!("DATAFILE");
		debug!("header_ver: {:?}", self.header_ver);
		debug!("header: {:?}", self.header);
		for type_id in self.item_types() {
			debug!("item_type type_id={:u}", type_id);
			for item in self.item_type_items(type_id) {
				debug!("\titem id={:u} data={:?}", item.id, item.data);
			}
		}
		for (i, data) in self.data_iter().enumerate() {
			let data = data.unwrap();
			debug!("data id={:u} size={:u}", i, data.len());
			if data.len() < 256 {
				match from_utf8(data) {
					Some(s) => debug!("\tstr={:s}", s),
					None => {},
				}
			}
		}
	}
}

impl Datafile for DatafileReader {
	fn item_type(&self, index: uint) -> u16 {
		self.item_types.as_slice()[index].type_id as u16
	}
	fn num_item_types(&self) -> uint {
		self.header.num_item_types as uint
	}

	fn item<'a>(&'a self, index: uint) -> DatafileItem<'a> {
		let item_header = self.item_header(index);
		let data = self.items_raw
			.slice_from(relative_size_of_mult::<u8,i32>(self.item_offsets.as_slice()[index] as uint))
			.slice_from(relative_size_of::<DatafileItemHeader,i32>())
			.slice_to(relative_size_of_mult::<u8,i32>(item_header.size as uint));
		DatafileItem {
			type_id: item_header.type_id(),
			id: item_header.id(),
			data: data,
		}
	}
	fn num_items(&self) -> uint {
		self.header.num_items as uint
	}

	fn data<'a>(&'a self, index: uint) -> Result<&'a [u8],()> {
		let self_uncomp_data_index = &self.uncomp_data.as_slice()[index];
		match self_uncomp_data_index.try_borrow() {
			// we have, return it
			Some(x) => match x {
				&Ok(ref y) => Ok(y.as_slice()),
				&Err(()) => Err(()),
			},
			// we don't have it, uncompress it from the file
			None => {
				let result: Result<Vec<u8>,()> = match self.uncomp_data_impl(index) {
					Ok(Ok(x)) => Ok(x),
					Ok(Err(x)) => {
						error!("datafile uncompression error {:?}", x);
						Err(())
					},
					Err(x) => {
						error!("IO error while uncompressing {}", x);
						Err(())
					},
				};
				self_uncomp_data_index.init(result);
				self_uncomp_data_index.borrow().as_ref().map(|x| x.as_slice()).map_err(|_| ())
			},
		}
	}
	fn num_data(&self) -> uint {
		self.header.num_data as uint
	}

	fn item_type_indexes_start_num(&self, type_id: u16) -> (uint, uint) {
		for t in self.item_types.iter() {
			if t.type_id as u16 == type_id {
				return (t.start as uint, t.num as uint);
			}
		}
		(0, 0)
	}
}
