# 017b-23 — Scenario `housekeeping`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-04, 017b-08, 017a-09, 017b-09, epic 015, epic 016

## Context
- `housekeeping` is in the epic's scenario list. Before the last rulings nothing said what it asserts; C-139 and C-145 now define it (new ticket, split from the former combined housekeeping and thread-death ticket).
- **Housekeeping restart policy (settled, C-139):** exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop: the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it.
- **Panic injection (settled, C-145):** the test-only hook of 017b-08 injects a panic into the housekeeping thread, stamped like any injection.
- **Ownership (C-144):** the restart machinery and the housekeeping-restart criterion AC-20 (c) are the hosted backend's (epic 015). This scenario is the L0 regression check of that behaviour end to end; it does not implement it.
- **Control isolation (C-161):** on hosted platforms housekeeping cannot delay the data path, by construction of the thread model, so content must be undisturbed throughout.
- Alarms (C-173): a connection alarm is raised on control-degraded.
- **Refusal while control-degraded (settled, C-185):** a new connection to the module is refused with ControlDegraded, answered by the housekeeping supervisor, which keeps a minimal refusal path; a refusal also raises a connection alarm (C-173).
- **Sync answering (settled, C-188):** the holder answers sync requests on its clock thread, never on housekeeping, so housekeeping panics do not interrupt sync. The scenario targets the module that holds the clock, so this is checked too.
- Periodic duties that must continue between panics: the stamp is refreshed every 10 s (C-18), receiver reports are produced per bundle (C-40), hellos carry the stamp version (C-18).
- L0 rules `[decision]`: never assert latency on loopback. Backoff intervals are programmed delays, not latencies: the scenario asserts only their lower bounds and the order of events, never an upper bound.

## Work
1. Scenario `housekeeping`, phases on one module, the clock holder (C-188):
   - (a) run 25 s undisturbed; assert stamp refreshes and receiver reports for every bundle are present and exact mismatches = 0;
   - (b) inject one housekeeping panic; assert it restarts, no restart comes earlier than 100 ms after the panic, and duties resume;
   - (c) inject panics until 5 have occurred within 60 s; assert the first restart interval is at least 100 ms and later intervals never shrink until the 10 s cap (C-139), and that after the fifth panic restarts stop;
   - (d) after (c): the module is control-degraded with an alarm, its live streams continue with zero exact-tier mismatches, its followers stay locked (C-188), and a new connection request to it is refused with ControlDegraded (C-185);
   - (e) with `suppress_hello` (017b-04), peers stop seeing the module's liveness counter advance.
2. Document the asserted behaviour in rustdoc, citing C-139, C-144, C-145, C-161, C-185, C-188 and epics 015/016.

## Acceptance criteria
1. `hadronyche-l0 housekeeping --duration 120` exits 0.
2. Phase (a): at least two stamp refreshes and receiver reports for every bundle in the record file; exact mismatches = 0.
3. Phases (b) and (c): the first restart comes no earlier than 100 ms after its panic, and each later restart interval is no shorter than the previous one until the 10 s cap (C-139); after 5 panics within 60 s no further restart occurs.
4. Phase (d): the module stamps control-degraded and raises an alarm (C-139, C-173); its existing streams keep zero exact-tier mismatches (C-161); a new connection to it is refused with ControlDegraded, raises a connection alarm and never goes live (C-173, C-185).
5. Phase (e): suppressed hellos are detected by peers.
6. Every injected panic appears in the stamp's injection record.
7. Through phases (b) to (d), every follower of the module stays locked (C-188).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Implementing the restart policy and AC-20 (c) (epic 015, C-144).
- Thread death (017b-22).
