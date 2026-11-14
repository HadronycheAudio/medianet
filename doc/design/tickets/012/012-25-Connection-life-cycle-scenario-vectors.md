# 012-25 — Connection life-cycle scenario vectors
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-16, 012-19, 012-23

## Context
Epic 012 criterion 012-A1 (AC-1) also covers the life cycle after Live. This ticket adds those vectors on the 012-19 harness (epic 007 format, explicit `now_ns`, deterministic, unprivileged) `[decision]`:
- L-1: a live connection becomes dead only from data-plane evidence; the producer stops only after 2 s without receiver reports (C-41; reports every 100 ms, C-40).
- L-5: destinations are pinned and change only by re-admission.
- Interrupted after 8 consecutive missed slices (C-38), where a marked gap or a slice on another timeline epoch resets the count (C-102); Dead 2 s after the last slice with content, a marked gap or a slice on another timeline epoch (C-111, C-135).
- Demoted (C-109): persistently late (C-101) with the offset at A; treated as gapped; cleared only by re-admission.
- Laptop sleep (C-124) is a departure; on wake the edge passes normal admission (Admitted, Locking, Aligning).
- Test-only hooks (C-120): force a stream onto a given timeline epoch; send a raw packet on a control socket.

Connection scenario scripts come from epic 005 (C-193) and run on the same harness; the vectors below are added where a script does not already cover them.

Who issues the new request on wake is still open (owner epic 005); the sleep vector issues it explicitly from the test.

## Work
Vectors for at least:
1. 012-A7, 012-A6 (AC-20 a, d): 30 s of hello silence; an address change during Live.
2. 012-A8 (AC-16): 8 missed slices → Interrupted; resume → Live at the frozen offset; 2 s → Dead. A marked gap at miss 7 resets the count (C-102); a slice forced onto another timeline epoch (C-120 hook) resets the count, restarts the 2 s dead clock and increments the other-epoch counter (C-135).
3. 012-A9 (AC-21): the producer stops after 2 s without receiver reports, not before.
4. 012-A12 (C-109): `Demoted` on Live → Demoted, unsatisfied in the graph view, unaffected by later evidence; a new request admits afresh.
5. 012-A15 (C-124): edge sleep → `Disconnect` → Ended; on wake a new request goes Admitted → Locking → Aligning → Live.
6. 012-A13: stamp records on both sides after an Interrupted cycle and a Dead, with every C-159 counter.
7. 012-A16 (C-173, C-174): alarms appear exactly on Interrupted, Dead, Demoted and `CannotMeetA`, and the connection is satisfied only while Live (C-154).

## Acceptance criteria
1. Every connection scenario script supplied by epic 005 (C-193) runs on the harness and passes, and all vectors listed above exist in the epic 007 format, in the vectors crate (C-192), and pass in CI.
2. Each vector asserts the exact sequence of `ConnEvent`s and outbox messages.
3. The run is deterministic (two runs produce identical traces).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: stamp fields are read, not defined, here
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: tests only
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Admission-phase vectors (012-19). Real-network tests (task 019). Device run (epic 026).
