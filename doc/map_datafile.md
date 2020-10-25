Introduction
============

Teeworlds and DDNet maps get saved as datafiles.
If you are not yet familiar with parsing datafiles, please go through the datafile documentation first.
Here we will assume that you know the datafile terminology.


-----------------------------------------------

Terminology
===========

**Data item indices**  point to a data item in the `datafile.data` section of the datafile. They will be prefixed with `&`.
**Optional data item index** on the other hand might also equal `-1`, meaning no data item is used. They will be prefixed with `opt &`

**CString** is a null terminated string.

The `item_data` of an item will be considered as an array of i32.
We will split the `item_data` up into its different elements, which differ for each item type.

Examples for the `item_data` syntax:
 
1. `[3] color: u8`: The next three i32 values represent the variable `color` (which will be explained afterwards) and each value should be in the range of an u8.
2. `[1] opt &name: CString`: `name` is a optional data item index on a CString.

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

- `version` = 1 for both Vanilla and DDNet
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
- if `version` = 1, the image is of type RGBA, for `version` = 2 `variant` holds the type
    - `variant` 0 -> RGB, `variant` = 1 -> RGBA
- Images can either be embedded or external.
    - Embedded images have `external` = false and have the image data stored in the data field.
    The image data is simply a 2d-array of pixels.
    RGBA pixels are 4 bytes each, RGB pixels 3 bytes each.
    - External images have `external` = true and the `data` field on `-1`.
    Those images can only be loaded by clients that have those in their `mapres` directory, meaning only a small set of images should be external.
    The client looks for those images by using the `name` field.

