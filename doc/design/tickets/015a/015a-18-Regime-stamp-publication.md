# 015a-18 — Counter board and regime stamp publication (C-18)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-15, epic 013
## Context
- Regime stamp transport (C-18): a dedicated control message split into sections that each fit the control message bound — at most 1 252 bytes of UDP payload (C-17, C-100). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gets an immediate reply; the hello carries only the stamp version number.
- The stamp format and its section encoder are core's (epic 013); sending the regime stamp message that epic 013 defines is a duty of the hosted backend (C-144). This ticket drives the encoder from the housekeeping thread, which owns control traffic (DSCP CS5, 40, C-13).
- Counters come from every thread: data threads (drops, DSCP, health — 015a-14, 015a-19; processing-overrun events — 015a-20), the clock thread (achieved rate, discards, sync-request DSCP per follower — 015a-10; sync requests dropped above 200 per second, per follower, C-204 — 015a-23; holder stall — 015a-21), sockets (send drops — 015a-05), the file service (record drops — 015a-08), the supervisor (control-degraded mark — 015a-16), strict mode (capability states — 015b-13), injections (015b-12).
- **L-8:** no lock is shared between housekeeping and the data or clock threads. The counter board must therefore be lock-free: each writer owns its slots (single-writer atomics or a per-slot seqlock), housekeeping only reads snapshots.
- Interface counters are sampled each time a stamp is built (015a-14).
- Commissioning window (C-153, C-178): the commissioning tool announces a window on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close. While it is open, ordinary production measurements in the stamp (latency, loss, alignment figures) are unqualified; the tool's own commissioning measurements are recorded with the window by the tool (epic 028). The window state machine and the stamp fields are core's (epic 013); housekeeping feeds it the announcements and the time, and stamps the result. Opening and closing are regime changes.
- Fields for some host counters (clock discards, send drops, control-degraded mark, processing-overrun event, per-follower sync-request drops of C-204) are not yet specified by the wire specification (epic 004); the board carries them so they can be stamped as soon as the fields exist.
## Work
1. Counter board in `hadronyche-std`: preallocated at registration, single writer per slot, lock-free snapshot read; no allocation on writers.
2. In housekeeping: build the stamp from a board snapshot with the core encoder; detect regime changes (a change in any non-counter field) and multicast the stamp on each; refresh every 10 s.
3. Answer a unicast stamp request immediately with the current stamp, unicast.
4. Give the current stamp version to the core hello encoder.
5. Verify each encoded section is at most 1 252 bytes before sending; an oversize section is a counted error, never fragmented.
6. Feed commissioning-window announcements received on the control group, and the time, to the core window state (C-153); while a window is open, stamp it and mark the production latency, loss and alignment figures unqualified (C-178).
## Acceptance criteria
1. A regime change is followed by a multicast stamp before the next 10 s refresh; with no change, stamps are sent every 10 s ± one housekeeping loop period over 60 s.
2. A unicast request is answered with the current stamp to the requester's unicast address only.
3. Every section on the wire is ≤ 1 252 bytes of UDP payload, DSCP 40, DF set.
4. The hello's stamp version equals the version of the last stamp sent.
5. A counter written by a data thread appears in the next stamp; writers never block and never allocate (counting allocator test).
6. An announced commissioning window triggers a multicast stamp that carries the window and marks the production latency, loss and alignment figures unqualified; at its expiry, and separately on an explicit close, a new stamp without the window and without the mark is multicast (C-153, C-178).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — stamp encoding and request decoding are core's (epic 013); "Design documents updated" — follows the epic; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — control path only.
## Out of scope
- The stamp format and section encoder (epic 013); stamp fields not yet specified (epic 004); the commissioning window state and the tool's own measurements (epics 013, 028).
- The counters themselves (015a-05, 015a-08, 015a-10, 015a-14, 015a-16, 015a-19, 015b-12, 015b-13, 015a-20, 015b-14).
