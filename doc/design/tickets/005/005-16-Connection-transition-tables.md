# 005-16 — Connection transition tables
**Epic:** 005-Control-state-machines-specification · **Effort:** M · **Depends on:** 005-03, 005-04, 005-15, 005-23, 005-24

## Context
- Machine 4 (connection states) of `doc/design/spec/control.md`; states and inputs from 005-15 in `doc/design/spec/control.toml` (`connection_consumer`, `connection_producer`); health guards and the decided exits from 005-24; regime stamp counter mapping from 005-23; checker from 005-03.
- Arcs to encode: Requested → Refused | Pending | Admitted; Pending → Requested | Ended (operator cancel, C-45; no timeout); Admitted → Locking | Aligning; Locking → Aligning; Aligning → Live (after 4 s, C-50; go-live on a slice boundary, flagged on the first slice, consumer's local decision); Live → Interrupted (8 consecutive slices absent at the deadline, C-38, C-102); Interrupted → Live (slices return, same session, immediate at the frozen offset); Interrupted → Dead (2 s since the last slice with content, marked gap or other timeline epoch, C-39, C-111, C-135); Live → Aligning (ratchet step of one slice period when persistently late, C-53, C-101, the last step landing on A exactly, C-163, taking effect at a position the fan-in announces one lead (64 slices, C-43) ahead in its own output, never silently on the next round, C-215; a sink, with no output to announce in, takes the step at its own position one lead ahead, unannounced, C-225; or announced re-alignment); Live → Demoted (persistently late with the offset at A, C-109); Demoted → Requested (re-admission); Live → Ended (end-of-stream marker or disconnect). Dead, Ended, Refused terminal.
- Producer: Sending → Stopped after 2 s without receiver reports (C-41; reports one per bundle every 100 ms, C-40); entry and restart as decided in 005-15. For a sink-only device the producer timeout is the only liveness judgement (C-168).
- Every state change raises `ControlEvent::ConnectionState(new state)`; refusal raises `ConnectionRefused(reason)`. Rows that enter Interrupted, Dead or Demoted carry the C-159 counter they increment (interrupted events, dead, demoted rounds), as mapped in 005-23. The reaction to an endpoint clock becoming unusable (C-162) follows the decision of 005-24.
- Invariants: L-1 (only data-plane evidence kills a healthy connection; producer stops only after 2 s without reports), L-2 (position-deadline control in-band), L-5 (destinations pinned; a hello never redirects).
- A sleeping laptop edge has departed (C-124): there is no resume row for sleep.
- Open sub-question owned by this ticket (epic 005, with epic 004 for the carrier): how a locking member's "declared silent" status is carried (output message, in-band, regime stamp). Falsifier: consumers count a locking member's inputs as gaps or loss.

## Work
1. Write both complete tables in `control.md`: every (state, input) pair has a row, including `ignore`, with guard, new state, outputs, events, counters, L-ids preserved.
2. Encode the rows in `control.toml`; checker passes.
3. Decide the carrier of the locking-member silence declaration and write it as the output of the Locking rows; record the decision with a `[decision]` tag and the falsifier, and name what epic 004 must carry.
4. Use the guards of 005-24 verbatim for Live → Interrupted, Interrupted → Dead, Live → Aligning (ratchet) and Live → Demoted.
5. "Worst-case work per input": a number.

## Acceptance criteria
1. The checker reports 0 missing pairs for both machines.
2. Live → Interrupted fires on the 8th consecutive slice absent at the deadline, not the 7th; a marked gap or another-epoch slice resets the count (C-102); Interrupted → Dead fires 2 s after the last slice with content, marked gap or other timeline epoch (C-111, C-135); Interrupted → Live keeps the same session and the frozen offset.
3. Live → Demoted fires only when the C-101 test holds with the offset at A; no row leaves Demoted except re-admission or the exits decided in 005-24.
4. No row leaving Live, Interrupted or Demoted has a hello-derived input (L-1).
5. No row changes the pinned addresses (L-5).
6. End-of-stream and re-alignment arrive only as in-band slice markers (L-2); a ratchet-step row of a fan-in outputs the in-band announcement of its re-alignment position one lead (C-43) ahead, a ratchet-step row of a sink has the same position and no announcement output (C-225), and no ratchet row changes the offset before that position (C-215).
7. Producer Sending → Stopped only via the 2 s report timer; one receiver report resets it.
8. Each state change row emits `ConnectionState(new state)`; every row entering Interrupted, Dead or Demoted lists its C-159 counter.
9. Every post-admission state has a row for the endpoint-clock-unusable input (C-162).

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The cross-machine invariant analysis (005-17).
- Scenarios (005-20, 005-25).
- Implementation (epic 012).
