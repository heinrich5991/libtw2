Introduction
============

The Teeworlds datafile format is the format which Teeworlds uses to save its
game maps. Despite having been used for quite some time, it has not yet been
formally described. The format enables one to store fixed-size "items" along
with variable-sized "data items".

The format is designed in a way that makes it easy to directly load most parts
into the memory (i. e. in version 4 everything except for the data block, as
the data block is stored compressed in the file). In this document the versions
3 and 4 of Teeworlds datafiles will be explained.


Terminology
===========

The following is an abstract description of the data contained in a Teeworlds
datafile. It does not specify how they are laid out in the file.

Items
-----

An item consists of a 16-bit unsigned integer `type_id`, a 16-bit unsigned
integer `id` and an array of 32-bit signed integers `data`. The combination of
`type_id` and `id` is unique amongst all items. The length of `data` is usually
the same for all items of a given `type_id`.

Examples of types in actual Teeworlds maps include metadata for layers, layer
groups, images (external or not), etc. Since only the metadata and not the
actual contents are stored, the items can remain fixed-size.

Data items
----------

A data item is an array of bytes (8-bit unsigned integers) `data`. `id` is
unique amongst all data items, the only possible IDs are from 0 (incl.) to the
number of data items (excl.). These data items are indexed via unsigned
integers, counting sequentially in the order they are laid out in the file.

In actual Teeworlds maps, data items are used e.g. for the tiles of a tile
layer or the image data of an embedded image. They are referred to in the
metadata items by their index.


Format
======

Datafile
--------

The format of datafiles looks like follows, all parts are explained later:

    datafile:
        [  8] version_header
        [ 28] header
        [*12] item_types
        [* 4] item_offsets
        [* 4] data_offsets
        [* 4] _data_sizes
        [   ] items
        [   ] data

The `_data_sizes` part is only present in version 4 of Teeworlds datafiles.

The `header` contains size information for the rest of the file:

- `item_types` has the length of `header.num_item_types` item types.
- `item_offsets` has the length of `header.num_items` 32-bit integers.
- `data_offsets` has the length of `header.num_data` 32-bit integers.
- `_data_sizes` is only present in version 4 of Teeworlds datafiles, it has the
  length of `header.num_data` 32-bit integers.
- `items` has the length of `header.item_size` bytes which must be divisible by
  four.
- `data` has the length of `header.data_size` bytes.


Version header
--------------

The version header consists of a magic byte sequence, identifying the file as a
Teeworlds datafile and a version number.

    version_header:
        [4] magic
        [4] version

The `magic` must exactly be the ASCII representations of the four characters,
'D', 'A', 'T', 'A'.

NOTE: Readers of Teeworlds datafiles should be able to read datafiles which
start with a reversed `magic` too, that is 'A', 'T', 'A', 'D'. A bug in the
reference implementation caused big-endian machines to save the reversed
`magic` bytes.

The `version` is a little-endian signed 32-bit integer, for version 3 or 4 of
Teeworlds datafiles, it must be 3 or 4, respectively.


Header
------

The header specific to version 3 and 4 consists of seven 32-bit signed
integers.

    header:
        [4] size
        [4] swaplen
        [4] num_item_types
        [4] num_items
        [4] num_data
        [4] item_size
        [4] data_size

The `size` is a little-endian integer and must be the size of the complete
datafile without the `version_header` and both `size` and `swaplen`.

NOTE: The reference implementation does not read this value.

The `swaplen` is a little-endian integer and must specify the number of
integers following the following the `size` and `swaplen` fields, up until the
data of the data items. It can therefore be used to reverse the endian on
big-endian machines.

NOTE: The reference implementation does not read datafiles correctly on
little-endian machines, because it interprets `swaplen` as starting after the
header.

NOTE: All further integers can be assumed to be already converted to
machine-native endian, if an endian swap was performed using the `swaplen`
field.

The `num_item_types` integer specifies the number of item types in the
`datafile.item_types` field.

The `num_items` integer specifies the number of items in the `datafile.items`
field.

The `num_data` integer specifies the number of raw data blocks in the
`datafile.data` field.

The `item_size` integer specifies the total size in bytes of the
`datafile.items` field.

The `data_size` integer specifies the total size in bytes of the
`datafile.data` field.


Item types
----------

The item types are an array of item types. The number of item types in that
array is `num_item_types`, each item type is identified by its unique `type_id`
(explained below). Each item type is of the following form:

    item_type:
        [4] type_id
        [4] start
        [4] num

The `type_id` 32-bit signed integer must be unique amongst all other
`item_type.type_id`s. Its value must fit into an unsigned 16-bit integer.

The `start` signed integer is the index of the first item in the `items` with
the type `type_id`.

The `num` signed integer must be the number of items with the the type
`type_id`.

NOTE: Since all items of the same type must be sequential in the `items` array,
exactly the items with the index `start` (incl.) to `start + num` (excl.) are
of the type `type_id`.


Item offsets, data offsets and data sizes
-----------------------------------------

The item offsets, the data offsets and the data sizes are 32-bit signed
integers.

Each item offset is the offset of the item with the corresponding index,
relative to the first item's position in the file.

Each data offset is an offset of the data with the corresponding index,
relative to the position of the first data item in the file. The data item's
size can then be calculated from the next data item's offset or the size of the
data section.

Each data size is the size of the uncompressed data of the data with the
corresponding index. Note that this field is only present in datafile version
4.


Items
-----

This is an array of items. Each is of the following form:

    item:
        [4] type_id__id
        [4] size
        [ ] item_data

The `type_id__id` integer consists of 16 bit `type_id` of the type the item
belongs to and 16 bit `id` that uniquely identifies the item among all others
of the same type, in that order, i.e. the upper 16 bit of `type_id__id` specify
the `type_id` and the lower 16 bit specify `id`.

The `size` signed 32-bit integer is the size of the `item_data` field, in
bytes, which must be divisible by four.

NOTE: Neither the `type_id` nor the `size` are interpreted by the reference
implementation.


Data
----

This section contains the data items. The order of the data items implicitly
defines their ID.

In version 3, this section solely consists of the concatenated data. In version
4, however, it stores the data compressed by zlib's `compress` function.
