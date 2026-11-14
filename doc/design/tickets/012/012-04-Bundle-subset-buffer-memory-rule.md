# 012-04 — Bundle subset buffer memory rule
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-03

## Context
Epic 012 once asked how buffer memory is declared for a bundle subset. It is now decided by C-96, with C-04 and C-27 `[decision]`:
- The skew budget of a consumer is its presentation group's audition offset A (C-55); alignment rings are sized to A (C-96).
- The receiver computes the buffer need from the members it subscribes: subscribed member streams × slices spanned by A (rounded up to whole slices) × audio bytes per slice. At defaults a stream carries 6 channels of 192 bytes per slice, 1 152 bytes of audio (C-04), at 2 000 slices/s (C-02).
- Check against C-27 (derived): 16 members × 20 slices (10 ms) × 1 152 bytes = 368 640 bytes, about 370 KB.
- `Declared.buffer_bytes` is the figure for the listed members only; an empty member list means the whole output. A bundle has at most 32 members, the controller member included (C-08, C-209).
- Refusal reason: `BufferMemory`, checked by the receiver; decisive on embedded devices.

The skew budget of a consumer that belongs to no presentation group is settled by C-142: the largest A of the downstream groups it feeds, learned at admission, or else an explicit budget from configuration. Resolving that A is 012-26; this ticket takes A as an input and does not define a default.

## Work
- Implement `pub fn subset_buffer_bytes(members: u8, a_ns: u64, slice_ns: u64, audio_bytes_per_slice: u32) -> u32` (names may be refined): `members × ceil(a_ns / slice_ns) × audio_bytes_per_slice`, saturating, integer arithmetic. Rounding never under-estimates.
- Use it in `check_receiver` (012-03) as the figure compared with the remaining buffer capacity; refuse a request whose `Declared.buffer_bytes` is below the computed figure with `BufferMemory`, since its declaration under-states the ring (record as `[decision]` in the module docs).
- Write the formula in the module docs with the C-ids.

## Acceptance criteria
1. 16 members, A = 10 ms, defaults → 368 640 bytes (C-27).
2. Subscribing to k of n members yields k × the single-member figure for every k in 1..=n, n ≤ 32 (property test).
3. A that is not a whole number of slices rounds up (A = 10.1 ms → 21 slices).
4. Falsifier test: for a ring capacity R and a subset whose computed need is R + 1 byte, admission returns `BufferMemory`; at exactly R it is accepted.
5. No panic for maximal inputs (saturates).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Choosing A (commissioning, C-55) and resolving the skew budget outside presentation groups (012-26, C-142).
- Ring implementation (alignment epic). Other receiver checks (012-03).
