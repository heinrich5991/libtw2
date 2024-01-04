meta:
  id: tw_demo_v3_v4_v5_v6
  file-extension: demo
  endian: be
  license: MIT/Apache-2.0
doc-ref: https://github.com/heinrich5991/libtw2/blob/0085b3eb76ff1ffc7136f874129c79fce0f955ee/doc/demo.md
seq:
  - id: header
    type: header
  - id: timeline_markers
    type: timeline_markers
    if: header.version >= 4
  - id: map_sha256
    type: map_sha256
    if: header.version >= 6
  - id: map
    size: header.map_size
  - id: chunks
    type: chunk
    repeat: eos
enums:
  chunk_type:
    1: snapshot
    2: message
    3: snapshot_delta
types:
  header:
    seq:
      - id: magic
        contents: ['TWDEMO', 0]
      - id: version
        type: u1
      - id: net_version
        type: strz
        encoding: utf8
        size: 64
      - id: map_name
        type: strz
        encoding: utf8
        size: 64
      - id: map_size
        type: s4
      - id: map_crc
        type: u4
      - id: type
        type: strz
        encoding: utf8
        size: 8
      - id: length
        type: s4
      - id: timestamp
        type: strz
        encoding: utf8
        size: 20
  timeline_markers:
    seq:
      - id: num_timeline_markers
        type: s4
      - id: timeline_marker
        type: s4
        repeat: expr
        repeat-expr: 64
  map_sha256:
    seq:
      - id: magic
        contents: [0x6b, 0xe6, 0xda, 0x4a, 0xce, 0xbd, 0x38, 0x0c, 0x9b, 0x5b, 0x12, 0x89, 0xc8, 0x42, 0xd7, 0x80]
      - id: map_sha256
        size: 32
  chunk:
    seq:
      - id: is_tick
        type: b1

      # tick
      - id: keyframe
        type: b1
        if: is_tick
      - id: inline_tick_delta
        type: b1
        if: _root.header.version >= 5 and is_tick
      - id: tick_delta_v5
        type: b5
        if: _root.header.version >= 5 and is_tick
      - id: tick_delta_v3
        type: b6
        if: _root.header.version < 5 and is_tick

      # non-tick
      - id: type
        type: b2
        enum: chunk_type
        if: not is_tick
      - id: size_inline
        type: b5
        if: not is_tick

      # tick
      - id: tick_absolute
        type: s4
        if: is_tick and (_root.header.version >= 5 ? not inline_tick_delta : tick_delta_v3 == 0)

      # non-tick
      - id: size_extern_8
        type: u1
        if: not is_tick and size_inline == 30
      - id: size_extern_16
        type: u2le
        if: not is_tick and size_inline == 31
      - id: compressed_data
        size: size

    instances:
      tick_delta:
        value: _root.header.version >= 5 ? tick_delta_v5 : tick_delta_v3
        if: is_tick and (_root.header.version >= 5 ? inline_tick_delta : tick_delta_v3 != 0)
      size:
        value: size_inline == 31 ? size_extern_16 : size_inline == 30 ? size_extern_8 : size_inline
        if: not is_tick
