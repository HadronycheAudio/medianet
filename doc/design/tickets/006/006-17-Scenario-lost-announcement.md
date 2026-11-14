# 006-17 — Scenario: announcement lost three times, received on the fourth
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-12
## Context
Epic 006 delivers prose scenario scripts that epic 007 turns into conformance vectors. This ticket writes: an announcement lost three times and received on the fourth re-issue. It exercises 006-AC-4: with a lead of 64 slices and re-issue every 16, an announcement survives the loss of three consecutive carrying slices.

Facts (`[decision]`):
- An announcement names a future position with a lead of 64 slices (C-43; 32 ms at defaults, slice = 48 frames = 0.5 ms at 96 kHz, C-02) and is re-issued every 16 slices (C-44; 8 ms) until the named position passes, in every bundle member.
- Transmissions are therefore at 64, 48, 32 and 16 slices before the named position `P`.
- Consumers process positions in order; at `P` the source ends its old session and starts a new one with a bumped session number (C-19) and the new epoch.
- A slice absent at the deadline is counted missing (C-108). Only consecutive misses count toward Interrupted (8 slices, C-38, C-102); the three dropped slices are 16 slices apart, so they never make the input Interrupted.

## Work
1. Add "S2 — announcement lost three times" to the Scenarios section of `doc/design/spec/timeline.md`: setup, stimulus, expected, pass/fail rule.
2. Setup: one source, one consumer, epoch `e`, concrete positions; named position `P` (slice-aligned).
3. Stimulus: the slices carrying the announcement at `P − 64`, `P − 48`, `P − 32` slices are dropped; the one at `P − 16` slices arrives.
4. Expected: the consumer records the switch at `P − 16` slices, switches cleanly at `P` with the new session and epoch `e+1`; no gap is caused by the epoch change itself (only the 3 dropped slices are missing, missing counter +3, C-108); the input never enters Interrupted.
5. Add a one-paragraph contrast with the residual case (all 4 lost): the input is on another epoch and is treated as a gap until it is observed to switch.

## Acceptance criteria
1. The script names the 4 transmission positions as concrete integers and marks 3 dropped and 1 received.
2. The expected outcome is a clean switch at `P` with no epoch-caused gap, a missing count of exactly 3, and no Interrupted event.
3. The residual 4-lost case and its outcome are stated.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Building vectors (epic 007).
- Scenarios S1 and S3 (006-16, 006-18).
