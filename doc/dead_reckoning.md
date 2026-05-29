Introduction
============

The server continually sends snapshots and snapshot deltas to all game clients to update them on the game state.
To avoid sending redundant information, snapshots are sent only occasionally and snapshot deltas are used to compactly represent what changed.
As tees move around, their position and other attributes often change on every tick.
Normally, the server would need to include all of these tees in every snapshot delta.
However, tees have a the system in place called dead reckoning in place to reduce how often a tee update needs to be sent by the server.

The idea of dead reckoning is that the client can run the physics calculations just as well as the server.
So we define a subset of physics we call dead reckoning which the client and server agree upon.
Dead reckoning only considers a singular tee, without interaction with projectiles or other tees.
Whenever the server sends a tee update, the state includes the tick on which from the state is.
The client then runs dead reckoning on the single tee.
The server knows which physics are run by the client and only sends a new tee state if the real tee position diverged from the dead reckoning simulation.
Alternatively, if the tee state the client has is 3 seconds old (150 ticks), the server also sends an update.

Assumptions
===========

- Snapshot ticks are strictly monotonically increasing
- Tee ticks are always equal or lower than the snapshot tick
- Tee ticks may not be monotonically increasing!
- The same tee's snap id along with the same tick always identify the same tee state(?)
- The maximum difference between the snapshot tick and tee tick is 150 ticks

Prediction
==========

If the client receives a snapshot with tick `n`, and finds a tee with tick `t`, it runs dead reckoning for `n - t` ticks to get the up-to-date tee.
In case the client has cached a dead reckoned tee from the original tick `t`, it can also use it to perform less dead reckoning.
A future snapshot does not have a "more correct" tee for this tick, even if that snapshot's tick is further back in the past.

Demos
-----

In demos, we receive a snapshot only every second tick and some snapshots are even fully left out.
A missing snapshot means that dead reckoning produces the expected snapshot which was left out for that reason(?).
The client will also fill the gap of every second tick by running dead reckoning(? or do we use full prediction here ?).
