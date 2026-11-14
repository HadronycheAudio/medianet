# 010-23 — Header cap and fallback omission
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-03, 010-07, 010-13
## Context
- C-137 (new): a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path.
- C-06 and C-100: the 1 280-byte fallback gives 1 252 bytes of UDP payload. C-04: 6 channels × 192 bytes = 1 152 bytes of audio.
- Consequence for this epic: a default 6-channel slice is always one packet, also at the fallback (C-05); the multi-packet regime (010-09) is never triggered by the fallback alone for a default stream.
- The exact header budget h comes from the wire specification (epic 004); C-137 bounds it. The room left for in-band content at the fallback is 100 − h bytes.
- C-181: a single-stream source may share the audio packet with its controllers only while headers plus controllers stay within the cap (it may always use a second stream instead, 010-18), so controller content is never omitted; on a fallback path the optional items fit in the room left after headers and controllers, or are omitted (010-18).
- Announcements (010-13) and the measurement extension (010-07) are optional parts of the packet. Re-issue every 16 slices (C-44) continues as scheduled; an omission only affects the slice where the content does not fit.
- Falsifier of C-43: a descendant misses an announcement despite four re-issues. A path where announcements never fit is reported, not hidden (counter below).

## Work
- In `hadronyche-core`, add `HEADER_CAP_BYTES = 100` (C-137) to the send-path constants of 010-01 and a compile-time assertion that `DEFAULT_HEADER_BYTES` (epic 004 headers and checksum, without optional in-band content) is at most `HEADER_CAP_BYTES`, and that 1 152 + `HEADER_CAP_BYTES` = 1 252.
- In `SliceWriter::finish`, for a single-packet default slice on a path whose UDP payload is below what the slice plus its optional in-band content needs (the fallback case), omit pending announcements and the measurement extension that do not fit, rather than switching to the multi-packet regime. Each optional item is written whole or omitted whole. When not all fit, keep items in layout order while they fit; document this order as provisional, owner epic 004, unless the wire specification sets one.
- Leave the announcement's pending entry intact so the next scheduled re-issue tries again.
- Add a per-stream, allocation-free monotonic counter of omitted in-band items, exposed next to `SendCounters` (010-21) so the regime stamp can show it.

## Acceptance criteria
1. The compile-time assertion `DEFAULT_HEADER_BYTES <= 100` holds; a test build with a header budget of 101 fails to compile.
2. A 6-channel, 48-frame slice with no in-band content at path MTU 1 280 encodes as exactly one packet of at most 1 252 bytes of UDP payload (epic AC-14).
3. The same slice with a pending announcement that does not fit within the cap at 1 280 encodes as one packet without the announcement, and the omission counter increases by 1; at MTU 1 500 the announcement is carried.
4. The same holds for a requested measurement extension: omitted at 1 280 when it does not fit, carried at 1 500.
5. After an omission, the announcement is carried at the next C-44 re-issue position where it fits, and its target is unchanged.
6. An announcement or extension that fits within the cap at 1 280 is carried.
7. No packet's UDP payload ever exceeds path MTU − 28 bytes in these tests (C-100).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The value of the header budget h and the byte sizes of announcements and the extension (epic 004).
- Multi-packet slices wider than one packet (010-09).
- Receive-side handling of a slice without its announcement (epic 009).
