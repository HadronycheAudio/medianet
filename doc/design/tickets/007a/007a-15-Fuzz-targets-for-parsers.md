# 007a-15 — Fuzz targets for parsers
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** M · **Depends on:** 007a-04, 007a-11, 007a-13

## Context
- Deliverable: a fuzz harness skeleton with one target per parser and per state machine, the invalid vectors as seed corpus, and the CI budget of C-95: each parser 60 s on every change and 1 hour nightly, corpus kept.
- Parsers in the protocol `[decision]`: packet header, slice header, in-band announcements (end-of-stream, re-alignment, timeline epoch switch, transport reference change), measurement extension, content messages (audio f32/Q1.31, dense controllers, sparse keyed controllers, read-only transport state), control messages (at most 1 252 bytes of UDP payload, C-17; the path-MTU probe pair at media packet size, C-116), regime stamp sections and their reassembly into one stamp (C-18). Plus the `no_std` vector reader of 007a-03.
- Required outcome on any input: never crash, never change state, bounded time; drop and count per reason `[decision]`. The time bound is C-164: linear in packet length and at most 10 µs per packet, a proposal pending L1; targets read it from configuration so an L1 revision needs no code change.
- Epic criterion 007-AC7 (system criterion AC-1) includes zero parser panics and zero hangs within the fuzz budget.
- Targets call through `UnderTest::decode(layer, bytes)` (007a-04), so any implementation can be fuzzed.
- The fuzzing tool is the implementing team's choice (C-192); it must run unprivileged.

## Work
1. Put the fuzz harness in the `hadronyche-tools` crate (harness and tool binaries live there, C-133), using a fuzzing tool chosen by the implementing team (C-192) that runs unprivileged, with one target per parser layer listed above and one for the vector reader.
2. Each target: decode; if `Ok`, re-encode and decode again and assert equality; never panic; assert the per-input time bound.
3. Seed each target's corpus from `vectors/wire/invalid/` and `vectors/wire/valid/` of the same layer.
4. Document how to run one target locally.

## Acceptance criteria
1. One target exists per parser layer in Context (including regime stamp sections) and one for the vector reader.
2. Each target starts with a non-empty seed corpus from the committed vectors.
3. Each target runs for 60 s locally against a stub without harness errors.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- State-machine targets (007b-20). CI job and corpus keeping (007b-21).
