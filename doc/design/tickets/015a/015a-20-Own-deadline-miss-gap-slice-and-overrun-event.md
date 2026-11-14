# 015a-20 — Own-deadline miss: gap slice and processing-overrun event (C-144)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-12, 015a-18, epic 010
## Context
- C-144 assigns to the hosted backend (epic 015) emitting the gap slice when a module misses its own deadline, and raising the processing-overrun event.
- 015a-12 counts missed deadlines: callback plus send not complete before the next slice deadline. This ticket acts on each miss.
- A marked gap proves the upstream is alive: downstream it resets the interrupted count (C-102) and restarts the dead clock (C-111), so emitting it on a miss keeps descendants from declaring the module interrupted for a processing overrun.
- The gap slice is built with the core packetiser (epic 010) and sent to the pinned destinations (L-5) through the pacer, in every output stream of the module, for the position whose deadline was missed.
- The processing-overrun event reaches the stamp through the lock-free counter board (015a-18, L-8). Its stamp field is not yet specified by the wire specification (epic 004), like the other host counters; the board carries it until the field exists.
- No allocation on the data thread after go-live (015b-09).
## Work
1. On a missed deadline, emit a marked-gap slice for the missed position on every output stream, from preallocated buffers, through the pacer to the pinned destinations.
2. If the late content for that position completes afterwards, discard it: the position has already been sent as a gap.
3. Raise the processing-overrun event: increment a per-module overrun counter and record the position, on the counter board.
4. Unit and loopback tests with a module callback that sleeps past its deadline on chosen slices.
## Acceptance criteria
1. A callback overrunning on one slice produces exactly one marked-gap slice per output stream for that position, and no content slice for it.
2. A consumer of that module sees a marked gap, not a missing slice: its miss count resets and it raises no interrupted event over 8 consecutive overruns (C-102).
3. Each overrun increments the processing-overrun counter by one, and the counter appears in the next stamp.
4. No allocation on the data thread when a gap slice is emitted (counting allocator test).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser, the gap slice is encoded by core (epic 010); "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — send side.
## Out of scope
- Gap slice encoding (epic 010); the stamp field for the overrun event (epic 004).
- Missed-deadline detection itself (015a-12).
