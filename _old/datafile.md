
Introduction
============

The general format of a Teeworlds datafile is designed in a way that makes it
easy to directly load most parts into the memory (i. e. everything except for
the data block). In this document the versions 3 and 4 of Teeworlds datafiles
will be explained.


Datafiles
=========

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

Note: The `_data_sizes` part is only present in version 4 of Teeworlds
datafiles.


Version header
--------------

The version header consists of a magic, identifying the file as a Teeworlds
datafile and a version number.

    version_header:
        [4] magic
        [4] version

The `magic` must exactly be the ASCII representations of the four characters,
'D', 'A', 'T', 'A'.

Note: Readers of Teeworlds datafiles should be able to read datafiles which
start with a reversed `magic` too, that is 'A', 'T', 'A', 'D'. A bug in the
reference implementation caused big-endian machines to save the reversed
`magic` bytes.

The `version` is a little-endian integer, for version 3 or 4 of Teeworlds
datafiles, it must be 3 or 4, respectively.


Header
------

The header specific to version 3 and 4 consists of seven integers.

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

The `swaplen` is a little-endian integer and must specify the number of
integers following little-endian-encoded integers. It can therefore be used to
reverse the endian on big-endian machines.

Note: All further integers can be assumed to be already converted to
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

The `type_id` integer must be unique amongst all other `item_type.type_id`s. It
represents.

The `start` integer is the index of the first item in the `items` with the type
`type_id`.

The `num` integer must be the number of items with the the type `type_id`.

Note: Since all items of the same type must be sequential in the `items` array,
exactly the items with the index `start` to `start` + `num` - 1 are of the type
`type_id`.


Item offsets, data offsets and data sizes
-----------------------------------------

The item offsets, the data offsets and the data sizes are arrays integers.

Each item offset is the offset of the item with the corresponding index,
relative to the first item's position in the file.

Each data offset are offset of the data with the corresponding index, relative
to the position of the first data in the file.

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
of the same type, in that order.

The `size` integer is the size of the `item_data` field, in bytes.

Note: Neither the `type_id` nor the `size` are interpreted by the reference
implementation.


Data
----

This is an array of data.
