# 005-20 — Scenarios: id collision and gossip robustness
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-07, 005-10, 005-16

## Context
- Deliverable 4 of epic 005 (scenario scripts in prose, to become vectors in task 007). This ticket covers: id collision; hello suppressed for 30 s with live streams; hello with a changed address during a live stream. It also covers the holder-hello-suppressed case of AC-20 (b).
- 005-AC-5 (system AC-22): id collision detected within one hello period, 1 s. Rule: id seen with a different instance nonce (C-11: 64-bit id and nonce); the later joiner redraws and rejoins; third parties quarantine the id until only one nonce has been seen for 3 s (C-46); both modules log it (`IdCollision`). The redrawn module is in every third party's map within 1.5 s + 1 s = 2.5 s of the redraw.
- A collision is produced in a vector with the test-only hook that forces a module id and nonce (C-120); a stamped injection like any other.
- 005-AC-7, contributes to system AC-20: (a) hellos suppressed 30 s, live streams continue; (b) holder hello suppressed while replies flow, no election; (d) a hello with a changed address, no redirect. Item (c), the housekeeping restart, belongs to epic 015 (C-144).
- Invariants: L-1 (gossip silence never tears down a live stream; a peer with a live connection is not removed by the 3 s expiry C-31), L-5 (destinations pinned; a hello with a new address never redirects), L-6 (hello expiry never opens an election while sync replies flow).
- Machine tables: `doc/design/spec/control.toml`.

## Work
1. In the "Scenarios" section of `control.md`, write as timed scripts (time ms, module, input, expected state, events):
   - S4 id collision: two modules forced to the same id with different nonces (C-120); detection ≤ 1 s; later joiner redraws and rejoins; a third party ignores both until one nonce has been seen for 3 s (C-46), and admits the redrawn id within 2.5 s of the redraw;
   - S5 (AC-20 a) all hellos suppressed for 30 s while streams are Live and slices flow: connections stay Live, no `PeerLeft` for live peers, no Interrupted/Dead;
   - S6 (AC-20 b) holder hellos suppressed while sync replies flow: no `ElectionOpened` over 30 s;
   - S7 (AC-20 d) a hello with a changed address from a peer with a live stream: the map entry may update, the pinned destination does not change.
2. Walk each through the tables and record that results match.

## Acceptance criteria
1. Four scripts S4–S7 exist with timestamps in ms.
2. S4 shows `IdCollision` at both colliding modules no later than 1 000 ms after the second module's first hello, the redrawn id in the third party's map no later than 2 500 ms after the redraw, and the old id's quarantine ending 3 000 ms after only one nonce is seen.
3. S5 runs for 30 000 ms of hello silence with 0 transitions out of Live.
4. S6 shows 0 `ElectionOpened` events over 30 000 ms.
5. S7 shows the pinned destination unchanged after the changed-address hello.
6. Every input used exists in the machine-readable tables.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- AC-20 (c), owned by epic 015 (C-144).
- Cold start, holder loss, partition (005-19); refusal scenarios (005-21); Pending and demotion (005-25).
- Vector generation (epic 007).
