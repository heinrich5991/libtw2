Introduction
============

Teeworlds and DDNet maps get saved as datafiles.
If you are not yet familiar with parsing datafiles, please go through the datafile documentation first.
Here we will assume that you know the datafile terminology.


-----------------------------------------------

Terminology
===========

**Data item indices**  point to a data item in the `datafile.data` section of the datafile.
They will be prefixed with `&`.
**Optional data item index** on the other hand might also equal `-1`, meaning no data item is used.
They will be prefixed with `opt &`

**CString** is a null terminated string.

**I32String** is a CString stored in consecutive i32 values.
To extract the string:
1. convert the i32s to their be (big endian) byte representation, join the bytes so that we have a single array of bytes
2. the last byte is a null byte, ignore that one for now
3. wrapping-subtract 128 from the remaining bytes
4. now you got a CString padded with zeroes.

**Point** is a struct with 2 i32, one for x, one for y.
It is usually used to describe a position in the map.
0, 0 is the top-left corner.

The `item_data` of an item will be considered as an array of i32.
We will split the `item_data` up into its different elements, which differ for each item type.

Examples for the `item_data` syntax:
 
1. `[2] point: Point` => The next two i32 values represent the variable `point` (which will be explained afterwards) which is of the type `Point`.
2. `[1] opt &name: CString` => `name` is an optional data item index on a CString.

Item Type Overview
==================

Maps consist of various elements that each have a `type_id` that identifies them.

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
        
Use them to figure out which purpose each of the item types in the `datafile.item_types` section of the datafile has.

Things to keep in mind:
1. When an item type appears in `datafile.item_types`, it means that there must be at least one item of that type
2. With the exception fo the UUID Index, the first item of an item type will have `id` = 0 and from there it will count up

UUID item types
---------------

In DDNet, some item types won't be assigned a type_id, but instead an uuid.

    uuid mappings:
        [0x3e,0x1b,0x27,0x16,0x17,0x8c,0x39,0x78,0x9b,0xd9,0xb1,0x1a,0xe0,0x41,0xd,0xd8] -> Auto mappers

To find the correct item type (in `datafile.item_types` for uuid item types, you will need their `type_id`.
You will need to figure out the `type_id` manually by looking into the **UUID Index items**.

    UUID Index Item structure:
        type_id: 0xffff
        id: type_id of the uuid item type that this item represents
        item_data:
            [3] UUID of the uuid item type that this item represents

The twelve bytes of the uuid are laid out in order in the `item_data`.

Let's suppose we are looking for the auto mapper items. What we will do is:

1. get the UUID item type
2. scan through its items
3. when an item has the correct uuid, copy the `type_id` from the `id` field
4. find the item type with the `type_id` that we just found out

Map Item Types
==============

Version
---------

- `type_id` = 0
- exactly one item


    item_data of the only version item:
        [1] version

`version` should always be set to `1`.

Info
------

- `type_id` = 1
- exactly one item


    item_data of the only version item:
        [1] (item) version
        [1] opt &author: CString
        [1] opt &version: CString
        [1] opt &credits: CString
        [1] opt &license: CString
        [1] opt &settings: [CString] (DDNet only)

- both vanilla and DDNet are at `version` = 1
- like indicated, all the other fields are optional data item indices
- the data item behind `settings` is an array of CStrings, all consecutive, split by their null bytes (with a null byte at the very end)

Images
------

- `type_id` = 2


    item_data of image items:
        [1] version
        [1] width
        [1] height
        [1] external: bool
        [1] &name: CString
        [1] opt &data: [Pixel]

        version 2 extension (Vanilla only):
        [1] variant

- Vanilla is at `version` = 2, DDNet is at `version` = 1
- `width` and `height` specify the dimensions of the image
- if `version` = 1, the image is of type RGBA, for `version` = 2 `variant` holds the type:
    - 0 -> RGB
    - 1 -> RGBA
- Images can either be embedded or external.
    - Embedded images have `external` = false and have the image data stored in the data field.
    The image data is simply a 2d-array of pixels.
    RGBA pixels are 4 bytes each, RGB pixels 3 bytes each.
    - External images have `external` = true and the `data` field on `-1`.
    Those images can only be loaded by clients that have those in their `mapres` directory, meaning only a small set of images should be external.
    The client looks for those images by using the `name` field.

Envelopes
---------

- `type_id` = 3

    item_data of envelope items:
        [1] version
        [1] channels
        [1] start_point
        [1] num_points
        
        extension without version change:
        [8] name: I32String
        
        version 2 extension:
        [1] synchronized: bool

- DDNet is at `version` = 2, Vanilla chooses 3 for all envelopes when one of them uses a bezier curve, but falls back to 2 when they is none.
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

    sound envelope point:
        [1] time
        [1] curve type
        [1] volume
        [3] -

    position envelope point:
        [1] time
        [1] curve_type
        [2] point: Point
        [1] rotation
        [1] -
    
    color envelope point:
        [1] time
        [1] curve type
        [4] color: I32Color

- `time` is the timestamp of the point, it should increase monotonously within each envelope
- `curve_type` holds how the curve should bend between this point and the next one
    - 0 -> Step (abrupt drop at second value)
    - 1 -> Linear (linear value change)
    - 2 -> Slow (first slow, later much faster value change)
    - 3 -> Fast (first fast, later much slower value change)
    - 4 -> Smooth (slow, faster, then once more slow value change)
    - 5 -> Bezier (very customizable curve)

- `point` holds the x and y movement
- **I32Color** actually means that the color values for r, g, b, a are i32 values

If bezier curves are used anywhere (envelope version 3), then there are 16 more i32 for each point.
These are only non-zero if the `curve_type` of the point is 5 (Bezier):

    bezier point extension:
        [4] in_tangent_dx
        [4] in_tangent_dy
        [4] out_tangent_dx
        [4] out_tangent_dy
