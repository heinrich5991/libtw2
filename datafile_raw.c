#include "datafile_raw.h"
#include "datafile_format.h"

#include "compression.h"

#include <assert.h> // assert
#include <stddef.h>
#include <stdlib.h> // free, malloc
#include <zlib.h> // crc32, uncompress

// ============================================================================
// tw_dfr_error
// ============================================================================

static int tw_dfr_error_set(tw_dfr_error *error, int errno, const char *fmt, ...);
static int tw_dfr_error_set_v(tw_dfr_error *error, int errno, const char *fmt, va_list va_args);

static int tw_dfr_error_set(tw_dfr_error *error, int errno, const char *fmt, ...)
{
	va_list va_args;
	int result;

	va_start(va_args, fmt);
	result = tw_dfr_error_set_v(error, errno, fmt, va_args);
	va_end(va_args);

	return result;
}

static int tw_dfr_error_set_v(tw_dfr_error *error, int errno, const char *fmt, va_list va_args)
{
	error->errno_ = errno;
	tw_str_format_v(error->string, sizeof(error->string), fmt, va_args);
	return errno;
}


// ============================================================================
// tw_datafile_raw
// ============================================================================

struct tw_datafile_raw
{
	tw_dfr_callback_read read;
	tw_dfr_callback_filesize filesize;
	tw_dfr_callback_alloc alloc;
	tw_dfr_callback_free free;

	tw_dfr_header_ver header_ver;
	tw_dfr_header header;

	void *memory;

	tw_dfr_item_type *item_types;
	int32_t *item_offsets;
	int32_t *data_offsets;
	int32_t *uncomp_data_sizes;
	tw_byte *items_start;

	size_t data_start_offset;
	size_t size;
};

#define TW_DFR_ITEM_BYINDEX(dfr, index) (*(tw_dfr_item *)(&dfr->items_start[dfr->item_offsets[index]]))


static int tw_dfr_check_header(tw_dfr_header *header, tw_dfr_error *error);
static int tw_dfr_check(tw_datafile_raw *dfr, tw_dfr_error *error);


static int tw_dfr_check_header(tw_dfr_header *header, tw_dfr_error *error)
{
	const char *error_str = NULL;

	if(0) ;
	// first, check that all lengths are non-negative
	else if(header->size < 0) error_str = "size is negative";
	else if(header->swaplen < 0) error_str = "swaplen is negative";
	else if(header->num_item_types < 0) error_str = "number of item types is negative";
	else if(header->num_items < 0) error_str = "number of items is negative";
	else if(header->num_data < 0) error_str = "number of data is negative";
	else if(header->size_items < 0) error_str = "total items size is negative";
	else if(header->size_data < 0) error_str = "total data size is negative";
	// various checks
	else if(header->size_items % sizeof(int32_t) != 0) error_str = "item size not divisible by 4";
	else if(header->size < header->swaplen) error_str = "size is less than swaplen";

	if(error_str != NULL)
		return tw_dfr_error_set(error, TW_DFR_ERRNO_MALFORMEDHEADER, "malformed header (%s)", error_str);

	return 0;
}

static int tw_dfr_check(tw_datafile_raw *dfr, tw_dfr_error *error)
{
#define _TW_DFR_ERROR(...) return tw_dfr_error_set(error, TW_DFR_ERRNO_MALFORMED, __VA_ARGS__)
	{
		// check item types for sanity
		int i;
		for(i = 0; i < dfr->header.num_item_types; i++)
		{
			tw_dfr_item_type *t = &dfr->item_types[i];

			if(!(0 <= t->type_id && t->type_id < 0x10000))
				_TW_DFR_ERROR("invalid item type id: must be in range 0 to 0x10000, item_type=%d type_id=%d", i, t->type_id);
			if(!(0 <= t->num && t->num <= dfr->header.num_items - t->start))
				_TW_DFR_ERROR("invalid item type num: must be in range 0 to num_items - start + 1, item_type=%d type_id=%d start=%d num=%d", i, t->type_id, t->start, t->num);

			// TODO: not sure if one should require sequential item types
			int expected_start = 0;
			if(i > 0)
				expected_start = dfr->item_types[i - 1].start + dfr->item_types[i - 1].num;
			if(t->start != expected_start)
				_TW_DFR_ERROR("item types are not sequential, item_type=%d type_id=%d", i, t->type_id);

			if(i == dfr->header.num_item_types - 1)
				if(t->start + t->num != dfr->header.num_items)
					_TW_DFR_ERROR("last item type does not contain last item, item_type=%d type_id=%d", i, t->type_id);

			// check for duplicate item type IDs
			int k;
			for(k = 0; k < i; k++)
				if(t->type_id == dfr->item_types[k].type_id)
					_TW_DFR_ERROR("item type id occurs twice, type_id=%d item_type1=%d item_type2=%d", t->type_id, i, k);
		}
	}

	{
		// check items
		size_t offset = 0;
		int i;
		for(i = 0; i < dfr->header.num_items; i++)
		{
			if(offset % sizeof(int32_t) != 0)
				_TW_DFR_ERROR("item not aligned, item=%d offset=%d wantedalign=%d", i, offset, sizeof(int32_t));

			if(offset != (size_t)dfr->item_offsets[i])
				_TW_DFR_ERROR("invalid item offset, item=%d offset=%d wanted=%d", i, dfr->item_offsets[i], offset);

			if(offset + sizeof(tw_dfr_item) > (size_t)dfr->header.size_items)
				_TW_DFR_ERROR("item header out of bounds, item=%d offset=%d size_items=%d", i, offset);

			tw_dfr_item *item = &TW_DFR_ITEM_BYINDEX(dfr, i);
			if(item->size < 0)
				_TW_DFR_ERROR("item has negative size, item=%d", i);

			if(offset + sizeof(tw_dfr_item) + item->size > (size_t)dfr->header.size_items)
				_TW_DFR_ERROR("item out of bounds, item=%d offset=%d size=%d size_items=%d", i, offset, item->size, dfr->header.size_items);

			if(i == dfr->header.num_items - 1)
				if(offset + sizeof(tw_dfr_item) + item->size != (size_t)dfr->header.size_items)
					_TW_DFR_ERROR("last item not large enough, item=%d offset=%d size=%d size_items=%d", i, offset, item->size, dfr->header.size_items);

			offset += sizeof(tw_dfr_item) + item->size;
		}
	}

	{
		// check data
		int i;
		for(i = 0; i < dfr->header.num_data; i++)
		{
			if(dfr->uncomp_data_sizes)
				if(dfr->uncomp_data_sizes[i] < 0)
					_TW_DFR_ERROR("invalid data's uncompressed size, data=%d uncomp_data_size=%d", i, dfr->uncomp_data_sizes[i]);

			if(dfr->data_offsets[i] < 0 || dfr->data_offsets[i] > dfr->header.size_data)
				_TW_DFR_ERROR("invalid data offset, data=%d offset=%d", i, dfr->data_offsets[i]);

			if(i > 0)
				if(dfr->data_offsets[i - 1] > dfr->data_offsets[i])
					_TW_DFR_ERROR("data overlaps, data1=%d data2=%d", i - 1, i);
		}
	}

	{
		// check item types <-> items relation
		int i;
		for(i = 0; i < dfr->header.num_item_types; i++)
		{
			tw_dfr_item_type *t = &dfr->item_types[i];
			int k;
			for(k = t->start; k < t->start + t->num; k++)
			{
				tw_dfr_item *item = &TW_DFR_ITEM_BYINDEX(dfr, k);
				if(TW_DFR_ITEM__TYPE_ID(item->type_id__id) != t->type_id)
					_TW_DFR_ERROR("item does not have right type_id, type=%d type_id1=%d item=%d type_id2=%d", i, t->type_id, k, TW_DFR_ITEM__TYPE_ID(item->type_id__id));
			}
		}
	}


	return 0;
#undef _TW_DFR_ERROR
}


tw_datafile_raw *tw_dfr_create(void)
{
	tw_datafile_raw *ret = malloc(sizeof(*ret));
	return ret;
}

void tw_dfr_free(tw_datafile_raw *dfr)
{
	free(dfr);
	dfr = NULL;
}

void tw_dfr_callbacks_set(tw_datafile_raw *dfr,
	tw_dfr_callback_read read,
	tw_dfr_callback_filesize filesize,
	tw_dfr_callback_alloc alloc,
	tw_dfr_callback_free free
)
{
	if(read)
		dfr->read = read;
	if(filesize)
		dfr->filesize = filesize;
	if(alloc)
		dfr->alloc = alloc;
	if(free)
		dfr->free = free;
}

int tw_dfr_open(tw_datafile_raw *dfr, tw_dfr_error *error, void *userdata)
{
	{
		// read version-agnostic header
		size_t read = 0;
		if(dfr->read(&dfr->header_ver, 0, sizeof(dfr->header_ver), &read, userdata) != 0)
			return -1;

		if(read != sizeof(dfr->header_ver))
			return tw_dfr_error_set(error, TW_DFR_ERRNO_FILETOOSHORT, "datafile too short for version header");;
	}

	// check for magic bytes
	assert(sizeof(TW_DFR_MAGIC) == sizeof(dfr->header_ver.magic)
		&& sizeof(TW_DFR_MAGIC_BIGENDIAN) == sizeof(dfr->header_ver.magic)
		&& "magic bytes have wrong length");
	if(tw_mem_comp(dfr->header_ver.magic, TW_DFR_MAGIC, sizeof(dfr->header_ver.magic)) != 0
		&& tw_mem_comp(dfr->header_ver.magic, TW_DFR_MAGIC_BIGENDIAN, sizeof(dfr->header_ver.magic)) != 0)
		return tw_dfr_error_set(error, TW_DFR_ERRNO_WRONGMAGIC, "wrong datafile signature, magic=%08x", dfr->header_ver.magic);

	// header consists of little-endian ints
	tw_endian_fromlittle(&dfr->header_ver, sizeof(int32_t), sizeof(dfr->header_ver) / sizeof(int32_t));
	// fix magic bytes after endian-swap
	tw_mem_copy(dfr->header_ver.magic, TW_DFR_MAGIC, sizeof(dfr->header_ver.magic));

	// check version - accept version 3 and 4
	if(dfr->header_ver.version != 3 && dfr->header_ver.version != 4)
		return tw_dfr_error_set(error, TW_DFR_ERRNO_UNSUPPORTEDVERSION, "unsupported datafile version, version=%d", dfr->header_ver.version);

	{
		// read version-dependent header
		size_t read = 0;
		if(dfr->read(&dfr->header, sizeof(dfr->header_ver), sizeof(dfr->header), &read, userdata) != 0)
			return -1;

		if(read != sizeof(dfr->header))
			return tw_dfr_error_set(error, TW_DFR_ERRNO_FILETOOSHORT, "datafile too short for header v3/v4");
	}

	// version-dependent header also consists of little-endian ints
	tw_endian_fromlittle(&dfr->header, sizeof(int32_t), sizeof(dfr->header) / sizeof(int32_t));

	if(tw_dfr_check_header(&dfr->header, error) != 0)
		return error->errno_;

	size_t readsize = 0;
	{
		// use this type to detect potential overflows
		uint64_t size = 0;
		// allocate data file struct, layout is mostly the same as in file
		// read item_types, item_offsets, data_offsets, data_sizes for version 4 and items
		// also allocate pointers for the uncompressed data
		size += sizeof(tw_dfr_item_type) * dfr->header.num_item_types; // item_types
		size += sizeof(int32_t) * dfr->header.num_items; // item_offsets
		size += sizeof(int32_t) * dfr->header.num_data; // data_offsets
		if(dfr->header_ver.version >= 4)
			size += sizeof(int32_t) * dfr->header.num_data; // data_sizes (only version 4)
		size += dfr->header.size_items; // items

		// potential overflow, detected later
		readsize = size; // read everything up to now directly from the file into the memory

		size += sizeof(tw_dfr_header_ver);
		size += sizeof(tw_dfr_header);

		// potential overflow
		dfr->data_start_offset = size; // offset of the data in the datafile

		size += dfr->header.size_data;

		dfr->size = size; // size of the complete datafile
		if(dfr->size != size)
			return tw_dfr_error_set(error, TW_DFR_ERRNO_MALFORMEDHEADER, "malformed header (total size overflows)");
	}

	{
		// check that the file is complete
		size_t filesize = 0;
		if(dfr->filesize(&filesize, userdata) != 0)
			return -1;

		if(filesize < dfr->size)
			return tw_dfr_error_set(error, TW_DFR_ERRNO_FILETOOSHORT, "datafile too short, size=%d wanted=%d", filesize, dfr->size);
	}

	// allocate the memory
	dfr->memory = NULL;
	if(dfr->alloc(&dfr->memory, readsize, userdata) != 0)
		return -1;

	{
		// read everything except the data
		size_t read = 0;
		if(dfr->read(dfr->memory, sizeof(tw_dfr_header_ver) + sizeof(tw_dfr_header), readsize, &read, userdata) != 0)
		{
			dfr->free(dfr->memory, userdata);
			return -1;
		}

		if(read != readsize)
		{
			dfr->free(dfr->memory, userdata);
			return tw_dfr_error_set(error, TW_DFR_ERRNO_FILETOOSHORT, "datafile too short for items");
		}
	}

	// everything up to the items is little-endian 32bit ints
	tw_endian_fromlittle(dfr->memory, sizeof(int32_t), readsize / sizeof(int32_t));

	{
		// set up pointers
		void *next = dfr->memory;

		// `dfr->item_types` gets the first chunk of `dfr->memory`
		// the chunk's size is `sizeof(tw_dfr_item_type) * dfr->header.num_item_types`
		// thus the length of the dfr->item_types array `dfr->header.num_item_types`
		// it contains type descriptions in the `tw_dfr_item_type` format
		dfr->item_types = next; next = &dfr->item_types[dfr->header.num_item_types];

		// next chunk, starting after the end of `dfr->item_types`
		// size is `sizeof(int32_t) * dfr->header.num_items`
		// thus the length of the array is `dfr->header.num_items`
		dfr->item_offsets = next; next = &dfr->item_offsets[dfr->header.num_items];

		// next chunk, starting after the end of `dfr->item_offsets`
		// size is `sizeof(int32_t) * dfr->header.num_data`
		// thus the length of the array is `dfr->header.num_data`
		dfr->data_offsets = next; next = &dfr->data_offsets[dfr->header.num_data];

		// next chunk, if the version is greater or equal to 4
		// starting after the end of the last chunk
		// size is `sizeof(int32_t) * dfr->header.num_data`
		// if the version is less than 4, the field is NULLed, so if it is not NULL
		// the length of the array is `dfr->header.num_data`
		if(dfr->header_ver.version >= 4)
		{
			dfr->uncomp_data_sizes = next; next = &dfr->uncomp_data_sizes[dfr->header.num_data];
		}
		else
			dfr->uncomp_data_sizes = NULL;

		// last chunk that is read from the file, starting after the last chunk (which depends on the version)
		// size is `dfr->header.size_items`
		// thus the length of the array is `dfr->header.size_items`
		dfr->items_start = next; next = &dfr->items_start[dfr->header.size_items];
	}

	if(tw_dfr_check(dfr, error) != 0)
	{
		dfr->free(dfr->memory, userdata);
		dfr->memory = NULL;
		return error->errno_;
	}

	return 0;
}

int tw_dfr_close(tw_datafile_raw *dfr, tw_dfr_error *error, void *userdata)
{
	(void)error;
	dfr->free(dfr->memory, userdata);
	dfr->memory = NULL;
	return 0;
}

int tw_dfr_data_read(tw_datafile_raw *dfr, void **data_o, size_t *data_size_o, int index, tw_dfr_error *error, void *userdata)
{
	*data_o = NULL;
	*data_size_o = 0;

	// check the index
	if(!(0 <= index && index < dfr->header.num_data))
		return tw_dfr_error_set(error, TW_DFR_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);

	// determine data offset in file
	size_t data_offset = dfr->data_start_offset + dfr->data_offsets[index];

	// determining read size
	size_t data_size;
	if(index < dfr->header.num_data - 1)
		data_size = dfr->data_offsets[index + 1] - dfr->data_offsets[index];
	else
		data_size = dfr->header.size_data - dfr->data_offsets[index];

	// allocate space for the data
	void *data = NULL;
	if(dfr->alloc(&data, data_size, userdata) != 0)
		return -1;

	{
		// read the data
		size_t read = 0;
		if(dfr->read(data, data_offset, data_size, &read, userdata) != 0)
		{
			dfr->free(data, userdata);
			return -1;
		}

		if(read != data_size)
		{
			dfr->free(data, userdata);
			return tw_dfr_error_set(error, TW_DFR_ERRNO_FILETOOSHORT, "could not read data, data=%d offset=%d size=%d", index, data_offset, data_size);
		}
	}

	// is the data compressed?
	if(dfr->uncomp_data_sizes)
	{
		void *compressed = data;
		size_t compressed_size = data_size;

		// allocate space for the uncompressed data
		data_size = dfr->uncomp_data_sizes[index];
		data = NULL;
		if(dfr->alloc(&data, data_size, userdata) != 0)
		{
			dfr->free(compressed, userdata);
			return -1;
		}

		{
			size_t wanted_size = data_size;
			int zlib_err = uncompress(data, &wanted_size, compressed, compressed_size); // zlib.h

			// throw away the compressed data
			dfr->free(compressed, userdata);
			compressed = NULL;
			compressed_size = 0;

			// check for errors
			if(zlib_err != Z_OK || wanted_size != data_size)
			{
				if(zlib_err != Z_OK)
					tw_dfr_error_set(error, TW_DFR_ERRNO_DATAUNCOMPRESS, "could not uncompress data, data=%d size=%d zlib_err=%d", index, data_size, zlib_err);
				else
					tw_dfr_error_set(error, TW_DFR_ERRNO_DATAUNCOMPRESS, "uncompressed data too short, data=%d size=%d wanted=%d", index, data_size, wanted_size);

				dfr->free(data, userdata);
				data = NULL;
				data_size = 0;
				return error->errno_;
			}
		}
	}

	*data_o = data;
	*data_size_o = data_size;
	return 0;
}

int tw_dfr_num_data(tw_datafile_raw *dfr, int *num, tw_dfr_error *error, void *userdata)
{
	(void)error;
	(void)userdata;
	*num = dfr->header.num_data;
	return 0;
}

int tw_dfr_item_read(tw_datafile_raw *dfr, int32_t **item, size_t *item_count, int *type_id, int *id, int index, tw_dfr_error *error, void *userdata)
{
	(void)error;
	(void)userdata;

	*item = NULL;
	*item_count = 0;

	// check the index
	if(!(0 <= index && index < dfr->header.num_items))
		return tw_dfr_error_set(error, TW_DFR_ERRNO_OUTOFRANGE, "item index out of range, item=%d", index);

	// load the item
	tw_dfr_item *item_header = &TW_DFR_ITEM_BYINDEX(dfr, index);

	// fill output parameters
	if(type_id)
		*type_id = TW_DFR_ITEM__TYPE_ID(item_header->type_id__id);
	if(id)
		*id = TW_DFR_ITEM__ID(item_header->type_id__id);

	assert(item_header->size % sizeof(int32_t) == 0 && "item not aligned");

	*item_count = item_header->size / sizeof(int32_t);
	*item = (void *)&item_header[1];

	return 0;
}

int tw_dfr_item_find(tw_datafile_raw *dfr, int32_t **item_o, size_t *item_count_o, int type_id, int id, tw_dfr_error *error, void *userdata)
{
	*item_o = NULL;
	*item_count_o = 0;

	int32_t *item = NULL;
	size_t item_count = 0;

	int start;
	int num;

	{
		// get the indexes of the items of the given type
		int result = tw_dfr_type_indexes(dfr, &start, &num, type_id, error, userdata);
		if(result != 0)
			return result;
	}

	{
		// iterate through said indexes
		int i;
		for(i = start; i < start + num; i++)
		{
			int id2;

			{
				// get the item
				int result = tw_dfr_item_read(dfr, &item, &item_count, NULL, &id2, i, error, userdata);
				if(result != 0)
					return result;
			}

			// have we found the item?
			if(id == id2)
			{
				*item_o = item;
				*item_count_o = item_count;
				return 0;
			}

			item = NULL;
			item_count = 0;
		}
	}

	// nothing found, return
	*item_o = NULL;
	*item_count_o = 0;
	return 0;
}

int tw_dfr_num_items(tw_datafile_raw *dfr, int *num, tw_dfr_error *error, void *userdata)
{
	(void)error;
	(void)userdata;

	*num = dfr->header.num_items;
	return 0;
}

int tw_dfr_type_indexes(tw_datafile_raw *dfr, int *start, int *num, int type_id, tw_dfr_error *error, void *userdata)
{
	(void)error;
	(void)userdata;

	*start = -1;
	*num = 0;

	// loop through the item types and look for the right type_id
	int i;
	for(i = 0; i < dfr->header.num_item_types; i++)
	{
		if(type_id == dfr->item_types[i].type_id)
		{
			// if you found it, fill the output parameters and return
			*start = dfr->item_types[i].start;
			*num = dfr->item_types[i].num;
			return 0;
		}
	}
	return 0;
}

int tw_dfr_crc_calc(tw_datafile_raw *dfr, tw_dfr_crc *crc_o, tw_dfr_error *error, void *userdata)
{
	(void)error;

	// NOTE: A proper implementation would only compute the checksum on the
	//       actual datafile, however in order to provide compatiblity with
	//       the reference implementation this crude behavior is actually
	//       wanted.

	*crc_o = 0;

	tw_dfr_crc crc = 0;

	tw_byte buf[TW_BUFSIZE];
	size_t pos = 0;
	while(1)
	{
		size_t read = 0;
		if(dfr->read(buf, pos, sizeof(buf), &read, userdata) != 0)
			return -1;
		crc = crc32(crc, buf, read); // zlib.h
		if(read != sizeof(buf))
			break;
	}

	*crc_o = crc;
	return 0;
}

int tw_dfr_dump(tw_datafile_raw *dfr, tw_dfr_error *error, void *userdata)
{
	(void)dfr;
	(void)userdata;
	return tw_dfr_error_set(error, TW_DFR_ERRNO_NOTIMPLEMENTED, "tw_dfr_dump not implemented");
}

/*
void tw_dfr_dump_header_ver(tw_dfr_header_ver *header_ver)
{
	printf("magic=0x%08x version=%d\n", *(int *)header_ver->magic, header_ver->version);
}

void tw_dfr_dump_header(tw_dfr_header *header)
{
	printf("size=%d swaplen=%d num_item_types=%d num_items=%d num_data=%d size_items=%d size_data=%d\n", header->size, header->swaplen, header->num_item_types, header->num_items, header->num_data, header->size_items, header->size_data);
}

void tw_dfr_dump_item_type(tw_dfr_item_type *type)
{
	printf("type_id=%d start=%-2d num=%-2d\n", type->type_id, type->start, type->num);
}

void tw_dfr_dump_item(tw_dfr_item *item)
{
	printf("type_id=%d id=%-2d size=%-3d\n", TW_DFR_ITEM__TYPE_ID(item->type_id__id), TW_DFR_ITEM__ID(item->type_id__id), item->size);
}

void tw_dfr_dump(tw_datafile *dfr)
{
	tw_dfr_dump_header_ver(&dfr->header_ver);
	tw_dfr_dump_header(&dfr->header);
	printf("\n");

	int i;
	for(i = 0; i < dfr->header.num_item_types; i++)
	{
		printf("type=%d ", i);
		tw_dfr_dump_item_type(&dfr->item_types[i]);
	}

	printf("\n");

	for(i = 0; i < dfr->header.num_item_types; i++)
	{
		tw_dfr_item_type *t = &dfr->item_types[i];

		printf("type=%d ", i);
		tw_dfr_dump_item_type(t);

		int k;
		for(k = t->start; k < t->start + t->num; k++)
		{
			printf("\titem=%-2d ", k);
			tw_dfr_dump_item(&TW_DFR_ITEM_INDEX(dfr, k));
		}
	}
}
*/
