# 006-16 — Scenario: two sources switch 10 slices apart into one fan-in
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-11, 006-12, 006-21
## Context
Epic 006 delivers prose scenario scripts that epic 007 turns into conformance vectors. This ticket writes the first: two sources feeding one fan-in switch epochs 10 slices apart. It exercises system criterion AC-18: a fan-in never aligns across timeline epochs; an input on another epoch is treated as a gap until it switches.

Facts (`[decision]`):
- Slice = 48 frames = 0.5 ms at 96 kHz (C-02), the only rate on the wire (C-01); positions 64-bit; epoch 16 bits (C-10).
- Each source announces its switch at current position + 64 slices (C-43), re-issued every 16 slices (C-44), in every bundle member; at the switch position it ends the old session and starts a new one with a bumped session number (C-19) and the new epoch.
- At a fan-in, an input still on another epoch is treated as a gap until it switches; a fan-in never combines samples from two epochs. The fan-in's output follows the source switch procedure itself.
- An output whose input at `p − L` was gapped is flagged degraded (C-122).
- A slice on another epoch resets the interrupted count (C-102), restarts the dead clock and increments the connection's other-epoch counter (C-135, C-159), so the 10-slice stagger never makes the input Interrupted even though it exceeds the 8-slice threshold (C-38).
- A vector can stage the stagger with the test-only hook that forces a stream onto a given timeline epoch (C-120).

## Work
1. Add a "Scenarios" subsection "S1 — staggered switch at a fan-in" to `doc/design/spec/timeline.md` (or a sibling file referenced from it) in this structure: setup, stimulus, expected observations, pass/fail rule.
2. Setup: sources A and B, both on epoch `e`, feeding fan-in F; give explicit starting positions, session numbers and the fan-in's own switch position.
3. Stimulus: epoch `e+1` is published; A switches at position `P` (slice-aligned), B at `P + 10 × 48` (10 slices later = 5 ms). State how a vector produces this (normal announcements, or the C-120 hook).
4. Expected: for each of the slices around the switch, list per input: epoch carried, whether F counts it as signal or gap, the counter incremented (received or other-epoch, C-159), the degraded flag of F's output (C-122), and F's output epoch; no output slice combines samples of `e` and `e+1`; B is never Interrupted (C-102).
5. State the observable outputs a vector checker reads (slice headers, gap flags, degraded flags) without defining the vector format.

## Acceptance criteria
1. The script lists, slice by slice, at least the 10 slices between the two switches plus 2 before and 2 after (≥ 14 rows).
2. In every row, F's output combines inputs of one epoch only; the input on the other epoch is a gap and the output is flagged degraded (system AC-18, C-122).
3. All positions are concrete integers on 48-sample slice boundaries.
4. The script states that no input enters Interrupted during the stagger (C-102).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Building the vector files and runner (epic 007); implementing the C-120 hook.
- The other two scenarios (006-17, 006-18).
