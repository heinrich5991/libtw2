meta:
  id: tw_map_v4
  file-extension: map
  endian: le
  license: MIT/Apache-2.0
doc-ref: https://github.com/heinrich5991/libtw2/blob/b510f20bc58ceb33f38ddc555b63989ccf5a90d7/doc/datafile.md
seq:
  - id: header
    type: header
  - id: item_types
    type: item_type
    repeat: expr
    repeat-expr: header.num_item_types
  - id: item_offsets
    type: s4
    repeat: expr
    repeat-expr: header.num_items
  - id: data_offsets
    type: s4
    repeat: expr
    repeat-expr: header.num_data
  - id: data_sizes
    type: s4
    repeat: expr
    repeat-expr: header.num_data
  - id: items
    type: item
    repeat: expr
    repeat-expr: header.num_items
  - id: data_items
    process: zlib
    type: dummy
    size: (_index == header.num_data - 1 ? header.data_size : data_offsets[_index + 1]) - data_offsets[_index]
    repeat: expr
    repeat-expr: header.num_data
types:
  # the dummy type is zero-sized. it leaves data streams complete and doesn't consume them, when instanciated
  # later parsing is only possible with data streams that have unparsed data in them
  # data_items and envelope points are not directly parsed, but only afterwards in instances, because they require more context
  dummy: {}
  header:
    seq:
      - id: magic
        contents: 'DATA'
      - id: version
        contents: "\x04\x00\x00\x00"
      - id: size
        type: s4
      - id: swaplen
        type: s4
      - id: num_item_types
        type: s4
      - id: num_items
        type: s4
      - id: num_data
        type: s4
      - id: item_size
        type: s4
      - id: data_size
        type: s4
  item_type:
    seq:
      - id: type_id
        type: s4
        enum: item_kind
      - id: start
        type: s4
      - id: num
        type: s4
  # In order to make values more readable
  fixed_point:
    params:
      - id: divisor
        type: f4
    seq:
      - id: x_raw
        type: s4
      - id: y_raw
        type: s4
    instances:
      x:
        value: x_raw / divisor
      y:
        value: y_raw / divisor
  color:
    seq:
      - id: r
        type: s4
      - id: g
        type: s4
      - id: b
        type: s4
      - id: a
        type: s4
  optional_string_data_index:
    seq:
      - id: data_index
        type: s4
    instances:
      string:
        if: data_index != -1
        io: _root.data_items[data_index]._io
        type: str
        encoding: UTF-8
        size-eos: true
  optional_multiple_strings_data_index:
    seq:
      - id: data_index
        type: s4
    instances:
      strings:
        if: data_index != -1
        io: _root.data_items[data_index]._io
        type: str
        terminator: 0
        encoding: UTF-8
        repeat: eos
  i32x3_string:
    seq:
      - id: data
        size: 4
        process: xor(0b10000000)
        repeat: expr
        repeat-expr: 3
    instances:
      string:
        value: data[0].to_s("UTF-8").reverse
          + data[1].to_s("UTF-8").reverse
          + data[2].to_s("UTF-8").substring(1, 4).reverse
  i32x8_string:
    seq:
      - id: data
        size: 4
        process: xor(0b10000000)
        repeat: expr
        repeat-expr: 8
    instances:
      string:
        value: data[0].to_s("UTF-8").reverse
          + data[1].to_s("UTF-8").reverse
          + data[2].to_s("UTF-8").reverse
          + data[3].to_s("UTF-8").reverse
          + data[4].to_s("UTF-8").reverse
          + data[5].to_s("UTF-8").reverse
          + data[6].to_s("UTF-8").reverse
          + data[7].to_s("UTF-8").substring(1, 4).reverse

  unknown_item:
    seq:
      - id: item_data
        type: s4
        repeat: eos
  item:
    seq:
      - id: id
        type: u2
      - id: type_id
        type: u2
        enum: item_kind
      - id: data_size
        type: s4
      - id: content
        type:
          switch-on: type_id
          cases:
            item_kind::version: version_item
            item_kind::info: info_item
            item_kind::image: image_item
            item_kind::envelope: envelope_item
            item_kind::group: group_item
            item_kind::layer: layer_item
            item_kind::env_points: env_points_item
            item_kind::ex_type_index: ex_type_index_item
            _: unknown_item
        size: data_size
  
  version_item:
    seq:
      - id: version
        type: s4
  
  info_item:
    seq:
      - id: item_version
        type: s4
      - id: author
        type: optional_string_data_index
      - id: version
        type: optional_string_data_index
      - id: credits
        type: optional_string_data_index
      - id: license
        type: optional_string_data_index
      - id: settings
        if: not _io.eof
        type: optional_multiple_strings_data_index
  
  image_item:
    seq:
      - id: version
        type: s4
      - id: width
        type: s4
      - id: height
        type: s4
      - id: external
        enum: bool
        type: s4
      - id: name
        type: optional_string_data_index
      - id: data_index
        enum: optional
        type: s4
  envelope_item:
    seq:
      - id: version
        type: s4
      - id: kind
        type: s4
        enum: envelope_kind
      - id: first_point_index
        type: s4
      - id: envelope_amount
        type: s4
      - id: name
        if: not _io.eof
        type: i32x8_string
      - id: synchronized
        if: version >= 2
        enum: bool
        type: s4
  
  group_item:
    seq:
      - id: version
        type: s4
      - id: offset
        type: fixed_point(32.)
        type: s4
      - id: parallax
        type: fixed_point(100.)
        type: s4
      - id: first_layer_index
        type: s4
      - id: layer_amount
        type: s4
      - id: clipping
        if: version >= 2
        enum: bool
        type: s4
      - id: clip_position
        if: version >= 2
        type: fixed_point(32.)
      - id: clip_size
        if: version >= 2
        type: fixed_point(32.)
        type: s4
      - id: name
        type: i32x3_string
  
  layer_item:
    seq:
      - id: unused_version
        type: s4
      - id: type
        type: s4
        enum: layer_kind
      - id: flags
        enum: layer_flags
        type: s4
      - id: content
        type:
          switch-on: type
          cases:
            layer_kind::tilemap: tilemap_layer_item
            layer_kind::quads: quads_layer_item
            layer_kind::sounds: sounds_layer_item
  tilemap_layer_item:
    seq:
      - id: version
        type: s4
      - id: width
        type: s4
      - id: height
        type: s4
      - id: type
        enum: tilemap_flags
        type: s4
      - id: color
        type: color
      - id: color_envelope_index
        type: s4
      - id: color_envelope_offset
        type: s4
      - id: image_index
        enum: optional
        type: s4
      - id: tiles_data_index
        type: s4
      - id: name
        type: i32x3_string
      - id: tele_data_index
        if: not _io.eof
        enum: optional
        type: s4
      - id: speedup_data_index
        if: not _io.eof
        enum: optional
        type: s4
      - id: front_data_index
        if: not _io.eof
        enum: optional
        type: s4
      - id: switch_data_index
        if: not _io.eof
        enum: optional
        type: s4
      - id: tune_data_index
        if: not _io.eof
        enum: optional
        type: s4
  quads_layer_item:
    seq:
      - id: version
        type: s4
      - id: quad_amount
        type: s4
      - id: data_index
        type: s4
      - id: image_index
        enum: optional
        type: s4
      - id: name
        if: version >= 2
        type: i32x3_string
    instances:
      quads:
        io: _root.data_items[data_index]._io
        type: quad
        repeat: eos
  quad:
    seq:
      - id: top_left_position
        type: fixed_point(1024. * 32.)
      - id: top_right_position
        type: fixed_point(1024. * 32.)
      - id: bot_left_position
        type: fixed_point(1024. * 32.)
      - id: bot_right_position
        type: fixed_point(1024. * 32.)
      - id: position
        type: fixed_point(1024. * 32.)
      - id: corner_colors
        type: color
        repeat: expr
        repeat-expr: 4
      - id: texture_coordinates
        type: fixed_point(1024.)
        repeat: expr
        repeat-expr: 4
      - id: position_envelope_index
        enum: optional
        type: s4
      - id: position_envelope_offset
        type: s4
      - id: color_envelope_index
        enum: optional
        type: s4
      - id: color_envelope_offset
        type: s4
  sounds_layer_item:
    seq:
      - id: version
        type: s4
      - id: source_amount
        type: s4
      - id: data_index
        type: s4
      - id: sound_index
        enum: optional
        type: s4
      - id: name
        type: i32x3_string
    instances:
      sound_sources:
        io: _root.data_items[data_index]._io
        type: sound_source
        repeat: eos
  sound_source:
    seq:
      - id: position
        type: fixed_point(1024. * 32.)
      - id: looping
        enum: bool
        type: s4
      - id: panning
        enum: bool
        type: s4
      - id: delay
        type: s4
      - id: falloff
        type: s4
      - id: position_envelope_index
        enum: optional
        type: s4
      - id: position_envelope_offset
        type: s4
      - id: sound_envelope_index
        enum: optional
        type: s4
      - id: sound_envelope_offset
        type: s4
      - id: shape
        enum: sound_source_shape
        type: s4
      - id: dimensions
        type: fixed_point(1024. * 32.)
  
  env_points_item:
    seq: []
    instances:
      ddnet_points:
        io: _io
        type: env_point
        repeat: eos
      teeworlds07_points:
        io: _io
        type: env_point_with_bezier
        repeat: eos
  env_point:
    seq:
      - id: time
        type: s4
      - id: curve_type
        enum: curve_kind
        type: s4
      - id: values
        type: s4
        repeat: expr
        repeat-expr: 4
    instances:
      time_ms:
        value: time / 1000.
  bezier:
    seq:
      - id: handle_in
        type: fixed_point(1024.)
      - id: handle_out
        type: fixed_point(1024.)
  env_point_with_bezier:
    seq:
      - id: point
        type: env_point
      - id: bezier
        type: bezier
  
  ex_type_index_item:
    seq:
      - id: uuid
        size: 16

enums:
  bool:
    0: false
    1: true
  optional:
    -1: not_set
  item_kind:
    0: version
    1: info
    2: image
    3: envelope
    4: group
    5: layer
    6: env_points
    7: sound
    0xffff: ex_type_index
  envelope_kind:
    4: color
    3: position
    1: volume
  layer_kind:
    2: tilemap
    3: quads
    9: deprecated_sounds
    10: sounds
  layer_flags:
    0: not_quality
    1: quality
  tilemap_flags:
    0x0000_0000: tiles
    0x0000_0001: game
    0x0000_0010: tele
    0x0000_0100: speedup
    0x0000_1000: front
    0x0001_0000: switch
    0x0010_0000: tune
  curve_kind:
    0: step
    1: linear
    2: slow
    3: fast
    4: smooth
    5: bezier
  sound_source_shape:
    0: rectangle
    1: circle
 
