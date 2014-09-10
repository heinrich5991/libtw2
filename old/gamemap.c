/*#include "gamemap.h"

#include "datafile.h"

struct tw_map
{
	tw_datafile *df;
};

typedef struct tw_map_fixed // fixed point - 22.10
{
	int32_t value;
} tw_map_fixed;

// TW_MAP_ITEMTYPE_VERSION
typedef struct tw_map_itemtype_version
{
	int32_t version;
} tw_map_itemtype_version;

// TW_MAP_ITEMTYPE_INFO
typedef struct tw_map_itemtype_info_v1
{
	int32_t version;
	int32_t map_author;
	int32_t map_version;
	int32_t map_credits;
	int32_t map_license;
} tw_map_itemtype_info_v1;

typedef union tw_map_itemtype_info
{
	int32_t version;
	tw_map_itemtype_info_v1 v1;
} tw_map_itemtype_info;

// TW_MAP_ITEMTYPE_IMAGE
typedef struct tw_map_itemtype_image_v1
{
	int32_t version;
	int32_t width;
	int32_t height;
	int32_t external;
	int32_t name;
	int32_t data;
} tw_map_itemtype_image_v1;

typedef struct tw_map_itemtype_image_v2
{
	int32_t version;
	int32_t width;
	int32_t height;
	int32_t external;
	int32_t name;
	int32_t data;
	int32_t format;
} tw_map_itemtype_image_v2;

typedef union tw_map_itemtype_image
{
	int32_t version;
	tw_map_itemtype_image_v1 v1;
	tw_map_itemtype_image_v2 v2;
} tw_map_itemtype_image;
*/
