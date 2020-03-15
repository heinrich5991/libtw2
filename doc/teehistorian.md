Introduction
============

The DDNet teehistorian format is the format which DDNet uses to save all input
to a server in order to be able to reproduce it faithfully. A teehistorian file
is fundamentally a stream of messages that describe input to the server and a
little data for sanity checks.

The format is designed in a way to make it easily compressible using standard
compression algorithms in order to make it suitable for long-term storage,
which Teeworlds demos are not due to their large size. It is easy to write out
in a stream (you just append data at the end) but for reading, one has to read
the whole file end-to-end because the format does not support seeking.

This document describes version 1 and 2 of the teehistorian file format. The
only difference between them is the existence of the `EX` message (see below).


Format
======

A teehistorian file is fundamentally a header followed by a stream of messages.


Header
------

The header starts with the teehistorian UUID
(699db17b-8efb-34ff-b1d8-da6f60c15dd1, version 3 UUID derived from the
Teeworlds namespace e05ddaaa-c4e6-4cfb-b642-5d48e80c0029 and the name
"teehistorian@ddnet.tw"), encoded as a [big endian binary encoded
UUID](https://en.wikipedia.org/w/index.php?title=Universally_unique_identifier&oldid=844235295#Encoding)
(16 bytes). It is followed by a null-terminated string that contains a JSON
object containing at least the following keys:

* `version`: This is the version number of the teehistorian format. It must be
  `"1"` or `"2"` for this document.

Messages
--------

Each message starts with a Teeworlds variable-width integer, which is the
message ID.

* PLAYER_DIFF(0-63): dx(int) dy(int) records that player with the message ID as cid (client ID) has changed position in a way that adds dx to the x coordinate and y to the y coordinate
* FINISH(-1): records the end of the teehistorian file
* TICK_SKIP(-2): dt(int) records that there were dt ticks in which nothing happened, i.e. the next tick is the last tick + dt + 1
* PLAYER_NEW(-3): cid(int) x(int) y(int) records that a new player character with cid appeared at (x, y)
* PLAYER_OLD(-4): cid(int) records that the player character with cid disappeared
* INPUT_DIFF(-5): cid(int) dinput(int[10]) records that a player with cid sent an input packet but has sent one before, add dinput to the previous input component-wise to obtain the new one
* INPUT_NEW(-6): cid(int) input(int[10]) records that a player with cid sent an input packet for the first time, containing input
* MESSAGE(-7): cid(int) msgsize(int) msg(raw[msgsize]) records that a player with cid sent a game-related packet msg
* JOIN(-8): cid(int) records that a player with cid joined, on the engine level
* DROP(-9): cid(int) reason(str) records that a player with cid left/was kicked/was dropped, on the engine level
* CONSOLE_COMMAND(-10): cid(int) flags(int) cmd(str) num_args(int) args(str[num_args]) records that a console command cmd was executed by client id cid (not necessarily a player, might be a vote as well), with flags (distinguishes chat commands, etc.) with parameters args
* EX(-11): uuid(uuid) size(int) data(raw[size]) records an extension message, identified by uuid and containing data

The following extra messages are known right now:
* TEST(teehistorian-test@ddnet.tw): is just a test message
* AUTH_INIT(teehistorian-auth-init@ddnet.tw): cid(int) level(int) auth_name(str) records that a player with cid got rcon access with level under the account name auth_name since the start of the map (because they had it before the map change as well)
* AUTH_LOGIN(teehistorian-auth-login@ddnet.tw): cid(int) level(int) auth_name(str) records that a player with cid just logged into rcon with level under the account name auth_name
* AUTH_LOGOUT(teehistorian-auth-logout@ddnet.tw): cid(int) records that a player with cid just logged out of rcon

The following data types are used:
* int is a [teeworlds variable-width integer](int.md)
* str is a null-terminated string
* raw[size] is simply size bytes
* str[num_args] is num_args null-terminated strings
* uuid is 16 bytes of a UUID

the UUIDs are version 3 UUIDs, with the teeworlds namespace e05ddaaa-c4e6-4cfb-b642-5d48e80c0029
a tick is implicit in these messages when a player with lower cid is recorded using any of PLAYER_DIFF, PLAYER_NEW, PLAYER_OLD
e.g.
PLAYER_DIFF cid=0 … PLAYER_NEW cid=5 … PLAYER_OLD cid=3 has an implicit tick between the cid=5 and the cid=3 message
another correction:
the header is the teehistorian uuid followed by a zero-terminated string containing json in a self-explanatory format
