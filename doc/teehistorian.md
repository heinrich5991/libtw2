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
  * introduced in DDNet 11.0.3, [6c378b972b70b055](https://github.com/ddnet/ddnet/commit/6c378b972b70b0556d3b434b26baa0b9ffe490f1)

The following extra messages are known right now:
* TEST(teehistorian-test@ddnet.tw): is just a test message
  * uuid: 6bb8ba88-0f0b-382e-8dae-dbf4052b8b7d
  * introduced in DDNet 11.0.3, [6c378b972b70b055](https://github.com/ddnet/ddnet/commit/6c378b972b70b0556d3b434b26baa0b9ffe490f1)
* DDNETVER_OLD(teehistorian-ddnetver-old@ddnet.tw): cid(int), version(int)
  * uuid: 41b49541-f26f-325d-8715-9baf4b544ef9
  * introduced in DDNet 13.2, [0d7872c79eaeb19b](https://github.com/ddnet/ddnet/commit/0d7872c79eaeb19b3fd08c39c013a1043db1fd9b)
* DDNETVER(teehistorian-ddnetver@ddnet.tw): cid(int), connection_id(uuid), version(int), version_str(str)
  * uuid: 1397b63e-ee4e-3919-b86a-b058887fcaf5
  * introduced in DDNet 13.2, [0d7872c79eaeb19b](https://github.com/ddnet/ddnet/commit/0d7872c79eaeb19b3fd08c39c013a1043db1fd9b)
* AUTH_INIT(teehistorian-auth-init@ddnet.tw): cid(int) level(int) auth_name(str) records that a player with cid got rcon access with level under the account name auth_name since the start of the map (because they had it before the map change as well)
  * uuid: 60daba5c-52c4-3aeb-b8ba-b2953fb55a17
  * introduced in DDNet 11.0.3, [1c3dc8c316c2bf37](https://github.com/ddnet/ddnet/commit/1c3dc8c316c2bf37b94814d390c1c214422d46a9)
* AUTH_LOGIN(teehistorian-auth-login@ddnet.tw): cid(int) level(int) auth_name(str) records that a player with cid just logged into rcon with level under the account name auth_name
  * uuid: 37ecd3b8-9218-3bb9-a71b-a935b86f6a81
  * introduced in DDNet 11.0.3, [1c3dc8c316c2bf37](https://github.com/ddnet/ddnet/commit/1c3dc8c316c2bf37b94814d390c1c214422d46a9)
* AUTH_LOGOUT(teehistorian-auth-logout@ddnet.tw): cid(int) records that a player with cid just logged out of rcon
  * uuid: d4f5abe8-edd2-3fb9-abd8-1c8bb84f4a63
  * introduced in DDNet 11.0.3, [1c3dc8c316c2bf37](https://github.com/ddnet/ddnet/commit/1c3dc8c316c2bf37b94814d390c1c214422d46a9)
* JOINVER6(teehistorian-joinver6@ddnet.tw): cid(int)
  * uuid: 1899a382-71e3-36da-937d-c9de6bb95b1d
  * introduced in DDNet 14.0, [e294da41ba7142cb](https://github.com/ddnet/ddnet/commit/e294da41ba7142cb583a5dd2eab45af2ec9a8447)
* JOINVER7(teehistorian-joinver7@ddnet.tw): cid(int)
  * uuid: 59239b05-0540-318d-bea4-9aa1e80e7d2b
  * introduced in DDNet 14.0 [e294da41ba7142cb](https://github.com/ddnet/ddnet/commit/e294da41ba7142cb583a5dd2eab45af2ec9a8447)
* TEAM_SAVE_SUCCESS(teehistorian-save-success@ddnet.tw): team(int), save_id(uuid), save(str)
  * uuid: 4560c756-da29-3036-81d4-90a50f0182cd
  * introduced in DDNet 14.0.2, [d8aab366fc8489c8](https://github.com/ddnet/ddnet/commit/d8aab366fc8489c8cba4c77d73a6a7bfcce83bbc)
* TEAM_SAVE_FAILURE(teehistorian-save-failure@ddnet.tw): team(int)
  * uuid: b29901d5-1244-3bd0-bbde-23d04b1f7ba9
  * introduced in DDNet 14.0.2, [d8aab366fc8489c8](https://github.com/ddnet/ddnet/commit/d8aab366fc8489c8cba4c77d73a6a7bfcce83bbc)
* TEAM_LOAD_SUCCESS(teehistorian-load-success@ddnet.tw): team(int), save_id(uuid), save(str)
  * uuid: e05408d3-a313-33df-9eb3-ddb990ab954a
  * introduced in DDNet 14.0.2, [d8aab366fc8489c8](https://github.com/ddnet/ddnet/commit/d8aab366fc8489c8cba4c77d73a6a7bfcce83bbc)
* TEAM_LOAD_FAILURE(teehistorian-load-failure@ddnet.tw): team(int)
  * uuid: ef8905a2-c695-3591-a1cd-53d2015992dd
  * introduced in DDNet 14.0.2, [d8aab366fc8489c8](https://github.com/ddnet/ddnet/commit/d8aab366fc8489c8cba4c77d73a6a7bfcce83bbc)
* TEEHISTORIAN_PLAYER_TEAM(teehistorian-player-team@ddnet.tw): cid(int), team(int) records team changes
  * uuid: a111c04e-1ea8-38e0-90b1-d7f993ca0da9
  * introduced in DDNet 15.6, [e9dec007b22a071e](https://github.com/ddnet/ddnet/commit/e9dec007b22a071e9d104682955c952633455c27)
* TEEHISTORIAN_TEAM_PRACTICE(teehistorian-team-practice@ddnet.tw): team(int), practice(int) records when a team enters practice mode, resulting ranks don't get submitted to the database
  * uuid: 5792834e-81d1-34c9-a29b-b5ff25dac3bc
  * introduced in DDNet 15.6, [81f4263428069526](https://github.com/ddnet/ddnet/commit/81f426342806952603a2d28290279e0a7107db5b)
* TEEHISTORIAN_PLAYER_READY(teehistorian-player-ready@ddnet.tw): cid(int) records when the client messages that it is ready to join the game, leading to the tee being spawned in the following tick
  * uuid: 638587c9-3f75-3887-918e-a3c2614ffaa0
  * introduced in DDNet 16.0, [3ea55dcc0ebc1c79](https://github.com/ddnet/ddnet/commit/3ea55dcc0ebc1c791e11cab0c268febe7e783504)
* TEEHISTORIAN_PLAYER_SWITCH("teehistorian-player-swap@ddnet.tw): cid1(int), cid2(int) records the ids of players swapping tees
  * uuid: 5de9b633-49cf-3e99-9a25-d4a78e9717d7
  * introduced in DDNet 16.1, [86f57289c6ff1926](https://github.com/ddnet/ddnet/commit/86f57289c6ff1926e1e9802de33ceae69a026717)

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
