# 010-17 — Belated gap slices for arrival-driven modules
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-08
## Context
- A **gap slice** is header-only and marks positions the sender did not produce or receive `[decision]`.
- An arrival-driven module that sees a jump in input positions emits gap slices for the missed positions belatedly; they serve classification, not timing.
- The former open question "is a belated gap slice worth its bytes?" is settled: yes. Downstream uses marked gaps: consumers count them as their own gap class (C-47), a marked gap resets the interrupted count (C-102), and the dead threshold runs from the last slice with content or a marked gap (C-111). Belated emission is therefore always on; no switch is needed.
- C-02: 48 frames per slice at defaults.

## Work
- In `hadronyche-core`, add a `no_std`, allocation-free helper (suggested `missed_positions(last: u64, next: u64, frames: u16) -> impl Iterator<Item = u64>`) that yields the positions of the slices skipped between two input positions, to be passed to `gap_slice` (010-08).
- Remove the open item from the design document and cite C-47, C-102 and C-111 as the reason.

## Acceptance criteria
1. `missed_positions(1_000, 1_000 + 4 × 48, 48)` yields 1 048, 1 096, 1 144.
2. Consecutive positions (`next = last + 48`) yield nothing; `next <= last` yields nothing and does not panic.
3. A jump of 3 missed slices passed through `gap_slice` produces 3 gap slices with those positions.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Gap-slice encoding (010-08).
- Downstream classification using these gaps.
