# 015a-15 — Housekeeping thread (housekeeping.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-04, 015a-06, 015a-08, 015a-10, epic 011, epic 012
## Context
- File: `hadronyche-std/src/housekeeping.rs`: drives the core discovery, election and admission state machines (epics 011, 012); talks to other threads by bounded lock-free queues.
- One housekeeping thread per process `[decision]`.
- **L-4:** housekeeping admits unknown sources only; it never touches a live data path.
- **L-5:** destinations are pinned at admission and change only by re-admission. A hello carrying a new address never redirects a live stream.
- **L-8:** no lock is shared between housekeeping and the data or clock threads.
- Control messages (hello, election, admission, regime stamp) use DSCP CS5 (40, C-13) and fit 1 252 bytes of UDP payload (C-17, C-100); the discovery socket is joined on the medianet interface, TTL 1.
- Elections start only on 1.5 s of sync-reply silence (C-37); hello expiry (3 s) applies to non-live peers only (C-31). The clock thread publishes the last valid reply time lock-free (015a-10); housekeeping feeds it to the election state machine, so a holder whose hellos stop while replies flow is never replaced (AC-20(b)).
- Housekeeping also builds and sends the regime stamp (015a-18) and samples interface counters (015a-14).
- Queue capacities are not in the constants table; they are owned by this ticket and recorded there (epic open sub-question; falsifier: a full queue is counted during a nominal admission burst of 64 modules, C-34).
## Work
1. Event loop over the discovery socket and timers, feeding the core state machines.
2. Outgoing queues to each data thread (admitted inputs/outputs with pinned addresses) and to the clock thread (holder), bounded and lock-free; a full queue is counted.
3. Read the clock thread's last-reply time on every loop and feed it to the election state machine as holder evidence.
4. Ignore address changes in hellos for admitted peers; only re-admission changes destinations.
5. Choose the queue capacities, justify them against 64 modules (C-34), and record them for the constants table (015b-11).
## Acceptance criteria
1. Two runtimes on loopback discover each other and complete admission; data threads receive the pinned addresses through the queue.
2. A hello with a changed address for an admitted peer does not produce a message to any data thread.
3. With holder hellos absent but sync replies flowing, the election state machine starts no election over 30 s; with replies silenced, an election starts after 1.5 s (C-37).
4. No `Mutex`, `RwLock` or other lock type is shared with data or clock threads (verified by 015b-02).
5. The chosen queue capacities and their justification are recorded.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" — control parsing is core's (epics 011, 012); "Regime stamp fields touched" — stamp built in 015a-18; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — data path only.
## Out of scope
- Supervision and restart (015a-16).
- Regime stamp publication (015a-18).
- The AC-20 test suite (015b-04).
