# 004-22 — Traceability table and review against the architecture specification
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-07, 004-14, 004-15, 004-21
## Context
Epic 004 acceptance criterion 1: every rule numbered 1 to 22 in the epic background maps to at least one field or validity rule in `wire.md`, shown in a traceability table. Acceptance criterion 8: every value the constants table places on the wire has a field with a width and a unit. Criterion 5: media packets contain no destination-specific and no per-receiver field. The definition of done requires `wire.md` reviewed against the architecture specification section on protocol and packetisation, with the traceability table attached.

The numbered media rules `[decision]`:
1. Packet header: magic, version, domain (16), stream number (16), session (32), timeline epoch (16), copy index (2), flags including gap and degraded.
2. Slice header: position (64-bit), frames, packet index and packet count within the slice, send timestamp of this packet.
3. Optional in-band announcements: end-of-stream, re-alignment, timeline epoch switch, transport reference change; each names a future position.
4. Optional measurement extension: hop count, accumulated residence time, accumulated declared delay.
5. Content messages: audio, dense controllers, sparse keyed controllers, read-only transport state.
6. Slice checksum over the reassembled slice.
7. Streams identified by source and stream number, never by destination; module id not in media packets.
8. Forward packets carry no per-receiver state.
9. Dedup by stream, position and packet index; first valid copy wins; no sequence number.
10. Gap slice is header-only; degraded flag marks a slice computed from a gapped input.
11. Every packet sent with don't-fragment; IP fragmentation forbidden.
12. Version mismatch is a named event; version also in the hello.
13. Foreign-domain packets dropped and counted.

Control rules: 14. binary format, fixed maxima for a 63-peer map (C-217); 15. hello contents including the regime stamp version number (C-18); 16. counter-as-liveness, live in v1 while scheme and tag are zero (C-115); 17. admission contents, including the manifest (C-04 channel map, C-103, C-104, C-105 L in whole samples, C-106, C-140, C-158), the probe pair at the intended media packet size (C-116), IPv4 addresses (C-100), the downstream A (C-142), an edge's conversion delay on the device-facing port (C-207) and the 18 refusal reasons named in C-185; 18. regime stamp in sections ≤ 1 252 bytes, multicast on change and every 10 s, unicast fetch (C-18).

Sync rules: 19. stateless echo; 20. epoch and holder identity in every reply; 21. equal sizes. Measurement rule: 22. exactly two v1 messages, reports unicast only, no query message (C-47).

Constants traced: C-01 to C-13, C-17 to C-22, C-34, C-40, C-43, C-44, C-100, C-137, and the wire-visible rulings C-47, C-103 to C-107, C-113, C-115, C-116, C-120, C-122, C-127, C-129, C-131, C-135, C-136, C-140, C-142, C-145, C-147, C-152, C-153, C-157 to C-161, C-164, C-173, C-178, C-181, C-182, C-185, C-198, C-204, C-205, C-207, C-208, C-209, C-213, C-219.

The review compares `wire.md` with the architecture specification section on protocol and packetisation. The architecture specification is `doc/design/01-architecture.md` (C-171); this ticket identifies the section on protocol and packetisation in it and names it in the review record.

## Work
1. Add a "Traceability" section to `wire.md`: one row per rule (1 to 22) and constant above, mapping to the field(s) and/or validity rule(s) that implement it, with section anchors.
2. Add a field-by-field audit of the media packet header, slice header, announcements, measurement extension and content messages confirming no destination-specific or per-receiver field (AC-5).
3. Review `wire.md` against the section on protocol and packetisation of `doc/design/01-architecture.md` (C-171); record each discrepancy and its resolution; attach the traceability table to the review record.

## Acceptance criteria
1. All 22 numbered rules have at least one mapped field or validity rule (epic AC-1).
2. All listed constants and rulings are mapped, each wire-visible one to a field with width and unit (epic AC-8).
3. The no-per-receiver-field audit covers every media field and finds none (epic AC-5).
4. A review record exists listing reviewer, date, discrepancies and resolutions, with the traceability table attached.

## Definition of done
- [ ] `wire.md` reviewed against the architecture specification section on protocol and packetisation, with the traceability table attached
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Changing design documents beyond recording discrepancies (004-23).
