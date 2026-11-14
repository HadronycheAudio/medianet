# 007a-14 — Dedup scenario vectors
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-07, 007a-04

## Context
- Dedup rule `[decision]`: packets are deduplicated by stream (16 bits), position (64 bits) and packet index; the first valid copy wins.
- The packet header has a 2-bit copy index; it is reserved and must be zero in v1 (C-07), so dedup cannot rely on it.
- A copy that is invalid (for example wrong slice checksum) is not a "valid copy" and must not win.
- Dropped copies are counted in the per-connection duplicates counter published in the regime stamp (C-159).

## Work
1. Write sequence vectors (input list → expected accepted packets and counters) covering:
   - two identical copies: first accepted, second dropped as duplicate;
   - first copy invalid (bad checksum), second valid: second accepted;
   - same position and stream, different packet index: both accepted;
   - same position and packet index, different stream: both accepted;
   - two valid copies with different payload: first one wins.
2. Store them under `vectors/wire/` in the format of 007a-02 and extend the driver to check them.

## Acceptance criteria
1. At least 5 vectors, one per case in step 1.
2. In each, the expected accepted payload equals the first valid copy byte for byte, and the duplicates counter (C-159) rises by exactly the number of valid copies dropped.
3. All files carry `spec_version`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- Redundant-path transmission mechanics and copy index semantics (reserved in v1).
