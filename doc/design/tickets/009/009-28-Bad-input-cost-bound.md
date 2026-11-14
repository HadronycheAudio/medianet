# 009-28 — Bad-input processing bound
**Epic:** 009-Gap-and-reassembly-core · **Effort:** XS · **Depends on:** 009-04, 009-07, 009-09

## Context
C-164 (proposal, pending L1) `[decision]`: processing one bad input is linear in packet length and takes at most 10 µs per packet. In this epic a bad input is a packet that `SourceFilter::admit` rejects (009-04) or that the reassembler discards (oversize, inconsistent packet count or index, 009-06, 009-07). The 10 µs figure is measured by L1 (task 019) on the reference hardware; this ticket proves the linear part in CI and provides the benchmark L1 runs.

## Work
1. A test that counts work per rejected packet without timing: instrument `admit` and the reassembler's discard paths with a test-only operation counter, and feed rejected packets of lengths 0 to 9 000 bytes.
2. A benchmark (`cargo bench` or a criterion target beside the tests) that reports the time per rejected packet for the same inputs, for L1 to run under its conditions.
3. Document both in the module doc, with the C-164 status (proposal, pending L1).

## Acceptance criteria
1. The operation count per rejected packet is at most a + b × length, with a and b fixed by the test, over all lengths 0 to 9 000 (linear, C-164).
2. A packet rejected at header validation costs the same count whatever its payload length.
3. The benchmark builds in CI and prints time per rejected packet; no timing threshold is asserted in per-change CI (the 10 µs figure is L1's).
4. Tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier (10 µs: L1, task 019; falsifier: a measured rejected packet above 10 µs)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Measuring on reference hardware (L1, task 019).
- Bad control messages (control-plane epics).
