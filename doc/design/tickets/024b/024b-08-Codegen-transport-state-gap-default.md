# 024b-08 — Codegen: transport-state gap default
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-07, 024b-10, epic 010

## Context
Read-only transport state is a v1 content domain; there is no transport authoring `[decision]`. It reaches a module as `InputSlice.transport: Option<&TransportState>` (type from epic 010). Rule (C-141): on a gap, the last known transport state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded. A stopped transport's playhead does not advance through a gap (C-200). The degraded flag follows from the general rule that an output is degraded when an input it depends on was gapped (C-122), implemented in 024b-10. This ticket was added because C-141 settled the former open sub-question "gap default for read-only transport state", which 024b-07 left out.

Rule (C-184): before any transport state has arrived, the transport state is stopped, playhead at zero, tempo 120 beats per minute (default tempo, C-199). A transport input gapped before any state has arrived therefore reads that state (not `None`, which remains the case when no transport is attached), and dependent outputs are degraded as for any gap (C-122). Being stopped, its playhead stays at zero through the gap (C-200).

Epic acceptance criterion 024b/AC-5 (transport part): a transport input flagged gap reads the last known state with the playhead advanced as that state dictates — a playing transport moves on at its tempo over the positions elapsed, a stopped one does not advance — and dependent outputs are degraded (C-141, C-200); before any state has arrived it reads stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). Slice 48 frames (C-02).

## Work
- In `hadronyche-dsl/src/codegen/gaps.rs`, for each transport input: keep the last received `TransportState` and the position at which it was received in `State` (counted in `state_bytes`); when `gap` is true, substitute the held state: if it is playing, with its playhead moved on at its tempo over the positions elapsed between that position and the current `position` argument; if it is stopped, with its playhead unchanged (C-141, C-200).
- Initialise the held transport state in `init` to stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199), so that a gap before any transport state has arrived presents that state.
- Tests on a transport-reading corpus variant: consecutive gaps, a gap after a received playing state, a gap after a received stopped state, a gap before any state.

## Acceptance criteria
1. After a received playing state with playhead h at position p0, a transport input flagged gap at position p reads that state with playhead h moved on at its tempo over the p − p0 positions elapsed (C-141).
2. Over several consecutive gapped slices a playing transport's playhead keeps moving on at its tempo, with no jump when content resumes at the expected value.
3. After a received stopped state with playhead h, a transport input flagged gap, over one or several consecutive gapped slices, reads that state with playhead h unchanged (C-200).
4. Every output that depends on the gapped transport input is marked degraded, and outputs that do not depend on it are not (with 024b-10).
5. A gap before any received state presents a stopped transport with playhead at zero and tempo 120 beats per minute (C-184, C-199), the playhead staying at zero over consecutive gapped slices (C-200), and dependent outputs are marked degraded.
6. The `const` state-size assertion passes with the held transport state included.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The `TransportState` type and its playhead unit (epic 010).
- Gap defaults for other content types (024b-07) and DSL overrides (024b-09).
- Transport authoring (not in v1).
