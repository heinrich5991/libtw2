#include <stdint.h>

enum
{
	TW_MAP_ITEMTYPE_VERSION=0,
	TW_MAP_ITEMTYPE_INFO,
	TW_MAP_ITEMTYPE_IMAGE,
	TW_MAP_ITEMTYPE_ENVELOPE,
	TW_MAP_ITEMTYPE_GROUP,
	TW_MAP_ITEMTYPE_LAYER,
	TW_MAP_ITEMTYPE_ENVPOINT,
	TW_MAP_NUM_ITEMTYPES,
};

// general structs

typedef struct tw_map_item_color
{
	int32_t red;
	int32_t green;
	int32_t blue;
	int32_t alpha;
} tw_map_item_color;

typedef struct tw_map_item_point
{
	int32_t x;
	int32_t y;
} tw_map_item_point;

typedef struct tw_map_item_tile
{
	uint8_t index;
	uint8_t flags;
	uint8_t unused[2];
} tw_map_item_tile;

enum
{
	TW_MAP_ITEM_TILE_FLAG_VFLIP=1,
	TW_MAP_ITEM_TILE_FLAG_HFLIP=2,
	TW_MAP_ITEM_TILE_FLAG_OPAQUE=4,
	TW_MAP_ITEM_TILE_FLAG_ROTATE=8,
};

typedef struct tw_map_item_quad
{
	tw_map_item_point points[5];
	tw_map_item_color colors[4];
	tw_map_item_point tex_coords[4];
	int32_t pos_env; // TW_MAP_ITEMTYPE_ENVELOPE item id
	int32_t pos_env_offset;
	int32_t color_env; // TW_MAP_ITEMTYPE_ENVELOPE item id
	int32_t color_env_offset;
} tw_map_item_quad;

//
// TW_MAP_ITEMTYPE_VERSION
//
typedef struct tw_map_item_version
{
	int32_t version;
} tw_map_item_version;

// 
// TW_MAP_ITEMTYPE_INFO
//
typedef struct tw_map_item_info_v1
{
	int32_t version;

	int32_t map_author; // data index
	int32_t map_version; // data index
	int32_t map_credits; // data index
	int32_t map_license; // data index
} tw_map_item_info_v1;

typedef union tw_map_item_info
{
	int32_t version;
	tw_map_item_info_v1 v1;
} tw_map_item_info;

//
// TW_MAP_ITEMTYPE_IMAGE
//

typedef struct tw_map_item_image_v1
{
	int32_t version;

	int32_t width;
	int32_t height;
	int32_t external;
	int32_t name; // data index
	int32_t data; // data index
} tw_map_item_image_v1;

typedef struct tw_map_item_image_v2
{
	int32_t version;

	int32_t width;
	int32_t height;
	int32_t external;
	int32_t name; // data index
	int32_t data; // data index

	int32_t format;
} tw_map_item_image_v2;

enum
{
	TW_MAP_ITEMTYPE_IMAGE_FORMAT_RGB=0,
	TW_MAP_ITEMTYPE_IMAGE_FORMAT_RGBA,
	TW_MAP_ITEMTYPE_IMAGE_NUM_FORMATS,
};

typedef union tw_map_item_image
{
	int32_t version;
	tw_map_item_image_v1 v1;
	tw_map_item_image_v2 v2;
} tw_map_item_image;

//
// TW_MAP_ITEMTYPE_ENVELOPE
//

typedef struct tw_map_item_envelope_v1
{
	int32_t version;

	int32_t channels;
	int32_t points_start; // TW_MAP_ITEMTYPE_ENVPOINT item id
	int32_t num_points;
	int32_t name[8];
} tw_map_item_envelope_v1;

typedef struct tw_map_item_envelope_v2
{
	int32_t version;

	int32_t channels;
	int32_t points_start; // TW_MAP_ITEMTYPE_ENVPOINT item id
	int32_t num_points;
	int32_t name[8];

	int32_t synchronized;
} tw_map_item_envelope_v2;

typedef union tw_map_item_envelope
{
	int32_t version;
	tw_map_item_envelope_v1 v1;
	tw_map_item_envelope_v2 v2;
} tw_map_item_envelope;

//
// TW_MAP_ITEMTYPE_GROUP
//

typedef struct tw_map_item_group_v1
{
	int32_t version;

	int32_t offset_x;
	int32_t offset_y;
	int32_t parallax_x;
	int32_t parallax_y;
	int32_t layers_start; // TW_MAP_ITEMTYPE_LAYER item id
	int32_t num_layers;
} tw_map_item_group_v1;

typedef struct tw_map_item_group_v2
{
	int32_t version;

	int32_t offset_x;
	int32_t offset_y;
	int32_t parallax_x;
	int32_t parallax_y;
	int32_t layers_start; // TW_MAP_ITEMTYPE_LAYER item id
	int32_t num_layers;

	int32_t clipping;
	int32_t clipping_x;
	int32_t clipping_y;
	int32_t clipping_height;
	int32_t clipping_width;
} tw_map_item_group_v2;

typedef struct tw_map_item_group_v3
{
	int32_t version;

	int32_t offset_x;
	int32_t offset_y;
	int32_t parallax_x;
	int32_t parallax_y;
	int32_t layers_start; // TW_MAP_ITEMTYPE_LAYER item id
	int32_t num_layers;

	int32_t clipping;
	int32_t clipping_x;
	int32_t clipping_y;
	int32_t clipping_height;
	int32_t clipping_width;

	int32_t name[3];
} tw_map_item_group_v3;

typedef union tw_map_item_group
{
	int32_t version;
	tw_map_item_group_v1 v1;
	tw_map_item_group_v2 v2;
	tw_map_item_group_v3 v3;
} tw_map_item_group;

//
// TW_MAP_ITEMTYPE_LAYER
//

typedef struct tw_map_item_layer_v1
{
	int32_t version;

	int32_t type;
	int32_t flags;
} tw_map_item_layer_v1;

enum
{
	TW_MAP_ITEM_LAYER_TYPE_UNUSED=0,
	TW_MAP_ITEM_LAYER_TYPE_UNUSED2,
	TW_MAP_ITEM_LAYER_TYPE_TILES,
	TW_MAP_ITEM_LAYER_TYPE_QUADS,
	TW_MAP_ITEM_LAYER_NUM_TYPE,
};

enum
{
	TW_MAP_ITEM_LAYER_FLAG_DETAIL=1,
};

typedef union tw_map_item_layer
{
	int32_t version;
	tw_map_item_layer_v1 v1;
} tw_map_item_layer;

// TW_MAP_ITEM_LAYER_TYPE_TILES
typedef struct tw_map_item_layer_tiles_v1
{
	tw_map_item_layer_v1 layer;
	int32_t version;

	int32_t width;
	int32_t height;
	int32_t flags;
	tw_map_item_color color;
	int32_t color_env; // TW_MAP_ITEMTYPE_ENVELOPE item id
	int32_t color_env_offset;
	int32_t image; // TW_MAP_ITEMTYPE_IMAGE item id
	int32_t data; // data index
} tw_map_item_layer_tiles_v1;

enum
{
	TW_MAP_ITEM_LAYER_TILES_FLAG_GAME=1,
};

typedef struct tw_map_item_layer_tiles_v2
{
	tw_map_item_layer_v1 layer;
	int32_t version;

	int32_t width;
	int32_t height;
	int32_t flags;
	tw_map_item_color color;
	int32_t color_env; // TW_MAP_ITEMTYPE_ENVELOPE item id
	int32_t color_env_offset;
	int32_t image; // TW_MAP_ITEMTYPE_IMAGE item id
	int32_t data; // data index

	int32_t name[3];
} tw_map_item_layer_tiles_v2;

typedef struct tw_map_item_layer_tiles
{
	struct
	{
		tw_map_item_layer_v1 layer;
		int32_t version;
	};
	tw_map_item_layer_tiles_v1 v1;
	tw_map_item_layer_tiles_v1 v2;
} tw_map_item_layer_tiles;

// TW_MAP_ITEM_LAYER_TYPE_QUADS
typedef struct tw_map_item_layer_quads_v1
{
	tw_map_item_layer_v1 layer;
	int32_t version;

	int32_t num_quads;
	int32_t data; // data index
	int32_t image; // TW_MAP_ITEMTYPE_IMAGE item id
} tw_map_item_layer_quads_v1;

typedef struct tw_map_item_layer_quads_v2
{
	tw_map_item_layer_v1 layer;
	int32_t version;

	int32_t num_quads;
	int32_t data; // data index
	int32_t image; // TW_MAP_ITEMTYPE_IMAGE item id

	int32_t name[3];
} tw_map_item_layer_quads_v2;

typedef union tw_map_item_layer_quads
{
	struct
	{
		tw_map_item_layer_v1 layer;
		int32_t version;
	};
	tw_map_item_layer_tiles_v1 v1;
	tw_map_item_layer_tiles_v1 v2;
} tw_map_item_layer_quads;

//
// TW_MAP_ITEMTYPE_ENVPOINT
//

typedef struct tw_map_item_envpoint
{
	int32_t time_ms;
	int32_t curvetype;
	int32_t values[4];
} tw_map_item_envpoint;

enum
{
	TW_MAP_ITEM_ENVPOINT_CURVETYPE_STEP=0,
	TW_MAP_ITEM_ENVPOINT_CURVETYPE_LINEAR,
	TW_MAP_ITEM_ENVPOINT_CURVETYPE_SLOW,
	TW_MAP_ITEM_ENVPOINT_CURVETYPE_FAST,
	TW_MAP_ITEM_ENVPOINT_CURVETYPE_SMOOTH,
	TW_MAP_ITEM_ENVPOINT_NUM_CURVETYPES,
};
