# 007b-07 — Connection lifecycle scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007b-05

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- Connection states `[decision]`: Requested, Pending, Refused, Admitted, Locking, Aligning, Live, Interrupted, Dead, Ended.
- Settled timings and rules:
  - Interrupted after 8 consecutive missed slices, 4 ms at defaults (C-38). Only a slice absent at the deadline counts as a miss: missing, or arriving late. A marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count (C-102);
  - Dead after 2 s without data (C-39), counted from the last slice received with content or a marked gap (C-111); a slice on another timeline epoch also restarts the dead clock, is not a miss, and is counted in its own per-connection other-epoch counter (C-135, C-159);
  - a producer stops after 2 s without receiver reports (C-41); the liveness of a sink-only device is judged by this producer timeout only, since Interrupted and Dead (C-38, C-39) describe streams a peer receives (C-168);
  - a receiver report is sent every 100 ms per bundle (C-40), unicast, never multicast (C-47);
  - laptop edge sleep is a departure: both timebase loops are reset and the edge returns through normal admission, Locking then Aligning (C-124).
- A slice is 0.5 ms at defaults (C-02).
- Admission scenarios up to Admitted are in 007b-05; late/missing reclassification, ratchet and Demoted are in 007b-08.

## Work
1. Write scenarios in `vectors/control/connection-*.json`:
   - Admitted → Locking → Aligning → Live;
   - Live → Interrupted after exactly 8 consecutive missed slices (7 does not);
   - miss counting: 7 missed slices, then 1 marked gap, then 7 missed slices: no Interrupted; 7 missed, then 1 slice on another timeline epoch, then 7 missed: no Interrupted; a mix of missing and late slices totalling 8 consecutive: Interrupted;
   - Interrupted → Live on recovery;
   - Interrupted → Dead 2 s after the last slice received with content or a marked gap (and not at 2 s − 1 ns); a run where marked gaps keep arriving does not reach Dead; a run where only other-epoch slices keep arriving does not reach Dead and raises the other-epoch counter by one per slice (C-135);
   - producer stop after 2 s without receiver reports (and not at 2 s − 1 ns);
   - receiver reports emitted every 100 ms per bundle, checked over at least 1 s (10 reports per bundle), each addressed unicast;
   - edge sleep: the edge departs, and on wake the connection passes again through admission, Locking and Aligning before Live;
   - sink-only device: its producer stops it 2 s after the last receiver report (and not at 2 s − 1 ns); the device itself reports no Interrupted or Dead (C-168);
   - normal end → Ended.
2. Add one scenario per remaining connection transition row.

## Acceptance criteria
1. All 10 states appear as an expected state in at least one scenario.
2. Boundaries tested at the value and just below: 8 missed slices, 2 s Dead, 2 s producer stop.
3. A marked gap and an other-epoch slice each reset the miss count, and a late slice counts as a miss.
4. The Dead count starts at the last slice with content, a marked gap or an other-epoch slice (C-111, C-135), and every other-epoch slice is counted in the other-epoch counter, not as a miss.
5. The receiver-report scenario expects exactly 10 reports per bundle per second, none multicast.
6. The sleep scenario expects Locking and Aligning after wake, never a direct return to Live.
7. Every connection transition row is covered (007b-13 report).
8. The sink-only scenario ends by producer timeout at 2 s and contains no Interrupted or Dead state for the device.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Admission and refusal (007b-05).
- Late/missing reclassification, ratchet steps and Demoted (007b-08).
- Multi-packet reassembly deadlines (007b-10).
- Gap handling in the reference mixer (007b-18).
- Connection alarms on interrupted and dead (007b-09).
