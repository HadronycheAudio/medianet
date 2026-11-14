# 004-06 — Media packet header layout
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-02, 004-03, 004-04, 004-05
## Context
Media packets carry slices: the content of one stream for one stretch of media time, addressed by position on a domain-wide timeline. They travel by UDP unicast, one packet per slice by default, DSCP EF (46) (C-13).

Rules this header must satisfy `[decision]`:
- Rule 1: packet header carries magic, version, domain (16 bits), stream number (16 bits), session (32 bits), timeline epoch (16 bits), copy index (2 bits), flags including *gap* and *degraded*.
- C-10: stream number / session / timeline epoch = 16 / 32 / 16 bits. C-09: at most 256 streams per source.
- C-19: the session number is assigned by the source, one per stream; it is bumped on restart and on timeline epoch change, and every consumer of a stream sees the same value. It is therefore not a per-receiver field.
- C-01: the wire carries one sample rate only (96 000 Hz), so the header needs no sample-rate field.
- C-122: the degraded flag follows the content: an output at position p is degraded if its input at p − L was gapped.
- C-11: domain id 16 bits; domain 0 = unconfigured.
- C-07: copy index 2 bits, reserved for a deferred feature (duplication); required value and receiver rule from 004-05.
- Rule 7: streams are identified by source and stream number, never by destination. The receiver demultiplexes on the source address pinned at admission plus the stream number. The 64-bit module id is **not** in media packets.
- Rule 8: forward packets carry no per-receiver state; round-trip closure uses the measurement path.
- Rule 10: a gap slice is header-only and marks positions the sender did not produce or receive; the degraded flag marks a slice computed from a gapped input.
- Rule 11: every packet is sent with don't-fragment set; IP fragmentation is forbidden. C-100: IPv4 only in v1, so this is the DF bit of the IPv4 header.
- Rule 13: foreign-domain packets are dropped and counted.
- The POC header was 88 bytes with direction-overloaded fields (`ack_seq`, `ack_t4`, `path_delay`) and per-receiver acknowledgements (`F_ACK`) `[code]`; all retired.
- Header bytes compete with audio: 6 channels use 1 152 bytes, and C-137 caps all headers plus the checksum at 100 bytes so that a default slice fits 1 252 bytes of UDP payload. This header's size counts against that cap.
- C-173: `ForeignDomain` is for a different non-zero domain.

## Work
1. Write the media packet header field table in `wire.md` (offset, width, type, meaning, valid range), using the magic and version from 004-04 and the copy index rule from 004-05.
2. Define the flags field: at least *gap*, *degraded*, and presence bits for optional parts (announcements, measurement extension), with unused bits reserved (004-05 rule).
3. Write the validity rules for the header with an outcome per violation: shorter than header (`TooShort`), bad magic, version mismatch, domain ≠ local (`ForeignDomain`), domain 0, stream number unknown for the pinned source (`UnknownStream`), source not admitted (`UnadmittedSource`), timeline epoch not current (`WrongEpoch`), reserved bits non-zero, gap flag with non-empty payload, flags combination invalid.
4. State explicitly that no field is destination-specific or per-receiver (rule 7, rule 8; the session is per stream, C-19), and that the don't-fragment requirement (rule 11) is a sender obligation on the IPv4 DF bit (C-100), outside the header.
5. Write the meaning of the session field (C-19) and of the degraded flag (C-122) beside the table.
6. Add the table to the machine-readable file.

## Acceptance criteria
1. The header table contains all rule-1 fields with the C-10/C-11/C-07 widths: domain 16, stream 16, session 32, epoch 16, copy index 2.
2. The header contains no module id, no destination address, no per-receiver acknowledgement field (checked field by field in a short note).
3. Header size in bytes is stated and every field offset is given.
4. At least 10 invalid forms are listed, each with a rejection reason or named event.
5. The session field's meaning cites C-19 and the degraded flag's meaning cites C-122.
6. The machine-readable table passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Slice header (004-08), announcements (004-09), measurement extension (004-10), content messages (004-11, 004-12), checksum (004-13).
- The 16-bit epoch wrap analysis (004-07).
- Emitting the gap slice when a module misses its own deadline: the hosted backend, epic 015 (C-144).
- The size budget (004-14).
