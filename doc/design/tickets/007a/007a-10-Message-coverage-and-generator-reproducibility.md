# 007a-10 — Message-type coverage check and generator reproducibility
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-06, 007a-07, 007a-08, 007a-09

## Context
- Epic criterion 007-AC1: every message type of the wire specification has valid vectors in both encode and decode direction.
- Epic criterion 007-AC4: the generator `gen` (in the `hadronyche-tools` crate, C-133) reproduces the committed wire vectors byte for byte from the specification tables.
- Valid vectors must exist as minimal, typical and maximal for every message type `[decision]`, including the regime stamp message, its unicast request (C-18) and the path-MTU probe pair (C-116).
- Header cap (C-137): a default slice must fit 1 252 bytes of UDP payload, 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. A single-stream source's controllers that share the audio packet count inside those 100 bytes (C-181).

## Work
1. Add a test that reads the message-type list from the wire tables and fails if any type lacks a minimal, a typical or a maximal vector in `vectors/wire/valid/`.
2. Add a CI step that runs `gen` into a temporary directory and compares it with the committed `vectors/wire/` byte for byte (file set and content); any difference fails the job.
3. Add a test that fails if any valid default-slice vector (6 channels, 48 frames) has more than 100 bytes of headers, in-packet controllers and checksum, or more than 1 252 bytes of UDP payload (C-137, C-181).
4. Add a test that, for every valid vector, checks the `.json` decoded structure re-encodes (through the generator's table-driven encoder) to exactly the `.bin` bytes, and the `.bin` decodes to exactly the `.json` structure.

## Acceptance criteria
1. Removing any one valid vector makes the coverage test fail, naming the message type and the missing variant.
2. Editing 1 byte of any committed wire vector makes the reproducibility step fail.
3. 100 % of valid vectors round-trip in both directions through the generator's codec.
4. A default-slice vector with 101 bytes of headers and checksum makes the header-cap test fail, and so does one whose headers plus in-packet controllers plus checksum reach 101 bytes (C-181).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- Running vectors against a real implementation (007b-22 for the CI wiring; epics 009–014 for the implementation).
- Invalid-vector coverage (007a-12).
