libtw2
======

*Some Teeworlds stuff in Rust.™*

This repository hosts some third-party Teeworlds/DDNet libraries and tooling,
written in Rust. Additionally, it has some programming language independent
**documentation** of some Teeworlds/DDNet protocols, in the [doc](doc)
directory.

The highlights are probably 
- [doc](doc). The documentation.
- [wireshark-dissector](wireshark-dissector). A working Wireshark dissector for
  Teeworlds 0.6, Teeworlds 0.7 and DDNet.
- [gamenet/generate/spec](gamenet/generate/spec). JSON files describing the
  high-level Teeworlds 0.6, Teeworlds 0.7 and DDNet protocol.

Documentation
-------------

- [connection](doc/connection.md). Small diagram showing network messages that
  are sent after the low-level connection establishment.
- [datafile](doc/datafile.md). Low-level file format of Teeworlds/DDNet maps.
- [datafile\_v4.ksy](doc/datafile_v4.ksy). [Kaitai Struct](https://kaitai.io/)
  definition for the low-level file format of Teeworlds/DDNet maps.
- [demo](doc/demo.md). Low-level file format of Teeworlds/DDNet demos
  (replays).
- [demo.ksy](doc/demo.ksy). [Kaitai Struct](https://kaitai.io/) definition for
  the low-level file format of Teeworlds/DDNet demos (replays).
- [huffman](doc/huffman.md). Homebrew compression format using [Huffman
  coding](https://en.wikipedia.org/wiki/Huffman_coding), used in demos and over
  the network.
- [int](doc/int.md). Definition of the [variable-length
  integer](https://en.wikipedia.org/wiki/Variable-length_quantity) used in
Teeworlds/DDNet.
- [int.ksy](doc/int.ksy). [Kaitai Struct](https://kaitai.io/) definition for
  the [variable-length
  integer](https://en.wikipedia.org/wiki/Variable-length_quantity).
- [map](doc/map.md). High-level format of Teeworlds/DDNet maps.
- [map\_v4.ksy](doc/map.md). [Kaitai Struct](https://kaitai.io/) definition for
  the high-level format of Teeworlds/DDNet maps.
- [packet](doc/packet.md). Definition of Teeworlds 0.6/DDNet packet/chunk
  headers.
- [packet7](doc/packet7.md). Definition of Teeworlds 0.7 packet/chunk
  headers.
- [protocol](doc/protocol.md). Low-level Teeworlds 0.6/DDNet network protocol.
  See also [ChillerDragon's documentation on the Teeworlds 0.6 and 0.7
  protocol](https://chillerdragon.github.io/teeworlds-protocol/).
- [quirks](doc/quirks.md). Small, random fact collections of quirks of
  Teeworlds.
- [serverinfo\_extended](doc/serverinfo_extended.md). DDNet's extended
  serverinfo protocol.
- [snapshot](doc/snapshot.md). Teeworlds/DDNet data structure for transferring
  gamestate.
- [tee\_rendering](doc/tee_rendering.md). Description how tees can be rendered
  from a skin file.
- [teehistorian](doc/teehistorian.md). DDNet file format for storing all player
  input.

More links to other people's documentation can be found in ["Resources" on the
DDNet Wiki](https://wiki.ddnet.org/wiki/Resources).

Code
----

The code is split into many smaller and larger libraries. **Bold** names
indicate that the libraries or executables might be useful outside of libtw2.

- [\_old](_old). Unmaintained implementation of the low-level file format of
  Teeworlds/DDNet maps ("datafiles"), written in C, before libtw2 turned to
  Rust.
- [common](common). Utilities for all the other crates. Number conversion, byte
  strings, etc.
- [**datafile**](datafile). Low-level file format of Teeworlds/DDNet maps.
- [**demo**](demo). Low-level file format of Teeworlds/DDNet demos (replays).
- [**downloader**](downloader). Downloader for maps from game servers.
- [event\_loop](event_loop). Helper for creating Teeworlds/DDNet protocol
  clients/servers.
- [**gamenet**](gamenet). Multiple crates for handling the high-level Teeworlds
  0.6, Teeworlds 0.7 and DDNet network protocols.
- [**gamenet/generate/spec**](gamenet/generate/spec). JSON files describing the
  high-level Teeworlds 0.6, Teeworlds 0.7 and DDNet protocol.
- [**huffman**](huffman). Homebrew compression format using [Huffman
  coding](https://en.wikipedia.org/wiki/Huffman_coding), used in demos and over
  the network. Alternative: Ryozuki's
  [rustyman](https://github.com/edg-l/rustyman).
- [logger](logger). Utility crate to unify logging across libtw2 code.
- [**map**](map). High-level format of Teeworlds/DDNet maps. **You should
  probably use Patiga's [TwMap](https://gitlab.com/Patiga/twmap) instead.**
- [**net**](net). Low-level network protocol of Teeworlds 0.6, Teeworlds 0.7 and
  DDNet.
- [packer](packer). Encodings for Teeworlds/DDNet network protocols and file
  formats. See also Ryozuki's [teeint](https://github.com/edg-l/teeint) for
  another implementation of Teeworlds/DDNet's [variable-length
  integers](https://en.wikipedia.org/wiki/Variable-length_quantity).
- [**render\_map**](render_map). Render Teeworlds/DDNet maps to images. **You
  should probably use Patiga's [TwGpu](https://gitlab.com/Patiga/twgpu)
  instead.**
- [server](server). Proof-of-concept Teeworlds 0.6 server implementation.
- [serverbrowse](serverbrowse). Server info protocol for Teeworlds 0.5,
  Teeworlds 0.6, Teeworlds 0.7 and DDNet. See also Ryozuki's
  [teestatus](https://github.com/edg-l/teestatus). Essentially superseded by
  the DDNet HTTPS masterserver protocol, server list is at
  **https://master1.ddnet.org/ddnet/15/servers.json, you should probably use
  that instead.**
- [**snapshot**](snapshot) Teeworlds/DDNet data structure for transferring
  gamestate.
- [socket](socket). Helper for creating UDP sockets.
- [stats\_browser](stats_browser). Used for adding entries to the DDNet HTTPS
  masterserver, for game servers not supporting the HTTPS masterserver
  protocol. Originally intended to provide a tracking for Teeworlds servers.
  That info can now be found at https://ddnet.org/stats/master/ and parsed
  using Ryozuki's [teemasterparser](https://github.com/edg-l/teemasterparser/).
- [**teehistorian**](teehistorian). DDNet file format for storing all player
  input. Alternative: Zwelf's
  [teehistorian](https://gitlab.com/zwelf/teehistorian).
- [tools](tools). Various tools.
- [uniffi](uniffi). Python bindings for huffman using Mozilla's
  [uniffi](https://github.com/mozilla/uniffi-rs/):
  [**libtw2-huffman**](https://pypi.org/project/libtw2-huffman/).
- [**wireshark-dissector**](wireshark-dissector). Working Wireshark dissector
  for Teeworlds 0.6, Teeworlds 0.7 and DDNet.
- [world](world). Proof-of-concept Teeworlds physics. **You should probably use
  Zwelf's [TwGame](https://gitlab.com/ddnet-rs/twgame) instead.**
- [zlib\_minimal](zlib_minimal). Minimal wrapper around
  [zlib](https://zlib.net/).
