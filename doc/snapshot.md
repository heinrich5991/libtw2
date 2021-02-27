Introduction
============

Teeworlds uses this format of so called "snapshots" to transmit the current
state of the world over the network. The format allows one to transmit
fixed-size "items".


Terminology
===========

All integers in this document are little-endian integers.


Item key
--------

An item key is a 32-bit unsigned integer that contains the 16-bit unsigned
integer `type_id` as the upper bits and `id` as the lower bits.


Items
-----

An item consists of an item key `key` and an array of 32-bit signed integers
`data`.


Snapshot
--------

A snapshot is a collection of items identified by their key.


Format
======

Checksum calculation
--------------------

The checksum of a snapshot is calculated by summing all the `data` array
elements of all items, using wrapping overflow behavior.


Snapshot deltas
---------------

    snapshot_delta:
        [ 4] num_removed_items
        [ 4] num_item_deltas
        [ 4] _zero
        [*4] removed_item_keys
        [  ] item_deltas

Snapshot deltas are used to describe the differences between two snapshots, the
"old" and the "new" one. You can use the delta together with the old snapshot
to construct the new one. Note that the reverse does not work, because the
values of snapshot items present in the old but not the new snapshot are not
recorded in the delta.

- `num_removed_items` is a positive signed 32-bit integer describing the length
  of the `removed_item_keys` array.
- `num_item_deltas` is a positive signed 32-bit integer describing the number
  of item deltas in the `item_deltas` field.
- `_zero` is a padding field which must be zeroed by any snapshot delta writer
  and ignored by any snapshot delta reader.
- `removed_item_keys` is an array of `num_removed_items` item keys that are
  present in old snapshot, but not in the new one.
- `item_deltas` is a concatenation of `num_item_deltas` item deltas.


Item deltas
-----------

    item_delta:
        [ 4] type_id
        [ 4] id
        [ 4] _size
        [*4] data_delta

- `type_id` is a 32-bit unsigned integer containing the 16-bit type id of the
  item.
- `id` is 32-bit unsigned integer containing the 16-bit id of the item.
- `_size` is a field that is only present if the size of items of type
  `type_id` is not pre-agreed for the protocol. A list of pre-agreed item sizes
  can be found in the appendix of this document.
- `data_delta` is the data delta (an array of `_size` 32-bit signed integers).
  If the item was not present in the old snapshot (determined by the item key),
  then `data_delta` just contains the new item data. Otherwise, the
  `data_delta` needs to be added onto the current item's data (elementwise)
  using 32-bit integer addition that wraps around on overflow. Note that in the
  case of item update, the new size must be the same as the old size of the
  item data.


Appendix
========

Pre-agreed item sizes
---------------------

This describes the 0.6 protocol of Teeworlds.

| `type_id` | `size` | name                   |
| --------: | -----: | ---------------------- |
|        1  |    10  | obj_player_input       |
|        2  |     6  | obj_projectile         |
|        3  |     5  | obj_laser              |
|        4  |     4  | obj_pickup             |
|        5  |     3  | obj_flag               |
|        6  |     8  | obj_game_info          |
|        7  |     4  | obj_game_data          |
|        8  |    15  | obj_character_core     |
|        9  |    22  | obj_character          |
|       10  |     5  | obj_player_info        |
|       11  |    17  | obj_client_info        |
|       12  |     3  | obj_spectator_info     |
|       13  |     2  | event_common           |
|       14  |     2  | event_explosion        |
|       15  |     2  | event_spawn            |
|       16  |     2  | event_hammerhit        |
|       17  |     3  | event_death            |
|       18  |     3  | event_sound_global     |
|       19  |     3  | event_sound_world      |
|       20  |     3  | event_damage_indicator |

