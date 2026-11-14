# 006-18 — Scenario: a source that never switches
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-11, 006-12, 006-21
## Context
Epic 006 delivers prose scenario scripts that epic 007 turns into conformance vectors. This ticket writes: a source that never switches epoch. It exercises system criterion AC-18: a fan-in never aligns across timeline epochs; an input on another epoch is treated as a gap until it switches.

Facts (`[decision]`):
- After an epoch change each source must announce a switch at current position + 64 slices (C-43), re-issued every 16 slices (C-44), and switch to a new session (bumped, C-19) with the new epoch.
- At a fan-in, an input still on another epoch is treated as a gap until it switches; a fan-in never combines samples from two epochs. The output is flagged degraded while the input is gapped (C-122).
- A slice on another epoch proves the upstream is alive and resets the interrupted count (C-102), so the faulty source never makes its input Interrupted.
- At a sink, `emit(p) = T(p) + A` uses the line of the epoch the slice carries; the map is never extrapolated across epochs.
- Slice = 48 frames = 0.5 ms at 96 kHz (C-02); epoch is 16 bits (C-10).
- A vector can make a source stay on the old epoch with the test-only hook that forces a stream onto a given timeline epoch (C-120).
- A slice on another epoch restarts the dead clock (C-111) and has its own per-connection counter (C-135), the other-epoch counter of the stamp (C-159). So B, which keeps sending on epoch `e`, is never declared dead, and F's other-epoch counter for B grows by one per slice.

## Work
1. Add "S3 — source never switches" to the Scenarios section of `doc/design/spec/timeline.md`: setup, stimulus, expected, pass/fail rule.
2. Setup: sources A (conformant) and B (faulty, held on epoch `e` by the C-120 hook) feeding fan-in F, plus one sink directly fed by B; epoch `e`, concrete positions.
3. Stimulus: epoch `e+1` is published; A switches at its announced position; B keeps sending on epoch `e`. Run for at least 6 000 slices (3 s), longer than the 2 s dead threshold (C-39), so that the absence of a dead verdict is observable.
4. Expected: F treats B as a gap for every slice after F's switch, flags its output degraded (C-122), and never combines B's `e` samples with `e+1`; B is never Interrupted (C-102); the sink fed by B behaves as specified in the consumer section for slices of a non-current epoch; B is never dead (C-135) and F's other-epoch counter for B increases by one per slice (C-159), per 006-21; any event or diagnostic emitted is listed.

## Acceptance criteria
1. The script states the run length in slices (≥ 6 000) and the switch positions as concrete integers.
2. Expected output: B is a gap at F for every slice after F's switch (system AC-18); 0 output slices mix epochs; every such output slice is flagged degraded.
3. No Interrupted event and no dead verdict for B (C-102, C-135); the other-epoch counter for B equals the number of B slices received after F's switch (C-159).
4. The sink-side outcome is stated and consistent with the consumer-side section.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Building vectors (epic 007); implementing the C-120 hook.
- Diagnosing or repairing a faulty source.
