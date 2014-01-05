from datafile_py import *

from weakref import WeakValueDictionary

class Datafile:
	def __init__(self, file):
		self._file, self._file_owned = _get_file(file)
		self.name = self._file.name

		self._dfr = DatafileRaw()
		self._dfr.open(self._file)

		self._crc = None

		self._data = {}

	def __enter__(self):
		return self

	def __exit__(self, type, value, traceback):
		self.close()

	def __repr__(self):
		return '<Datafile {!r}>'.format(self.name)

	@property
	def crc(self):
		if self._crc is None:
			self._crc = self._dfr.crc_calc()
		return self._crc

	@property
	def data(self):
		return _DatafileData(self)

	@property
	def types(self):
		return _DatafileTypes(self)

	@property
	def items(self):
		return _DatafileItems(self)

	def close(self):
		self._dfr.close()
		self._dfr = None
		if self._file_owned:
			self._file.close()
		self._file = None

class _DatafileItems:
	def __init__(self, df):
		self._df = df
	def __getitem__(self, index):
		return self._df._dfr.item(index)
	def __iter__(self):
		return (self[i] for i in range(len(self)))
	def __len__(self):
		return self._df._dfr.num_items()

class _DatafileTypeItems:
	def __init__(self, df, type_id):
		self._df = df
		self._type_id = type_id
	def __getitem__(self, id_):
		return self._df._dfr.item_find(self._type_id, id_)
	def __iter__(self):
		return (self._df._dfr.item(i) for i in self._df._dfr.type_indexes(self._type_id))
	def __len__(self):
		return len(self._df._dfr.type_indexes(self._type_id))

class _DatafileTypes:
	def __init__(self, df):
		self._df = df
	def __getitem__(self, type_id):
		return _DatafileTypeItems(self._df, type_id)

class _DatafileData:
	def __init__(self, df):
		self._df = df
	def __getitem__(self, index):
		try:
			return self._df._data[index]
		except KeyError:
			d = self._df._dfr.data(index)
			self._df._data[index] = d
			return d
	def drop(self, index):
		del self._df._data[index]
	def __iter__(self):
		return (self[i] for i in range(len(self)))
	def __len__(self):
		return self._df._dfr.num_data()

def _get_file(file, mode='rb'):
	try:
		file.read
		file.seek
		file.tell
	except AttributeError:
		return open(file, mode), True
	else:
		return file, False
