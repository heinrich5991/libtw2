/*#include "common.h"

#include <stdbool.h>

typedef struct tw_map tw_map;

// general struct "color"
typedef struct tw_map_color
{
	int red;
	int green;
	int blue;
	int alpha;
} tw_map_color;

typedef struct tw_map_point
{
	float x;
	float y;
} tw_map_point;


typedef struct tw_map_info
{
	const char *author;
	const char *version;
	const char *credits;
	const char *license;
} tw_map_info;

typedef struct tw_map_image
{
	int width;
	int height;
	bool external;
	const char *name;

	size_t data_size;
	const tw_byte *data;
} tw_map_image;

typedef struct tw_map_envpoint
{
	int time;
	int curvetype;
	float values[4];
} tw_map_envpoint;

enum
{
	TW_MAP_ENVPOINT_CURVETYPE_STEP=0,
	TW_MAP_ENVPOINT_CURVETYPE_LINEAR,
	TW_MAP_ENVPOINT_CURVETYPE_SLOW,
	TW_MAP_ENVPOINT_CURVETYPE_FAST,
	TW_MAP_ENVPOINT_CURVETYPE_SMOOTH,
	TW_MAP_ENVPOINT_NUM_CURVETYPES,
};

typedef struct tw_map_envelope
{
	int channels;
	int num_points;
	tw_map_envpoint *points;
	const char *name;
	bool synchronized;
} tw_map_envelope;

typedef struct tw_map_group
{
	int offset_x;
	int offset_y;
	int parallax_x;
	int parallax_y;
	bool clipping;
	int clipping_x;
	int clipping_y;
	int clipping_width;
	int clipping_height;
	const char *name;

	int layers_start; // layer index
	int num_layers;
} tw_map_group;

typedef struct tw_map_layer
{
	bool detail;
} tw_map_layer;

typedef struct tw_map_tile
{
	tw_byte index;
	tw_byte flags;
	tw_byte unused[2];
} tw_map_tile;

enum
{
	TW_MAP_TILE_FLAG_VFLIP=1,
	TW_MAP_TILE_FLAG_HFLIP=2,
	TW_MAP_TILE_FLAG_OPAQUE=4,
	TW_MAP_TILE_FLAG_ROTATE=8,
};

typedef struct tw_map_layer_tilemap
{
	tw_map_layer layer;
	bool game;
	int width;
	int height;
	tw_map_color color;
	int color_env; // envelope index
	int color_env_offset;
	int image; // image index
	const tw_map_tile *data;
	const char *name;
} tw_map_layer_tilemap;

typedef struct tw_map_quad
{
	tw_map_point points[5];
	tw_map_color colors[4];
	tw_map_point texcoords[4];
	int pos_env; // envelope index
	int pos_env_offset;
	int color_env; // envelope index
	int color_env_offset;
} tw_map_quad;

typedef struct tw_map_layer_quads
{
	int num_quads;
	tw_map_quad *data;
} tw_map_layer_quads;

tw_map *tw_map_open(const char *filename);*/
