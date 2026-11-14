# 007a-06 — Valid vectors: announcements and measurement extension
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-05

## Context
- The generator `gen` (in the `hadronyche-tools` crate, C-133) (from 007a-05) derives wire vectors from the machine-readable tables of the wire specification. Each message type needs at least one minimal, one typical and one maximal valid encoding with its decoded structure.
- Optional in-band announcements carried in media packets `[decision]`: end-of-stream, re-alignment, timeline epoch switch, transport reference change.
- Optional measurement extension `[decision]`: hop count, accumulated residence, accumulated declared delay.
- In-band announcements have a flat lead of 64 slices (C-43) and are re-issued every 16 slices until the named position passes, in every bundle member (C-44). This bounds the typical and maximal values of any "slices until" field.
- Header cap (C-137): a default slice must fit 1 252 bytes of UDP payload, that is 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path.

## Work
1. Extend `gen` to emit minimal, typical and maximal vectors for each of the 4 announcement types and for the measurement extension.
2. Include at least one vector with an announcement and the measurement extension present together in one default audio packet (6 channels, 48 frames), and one default audio packet for a fallback path in which they are carried inside the 1 252-byte cap or, where the wire tables say they do not fit, omitted (C-137).
3. Commit the generated `.bin` + `.json` files to `vectors/wire/valid/`.

## Acceptance criteria
1. At least 15 vectors exist: 3 each for end-of-stream, re-alignment, timeline epoch switch, transport reference change and the measurement extension.
2. At least 1 vector combines an announcement with the measurement extension in a default audio packet whose UDP payload is at most 1 472 bytes, and at least 1 default audio packet for a fallback path has a UDP payload of at most 1 252 bytes with the announcement and extension either inside it or omitted (C-137).
3. No vector carries a lead above 64 slices.
4. All files pass the 007a-02 schema test and carry `spec_version`.
5. `gen` output stays deterministic (two runs, identical bytes).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Invalid announcement encodings (007a-11).
- Epoch-switch timeline semantics and re-issue timing (007b-14).
