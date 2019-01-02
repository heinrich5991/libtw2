Unless specified otherwise, all of the following sizes are measured in bytes.

    demo:
        [  8] version_header
        [168] header
        [260] _timeline_markers
        [   ] map
        [   ] data

`_timeline_markers` are only available in version 4 and 5.

The length of the `map` field is determined by the `map_size` field in
`header`. The `map` field contains a Teeworlds map (datafile).

`data` consists of many `chunk`s laid out one after the other.

    version_header:
        [  7] magic
        [  1] version

`magic` is "TWDEMO\0".

`version` is any number from 3 to 5 for this document.

    header:
        [ 64] net_version
        [ 64] map_name
        [  4] map_size
        [  4] map_crc
        [  8] type
        [  4] length
        [ 20] timestamp

`net_version`, `map_name`, `type` and `timestamp` are null-terminated strings.
`map_size`, `map_crc` and `length` are signed big-endian 32-bit integers.

`map_size` determines the size of the `map` field of the demo.

`type` should be either "client" or "server" for a client or server demo,
respectively.

    _timeline_markers:
        [  4] num_timeline_markers
        [256] timeline_markers

`num_timeline_markers` is a signed big-endian 32-bit integer.
`timeline_markers` is an array of signed big-endian 32-bit integers of size 64.

`num_timeline_markers` gives the number of valid `timeline_markers`. This
number should be less or equal to 64.

    chunk:
        [   ] chunk_header
        [   ] chunk_data

`chunk_header` is at least a single byte. If the `chunk` does not indicate a
new tick, it looks like follows (sizes specified in bits):

    chunk_header_normal_first:
        [1] is_tick (set to 0 for normal chunks)
        [2] type
        [5] size

    0ttS SSSS

`is_tick` is set to `0` for normal chunks.

`type` can be one of `1` (snapshot), `2` (message) or `3` (snapshot delta).

`size` determines the size of the following `chunk_data` unless one of the
special values `30` or `31` are used. `30` indicates that `chunk_header`
contains one additional byte whose numerical value specifies the length of the
following `chunk_data`, `31` indicates that `chunk_header` contains one
additional little-endian 16-bit integer specifying the length of the following
`chunk_data`.

If the `chunk` does indicate a new tick, its first byte looks like follows in
version 3 and 4 (sizes specified in bits):

    chunk_header_tick_first_34:
        [1] is_tick (set to 1 for tick markers)
        [1] keyframe
        [6] tick_delta

    1kDD DDDD

`is_tick` is set to `1` for tick markers.

`keyframe` is a hint to the demo player that the next tick will contain a full
snapshot, which allows it to jump to that specific position.

`tick_delta` either specifies the tick delta (delta = difference) to the
previous tick, or it is `0` to indicate that a big-endian 32-bit integer
specifying the absolute tick follows.

In version 5, the first byte of the `chunk_header` looks like follows if it
does indicate a new tick (sizes specified in bits):

    chunk_header_tick_first_5:
        [1] is_tick (set to 1 for tick markers)
        [1] keyframe
        [1] inline_tick
        [5] tick_delta

    1kiD DDDD

`is_tick` and `keyframe` have the same meaning as in version 3 and 4. If
`inline_tick` is `1`, `tick_delta` specifies the tick delta. If it is `0`,
`tick_delta` is padding that should be zeroed and a big-endian 32-bit integer
specifying the absolute tick follows.
