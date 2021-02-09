Introduction
============

Teeworlds and DDNet maps get saved as datafiles.
If you are not yet familiar with parsing datafiles, please go through the [datafile documentation](datafile.md) first.

Terminology
===========

- integer types are usually declared in the same notation as in rust. Examples:
    - `i32` is a signed 32 bit integer
    - `u8` is a unsigned 8 bit integer, `byte` is used synonymously
    
- `&` is the prefix for data item indices.
  Those point to a data item in the `datafile.data` section of the datafile.

- `opt` is the prefix for optional indices. They are either a normal index or `-1`, marking it as unused.

- `*` is the prefix for indices that point to another item in the datafile.
    - For example `*image` means that the field points to an image item.
      `*color_envelope` would mean that the field points to the envelope item with that index, which should be a color envelope.
 
- **CString** is a null terminated UTF-8 string.

- **I32String** is a CString stored in consecutive i32 values.
  To extract the string:
    1. convert the i32s to their be (big endian) byte representation, join the bytes so that we have a single array of bytes
    2. the last byte is a null byte, ignore that one for now
    3. wrapping-subtract 128 from the remaining bytes
    4. now you got a CString padded with zeroes
    
- **Point** is a struct with two i32s, one for x, one for y.
  It is usually used to describe a position in the map.
  0, 0 is the top-left corner.

- **Color** is a struct with the 4 u8 values (in order): r, g, b, a.
  Its still usually parsed from 4 i32s, meaning each one should hold a value that fits into an u8.

- the `item_data` of an item is an array of i32s.
  We will split the `item_data` up into its different elements, which differ for each item type.
  Examples for the `item_data` syntax:
    1. `[2] point: Point` => The next two i32 values represent the variable `point` (which will be explained afterwards) which is of the type `Point`.
    2. `[1] opt &name: CString` => The next i32 represents `name` and is an optional data item index to a CString.

Item Type Overview
==================

```
General map structure:
    > Info
    > Images
    > Envelopes
        > Envelope Points
    > Groups
        > Layers
            > Auto Mappers (DDNet only)
    > Sounds (DDNet only)
```

Maps consist of various elements that each have a `type_id` that identifies them.

```
type_id mappings:
    0 -> Version
    1 -> Info
    2 -> Images
    3 -> Envelopes
    4 -> Groups
    5 -> Layers
    6 -> Envelope Points
    7 -> Sounds (DDNet only)
    0xffff -> UUID Index (see below, DDNet only)
```
        
Use them to figure out which purpose each of the item types in the `datafile.item_types` section of the datafile has.

Things to keep in mind:
1. When an item type appears in `datafile.item_types`, it means that there must be at least one item of that type
2. With the exception of the UUID Index, the first item of an item type will have `id` = 0 and from there it will count up

UUID item types
---------------

In DDNet, some item types won't be assigned a type_id, but instead an uuid.

To find the correct item type (in `datafile.item_types` for uuid item types, you will need their `type_id`.
You will need to figure out the `type_id` manually by looking into the **UUID Index items**.

```
UUID Index Item structure:
    type_id: 0xffff
    id: type_id of the uuid item type that this item represents
    item_data:
        [3] UUID of the uuid item type that this item represents
```

- the twelve bytes of the uuid are laid out in order in the `item_data` when viewing the integers as big endian
- steps to find an uuid item type:
    1. get the UUID item type
    2. scan through its items
    3. when an item has the correct uuid in its `item_data`, copy the `type_id` from the `id` field
    4. find the item type with the `type_id` that we just found out

Map Item Types
==============

Version
-------

- `type_id` = 0
- exactly one item

```
item_data of the only version item:
    [1] version
```

- `version` = 1
- no actual data is stored using the Version item type

Info
----

- `type_id` = 1
- exactly one item

```
item_data of the only version item:
    [1] (item) version
    [1] opt &author: CString
    [1] opt &version: CString
    [1] opt &credits: CString
    [1] opt &license: CString
    [1] opt &settings: [CString] (DDNet only)
```

- both vanilla and DDNet are at `version` = 1
- like indicated, all the other fields are optional data item indices
- the data item behind `settings` is an array of CStrings, all consecutive, split by their null bytes (with a null byte at the very end)
- maximum amount of bytes for each CString (including the null byte):
    - author: 32
    - version: 16
    - credits: 128
    - license: 32

Images
------

- `type_id` = 2

```
item_data of image items:
    [1] version
    [1] width
    [1] height
    [1] external: bool
    [1] &name: CString
    [1] opt &data: [Pixel]

    version 2 extension (Vanilla only):
    [1] variant
```

- Vanilla is at `version` = 2, DDNet is at `version` = 1
- `width` and `height` specify the dimensions of the image
- if `version` = 1, the image is of type RGBA, for `version` = 2 `variant` holds the type:
    - 0 -> RGB
    - 1 -> RGBA
- Images can either be embedded or external.
    - Embedded images have `external` = false and have the image data stored in the data field.
      The image data is simply a 2d-array of pixels in row-major ordering.
      RGBA pixels are 4 bytes each, RGB pixels 3 bytes each.
    - External images have `external` = true and the `data` field on `-1`.
      Those images can only be loaded by clients that have those in their `mapres` directory, meaning only a small set of images should be external.
      The client looks for those images by using the `name` field.
- the CString behind `name` must fit into 128 bytes
- External images for both 0.6 and 0.7 (note that they might differ between versions!): `bg_cloud1`, `bg_cloud2`,
  `bg_cloud3`, `desert_doodads`, `desert_main`, `desert_mountains2`, `desert_mountains`, `desert_sun`,
  `generic_deathtiles`,  `generic_unhookable`, `grass_doodads`, `grass_main`, `jungle_background`, `jungle_deathtiles`,
  `jungle_doodads`, `jungle_main`, `jungle_midground`, `jungle_unhookables`, `moon`, `mountains`, `snow`, `stars`,
  `sun`, `winter_doodads`, `winter_main`, `winter_mountains2`, `winter_mountains3`, `winter_mountains`
- Further external images for 0.7 maps: `easter`, `generic_lamps`, `generic_shadows`, `light`

Envelopes
---------

- `type_id` = 3

```
item_data of envelope items:
    [1] version
    [1] channels
    [1] start_point
    [1] num_points
    
    extension without version change:
    [8] name: I32String
    
    version 2 extension:
    [1] synchronized: bool
```

- DDNet is at `version` = 2, Vanilla chooses 3 for all envelopes when one of them uses a bezier curve, but falls back to 2 when there is none.
- `channel` holds the type of the envelope
    - 1 -> Sound envelope
    - 3 -> Position envelope
    - 4 -> Color envelope
- `synchronized` has the effect that the envelope syncs to server time, not player join time
- `start_point` is the index of its first envelope point
- `num_points` is the number of envelope points for this envelope

See Envelope Points to see how the envelope points are stored.

Envelope Points
---------------

- `type_id` = 6
- exactly one item

The `item_data` of the only item contains all the envelope points used for the envelopes.

- Size of each envelope point:
    - 22 i32s, if all envelopes have `version` = 3
    - 6 i32s, if all envelopes have a `version` <= 2
- Note that all unused fields are zeroed

The first 6 i32 of each envelope point, depending on the envelope type it belongs to:

```
sound envelope point:
    [1] time
    [1] curve type
    [1] volume
    [3] -

position envelope point:
    [1] time
    [1] curve_type
    [1] x
    [1] y
    [1] rotation
    [1] -

color envelope point:
    [1] time
    [1] curve type
    [4] color: I32Color
```

- `time` is the timestamp of the point, it should increase monotonously within each envelope
- `curve_type` holds how the curve should bend between this point and the next one
    - 0 -> Step (abrupt drop at second value)
    - 1 -> Linear (linear value change)
    - 2 -> Slow (first slow, later much faster value change)
    - 3 -> Fast (first fast, later much slower value change)
    - 4 -> Smooth (slow, faster, then once more slow value change)
    - 5 -> Bezier (Vanilla only, very customizable curve)

- `x` and `y` hold the movement
- **I32Color** actually means that the color values for r, g, b, a are i32 values

If bezier curves are used anywhere (envelope version 3), then there are 16 more i32 for each point.
These are only non-zero if the `curve_type` of the point is 5 (Bezier):

```
bezier point extension:
    [4] in_tangent_dx
    [4] in_tangent_dy
    [4] out_tangent_dx
    [4] out_tangent_dy
```

Groups
------

- `type_id` = 4

```
item_data of group items
    [1] version
    [1] x_offset
    [1] y_offset
    [1] x_parallax
    [1] y_parallax
    [1] start_layer
    [1] num_layers
    
    version 2 extension:
    [1] clipping: bool
    [1] clip_x
    [1] clip_y
    [1] clip_width
    [1] clip_height
    
    version 3 extension:
    [3] name: I32String
```
        
- both Vanilla and DDNet are at `version` = 3
- `start_layer` and `num_layers` tell you which layers belong to this group.
  Groups are not allowed to overlap, however, the reference implementation has no such checks while loading.
- the 'Game' group, which is the only one that is allowed to hold physics layers, should have every field zeroed,
  only `x_parallax` and `y_parallax` should each be 100 and the `name` should be "Game".
  Note that the reference implementation does not verify this but instead just overwrites those values
- all maps must have a 'Game' group, since every map must have a 'Game' layer which can only be in the 'Game' group

Layers
------

- `type_id` = 5

Layer types:

- Tilemap layers:
    - Tiles layer
    - Physics layers:
        - Game layer
        - Front layer (DDNet only)
        - Tele layer (DDNet only)
        - Speedup layer (DDNet only)
        - Switch layer (DDNet only)
        - Tune layer (DDNet only)
- Quads layer
- Sounds layer (DDNet only)
- Deprecated Sounds layer (DDNet only, replaced by Sounds layer)

Note that:
1. All physics layers *should* be unique, but this isn't properly enforced on all DDNet maps.
   The reference implementation uses the last physics layer of each type.
2. All maps must have a Game layer

```
item_data base for all layer items (different types have different extensions):
    [1] _version (not used, was uninitialized)
    [1] type
    [1] flags
```

- `flags` currently only has the detail flag (at 2^0), which is used in Quad-, Tile- and Sound layers.
- `type` holds the type of layer:
    - 2 -> Tilemap layer
    - 3 -> Quads layer
    - 9 -> Deprecated Sounds layer
    - 10 -> Sounds layer

```
item_data extension for tilemap layers:
    [1] version
    [1] width
    [1] height
    [1] flags
    [4] color: Color
    [1] opt *color_envelope
    [1] color_envelope_offset
    [1] opt *image
    [1] &data: 2d-array of the the tile type 'Tile'
    
    version 3 extension:
    [3] name: I32String
    
    DDNet extension (no version change):
    [1] opt &data_tele
    [1] opt &data_speedup
    [1] opt &data_front
    [1] opt &data_switch
    [1] opt &data_tune
```

- Vanilla is at `version` = 4, DDNet at `version` = 3
- `width` and `height` specify the dimensions of the layer
- `flags` tells you what kind of tilemap layer this is:
    - 0 -> Tiles
    - 1 -> Game
    - 2 -> Tele
    - 4 -> Speedup
    - 8 -> Front
    - 16 -> Switch
    - 32 -> Tune
- `color`, `color_envelope`, `color_envelope_offset`, `image` are only used by the tiles layer

- all tile types consist of bytes (u8)
- all 2d-arrays of tiles use row-major ordering
- all tile types have the `id` byte, which identifies its use
    - for example in the game layer, 0 is air, 1 is hookable, etc.
- many have a `flags` byte, which is a bitflag with the following bits:
    - 2^0 -> vertical flip
    - 2^1 -> horizontal flip
    - 2^2 -> opaque
    - 2^3 -> 90° rotation
    - order of flips and rotations: vertical flip -> horizontal flip -> rotation

```
'Tile' tile type (consisting of bytes, used by all vanilla layers and the front layer):
    [1] id
    [1] flags
    [1] skip
    [1] - unused
```

- the `skip` byte is used for the 0.7 compression, which is used if `version` >= 4:
    - the `data` field no longer points to an 2d-array of tiles, but instead to an array of 'Tile' tiles which must be expanded into the 2d-array
    - the `skip` field of each tile in the array tells you how many times this tile is used in a row.
      For example:
        - 0 means that it appears only once there
        - 3 means that you need to add 3 more copies of that tile after this one
    - note that the maximum value for `skip` is 255
    - set the `skip` field to 0 while expanding
        - Teeworlds rendering assumes that those values are set to 0
        - saving the tiles with 0.7 compression is less tedious, since you can check for equality of the entire tile struct
        - saving the tiles again without the compression is less error prone.
          Since any saved map could be fed back into Teeworlds rendering, you need to set the `skip` values to 0 in this step

DDNet only content:
- each physics layer uses a different data field pointer, keep in mind to use the correct one, when saving maps, set the unused pointers to -1
- the DDNet extension came before the `version` = 3 extension, meaning you have to subtract 3 (the length of the `name` field) from the data index
- you might have noticed that the `data` field is not actually optional like all the other data fields.
  For vanilla compatibility, the `data` field always points to a 2d-array of tiles of the type 'Tile', with the same dimensions as the actual layer, but everything zeroed out

Special tile types:

```
'Tele' tile type (consisting of bytes):
    [1] number
    [1] id
```

- `number` is the number of the teleporter exit/entry to group them together

```
'Speedup' tile type (consisting of bytes):
    [1] force
    [1] max_speed
    [1] id
    [1] - unused padding byte
    [2] angle: i16
```

- angle is LE

```
'Switch' tile type (consisting of bytes):
    [1] number
    [1] id
    [1] flags
    [1] delay
```

- `number` once again tells you which tiles interact with each other

```
'Tune' tile type (consisting of bytes):
    [1] number
    [1] id
```

- `number` stores which zone this is, zones are defined in the map info -> settings

**Quads layer**

```
item_data extension for quads layers:
    [1] version
    [1] num_quads
    [1] &data: [Quads]
    [1] opt *image
    
    version 2 extension:
    [3] name: I32String
```

- both Vanilla and DDNet are at `version` = 2
- `num_quads` is the amount of quads found behind the data item pointer `data`
- the size of a quad in bytes is 152, however we will pretend that the data consists of i32 when looking at the Quad structure:

```
Quad:
    [8] positions: [Point; 5]
    [16] corner_colors: [Color; 4]
    [8] texture_coordinates: [Point; 4]
    [1] opt *position_envelope
    [1] position_envelope_offset
    [1] opt *color_envelope
    [1] color_envelope_offset
```

- `positions` elements 1 - 4 are the corner positions and `positions` element 5 contains the pivot
- to map the `positions` to world coordinates divide them by 512 
- corners are in the order top-left -> top-right -> bottom-left -> bottom-right
- the `texture_coordinates` are in range (0, 1024). To get the actual texture coordinates, divide by 1024 to normalize to (0, 1) range and multiply by the dimension of the quad image.

**Sounds layer**

```
item_data extension for sounds layers:
    [1] version
    [1] num_sources
    [1] &data: [SoundSource]
    [1] opt *sound
    [3] name: I32String
```

- num_sources is the amount of sources behind the data item pointer `data`
- the size of a sound source in bytes is 52, however we will pretend that the data consists of i32 when looking at the SoundSource structure:
- the CString behind `name` must fit into 128 bytes

```
SoundSource:
    [2] position: Point
    [1] looping: bool
    [1] panning: bool
    [1] delay (in seconds)
    [1] falloff: u8
    [1] *position_envelope
    [1] position_envelope_offset
    [1] *sound_envelope
    [1] sound_envelope_offset
    [3] shape: SoundShape

SoundShape:
    [1] kind
    [1] width  / radius
    [1] height / - unused
```

- `kind`:
    - 0 -> rectangle (use `width` and `height`)
    - 1 -> circle (use `radius`)

**Deprecated Sounds layer**

- the `item_data` is the same as in the Sounds layer
- difference is the SoundSource struct, which here only uses 36 bytes:

```
deprecated SoundSource:
    [2] position: Point
    [1] looping: bool
    [1] delay
    [1] radius
    [1] *position_envelope
    [1] position_envelope_offset
    [1] *sound_envelope
    [1] sound_envelope_offset
```

Use the following values to convert a deprecated SoundSource:
- `panning` = true
- `falloff` = 0
- `shape`: kind = circle, with shared `radius`

Sounds
------

- `type_id` = 7
- DDNet only

```
item_data of sound items:
    [1] version
    [1] external: bool
    [1] &name: CString
    [1] &data
    [1] data_size
```

- DDNet is at `version` = 1
- in theory, sounds can be external like images.
  However, since there are no sounds that could currently be loaded externally, this feature has been removed.
  This means that `external` should always be false and `data` should not be considered an option index
- the data item index `data` points to opus sound data

Auto Mappers
------------

- `uuid` = `16271b3e-8c17-7839-9bd9-b11ae041d0d8`
- DDNet only

```
item_data of auto mapper items:
    [1] _version (not used, was uninitialized)
    [1] *group
    [1] *layer
    [1] opt config
    [1] seed
    [1] flags
```

- `group` points to a group, `layer` is the layer index within the group
- `flags` currently only has the `automatic` flag at 2^0, which tells the client to auto map after any changes
- while only Tiles layer use auto mappers, physics layers may also have one. When saving, only save auto mappers for Tiles layers
