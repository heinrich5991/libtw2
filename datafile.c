#include "datafile.h"

#include "common.h"
#include "error.h"
#include "io.h"

#include <assert.h>
#include <inttypes.h>
#include <stdlib.h>

#include "datafile_raw.h"

struct tw_datafile
{
	tw_datafile_raw *dfr;

	tw_io *file;

	uint32_t crc;
	int crc_calc;

	int num_data;
	size_t *data_sizes;
	void **data;
};

static int tw_df_callback_read(void *buffer, size_t start, size_t buffer_size, size_t *read, void *userdata);
static int tw_df_callback_filesize(size_t *filesize, void *userdata);
static int tw_df_callback_alloc(void **result, size_t size, void *userdata);
static void tw_df_callback_free(void *ptr, void *userdata);

static int tw_df_handle_error(int result, tw_dfr_error *error);

tw_datafile *tw_df_open(const char *filename)
{
	tw_datafile df;
	tw_dfr_error error;

	// open file
	df.file = tw_io_open(filename, "rb");
	if(df.file == NULL)
		return NULL;

	df.dfr = tw_dfr_create();
	tw_dfr_callbacks_set(df.dfr,
		tw_df_callback_read,
		tw_df_callback_filesize,
		tw_df_callback_alloc,
		tw_df_callback_free
	);

	// actually read the datafile
	if(tw_df_handle_error(tw_dfr_open(df.dfr, &error, &df), &error))
	{
		tw_io_close(df.file);
		return NULL;
	}

	df.num_data = tw_df_num_data(&df);

	df.data = malloc(sizeof(void *) * df.num_data);
	tw_mem_zero(df.data, sizeof(void *) * df.num_data);

	df.data_sizes = malloc(sizeof(size_t) * df.num_data);
	tw_mem_zero(df.data_sizes, sizeof(size_t) * df.num_data);

	df.crc_calc = 0;

	// copy the struct on to the heap for the user
	// should be freed tw_df_close
	tw_datafile *ret = malloc(sizeof(*ret));
	*ret = df;
	return ret;
}

int tw_df_close(tw_datafile *df)
{
	// make a local copy so we can free the pointer
	tw_datafile df_c = *df;
	free(df);
	df = NULL;

	tw_dfr_close(df_c.dfr, NULL, &df_c);
	tw_dfr_free(df_c.dfr);
	df_c.dfr = NULL;

	{
		int i;
		for(i = 0; i < df_c.num_data; i++)
		{
			if(df_c.data[i] != NULL)
			{
				free(df_c.data[i]);
				df_c.data[i] = NULL;
			}
		}
	}

	free(df_c.data);
	df_c.data = NULL;

	free(df_c.data_sizes);
	df_c.data_sizes = NULL;

	{
		int close_result = tw_io_close(df_c.file);
		df_c.file = NULL;
		if(close_result != 0)
			return 1;
	}
	return 0;
}

void *tw_df_data_load(tw_datafile *df, int index, size_t *size)
{
	tw_dfr_error error;

	// check the index
	if(!(0 <= index && index < df->num_data))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);
		return NULL;
	}

	// if it's not already loaded, load it
	if(df->data[index] == NULL)
	{
		if(tw_df_handle_error(tw_dfr_data_read(df->dfr, &df->data[index], &df->data_sizes[index], index, &error, &df), &error))
		{
			*size = 0;
			return NULL;
		}
	}

	*size = df->data_sizes[index];
	return df->data[index];
}

void tw_df_data_unload(tw_datafile *df, int index)
{
	// check the index
	if(!(0 <= index && index < df->num_data))
	{
		tw_error_set(TW_ERRNO_OUTOFRANGE, "data index out of range, data=%d", index);
		return;
	}

	if(df->data[index] != NULL)
	{
		free(df->data[index]);
		df->data[index] = NULL;
		df->data_sizes[index] = 0;
	}
}

int tw_df_num_data(tw_datafile *df)
{
	int result = -1;
	tw_dfr_num_data(df->dfr, &result, NULL, df);
	return result;
}

void *tw_df_item_read(tw_datafile *df, int index, size_t *size, int *type_id, int *id)
{
	tw_dfr_error error;

	void *result = NULL;
	if(tw_df_handle_error(tw_dfr_item_read(df->dfr, &result, size, type_id, id, index, &error, &df), &error))
	{
		*size = 0;
		return NULL;
	}
	return result;
}

void *tw_df_item_find(tw_datafile *df, size_t *size, int type_id, int id)
{
	tw_dfr_error error;

	void *result = NULL;
	if(tw_df_handle_error(tw_dfr_item_find(df->dfr, &result, size, type_id, id, &error, &df), &error))
	{
		*size = 0;
		return NULL;
	}
	return result;
}

void tw_df_type_indexes(tw_datafile *df, int type_id, int *start, int *num)
{
	tw_dfr_type_indexes(df->dfr, start, num, type_id, NULL, &df);
}

int tw_df_num_items(tw_datafile *df)
{
	int result = -1;
	tw_dfr_num_items(df->dfr, &result, NULL, &df);
	return result;
}

uint32_t tw_df_crc(tw_datafile *df)
{
	if(!df->crc_calc)
	{
		tw_dfr_crc temp;
		tw_dfr_crc_calc(df->dfr, &temp, NULL, &df);
		df->crc = temp;

		df->crc_calc = 1;
	}
	return df->crc;
}

static int tw_df_callback_read(void *buffer, size_t start, size_t buffer_size, size_t *read, void *userdata)
{
	tw_datafile *df = userdata;

	if(tw_io_seek(df->file, start) != 0)
		return 1;

	if((*read = tw_io_read(df->file, buffer, buffer_size)) != buffer_size)
		if(tw_error_errno() != TW_ERRNO_IO_EOF)
			return 1;

	return 0;
}

static int tw_df_callback_filesize(size_t *filesize, void *userdata)
{
	tw_datafile *df = userdata;

	if(tw_io_seek_end(df->file) != 0)
		return 1;
	long length = tw_io_tell(df->file);
	if(length < 0)
		return 1;

	*filesize = length;
	return 0;
}

static int tw_df_callback_alloc(void **result, size_t size, void *userdata)
{
	tw_datafile *df = userdata;
	(void)df;

	*result = malloc(size);
	assert(*result != NULL);
	return 0;
}

static void tw_df_callback_free(void *ptr, void *userdata)
{
	tw_datafile *df = userdata;
	(void)df;

	free(ptr);
}

static int tw_df_handle_error(int result, tw_dfr_error *error)
{
	// no error -- nothing to do
	if(result == 0)
		return 0;

	// own error -- report to user and return
	if(result < 0)
		return 1;
	else
	{
		if(error->errno == TW_DFR_ERRNO_OUTOFRANGE)
			tw_error_set(TW_ERRNO_OUTOFRANGE, "%s", error->string);
		else if(error->errno == TW_DFR_ERRNO_NOTIMPLEMENTED)
			tw_error_set(TW_ERRNO_NOTIMPLEMENTED, "%s", error->string);
		else
			tw_error_set(TW_ERRNO_DF + error->errno, "datafile error: %s", error->string);
		return 1;
	}
}
