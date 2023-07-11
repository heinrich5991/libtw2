This document describes the low-level Teeworlds protocol up to 0.6.x and in
DDNet. The packet headers are described in [packet.md](packet.md).

The Teeworlds protocol is a layer over UDP. It offers connectionless
(stateless) messages, and connections. Connections can be established and torn
down. Within connections, reliable ("vital") and unreliable ("non-vital")
messages can be sent. Unreliable messages might or might not be delivered and
might be received in any order. Reliable messages are guaranteed to arrive
in-order, and will get re-sent if they're lost, so they will arrive eventually
as long as the connection is not closed.

The Teeworlds 0.6 protocol does not offer protection against IP spoofing,
neither for connectionless messages nor for the connections themselves. DDNet
extends this network protocol to offer protection against IP spoofing for
connections.

The Teeworlds protocol defines three different kinds of packets. There are
connectionless packets, and two kinds of connection-oriented packets, namely
control packets and "normal" packets.


Connectionless packets
======================

Any packet having the `flag_connless` header flag set is considered
connectionless. All other header flags and fields are ignored. After the
header, there are 3 bytes of padding. The bits of these first 6 bytes are
supposed to be all-ones.

All sizes in bits.

    packet_connless:
        [24] packet_header
        [24] padding
        [  ] payload

NOTE: The reference implementation only care about the `flag_connless` header
flag being set and ignores all of the other 47 bits.


Connection-oriented packets
===========================

In order to guarantee reliable messages being received in the right order, each
reliable message gets assigned a 10-bit sequence number, starting at 0,
incremented for each message and wrapping around after 1023 to 0.

All connection-oriented packets have `flag_connless` set to 0. Each peer keeps
track of the highest sequence number such that it has received all reliable
messages with sequence numbers smaller or equal to it (note: this is
simplified: sequence numbers wrap around, so it's not straightforward to say
which sequence numbers are larger and which are smaller). This highest sequence
number must be reported in the `ack` field, it tells the other peer that it can
forget about these reliable messages.

If a peer detects that it's missing reliable messages (when receiving higher
sequence numbers without the ones in between), it sets `flag_resend`. This
tells the other peer to start re-sending messages starting from the client's
current `ack`.

`flag_compression` tells us whether the `maybe_compressed_payload` of the
packet (everything after the header) has been compressed using the Huffman
compression described in [huffman.md](huffman.md). Peers compress the payload
and then send whichever of the compressed or uncompressed payload is smaller.
Packets with `flag_control` must not have `flag_compression` set.

NOTE: The reference implementation still interprets `flag_compression` for
packets with `flag_control`.

Finally, `flag_control` determines if the payload is a control packet or a
"normal" packet.

    packet_connected:
        [24] packet_header
        [  ] maybe_compressed_payload


Control packet
--------------

`num_chunks` should be 0.

There are five different control messages. The first byte of the payload
determines the kind of control message. There are five different defined
control messages.

    keepalive = 0
    connect = 1
    connectaccept = 2
    accept = 3
    close = 4

All of these do not have extra data, except for the `close` control message
which can optionally take a nul-terminated UTF-8 string as the close reason.

    packet_control:
        [24] packet_header
        [ 8] control_message

    packet_control_close:
        [24] packet_header
        [ 8] control_message
        [  ] reason

<!-- TODO: a diagram for connection establishment would be nice here -->

When a connection is being established, there's a clear difference between
client and server. The client starts by sending a `connect` control message.
When the server receives a `connect`, it responds with a `connectaccept`. When
the client sees a `connectaccept`, it sends an `accept` (which is ignored by
the server) and considers the connection active. When the server receives its
first normal packet, it also considers the connection active.

Either party may send a `close` at any point (as a response to `connect`,
`connectaccept`, during an active connection or even just upon receiving a
packet belonging to an unknown connection).


Normal packets
--------------

Normal packets carry reliable and unreliable messages ("chunks").

Each message is prepended a `chunk_header_nonvital` or `chunk_header_vital`,
depending on whether `flag_vital` is set. `flag_resend` indicates whether the
message was sent in response to a `flag_request_resend` of a packet header.
`size` is the size of the message *excluding* the header. `sequence` indicates
the 10-bit sequence number of a reliable message.

Then all of the messages are concatenated (up to a maximum packet size of 1400
bytes), `num_chunks` specifies the number of these messages.

NOTE: The reference implementation ignores `num_chunks` and simply reads chunks
from the payload until it reaches the end.


DDNet token extension
=====================

In order to secure the protocol against IP spoofing. It introduces a 4-byte
token that needs to be included as part of every conection-oriented message.
The server decides the token, usually based on IP address of the client, so
that it doesn't need to save anything before the client IP address is
validated.

There are two special tokens. The first one is the all-zero token which is used
by the reference implementation for internal tracking and should not be used.
The other is the all-ones token which is used before the actual token is known,
i.e. in the `connect` control message.

**The token is simply appended to each of the connection-oriented packets,**
before compression takes place. This means that the receiver needs to
decompress the packet before it can validate the token. This is a design flaw,
because it requires computational overhead before a packet can be validated.

    packet_control_ddnet:
        [24] packet_header
        [ 8] control_message
        [32] token

    packet_control_close_ddnet:
        [24] packet_header
        [ 8] control_message
        [  ] close_reason
        [32] token

In the `connect` packet, since the token isn't known at that time (the server
decides on which token to use), it is set to the all-ones token. The `connect`
packet is additionally special in the sense that it is used to signal support
for the DDNet token extension.

    packet_control_connect_ddnet:
        [24] packet_header
        [32] token_magic
        [32] token

The `token_magic` is the ASCII string `TKEN`, in hex `54 4b 45 4e`.

NOTE: The DDNet reference implementation unfortunately uses the `accept`
message to actually let a client connect. Since that one is not re-sent, it
means that a single lost packet can cause the connection establishment to get
stuck.
