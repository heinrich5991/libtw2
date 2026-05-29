Introduction
============

The server continually sends snapshots and snapshot deltas to all game clients to update them on the game state.
To avoid sending redundant information, snapshots might not be sent every tick, and snapshot deltas are used to compactly represent what changed.
As tees move around, their position and other attributes often change on every tick.
Normally, the server would need to include all of these tees in every snapshot delta.
Dead reckoning is a system to reduce how often tee updates need to be sent by the server.

The idea of dead reckoning is that the client can run standard physics calculations just as well as the server.
Dead reckoning is a subset of unmodified Teeworlds physics which the client and server are aware of.
Whenever the server sends a tee update, the state includes the tick that the state is for.
The server knows what the standard Teeworlds physics would simulate on the client without a new update.
The server only needs to send an update, if the client side simulation diverges from the servers tee state.
Alternatively, if the tee state the client has is 3 seconds old (150 ticks), the server also sends an update.

`CharacterCore` is the snap item which contains the tee info for which dead reckoning is used.
Dead reckoning is not used for other prediction, only to get `CharacterCore`s up-to-date to the tick the snapshot which it belongs to.

Physics
=======

Dead reckoning is a subset of Teeworlds physics and doesn't include:

- Projectiles
- Other tees
- Changed tunings

Assumptions
===========

- Snapshot ticks are strictly monotonically increasing
- Tee ticks are always equal or lower than the snapshot tick
- Tee ticks should typically be monotonically increasing, see the section "Zeroed `CharacterCore`s"!
- The same tee's snap id along with the same tick always identify the same tee state(?)
- The maximum difference between the snapshot tick and tee tick is 150 ticks

Prediction
==========

If the client receives a snapshot with tick $n$, and finds a tee with tick $t$, it runs dead reckoning for $n - t$ ticks to get the up-to-date tee.
In case the client has cached a dead reckoned tee from the original tick $t$, it can also use it to perform less dead reckoning.
A future snapshot does not have a "more correct" tee for this tick, even if that snapshot's tick is further back in the past.

Zeroed `CharacterCore`s
===========================

When a `CharacterCore` is reset by a tee spawning, occasionally the entire `CharacterCore` is zeroed out (see https://github.com/ddnet/ddnet/issues/12337).
This also includes the tick field, so the `CharacterCore` appears out-of-date for server-time amount of ticks.
Trying to dead reckon in this case can cause a denial of service of the client, as this could result in simulating millions of ticks at once.
For this reason and also to protect from malicious servers, we recommend to implement an upper limit on how many ticks dead reckoning will simulate.
