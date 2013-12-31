#include <stddef.h>
#include <stdint.h>

typedef struct tw_datafile_raw tw_datafile_raw;

typedef int (*tw_dfr_callback_read)(void *buffer, size_t start, size_t buffer_size, size_t *read, void *userdata);
typedef int (*tw_dfr_callback_filesize)(size_t *filesize, void *userdata);
typedef int (*tw_dfr_callback_alloc)(void **result, size_t size, void *userdata);
typedef void (*tw_dfr_callback_free)(void *ptr, void *userdata);

typedef uint32_t tw_dfr_crc;


typedef struct tw_dfr_error
{
	int errno;
	char string[256];
} tw_dfr_error;

enum
{
	TW_DFR_ERRNO_NONE=0,
	TW_DFR_ERRNO_FILETOOSHORT,
	TW_DFR_ERRNO_WRONGMAGIC,
	TW_DFR_ERRNO_UNSUPPORTEDVERSION,
	TW_DFR_ERRNO_MALFORMEDHEADER,
	TW_DFR_ERRNO_MALFORMED,
	TW_DFR_ERRNO_OUTOFRANGE,
	TW_DFR_ERRNO_DATAUNCOMPRESS,
	TW_DFR_ERRNO_NOTIMPLEMENTED,
};


tw_datafile_raw *tw_dfr_create();
void tw_dfr_free(tw_datafile_raw *dfr);

void tw_dfr_callbacks_set(tw_datafile_raw *dfr,
	tw_dfr_callback_read read,
	tw_dfr_callback_filesize filesize,
	tw_dfr_callback_alloc alloc,
	tw_dfr_callback_free free
);

int tw_dfr_open(tw_datafile_raw *dfr, tw_dfr_error *error, void *userdata);
int tw_dfr_close(tw_datafile_raw *dfr, tw_dfr_error *error, void *userdata);

int tw_dfr_data_read(tw_datafile_raw *dfr, void **data, size_t *data_size, int index, tw_dfr_error *error, void *userdata);
int tw_dfr_num_data(tw_datafile_raw *dfr, int *num, tw_dfr_error *error, void *userdata);

int tw_dfr_item_read(tw_datafile_raw *dfr, int32_t **item, size_t *item_count, int *type_id, int *id, int index, tw_dfr_error *error, void *userdata);
int tw_dfr_item_find(tw_datafile_raw *dfr, int32_t **item, size_t *item_count, int type_id, int id, tw_dfr_error *error, void *userdata);
int tw_dfr_num_items(tw_datafile_raw *dfr, int *num, tw_dfr_error *error, void *userdata);

int tw_dfr_type_indexes(tw_datafile_raw *dfr, int *start, int *num, int type_id, tw_dfr_error *error, void *userdata);

int tw_dfr_crc_calc(tw_datafile_raw *dfr, tw_dfr_crc *crc, tw_dfr_error *error, void *userdata);

int tw_dfr_dump(tw_datafile_raw *dfr, tw_dfr_error *error, void *userdata);
