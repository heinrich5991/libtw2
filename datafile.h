#include <stddef.h>
#include <stdint.h>

typedef struct tw_datafile tw_datafile;

// datafile reader
tw_datafile *tw_df_open(const char *filename);
int tw_df_close(tw_datafile *df);

void *tw_df_data_load(tw_datafile *df, int index, size_t *size);
void tw_df_data_unload(tw_datafile *df, int index);
int tw_df_num_data(tw_datafile *df);

void *tw_df_item_read(tw_datafile *df, int index, size_t *size, int *type_id, int *id);
void *tw_df_item_find(tw_datafile *df, size_t *size, int type_id, int id);
void tw_df_type_indexes(tw_datafile *df, int type_id, int *start, int *num);
int tw_df_num_items(tw_datafile *df);

uint32_t tw_df_crc(tw_datafile *df);

// errors
enum
{
	TW_ERRNO_DF=300,
	// see datafile_raw.h, add 300 to each
};
