#include "common.h"

#include <stdint.h>

typedef struct tw_dfr_header_ver
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

static const tw_byte TW_DFR_MAGIC[] = {'D', 'A', 'T', 'A'};
static const tw_byte TW_DFR_MAGIC_BIGENDIAN[] = {'A', 'T', 'A', 'D'};
