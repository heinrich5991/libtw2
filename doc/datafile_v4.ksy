meta:
  id: tw_datafile_v4
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
  - id: data
    size: (_index == header.num_data - 1 ? header.data_size : data_offsets[_index + 1]) - data_offsets[_index]
    repeat: expr
    repeat-expr: header.num_data
types:
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
      - id: start
        type: s4
      - id: num
        type: s4
  item:
    seq:
      - id: id
        type: u2
      - id: type_id
        type: u2
      - id: size
        type: s4
      - id: item_data
        type: s4
        repeat: expr
        repeat-expr: size / 4