from common import ProxyNoClose, cached_property, file_chunks, namedstruct
from common_tw import crc32tw

from collections import namedtuple
from io import BytesIO
from weakref import WeakValueDictionary
from zlib import decompressobj

def get_file(filename__file):
	try:
		filename__file.read
		filename__file.seek
		filename__file.tell
	except AttributeError:
		return open(filename__file, 'rb')
	else:
		return ProxyNoClose(filename__file)

DatafileHeaderVersion = namedstruct('DatafileHeaderVersion', '<4si', 'magic version')
DatafileHeader = namedstruct('DatafileHeader', '<iiiiiii', 'size swaplen num_item_types num_items num_data size_items size_data')

DatafileItemType = namedstruct('DatafileItemType', '<iii', 'type_id start num')

DatafileItemHeader = namedstruct('DatafileItemHeader', '<ii', 'type_id__id size')
DatafileItemHeader.type_id = property(lambda self: (self.type_id__id&0xffff0000)>>16)
DatafileItemHeader.id = property(lambda self: self.type_id__id&0xffff)


DatafileInt = namedstruct('DatafileInt', '<i', 'value')

DatafileItem = namedtuple('DatafileItem', 'type_id id data')

MAGIC=b'DATA'
MAGIC_BIGENDIAN=b'ATAD'

class DatafileError(Exception):
	pass

class Datafile:
	def __init__(self, filename__file):
		try:
			self._file = get_file(filename__file)
			self.__initial_read()
		except Exception:
			self.close()
			raise
		self.name = self._file.name

	@cached_property
	def crc(self):
		self._file.seek(0)
		crc = crc32tw(b'')
		for chunk in file_chunks(self._file):
			crc = crc32tw(chunk, crc)
		return crc

	def close(self):
		self._file.close()

	def items(self):
		raise NotImplementedError

	def items_by_type(self, type_id):
		raise NotImplementedError

	def get_data(self, index):
		try:
			return self._uncomp_data[index]
		except KeyError:
			pass

		offset = self.data_offsets[index]
		if index < self.header.num_data:
			length = self.data_offsets[index + 1] - offset
		else:
			length = self.header.size_data - offset

		self._file.seek(offset)
		compressed_data = self._file.read(length)
		if len(compressed_data) < length:
			raise DatafileError("data incomplete, data={} wanted={} got={}".format(index, length, len(compressed_data)))

		d = decompressobj()
		uncompressed_data = d.decompress(compressed_data, self.data_sizes[index])
		if not d.eof:
			raise DatafileError("data size invalid, data={} wanted={}".format(index, self.data_sizes[index]))

		self._uncomp_data[index] = uncompressed_data
		return self._uncomp_data[index]

	def _get_item(self, index):
		raise NotImplementedError

	def __initial_read(self):
		self._offset = self._file.tell()

		self.header_ver = self.__read_struct(DatafileHeaderVersion)
		if self.header_ver.magic != MAGIC and self.header_ver.magic != MAGIC_BIGENDIAN:
			raise DatafileError("wrong datafile signature, magic={}".format(self.header_ver.magic))
		if not 3 <= self.header_ver.version <= 4:
			raise DatafileError("wrong datafile version, version={}".format(self.header_ver.version))

		self.header = self.__read_struct(DatafileHeader)
		self.__check_header(self.header)

		size = 0
		size += DatafileItemType._size * self.header.num_item_types # item_types
		size += DatafileInt._size * self.header.num_items # item_offsets
		size += DatafileInt._size * self.header.num_data # data_offsets
		if self.header_ver.version >= 4:
			size += DatafileInt._size * self.header.num_data # data_sizes (only version 4)
		size += self.header.size_items # items

		raw = self._file.read(size)
		if len(raw) < size:
			raise DatafileError("datafile too short, wanted={} got={}".format(size, len(raw)))

		next = raw
		def claim(size):
			nonlocal next
			result = next[:size]
			assert len(result) == size

			next = next[size:]
			return BytesIO(result)

		raw_item_types = claim(DatafileItemType._size * self.header.num_item_types)
		raw_item_offsets = claim(DatafileInt._size * self.header.num_items)
		raw_data_offsets = claim(DatafileInt._size * self.header.num_data)
		if self.header_ver.version >= 4:
			raw_uncomp_data_sizes = claim(DatafileInt._size * self.header.num_data)
		else:
			raw_uncomp_data_sizes = None
		raw_items = claim(self.header.size_items)
		assert len(next) == 0

		self.item_types = self.__unpack_item_types(raw_item_types, self.header.num_item_types, self.header.num_items)
		self.item_offsets = self.__unpack_item_offsets(raw_item_offsets, self.header.num_items)
		self.data_offsets = self.__unpack_data_offsets(raw_data_offsets, self.header.num_data)
		if raw_uncomp_data_sizes is not None:
			self.uncomp_data_sizes = self.__unpack_uncomp_data_sizes(raw_uncomp_data_sizes, self.header.num_data)
		else:
			self.uncomp_data_sizes = None
		self.items = self.__unpack_items(raw_items, self.header.num_items, self.item_offsets)

		self._uncomp_data = WeakValueDictionary()

	@staticmethod
	def __unpack_item_types(raw, num, num_items):
		item_types = []
		expected_start = 0
		for i in range(num):
			t = DatafileItemType._unpack(raw.read(DatafileItemType._size))

			if t.type_id not in range(0x10000):
				raise DatafileError("invalid item type id, item_type={} type_id={}".format(i, t.type_id))

			if t.start != expected_start:
				raise DatafileError("unexpected item type start, item_type={} type_id={} start={} expected={}".format(i, t.type_id, t.start, expected_start))

			if t.num not in range(num_items - t.start + 1):
				raise DatafileError("invalid item type num, item_type={} type_id={} num={}".format(i, t.type_id, t.num))

			# last item? check for full coverage
			if i == num - 1:
				if t.start + t.num != num_items:
					raise DatafileError("last item type does not contain last item, item_type={} type_id={}".format(i, t.type_id))
			expected_start = t.start + t.num

			# check for duplicate item type IDs
			for k, t2 in enumerate(item_types):
				if t.type_id == t2.type_id:
					raise DatafileError("item type id occurs twice, type_id={} item_type1={} item_type2={}".format(t.type_id, i, k))
			item_types.append(t)

	@staticmethod
	def __unpack_item_offsets(raw, num):
		item_offsets = []
		for i in range(num):
			o = DatafileInt._unpack(raw.read(DatafileInt._size)).value
			# item offset checking is performed in the item reading
			item_offsets.append(o)
		return item_offsets

	@staticmethod
	def __unpack_data_offsets(raw, num):
		data_offsets = []
		prev = 0
		for i in range(num):
			o = DatafileInt._unpack(raw.read(DatafileInt._size)).value
			if o < prev:
				raise DatafileError("data item has negative compressed size, data={}".format(i))
			data_offsets.append(o)
		return data_offsets

	@staticmethod
	def __unpack_uncomp_data_sizes(raw, num):
		uncomp_data_sizes = []
		for i in range(num):
			s = DatafileInt._unpack(raw.read(DatafileInt._size)).value
			if s < 0:
				raise DatafileError("data item has negative uncompressed size, data={}".format(i))
			uncomp_data_sizes.append(s)
		return uncomp_data_sizes

	@staticmethod
	def __unpack_items(raw, num, offsets):
		offset = 0
		items = []

		def read(required_size, name):
			nonlocal offset
			offset += required_size
			result = raw.read(required_size)
			if len(result) < required_size:
				raise DatafileError("{} out of bounds, item={} offset={}".format(name, i, offset))
			return result

		for i in range(num):
			if offset != offsets[i]:
				raise DatafileError("invalid item offset, item={} wanted={} got={}".format(i, offsets[i], offset))
			item_header = DatafileItemHeader._unpack(read(DatafileItemHeader._size, "item header"))
			if item_header.size < 0:
				raise DatafileError("item has negative size, item={}".format(i))
			item_data = read(item_header.size, "item data")
			items.append(DatafileItem(type_id=item_header.type_id, id=item_header.id, data=item_data))

		if len(raw.read(1)) > 0:
			raise DatafileError("last item not large enough")

		return items

	@staticmethod
	def __check_header(header):
		for name, value in vars(header):
			if value < 0:
				raise DatafileError("header field {} is negative, value={}".format(name, value))
		if header.size < header.swaplen:
			print(header)
			raise DatafileError("size is less than swaplen, size={} swaplen={}".format(header.size, header.swaplen))

	def __read_struct(self, struct):
		read = self._file.read(struct._size)
		if len(read) < struct._size:
			raise DatafileError("datafile too short for {}, wanted={} got={}".format(struct._name, struct._size, len(read)))
		return struct._unpack(read)

	def __enter__(self):
		return self

	def __exit__(self, type, value, traceback):
		self.close()

	def __del__(self):
		self.close()
