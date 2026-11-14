# 009-09 — Fuzz targets for packet validation and reassembly
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-04, 009-07

## Context
Epic 009 must deliver fuzz targets for packet validation and reassembly, and every parser touched must be fuzzed. System criterion AC-1 (epic EC-1): zero parser panics or hangs under fuzzing.

- C-95 `[decision]`: fuzz budget = each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- C-192 `[decision]`: the fuzzing tool is the implementing team's choice. This ticket writes the targets for cargo-fuzz (libFuzzer) as a proposal; another tool is acceptable if it meets the C-95 budget and the criteria below with its equivalent options.
- A "hang" is an input that takes longer than the fuzzer's per-input timeout; this ticket sets it to 1 s (a test parameter, not a design constant).

Code under fuzz:
- `SourceFilter<N>::admit(&self, from: &Addr, pkt: &[u8]) -> Result<InputIndex, Reject>` (009-04): epic 004 header validation + pinned (IPv4 addr + port, stream) lookup (C-100).
- `Reassembler<SLOTS, MAX_SLICE>::push(&mut self, pkt: &MediaPacket, arrival_ns: i64)`, `expire(now_ns)` and `expire_before(position)` (009-06, 009-07): dedup by (stream, position, packet index), copy index 2 bits (C-07); multi-packet assembly (C-05); reassembly deadline 2 slice periods (C-15) or the consumer's deadline (C-110).

## Work
1. Add a `fuzz/` project (cargo-fuzz unless the team chooses another tool, C-192) for `hadronyche-core` with two targets:
   - `fuzz_packet_validate`: arbitrary bytes and an arbitrary IPv4 source address and port into a `SourceFilter` with a few pinned pairs.
   - `fuzz_reassembly`: arbitrary sequence of (packet bytes, arrival delta, push / expire / expire_before) operations into a small `Reassembler` (for example `SLOTS = 4`, `MAX_SLICE = 4 × 1152`); invariants asserted: no panic, returned payload length ≤ `MAX_SLICE`, `packets_missing` never exceeds the packet count claimed.
2. Seed corpora from the epic 007 media-packet vectors; keep the corpus in the repository (or the CI cache named in the job) so it grows across runs (C-95).
3. CI per C-95: 60 s per target on every change, in an unprivileged job; 1 hour per target in a nightly job.

## Acceptance criteria
1. Both targets build and run with the chosen fuzzing tool (C-192).
2. Per-change CI run: 0 crashes, 0 timeouts (1 s per input, for example libFuzzer `-timeout=1`) for each target over 60 s (C-95).
3. Nightly job: each target runs 1 hour (C-95) and fails the job on any crash or timeout.
4. The corpus is kept between runs: a second run starts from the corpus left by the first.
5. Invariants of step 1 are asserted inside the targets.
6. A deliberately injected panic in `admit` is found by `fuzz_packet_validate` within the 60 s run (verified once, then reverted).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Conformance vectors (009-08).
- Fuzzing parsers owned by other epics beyond what `admit` calls.
