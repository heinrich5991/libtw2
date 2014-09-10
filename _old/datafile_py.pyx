from datafile_raw cimport *

from libc.stdint cimport uintptr_t
from libc.string cimport memcpy

from collections import namedtuple

cimport cython

cdef extern from "Python.h":
	char* PyByteArray_AsString(bytearray)

cdef int callback_read(void *buffer, size_t start, size_t buffer_size, size_t *read, void *userdata) except 1:
	cdef DatafileRaw self = <object>userdata
	cdef uintptr_t buffer_id = <uintptr_t>buffer

	self._file.seek(start)
	if buffer_id in self._memory:
		read[0] = self._file.readinto(self._memory[buffer_id])
	else:
		data = self._file.read(buffer_size)
		read[0] = len(data)
		memcpy(buffer, <char *>data, len(data))
	return 0

cdef int callback_filesize(size_t *filesize, void *userdata) except 1:
	cdef DatafileRaw self = <object>userdata
	self._file.seek(0, 2) # seek to the end
	filesize[0] = self._file.tell() # return position
	return 0

cdef int callback_alloc(void **result, size_t size, void *userdata) except 1:
	cdef DatafileRaw self = <object>userdata
	cdef memory = bytearray(size)

	#cython 0.20
	#result[0] = <char *>memory
	#cython 0.19
	result[0] = PyByteArray_AsString(memory)

	cdef uintptr_t result_id = <uintptr_t>result[0]
	self._memory[result_id] = memory
	return 0

cdef void callback_free(void *ptr, void *userdata):
	cdef DatafileRaw self = <object>userdata
	cdef uintptr_t ptr_id = <uintptr_t>ptr
	del self._memory[ptr_id]

class DatafileError(Exception): pass
class DatafileFileTooShortError(DatafileError): pass
class DatafileWrongMagicError(DatafileError): pass
class DatafileUnsupportedVersionError(DatafileError): pass
class DatafileMalformedHeaderError(DatafileError): pass
class DatafileMalformedError(DatafileError): pass
class DatafileDataUncompressError(DatafileError): pass

cdef int handle_error(int errno, tw_dfr_error *error) except 1:
	if errno == 0:
		return 0

	if errno > 0:
		err = DatafileError

		if False: pass
		elif errno == TW_DFR_ERRNO_FILETOOSHORT: err = DatafileFileTooShortError
		elif errno == TW_DFR_ERRNO_WRONGMAGIC: err = DatafileWrongMagicError
		elif errno == TW_DFR_ERRNO_UNSUPPORTEDVERSION: err = DatafileUnsupportedVersionError
		elif errno == TW_DFR_ERRNO_MALFORMEDHEADER: err = DatafileMalformedHeaderError
		elif errno == TW_DFR_ERRNO_MALFORMED: err = DatafileMalformedError
		elif errno == TW_DFR_ERRNO_OUTOFRANGE: err = IndexError
		elif errno == TW_DFR_ERRNO_DATAUNCOMPRESS: err = DatafileDataUncompressError
		elif errno == TW_DFR_ERRNO_NOTIMPLEMENTED: err = NotImplementedError

		raise err
	else: # errno < 0
		return 1 # error already raised, make cython aware of it

class DatafileItem(namedtuple('DatafileItem', 'type_id id data')):
	pass

@cython.freelist(8)
cdef class DatafileRaw:
	cdef tw_datafile_raw *_dfr
	cdef _memory
	cdef _file

	def __cinit__(self):
		self._dfr = tw_dfr_create()
		if self._dfr is NULL:
			raise MemoryError()
		tw_dfr_callbacks_set(
			self._dfr,
			<tw_dfr_callback_read>callback_read,
			<tw_dfr_callback_filesize>callback_filesize,
			<tw_dfr_callback_alloc>callback_alloc,
			<tw_dfr_callback_free>callback_free
		)
		self._memory = {}
		self._file = None

	def __dealloc__(self):
		if self._file is not None:
			self.close()

		if self._dfr is not NULL:
			tw_dfr_free(self._dfr)
			self._dfr = NULL

	def open(self, file):
		cdef tw_dfr_error error

		self._file = file
		try:
			handle_error(tw_dfr_open(self._dfr, &error, <void *>self), &error)
		except:
			self._file = None
			raise

	def close(self):
		cdef tw_dfr_error error

		try:
			handle_error(tw_dfr_close(self._dfr, &error, <void *>self), &error)
		finally:
			self._file = None

	def data(self, int index):
		cdef tw_dfr_error error
		cdef void *data
		cdef size_t data_size

		handle_error(tw_dfr_data_read(self._dfr, &data, &data_size, index, &error, <void *>self), &error)

		cdef uintptr_t data_id = <uintptr_t>data

		result = self._memory[data_id][:data_size]
		del self._memory[data_id]
		return result

	def num_data(self):
		cdef tw_dfr_error error
		cdef int num

		handle_error(tw_dfr_num_data(self._dfr, &num, &error, <void *>self), &error)

		return num

	def item(self, int index):
		cdef tw_dfr_error error
		cdef int32_t *item
		cdef size_t item_count
		cdef int type_id
		cdef int id

		handle_error(tw_dfr_item_read(self._dfr, &item, &item_count, &type_id, &id, index, &error, <void *>self), &error)

		return DatafileItem(type_id, id, tuple(item[i] for i in range(item_count)))

	def item_find(self, int type_id, int id):
		cdef tw_dfr_error error
		cdef int32_t *item
		cdef size_t item_count

		handle_error(tw_dfr_item_find(self._dfr, &item, &item_count, type_id, id, &error, <void *>self), &error)

		if item is NULL:
			raise IndexError(type_id, id)

		return DatafileItem(type_id, id, tuple(item[i] for i in range(item_count)))

	def num_items(self):
		cdef tw_dfr_error error
		cdef int num

		handle_error(tw_dfr_num_items(self._dfr, &num, &error, <void *>self), &error)

		return num

	def type_indexes(self, int type_id):
		cdef tw_dfr_error error
		cdef int start
		cdef int num

		handle_error(tw_dfr_type_indexes(self._dfr, &start, &num, type_id, &error, <void *>self), &error)

		return range(start, start + num)

	def crc_calc(self):
		cdef tw_dfr_error error
		cdef tw_dfr_crc crc

		handle_error(tw_dfr_crc_calc(self._dfr, &crc, &error, <void *>self), &error)

		return <object>crc
