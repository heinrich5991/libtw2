    demo:
        [  8] version_header
        [168] header
        [260] _timeline_markers
        [   ] map
        [   ] data

`_timeline_markers` are only available in version 4 and 5.

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

`net_version`, `map_name`, `type` and `timestamp` are nul-terminated strings.
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
