- Huffman compression has an extra byte if the actual end is at a byte
  boundary.
- The size field has a weird splitting in the chunk header.
- The ack field of the packet header is 12 instead of 10 bit wide.
- The sequence field of the chunk header has overlapping bits.
- The client doesn't care about the actual lengths of the snapshot chunks,
  it'll always pad to `MAX_SNAPSHOT_PADSIZE`.
- The client wants to receive the last snapshot part last, otherwise the
  resulting delta is too long.
- CCharacterCore has unused fields `m_HookDx`, `m_HookDy`
- The packet payload can be empty. This happens when chunks from the peer are
  lost, a resend of these is requested but no new chunks to the peer are queued
  yet.
