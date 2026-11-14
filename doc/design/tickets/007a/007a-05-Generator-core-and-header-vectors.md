# 007a-05 — Generator core and header vectors
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** M · **Depends on:** 007a-02, epic 004

## Context
- The generator `gen` (hosted only; a tool binary in the `hadronyche-tools` crate of the design set's repository, C-133) derives the wire vectors from the machine-readable tables of the wire specification, so vectors and specification cannot drift `[decision]`. The wire tables (layouts and reject reasons) come from epic 004; epic 005 supplies the state tables and scenario scripts, epic 006 the timeline and epoch scenarios, both used by 007b (C-193).
- For every message type the committed set needs at least one minimal, one typical and one maximal valid encoding, each with its decoded structure.
- Packet header fields `[decision]`: magic, version, domain (16 bits, 0 = unconfigured; C-11), stream number (16 bits), session (32 bits), timeline epoch (16 bits) (C-10), copy index (2 bits, reserved; C-07), flags including gap and degraded.
- The session number is assigned by the source, one per stream, bumped on restart and on timeline epoch change; every consumer of a stream sees the same value (C-19).
- Slice header fields `[decision]`: position (64 bits), frames, packet index and count, send timestamp.
- Every stream is at the domain rate of 96 000 Hz, the only rate on the wire (C-01).
- Default slice: 6 channels, 48 frames, one packet, at most 1 472 bytes of UDP payload on a 1 500-byte path (C-02, C-04, C-05). IPv4 only (C-100). Fallback path MTU 1 280 bytes, giving 1 252 bytes of UDP payload (C-06, C-100).
- Header cap (C-137): a default slice must fit 1 252 bytes of UDP payload, so all headers and the checksum together take at most 100 bytes beside the 1 152 bytes of audio. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path.
- Reserved in v1 `[decision]`: the copy index must be zero (C-07). The authentication reservation of control messages (scheme 8 bits + counter 64 bits + tag 128 bits, C-12) has scheme 0 and tag zero; the 64-bit counter is live, because it is the liveness counter (C-115).

## Work
1. In `gen`, load the machine-readable wire tables and build a table-driven encoder and decoder for field layouts.
2. Emit to `vectors/wire/valid/` the packet header and slice header vectors: for each, minimal, typical and maximal, as `.bin` + `.json` with the decoded structure, the `spec_version` taken from the tables, and a stable `id`.
3. Generation must be deterministic: same tables, same bytes, same file names. The file naming scheme within `vectors/wire/valid/` and `vectors/wire/invalid/` is this epic's choice (C-206); record it in the crate documentation.
4. Commit the generated files.

## Acceptance criteria
1. At least 3 vectors (minimal, typical, maximal) exist for the packet header and 3 for the slice header.
2. All emitted files pass the schema test of 007a-02 and carry the `spec_version` of the input tables.
3. Running `gen` twice on the same tables produces byte-identical output.
4. The copy index is zero in every valid vector; wherever the authentication reservation appears, the scheme is 0 and the tag is zero, and at least one typical vector carries a non-zero liveness counter (C-115).
5. The maximal vectors respect 1 472 bytes of UDP payload for the default path.
6. The packet header plus slice header of a default slice, with the checksum, take at most 100 bytes (C-137).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Announcements and the measurement extension (007a-06), content messages and the checksum (007a-07), control messages (007a-08), the regime stamp message (007a-09).
- Invalid vectors (007a-11).
- The CI reproducibility check (007a-10).
