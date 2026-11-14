# 007a-03 — No-std vector reader
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-02

## Context
- Crate `hadronyche-vectors` is "data plus a tiny `no_std` reader" `[decision]`. The reader API is a deliverable of epic 007 itself. The same vectors are run by the hosted core (epics 009–014), by DSL-generated code (epic 025) and on embedded devices (epic 026), so the reader must work without `std` and without an allocator if possible.
- The vector kinds and the mandatory `spec_version` and `id` fields are defined in 007a-02.

## Work
1. In `hadronyche-vectors/src/`, add a reader API that enumerates wire vectors (valid and invalid) and yields, for each: `id`, `spec_version`, the raw bytes, and either the decoded structure (valid) or the rejection reason (invalid).
2. Add equivalent enumeration for control, timeline, transport and reference vectors.
3. Embed the data in the crate at build time (for example with `include_bytes!` from a build step) so a device without a file system can iterate it.
4. Keep the reader free of `std`; if an allocator is needed, put it behind a feature flag and document why.

## Acceptance criteria
1. The reader compiles with `#![no_std]` for at least one target without `std`.
2. On the host, a test enumerates all 6 example fixtures from 007a-02 and gets back their `id` and `spec_version` unchanged.
3. Byte content returned for a `.bin` fixture equals the file content byte for byte.
4. The crate has no `std` dependency in its default feature set.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The reader's own parsing is listed as a fuzz target for 007a-15
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- The runner trait and the driver that feeds vectors to an implementation (007a-04).
- Generating vectors (007a-05 onward).
- Running on a real embedded device (epic 026).
