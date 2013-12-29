#include "datafile.h"

#include "compression.h"
#include "error.h"
#include "io.h"

#include <assert.h>
#include <inttypes.h>
#include <stdlib.h>

#include "datafile_format.h"

/*typedef struct tw_dfr_header_ver
{
	tw_byte magic[4];
	int32_t version;
} tw_dfr_header_ver;

typedef struct tw_dfr_header
{
	int32_t size;
	int32_t swaplen;
	int32_t num_item_types;
	int32_t num_items;
	int32_t num_data;
	int32_t size_items;
	int32_t size_data;
} tw_dfr_header;

typedef struct tw_dfr_item_type
{
	int32_t type_id;
	int32_t start;
	int32_t num;
} tw_dfr_item_type;

typedef struct tw_dfr_item
{
	int32_t type_id__id;
	int32_t size;
} tw_dfr_item;

#define TW_DFR_ITEM__TYPE_ID(type_id__id) ((type_id__id >> 16) & 0xffff)
#define TW_DFR_ITEM__ID(type_id__id) (type_id__id & 0xffff)
#define TW_DFR_ITEM__TYPE_ID__ID(type_id, id) ((type_id << 16) | id)
*/

struct tw_datafile
{
	tw_io *file;
	long file_offset;

	uint32_t crc;
	int crc_calc;
	tw_dfr_header_ver header_ver;
	tw_dfr_header header;

	long size;
	long data_start_offset;

	void *alloc;
	tw_dfr_item_type *item_types;
	int *item_offsets;
	int *data_offsets;
	int *uncomp_data_sizes;
	tw_byte *items_start;
	void **uncomp_data;
};

#define TW_DF_ITEM_INDEX(df, index) (*(tw_dfr_item *)(&df->items_start[df->item_offsets[index]]))

static const tw_byte TW_DF_MAGIC[] = {'D', 'A', 'T', 'A'};
static const tw_byte TW_DF_MAGIC_BIGENDIAN[] = {'A', 'T', 'A', 'D'};

static int tw_df_open_read_check_header(tw_dfr_header *header)
{
	const char *error = NULL;

	if(0) ;
	// first, check that all lengths are non-negative
	else if(header->size < 0) error = "size is negative";
	else if(header->swaplen < 0) error = "swaplen is negative";
	else if(header->num_item_types < 0) error = "number of item types is negative";
	else if(header->num_items < 0) error = "number of items is negative";
	else if(header->num_data < 0) error = "number of data is negative";
	else if(header->size_items < 0) error = "total items size is negative";
	else if(header->size_data < 0) error = "total data size is negative";
	// various checks
	else if(header->size < header->swaplen) error = "size is less than swaplen";

	if(error != NULL)
	{
		tw_error_set(TW_ERRNO_DF_OPEN, "malformed datafile header: %s", error);
		return 1;
	}

	return 0;
}

// NOTE: modifies df->file cursor position
static int tw_df_open_read_check(tw_datafile *df)
{
#define _TW_DF_ERROR(...) do { tw_error_set(TW_ERRNO_DF_OPEN, __VA_ARGS__); return 1; } while(0)
	int i;
	// check item types for sanity
	for(i = 0; i < df->header.num_item_types; i++)
	{
		tw_dfr_item_type *t = &df->item_types[i];

		if(!(0 <= t->type_id && t->type_id < 0x10000))
			_TW_DF_ERROR("invalid item type id: must be in range 0 to 0x10000, item_type=%d type_id=%d", i, t->type_id);
		if(!(0 <= t->num && t->num <= df->header.num_items - t->start))
			_TW_DF_ERROR("invalid item type num: must be in range 0 to num_items - start + 1, item_type=%d type_id=%d start=%d num=%d", i, t->type_id, t->start, t->num);

		// TODO: not sure if one should require sequential item types
		int expected_start = 0;
		if(i > 0)
			expected_start = df->item_types[i - 1].start + df->item_types[i - 1].num;
		if(t->start != expected_start)
			_TW_DF_ERROR("item types are not sequential, item_type=%d type_id=%d", i, t->type_id);

		if(i == df->header.num_item_types - 1)
			if(t->start + t->num != df->header.num_items)
				_TW_DF_ERROR("last item type does not contain last item, item_type=%d type_id=%d", i, t->type_id);

		// check for duplicate item type IDs
		int k;
		for(k = 0; k < i; k++)
			if(t->type_id == df->item_types[k].type_id)
				_TW_DF_ERROR("item type id occurs twice, type_id=%d item_type1=%d item_type2=%d", t->type_id, i, k);
	}

	// check items
	size_t offset = 0;
	for(i = 0; i < df->header.num_items; i++)
	{
		if(offset != (size_t)df->item_offsets[i])
			_TW_DF_ERROR("invalid item offset, item=%d offset=%d", i, df->item_offsets[i]);

		if(offset + sizeof(tw_dfr_item) > (size_t)df->header.size_items)
			_TW_DF_ERROR("item header out of bounds, item=%d offset=%d size_items=%d", i, offset);

		tw_dfr_item *item = &TW_DF_ITEM_INDEX(df, i);
		if(item->size < 0)
			_TW_DF_ERROR("item has negative size, item=%d", i);

		if(offset + sizeof(tw_dfr_item) + item->size > (size_t)df->header.size_items)
			_TW_DF_ERROR("item out of bounds, item=%d offset=%d size=%d size_items=%d", i, offset, item->size, df->header.size_items);

		if(i == df->header.num_items - 1)
			if(offset + sizeof(tw_dfr_item) + item->size != (size_t)df->header.size_items)
				_TW_DF_ERROR("last item not large enough, item=%d offset=%d size=%d size_items=%d", i, offset, item->size, df->header.size_items);

		offset += sizeof(tw_dfr_item) + item->size;
	}

	// check data
	for(i = 0; i < df->header.num_data; i++)
	{
		if(df->uncomp_data_sizes)
			if(df->uncomp_data_sizes[i] < 0)
				_TW_DF_ERROR("invalid data's uncompressed size, data=%d uncomp_data_size=%d", i, df->uncomp_data_sizes[i]);

		if(df->data_offsets[i] < 0 || df->data_offsets[i] > df->header.size_data)
			_TW_DF_ERROR("invalid data offset, data=%d offset=%d", i, df->data_offsets[i]);

		if(i > 0)
			if(df->data_offsets[i - 1] > df->data_offsets[i])
				_TW_DF_ERROR("data overlaps, data1=%d data2=%d", i - 1, i);
	}

	// check item types <-> items relation
	for(i = 0; i < df->header.num_item_types; i++)
	{
		tw_dfr_item_type *t = &df->item_types[i];
		int k;
		for(k = t->start; k < t->start + t->num; k++)
		{
			tw_dfr_item *item = &TW_DF_ITEM_INDEX(df, k);
			if(TW_DFR_ITEM__TYPE_ID(item->type_id__id) != t->type_id)
				_TW_DF_ERROR("item does not have right type_id, type=%d type_id1=%d item=%d type_id2=%d", i, t->type_id, k, TW_DFR_ITEM__TYPE_ID(item->type_id__id));
		}
	}

	// check that the file is complete
	{
		if(tw_io_seek_end(df->file) != 0)
			return 1;
		long apparant_length;
		if((apparant_length = tw_io_tell(df->file)) < 0)
			return 1;

		if(apparant_length - df->file_offset != df->size)
			_TW_DF_ERROR("datafile too short, size=%d wanted=%d", apparant_length - df->file_offset, df->size);
	}

	return 0;
#undef _TW_DF_ERROR
}

static uint32_t tw_df_crc_calc(tw_datafile *df)
{
	// NOTE: A proper implementation would only compute the checksum on the
	//       actual datafile, however in order to provide compatiblity with
	//       the reference implementation this crude behavior is actually
	//       wanted.

	// go to the start of the file
	if(tw_io_seek(df->file, 0) != 0)
		return 1;

	uint32_t crc = 0;

	tw_byte buf[TW_BUFSIZE];
	size_t length;
	while((length = tw_io_read(df->file, buf, sizeof(buf))) == sizeof(buf))
		crc = tw_comp_crc(crc, buf, length);

	if(tw_error_errno() != TW_ERRNO_IO_EOF)
		return 0;
	tw_error_clear();

	return crc;
}

static int tw_df_open_read(tw_datafile *df)
{
	// say that the crc hasn't been calculated yet
	df->crc_calc = 0;

	// go to the start of the datafile
	if(tw_io_seek(df->file, df->file_offset) != 0)
		return 1;

	// read version-agnostic header
	if(tw_io_read(df->file, &df->header_ver, sizeof(df->header_ver)) != sizeof(df->header_ver))
	{
		if(tw_error_errno() == TW_ERRNO_IO_EOF)
			tw_error_set(TW_ERRNO_DF_OPEN, "datafile too short");
		return 1;
	}

	// check for magic bytes
	assert(sizeof(TW_DF_MAGIC) == sizeof(df->header_ver.magic)
		&& sizeof(TW_DF_MAGIC_BIGENDIAN) == sizeof(df->header_ver.magic)
		&& "magic bytes have wrong length");
	if(tw_mem_comp(df->header_ver.magic, TW_DF_MAGIC, sizeof(df->header_ver.magic)) != 0
		&& tw_mem_comp(df->header_ver.magic, TW_DF_MAGIC_BIGENDIAN, sizeof(df->header_ver.magic)) != 0)
	{
		tw_error_set(TW_ERRNO_DF_OPEN, "wrong datafile signature");
		return 1;
	}

	// header consists of little-endian ints
	tw_endian_fromlittle(&df->header_ver, sizeof(int), sizeof(df->header_ver) / sizeof(int));
	// fix magic bytes after endian-swap
	tw_mem_copy(df->header_ver.magic, TW_DF_MAGIC, sizeof(df->header_ver.magic));

	// check version - accept version 3 and 4
	if(df->header_ver.version != 3 && df->header_ver.version != 4)
	{
		tw_error_set(TW_ERRNO_DF_OPEN, "wrong datafile version, version=%d", df->header_ver.version);
		return 1;
	}

	// read version-dependent header
	if(tw_io_read(df->file, &df->header, sizeof(df->header)) != sizeof(df->header))
	{
		if(tw_error_errno() == TW_ERRNO_IO_EOF)
			tw_error_set(TW_ERRNO_DF_OPEN, "datafile too short for header v3/4");
		return 1;
	}

	// version-dependent header also consists of little-endian ints
	tw_endian_fromlittle(&df->header, sizeof(int), sizeof(df->header) / sizeof(int));

	if(tw_df_open_read_check_header(&df->header) != 0)
		return 1;

	// use this type to detect potential overflows
	uint64_t size = 0;
	// allocate data file struct, layout is mostly the same as in file
	// read item_types, item_offsets, data_offsets, data_sizes for version 4 and items
	// also allocate pointers for the uncompressed data
	size += sizeof(tw_dfr_item_type) * df->header.num_item_types; // item_types
	size += sizeof(int32_t) * df->header.num_items; // item_offsets
	size += sizeof(int32_t) * df->header.num_data; // data_offsets
	if(df->header_ver.version >= 4)
		size += sizeof(int32_t) * df->header.num_data; // data_sizes (only version 4)
	size += df->header.size_items; // items

	uint64_t readsize = size; // read everything up to now directly from the file into the memory

	size += sizeof(void *) * df->header.num_data; // uncompressed data pointers

	// offset of the datafile, where the data starts, size of complete datafile
	int64_t data_start_offset = sizeof(tw_dfr_header_ver) + sizeof(tw_dfr_header) + readsize;
	int64_t datafile_size = data_start_offset + df->header.size_data;

	// detect overflows
	if(size != (uint32_t)size || readsize != (uint32_t)readsize
		|| data_start_offset != (int32_t)data_start_offset
		|| datafile_size != (int32_t)datafile_size)
	{
		tw_error_set(TW_ERRNO_DF_OPEN, "malicious header, size=%"PRId64" readsize=%"PRId64" data_start_offset=%"PRId64" datafile_size=%"PRId64, size, readsize, data_start_offset, datafile_size);
		return 1;
	}

	df->data_start_offset = data_start_offset;
	df->size = datafile_size;

	df->alloc = malloc(size);

	if(tw_io_read(df->file, df->alloc, readsize) != readsize)
	{
		if(tw_error_errno() == TW_ERRNO_IO_EOF)
			tw_error_set(TW_ERRNO_DF_OPEN, "datafile too short (can't read to items' end), wanted=%d", readsize);
		free(df->alloc);
		return 1;
	}

	{
		// set up pointers
		void *next = df->alloc;

		// `df->item_types` gets the first chunk of `df->alloc`
		// the chunk's size is `sizeof(tw_dfr_item_type) * df->header.num_item_types`
		// thus the length of the df->item_types array `df->header.num_item_types`
		// it contains type descriptions in the `tw_dfr_item_type` format
		df->item_types = next; next = &df->item_types[df->header.num_item_types];

		// next chunk, starting after the end of `df->item_types`
		// size is `sizeof(int32_t) * df->header.num_items`
		// thus the length of the array is `df->header.num_items`
		df->item_offsets = next; next = &df->item_offsets[df->header.num_items];

		// next chunk, starting after the end of `df->item_offsets`
		// size is `sizeof(int32_t) * df->header.num_data`
		// thus the length of the array is `df->header.num_data`
		df->data_offsets = next; next = &df->data_offsets[df->header.num_data];

		// next chunk, if the version is greater or equal to 4
		// starting after the end of the last chunk
		// size is `sizeof(int32_t) * df->header.num_data`
		// if the version is less than 4, the field is NULLed, so if it is not NULL
		// the length of the array is `df->header.num_data`
		if(df->header_ver.version >= 4)
		{
			df->uncomp_data_sizes = next; next = &df->uncomp_data_sizes[df->header.num_data];
		}
		else
			df->uncomp_data_sizes = NULL;

		// last chunk that is read from the file, starting after the last chunk (which depends on the version)
		// size is `df->header.size_items`
		// thus the length of the array is `df->header.size_items`
		df->items_start = next; next = &df->items_start[df->header.size_items];

		// actual last chunk
		// size is `sizeof(void *) * df->header.num_data`
		// thus the array has the length `df->header.num_data`
		df->uncomp_data = next; next = &df->uncomp_data[df->header.num_data];

		// zero out the pointers so we know when whether we can free them
		tw_mem_zero(df->uncomp_data, sizeof(*df->uncomp_data) * df->header.num_data);
	}

	if(tw_df_open_read_check(df) != 0)
	{
		free(df->alloc);
		return 1;
	}

	return 0;
}

tw_datafile *tw_df_open(const char *filename)
{
	tw_datafile df;

	// open file
	df.file = tw_io_open(filename, "rb");
	if(df.file == NULL)
		return NULL;

	df.file_offset = 0;

	// actually read the datafile
	if(tw_df_open_read(&df) != 0)
	{
		tw_io_close(df.file);
		return NULL;
	}

	// copy the struct on to the heap for the user
	// should be freed tw_df_close
	tw_datafile *ret = malloc(sizeof(*ret));
	*ret = df;
	return ret;
}

int tw_df_close_read(tw_datafile *df)
{
	int i;
	for(i = 0; i < df->header.num_data; i++)
	{
		if(df->uncomp_data[i] != NULL)
		{
			free(df->uncomp_data[i]);
			df->uncomp_data[i] = NULL;
		}
	}

	free(df->alloc);
	df->alloc = NULL;
	return 0;
}

int tw_df_close(tw_datafile *df)
{
	// make a local copy so we can free
	tw_datafile df_c = *df;
	free(df);
	df = NULL;

	// the return value
	int ret = tw_df_close_read(&df_c);

	// close the file
	if(tw_io_close(df_c.file) != 0)
		return 1;

	return ret;
}

size_t tw_df_data_size_file(tw_datafile *df, int index)
{
	// check the index
	if(!(0 <= index && index < df->header.num_data))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);
		return -1;
	}

	if(index < df->header.num_data - 1)
		return df->data_offsets[index + 1] - df->data_offsets[index];
	else
		return df->header.size_data - df->data_offsets[index];
}

size_t tw_df_data_size(tw_datafile *df, int index)
{
	// check the index
	if(!(0 <= index && index < df->header.num_data))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);
		return -1;
	}

	if(df->uncomp_data_sizes)
		return df->uncomp_data_sizes[index];
	else
		return tw_df_data_size_file(df, index);
}

void *tw_df_data_load(tw_datafile *df, int index)
{
	// check the index
	if(!(0 <= index && index < df->header.num_data))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);
		return NULL;
	}

	// if it's already loaded, return it
	if(df->uncomp_data[index])
		return df->uncomp_data[index];

	// otherwise, load it

	// determining read size
	size_t data_size_file = tw_df_data_size_file(df, index);

	// seek to the data
	if(tw_io_seek(df->file, df->file_offset + df->data_start_offset + df->data_offsets[index]) != 0)
	{
		tw_error_set(TW_ERRNO_DF_READDATA, "could not seek to data, data=%d offset=%d", index, df->data_start_offset + df->data_offsets[index]);
		return NULL;
	}

	// allocate space for the data
	df->uncomp_data[index] = malloc(data_size_file);
	// read the data
	if(tw_io_read(df->file, df->uncomp_data[index], data_size_file) != data_size_file)
	{
		tw_error_set(TW_ERRNO_DF_READDATA, "could not read data, data=%d offset=%d size=%d", index, df->data_start_offset + df->data_offsets[index], data_size_file);
		free(df->uncomp_data[index]);
		df->uncomp_data[index] = NULL;
		return NULL;
	}

	// is the data compressed?
	if(df->uncomp_data_sizes)
	{
		void *compressed = df->uncomp_data[index];

		// allocate space for the uncompressed data
		df->uncomp_data[index] = malloc(df->uncomp_data_sizes[index]);

		size_t uncomp_size = df->uncomp_data_sizes[index];
		int uncomp_error = tw_comp_uncompress(df->uncomp_data[index], &uncomp_size, compressed, data_size_file);
		if(uncomp_error != 0 || uncomp_size != (size_t)df->uncomp_data_sizes[index])
		{
			if(uncomp_error == 0)
				tw_error_set(TW_ERRNO_DF_READDATA, "could not uncompress data, data=%d size=%d wanted=%d", index, uncomp_size, df->uncomp_data_sizes[index]);

			free(compressed);
			compressed = NULL;
			free(df->uncomp_data[index]);
			df->uncomp_data[index] = NULL;
			return NULL;
		}

		// throw away the compressed data
		free(compressed);
		compressed = NULL;
	}
	return df->uncomp_data[index];
}

int tw_df_data_unload(tw_datafile *df, int index)
{
	// check the index
	if(!(0 <= index && index < df->header.num_data))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);
		return 1;
	}

	if(df->uncomp_data[index])
	{
		free(df->uncomp_data[index]);
		df->uncomp_data[index] = NULL;
	}
	return 0;
}

int tw_df_data_num(tw_datafile *df)
{
	return df->header.num_data;
}

void *tw_df_item_read(tw_datafile *df, int index, size_t *size, int *type_id, int *id)
{
	// check the index
	if(!(0 <= index && index < df->header.num_items))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "item index out of range, item=%d", index);
		return NULL;
	}

	// load the item
	tw_dfr_item *item = &TW_DF_ITEM_INDEX(df, index);

	// fill output parameter
	if(type_id)
		*type_id = TW_DFR_ITEM__TYPE_ID(item->type_id__id);
	if(id)
		*id = TW_DFR_ITEM__ID(item->type_id__id);
	if(size)
		*size = item->size;

	// return the actual item
	return (void *)(item + 1);
}

void *tw_df_item_find(tw_datafile *df, size_t *size, int type_id, int id)
{
	int start;
	int num;

	// get the indexes of the items of the given type
	tw_df_type_indexes(df, type_id, &start, &num);

	// look through said indexes
	int i;
	for(i = start; i < start + num; i++)
	{
		int id2;

		// get the item (this also sets id2 and the output parameter size)
		void *item = tw_df_item_read(df, i, size, NULL, &id2);

		// have we found the item?
		if(id == id2)
			return item;
	}

	// nothing found, return NULL
	return NULL;
}

void tw_df_type_indexes(tw_datafile *df, int type_id, int *start, int *num)
{
	if(start)
		*start = -1;
	if(num)
		*num = 0;

	// loop through the item types and look for the right type_id
	int i;
	for(i = 0; i < df->header.num_item_types; i++)
	{
		if(type_id == df->item_types[i].type_id)
		{
			// if you found it, fill the output parameters and return
			if(start)
				*start = df->item_types[i].start;
			if(num)
				*num = df->item_types[i].num;
			return;
		}
	}
}

int tw_df_item_num(tw_datafile *df)
{
	return df->header.num_items;
}

uint32_t tw_df_crc(tw_datafile *df)
{
	if(df->crc_calc == 0)
	{
		df->crc = tw_df_crc_calc(df);
		df->crc_calc = 1;
	}
	return df->crc;
}

void tw_df_dump_header_ver(tw_dfr_header_ver *header_ver)
{
	printf("magic=0x%08x version=%d\n", *(int *)header_ver->magic, header_ver->version);
}

void tw_df_dump_header(tw_dfr_header *header)
{
	printf("size=%d swaplen=%d num_item_types=%d num_items=%d num_data=%d size_items=%d size_data=%d\n", header->size, header->swaplen, header->num_item_types, header->num_items, header->num_data, header->size_items, header->size_data);
}

void tw_df_dump_item_type(tw_dfr_item_type *type)
{
	printf("type_id=%d start=%-2d num=%-2d\n", type->type_id, type->start, type->num);
}

void tw_df_dump_item(tw_dfr_item *item)
{
	printf("type_id=%d id=%-2d size=%-3d\n", TW_DFR_ITEM__TYPE_ID(item->type_id__id), TW_DFR_ITEM__ID(item->type_id__id), item->size);
}

void tw_df_dump(tw_datafile *df)
{
	tw_df_dump_header_ver(&df->header_ver);
	tw_df_dump_header(&df->header);
	printf("\n");

	int i;
	for(i = 0; i < df->header.num_item_types; i++)
	{
		printf("type=%d ", i);
		tw_df_dump_item_type(&df->item_types[i]);
	}

	printf("\n");

	for(i = 0; i < df->header.num_item_types; i++)
	{
		tw_dfr_item_type *t = &df->item_types[i];

		printf("type=%d ", i);
		tw_df_dump_item_type(t);

		int k;
		for(k = t->start; k < t->start + t->num; k++)
		{
			printf("\titem=%-2d ", k);
			tw_df_dump_item(&TW_DF_ITEM_INDEX(df, k));
		}
	}
}

// datafile writer
tw_datafile_writer *tw_dfw_open(const char *filename)
{
	(void)filename;
	tw_error_set(TW_ERRNO_NOTIMPLEMENTED, "write support not implemented");
	return NULL;
}

int tw_dfw_data_add(tw_datafile_writer *dfw, void *data, size_t size)
{
	(void)dfw;
	(void)data;
	(void)size;
	tw_error_set(TW_ERRNO_NOTIMPLEMENTED, "write support not implemented");
	return -1;
}

int tw_dfw_item_add(tw_datafile_writer *dfw, int type_id, int id, void *data, size_t size)
{
	(void)dfw;
	(void)type_id;
	(void)id;
	(void)data;
	(void)size;
	tw_error_set(TW_ERRNO_NOTIMPLEMENTED, "write support not implemented");
	return 1;
}

int tw_dfw_close(tw_datafile_writer *dfw)
{
	(void)dfw;
	tw_error_set(TW_ERRNO_NOTIMPLEMENTED, "write support not implemented");
	return 1;
}
