# 004-08 — Slice header and send-timestamp format
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-06
## Context
Each media packet carries a slice header after the packet header. A slice is the content of one stream for one stretch of media time, addressed by position (network time counted in samples from a domain epoch).

Rules `[decision]`:
- Rule 2: slice header carries position (64-bit, content position on the domain line), frames, packet index and packet count within the slice, and the send timestamp of this packet.
- C-01: the domain sample rate is 96 000 Hz and it is the only rate on the wire; position is therefore always in samples at 96 000 Hz. C-02: default slice length 48 frames = 0.5 ms = 2 000 slices/s.
- C-05: the default slice fits one packet at the path MTU; multi-packet slices are an explicit, stamped regime.
- C-18: the regime stamp is a dedicated control message in sections; the hello carries only its version number. The regime stamp section that declares a stream's multi-packet regime is laid out in 004-25; this ticket supplies the fields it must contain.
- C-15 and C-110 fix the reassembly deadlines (2 slice periods for arrival-driven inputs; the consumer's own deadline for aligned inputs); they are behaviour, not wire fields.
- Rule 9: dedup by stream, position and packet index; first valid copy wins; no separate sequence number.
- Rule 10: a gap slice is header-only.
- C-137: all headers plus the checksum take at most 100 bytes beside 1 152 bytes of audio; the slice header counts against that cap.
- Open sub-question owned by epic 004: **send-timestamp width and unit in the slice header**. Falsifier: the width wraps within one reassembly or alignment span, or the unit cannot resolve one sample period at 96 000 Hz.
- Epic 001 delivers a report on receive timestamps and don't-fragment per OS; it is non-blocking and affects only notes on timestamp fields.

## Work
1. Decide the send-timestamp width and unit (e.g. ns of local or network clock, or samples), state the clock it is read from, its wrap period, and justify (`[decision]`). Record the decision as the answer to the open sub-question.
2. Write the slice header field table in `wire.md`: position (64), frames, packet index, packet count, send timestamp, with valid ranges (frames ≥ 1 and an upper bound; packet index < packet count; packet count = 1 in the default regime).
3. Define the multi-packet regime (C-05) on the media side: packet count > 1 in the slice header. List the values the regime stamp must declare for it (for example stream number, packets per slice, frames per slice) and hand them to 004-25, which places them in a regime stamp section (C-18).
4. Validity rules with outcomes: packet index ≥ packet count (`BadPacketIndex`), packet count = 0, frames = 0 or above maximum, frames inconsistent with payload length (`BadLength`), packet count inconsistent across packets of one slice, position not a multiple expected for the stream (if a rule applies).
5. Add a note on receive-timestamp behaviour: the receive-timestamp source is one of kernel, userspace, mac-hardware or interrupt, each reported with a capability state (C-131, C-129) in the regime stamp (004-25), not in the slice header; per-OS detail is completed from epic 001's report.
6. Add the table to the machine-readable file.

## Acceptance criteria
1. The send timestamp has a stated width, unit, clock source and wrap period.
2. The slice header table contains position as 64 bits and all rule-2 fields with offsets and valid ranges.
3. The multi-packet regime has its regime stamp values named and handed to 004-25; the slice-header side of the regime is documented.
4. At least 5 invalid forms are listed with rejection reasons.
5. The machine-readable table passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Reassembly logic and jitter handling (implementation epics).
- Checksum over the reassembled slice (004-13).
