# 004-15 — Recompute capacity figures from the final layout
**Epic:** 004-Wire-specification · **Effort:** XS · **Depends on:** 004-14
## Context
The design derived the wire size of one default packet as about 1 320 bytes including all Ethernet overhead (C-20) and about 21.1 Mbit/s per stream, about 3.5 Mbit/s per channel (C-21), at 2 000 slices/s (C-02: 48 frames at 96 000 Hz, the only rate on the wire, C-01). Per source: 24 ch = 4 streams, 84 Mbit/s; 48 ch = 8 streams, 169 Mbit/s; 96 ch = 16 streams, 338 Mbit/s (C-22).

Overheads `[decision]` unless tagged: IPv4 only (C-100), so 20 bytes of IPv4 header and 8 of UDP header. Ethernet framing adds 14 bytes of header, 4 of FCS, 8 of preamble and start-of-frame delimiter and 12 of inter-frame gap, 38 bytes in all `[memory]`; once the IEEE 802.3 clause is checked the tag becomes `[standard]` with the clause (C-132). With 1 152 bytes of audio (C-04), 1 320 bytes on the wire leaves about 102 bytes for the protocol's own headers. The header cap C-137 (at most 100 bytes for all headers and the checksum) bounds a default packet without announcements or measurement extension at 1 152 + 100 + 28 + 38 = 1 318 bytes on the wire.

C-219: the wire cost of a controller stream (a bundle source's controller member, C-205, or a single-stream source's controller second stream, C-181) is computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. The 66 bytes are the 28 of IPv4 and UDP plus the 38 of Ethernet framing above. Admission and the commissioning tool use this same figure.

Constants computed from C-20 and C-21, to be listed if a recompute is needed: C-22, C-24, C-25, C-26, C-90, C-92 and C-118. If the final layout changes the packet size by more than 5 %, the capacity constants must be recomputed `[decision]`. Epic acceptance criterion 6: the capacity figures are recomputed from the final layout and the difference stated.

## Work
1. From the 004-14 default budget, compute the on-wire size per packet: UDP payload + 8 (UDP) + 20 (IPv4) + 38 (Ethernet header, FCS, preamble/SFD, inter-frame gap), with each overhead stated and tagged (`[standard]` with clause where checked, C-132).
2. Compute bit rate per stream = wire bytes × 8 × 2 000, per channel (÷ 6), and per source for 24, 48 and 96 channels (C-22).
3. State the difference from 1 320 bytes and 21.1 Mbit/s in bytes and percent.
4. If the difference exceeds 5 %, mark C-20, C-21 and the dependent constants listed above for recompute in the design constants table and record which are affected.
5. State the wire cost of a controller stream by the formula of C-219 (bytes per packet and bit/s as a function of controller bytes per slice), with the header term taken from the 004-14 budget, so that admission's declared rates (004-18) and the commissioning tool use the same figure.

## Acceptance criteria
1. `wire.md` states the new wire size per default packet and bit rate per stream with the arithmetic shown.
2. The difference from 1 320 bytes / 21.1 Mbit/s is stated in absolute and percent, and the per-source rates are compared with C-22.
3. If > 5 %, the constants table is updated (or an open item with owner and falsifier is recorded).
4. `wire.md` states the controller-stream wire cost of C-219 as a formula in controller bytes per slice, at 2 000 packets/s, with the 66 bytes of overhead shown.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Network capacity planning beyond C-20/C-21 and the controller-stream figure of C-219.
