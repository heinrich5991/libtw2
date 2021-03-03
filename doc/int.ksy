meta:
  id: tw_int
  title: Variable length quantity, signed integer, at most 32 bit, little-endian
  ks-version: 0.7
doc: |
  A variable-length signed integer. The highest bit of each byte specifies
  whether the number continues after this byte. The first byte also has the
  sign flag as the second-to-highest bit. The rest of the bits are used as
  binary digits of the represented number.

  The number can consist of 1 to 5 bytes, if it's five bytes long, the last
  byte's four upper bits (including the continuation bit) should be ignored.

  If the sign bit is set, the number's value should have all of its bits
  (including the sign bit) flipped, assuming two's complement.

seq:
  - id: byte0
    type: first_byte
  - id: byte1
    type: continuation_byte
    if: byte0.has_continuation
  - id: byte2
    type: continuation_byte
    if: byte0.has_continuation and byte1.has_continuation
  - id: byte3
    type: continuation_byte
    if: byte0.has_continuation and byte1.has_continuation and byte2.has_continuation
  - id: byte4
    type: last_byte
    if: byte0.has_continuation and byte1.has_continuation and byte2.has_continuation and byte3.has_continuation
types:
  first_byte:
    seq:
      - id: has_continuation
        type: b1
      - id: sign
        type: b1
      - id: bits
        type: b6
  continuation_byte:
    seq:
      - id: has_continuation
        type: b1
      - id: bits
        type: b7
  last_byte:
    seq:
      - id: padding
        type: b4
      - id: bits
        type: b4
instances:
  len:
    value: >-
      (not byte0.has_continuation ? 1 :
      (not byte1.has_continuation ? 2 :
      (not byte2.has_continuation ? 3 :
      (not byte3.has_continuation ? 4 : 5))))
    doc: Number of bytes this value occupies
  value:
    value: >-
      (byte0.bits
      | (len >= 2 ? byte1.bits << 6 : 0)
      | (len >= 3 ? byte2.bits << 13 : 0)
      | (len >= 4 ? byte3.bits << 20 : 0)
      | (len >= 5 ? byte4.bits << 27 : 0))
      ^ (byte0.sign ? ~0 : 0)
    doc: Resulting value as normal integer
