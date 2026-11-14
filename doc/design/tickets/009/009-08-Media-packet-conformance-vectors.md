# 009-08 — Media packet conformance vectors
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-04, 009-07, epic 007

## Context
System criterion AC-1 requires the conformance vectors for media packets to pass at 100 %, with zero parser panics or hangs under fuzzing (epic EC-1). The vectors are produced by epic 007, from the wire layouts and reject reasons of epic 004 and the timeline and epoch scenarios of epic 006 (C-193); the vectors crate sits at the workspace root (C-192). This epic's receive path must pass them.

The receive path under test is: `SourceFilter::admit` (009-04: header validation + pinned (IPv4 addr + port, stream) lookup; IPv4 only, C-100) → `Reassembler::push` / `expire` / `expire_before` (009-06, 009-07: dedup by (stream, position, packet index) with a 2-bit copy index, C-07; multi-packet assembly, C-05; reassembly deadline 2 slice periods for arrival-driven inputs, C-15, and the consumer's deadline for aligned inputs, C-110).

Facts the vectors are expected to reflect `[decision]`: one sample rate on the wire, 96 kHz (C-01); 4-byte samples (C-03); a default slice of 6 channels × 192 bytes = 1 152 bytes of audio with no per-channel id byte (C-04).

## Work
1. Add an integration test `hadronyche-core/tests/conformance_media.rs` that loads every media-packet vector from epic 007's vectors crate at the workspace root (C-192; embedded with `include_bytes!` or read in the test harness, not in the receive path) and, per vector, checks the expected outcome: accepted slice bytes, or the expected rejection reason (epic 004 reasons), or the expected dedup/reassembly result.
2. The test prints the count passed / total and fails on any mismatch.
3. Add the test to the unprivileged CI job.

## Acceptance criteria
1. 100 % of epic 007 media-packet vectors pass (AC-1).
2. Each rejection vector yields exactly the rejection reason the vector expects.
3. No vector causes a panic; the whole suite completes in under 10 s in CI (hang guard).
4. The receive-path calls inside the test run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Any vector disagreement is resolved by fixing code, or reported to epic 007 with the vector id (never by editing vectors locally)

## Out of scope
- Writing the vectors (epic 007).
- Fuzz targets (009-09).
- Fan-in and alignment vectors, if any (exercised in 009-12 to 009-21).
