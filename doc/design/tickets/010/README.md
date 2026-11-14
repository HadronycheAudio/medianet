# Epic 010 — Packetiser and bundles core: tickets

## Summary
Epic 010 (phase P3, depends on epics 004, 006, 007, 009; blocks 012, 014, 015, 017, 024, 026; scope L) implements the send side of the data plane in `hadronyche-core` as pure `no_std` code with no sockets and no allocation. It covers slice and packet encoding (one packet per slice by default, an explicit multi-packet regime with one send timestamp per packet, no IP fragmentation, IPv4 only so the UDP payload is the path MTU − 28 bytes, headers and checksum capped at 100 bytes so a default slice fits the 1 280-byte fallback, C-137), position stamping (output = p − L with L a whole number of samples, C-105, never restamped), channel bundles (narrow 6-channel members of 192 bytes per channel per slice, at most 32 per bundle with the controller member included (C-209), one session per member stream assigned by the source, controllers in their own member), sizing and wire-rate functions used for admission (single wire rate, 96 kHz, controller streams costed per C-219), the intra-slice pacing schedule (spread over at most 25 % of the 0.5 ms slice period plus a per-source phase offset from the module id in [0, 75 %) of the period, C-155), header-only gap slices (including belated ones) and the degraded flag, in-band announcements (target = issue position + 64 slices, re-issued every 16 slices in every member, relayed unchanged) including the source-side timeline epoch switch, the producer-side send counters for the regime stamp, and a test-only epoch forcing hook.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 010-01 | Send-path constants and no_std build gate | S | epic 004 |
| 010-02 | Output position stamping | XS | 010-01 |
| 010-03 | Packet sizing functions | S | 010-01, epic 004 |
| 010-04 | Bundle and member layouts | S | 010-03, epic 004 |
| 010-05 | Wire rate of a bundle | S | 010-03, 010-04 |
| 010-06 | Single-packet slice encoder | M | 010-02, 010-03, epic 004, epic 007 |
| 010-07 | Measurement extension slot | XS | 010-06, epic 004 |
| 010-08 | Gap slice encoding | S | 010-06 |
| 010-09 | Multi-packet regime | M | 010-03, 010-06, epic 004, epic 007 |
| 010-10 | Phase offset from module id | S | 010-01 |
| 010-11 | Intra-slice pacing schedule | S | 010-10 |
| 010-12 | Regime stamp exposure | XS | 010-09, 010-11 |
| 010-13 | Announcements: lead and re-issue | S | 010-06, 010-09, epic 004 |
| 010-14 | Announcements in every bundle member | S | 010-04, 010-13 |
| 010-15 | Relaying announcements unchanged | S | 010-02, 010-13 |
| 010-16 | Source-side epoch switch | M | 010-13, 010-14, epic 006 |
| 010-17 | Belated gap slices for arrival-driven modules | XS | 010-08 |
| 010-18 | Controllers for a single-stream source (revised, C-181 permissive) | S | 010-04, 010-06, epic 004 |
| 010-19 | Packetiser round-trip fuzz target | S | 010-08, 010-09, 010-13, epic 009 |
| 010-20 | Send-path audit and design document update | S | 010-01 to 010-19, 010-21 to 010-24 |
| 010-21 | Send counters for the regime stamp (new, C-47) | XS | 010-08, 010-14 |
| 010-22 | Test-only epoch forcing hook (new, C-120) | XS | 010-06, 010-14 |
| 010-23 | Header cap and fallback omission (new, C-137) | S | 010-03, 010-07, 010-13 |
| 010-24 | Wire cost of a controller stream (new, C-219) | XS | 010-04, 010-05, 010-18 |

Changes since the split: 010-21, 010-22, 010-23 and 010-24 added. 010-02 went from XS to S for the rational-L rule and is back to XS: C-105 now makes L a whole number of samples and withdraws rational L, so its rational type, rounding error path and their tests are removed. No ticket deleted: no ticket existed only for rational L (010-02 still delivers `output_position`), and 010-17 is kept because its open question was settled in favour of the work (C-47, C-102, C-111).

## Effort tally
- XS: 7
- S: 14
- M: 3
- Total: 24 tickets, about 23 days (XS 0.25, S 1, M 2.5 days: 1.75 + 14 + 7.5 = 23.25) → scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `StreamId`, `MemberLayout`, `BundleLayout<M>` (M ≤ 32, controller member included, C-209) | 010-04 |
| `channels_per_packet`, `packets_per_slice`, `media_packet_bytes` (C-116 probe size) | 010-03 |
| `wire_bits_per_second` (admission unit) and `packets_per_second` for epic 012 | 010-05, 010-24 |
| Wire cost of a controller stream: one packet per slice of headers + controller bytes + 66 bytes, 2 000 packets/s (settled, C-219) | 010-24 |
| `Packetiser::new` (session from the source, C-19), `begin_slice`, `SliceWriter::content`, `SliceWriter::finish` (single packet, `OutTooSmall`) | 010-06 |
| `SliceWriter::finish` multi-packet regime, one stamp per packet | 010-09 |
| Measurement extension location for epic 014 | 010-07 |
| `Packetiser::gap_slice` (with `send_ns`, `OutTooSmall`) | 010-08, 010-17 |
| `Packetiser::announce` (lead, re-issue) | 010-13 |
| `BundlePacketiser` (every member) | 010-14 |
| `Packetiser::relay_announcement` | 010-15 |
| `Packetiser::switch_epoch` (issues the announcement, returns T, bumps sessions) | 010-16 |
| `PaceSchedule`, `pace` (incl. phase offset, errors for capacity and spread) | 010-10, 010-11 |
| `output_position` (p − L with L in whole samples, C-105; none when p < L) | 010-02 |
| `SendRegime` (packets per slice and pacing mode) | 010-12 |
| `SendCounters` (C-47/C-159 producer side) | 010-21 |
| Header cap and omission of in-band content on a fallback path, omission counter (C-137) | 010-23 |
| Test-only epoch hook (C-120) | 010-22 |
| Unit tests | every ticket |
| Conformance-vector tests | 010-06, 010-07, 010-08, 010-09, 010-13, 010-20 |
| Fuzz target round-tripping through the epic 009 reassembler, C-95 budget | 010-19 |
| `no_std`, no allocation, no lock | 010-01, 010-20 |
| Open sub-question: phase derivation and sharing for 64 modules | 010-10 |
| Phase range [0, 75 %) of the period (settled, C-155) | 010-10, 010-11 |
| Controllers of a single-stream source: second stream always allowed and required beyond the 100-byte cap, shared audio packet allowed while within it (settled, C-181 revised) | 010-18, 010-23 |
| Controllers of a bundle source always in their own member stream (settled, C-205, C-181) | 010-04, 010-09 |
| Content-position stamping and sign convention (settled, C-105, C-122); resampler delay in D, never in L (settled, C-207) | 010-02 |
| A fan-in's own re-alignment after a changed A, or a ratchet step, is announced, not relayed (settled, C-208, C-186, C-215) | 010-15 |
| Missed-deadline gap slice emitted by epic 015 through `gap_slice` (settled, C-144) | 010-08 |
| Open sub-question: spread plus phase vs queue drops (L1, task 019) | noted in 010-10, 010-11, 010-20 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 1. Media-packet conformance vectors byte for byte (system AC-1) | 010-06, 010-07, 010-08, 010-09, 010-13, 010-20 |
| 2. `channels_per_packet(1500, 48, h) ≥ 6`; at 1 280 equals ⌊(1 252 − h)/192⌋ ≥ 6 (C-137) | 010-03, 010-23 |
| 3. 24/48/96 ch → 4/8/16 streams, ≈ 84/169/338 Mbit/s and 8 000/16 000/32 000 packets/s within 5 % | 010-04, 010-05 |
| 4. Own send timestamp per packet, UDP payload ≤ path MTU − 28 | 010-09, 010-19 |
| 5. No destination-specific byte | 010-06 |
| 6. Pacing within 25 % after phase; deterministic phase in [0, 75 %) (C-155), distinct for test ids; spread 0 = one instant; refusals without panic | 010-10, 010-11 |
| 7. Announcement target +64, every member, re-issue every 16, absent after | 010-13, 010-14 |
| 8. System AC-17 send part: announced position unchanged through arrival-driven module | 010-15 |
| 9. `switch_epoch` announces T = +64 slices; every member switches at T to a bumped session and the new epoch (AC-18) | 010-16 |
| 10. Compiles for `no_std` without allocator | 010-01, 010-20 |
| 11. `output_position`: p − L for L in whole samples, none when p < L | 010-02 |
| 12. Per-stream content and gap slice counters | 010-21 |
| 13. Fuzz 60 s per change, 1 hour nightly, corpus kept (C-95) | 010-19 |
| 14. Default slice one packet at the fallback; in-band content over the cap omitted (C-137) | 010-23 |
| 15. Controller member counts towards the 32: at most 31 audio members with controllers, refusal without panic (C-209) | 010-04 |
| 16. Controller member costed as one packet per slice of headers + controller bytes + 66 bytes at 2 000 packets/s (C-219) | 010-24 |

### Definition of done

| Item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all |
| Every parser touched is fuzzed | 010-19, 010-20 |
| Regime stamp fields touched are complete and documented | 010-09, 010-12, 010-20, 010-21, 010-22, 010-23 |
| Design documents updated where behaviour differs, constants table first | 010-05, 010-10, 010-16, 010-17, 010-18, 010-20 |
| No new open item without an owner and a falsifier | 010-10, 010-13, 010-16, 010-18, 010-20, 010-23 |
| The frozen POC is not modified | all |
| Every new factual claim carries an evidence tag | 010-03, 010-05, 010-10, 010-18, 010-20 |
| No allocation and no lock anywhere on the send path | all code tickets, audited in 010-20 |
| Packets per slice and pacing mode are exposed for the regime stamp | 010-12, 010-20 |

The epic marks no checklist line as not applicable (C-125); its note explains why each line has a subject.

## Remaining gaps
Only what neither the epic nor the constants table answers. Each is recorded in the epic's open sub-questions with an owner and a falsifier; the tickets carry a provisional behaviour where they can.

1. **Header budget h within the 100-byte cap (C-137).** The cap is settled and a default 6-channel slice is one packet at the fallback; the exact h, and so the room left for announcements and the measurement extension at 1 280 bytes, comes from the wire specification. — 010-03, 010-05, 010-18, 010-23. Answer: epic 004.
2. **Ethernet terms in C-20** (VLAN tag counted or not). Effect under 0.5 %, inside the 5 % falsifier; the term sizes get `[standard]` citations (C-132). — 010-05. Answer: epic 004.
3. **Acceptable phase sharing among 64 ids.** The threshold behind "often enough to matter". — 010-10. Answer: measurement, L1 (task 019).
4. **Announcement edge rules.** Carried at the target or not; a re-issue falling on a gap slice; slice-length change while pending. Provisional answers in the ticket. — 010-13, 010-14. Answer: epic 004.
5. **Epoch-switch boundary reading** of AC-9. Provisional: boundary at T. — 010-16. Answer: epic 006 (switch procedure).
6. **Multi-packet placement** of the slice checksum and the measurement extension, and the content of "transport state" in packet 0. — 010-07, 010-09. Answer: epic 004.
7. **`no_std` CI target.** — 010-01. Answer: epic 026 (device).
8. **Spread plus phase versus queue drops and consumer margins.** — 010-10, 010-11, 010-20. Answer: measurement, L1 (task 019).

## Settled since the split
Former gaps (numbering of the previous "Gaps in the epic" list):

- 1 Header budget: the formula and the fallback payload are settled (C-04, C-100); the value of h stays open (remaining gap 1).
- 2 192 bytes per channel: settled by C-04 — 48 × 4 bytes, no per-channel id byte, channel map in the manifest.
- 3 IP version: settled by C-100 — IPv4 only, 28 bytes of IPv4 + UDP, 1 252 bytes of payload at the 1 280 fallback.
- 4 Ethernet overhead breakdown: not settled (remaining gap 2); the tag rule is now C-132.
- 5 Phase offset: AC-6 rewritten in the epic (deterministic, distinct for test ids, even spread over at most 64 modules, C-34); range and threshold stay open (remaining gaps 3 and 4).
- 6 `PaceSchedule<P>` size: settled by epic change — u16 count up to 256 packets (C-09), caller-chosen P, error above P, spread above 250 ‰ refused (C-14).
- 7 Announcement details: lead conversion settled (C-43 flat, slice length of the stream); pending-table capacity left to the ticket as an implementation choice; the edge rules stay open (remaining gap 5).
- 8 Relay API: settled by epic change — `relay_announcement` in the deliverables; target not shifted by L (AC-17 "unchanged").
- 9 Epoch switch: session source settled by C-19 (assigned by the source, bumped at the switch); `announce`/`switch_epoch` contract settled by epic change with C-43 flat; boundary reading stays open (remaining gap 6).
- 10 Bundle-level packetiser: settled by epic change — `BundlePacketiser` in the deliverables; sessions per member stream (C-19).
- 11 Undeclared dependency: settled by epic change — 009 added to the header and Dependencies.
- 12 Multi-packet placement: not settled (remaining gap 7).
- 13 Error and edge behaviour: settled by epic change — `finish` and `gap_slice` return `OutTooSmall`, `gap_slice` takes `send_ns`, gap slices are header-only and carry no announcements; the re-issue-on-gap question moved into remaining gap 5.
- 14 Regime stamp structure: settled by C-18 (sectioned control message owned elsewhere) and C-47 (producer counters); this epic exposes `SendRegime` and `SendCounters`.
- 15 `output_position` when p < L: settled by epic change — returns none; C-105 makes L rational, which opens remaining gaps 11 and 12.
- 16 `no_std` target: not settled (remaining gap 8).
- 17 Runtime gap emission: not settled (remaining gap 9).
- 18 Dense controllers: not settled (remaining gap 10); the fallback figure is now known from C-100.
- 19 Evidence tags: settled by epic change — the epic lists `[code]`, `[decision]`, `[measured]`, `[kernel-doc]`, `[standard]` (C-132) and `[memory]`.
- Epic open sub-question "is a belated gap slice worth its bytes?": settled — yes, marked gaps are used downstream (C-47, C-102, C-111).

Settled in this refinement (numbering of the previous "Remaining gaps" list):

- 1 Header budget: the cap is settled by C-137 (headers and checksum at most 100 bytes; a default slice is one packet at the fallback; announcements and the measurement extension fit or are omitted on a fallback path, new ticket 010-23); the exact h stays open (remaining gap 1).
- 3 Phase offset range: settled by C-155 — the phase offset is in [0, 75 %) of the period, so offset plus spread stay inside it (010-10, 010-11).
- 9 Runtime emission of gap slices on a missed deadline: settled by C-144 — the hosted backend (epic 015) emits it through `gap_slice` (010-08).
- 11 Rounding of a non-integer L: void — C-105 is revised, L is always a whole number of samples and the rational-L rule is withdrawn. 010-02 drops its rational type and error path and returns to XS; no ticket is deleted.
- 12 Sign convention of L: settled by C-122 — stamps denote content position; the output emitted while processing p is stamped p − L, carries content from input p − L, and is degraded if that input was gapped (010-02, 010-06).
- Regime stamp ownership: settled by C-144 — epic 013 defines the message, epic 015 sends it (010-12, 010-21, 010-22).

Settled in the latest refinement (C-181, C-137; numbering of the previous "Remaining gaps" list, which had 9 entries; former gap 9 is now gap 8):

- 8 Dense controllers on a single-stream source: settled by C-181 — controllers share the audio packet while headers plus controllers stay within the 100-byte cap (C-137), at every path MTU; beyond that the source emits a second stream (a 2-member bundle, 010-04). No byte-rate threshold is needed; the compiler checks the case from the manifest. 010-18 no longer waits on the design owner: its Work and Acceptance criteria now implement both layouts; 010-23 notes that controllers count against the cap before the optional items. No ticket added; effort unchanged.
- C-137 revised wording ("must fit"): no change of substance for this epic; the epic's constants row now matches the table.

Settled in this refinement (C-205, C-181 revised, C-199, C-200; no entry of the "Remaining gaps" list is closed, the list keeps its 8 entries):

- C-205, and C-181 revised to cite it: a bundle source's controllers always ride in their own member stream. The epic already put them in their own member, but 010-04 made the controller member optional and 010-09 placed controllers in packet 0 of any multi-packet slice. 010-04 now makes the controller member mandatory for a bundle source, with no controller-size test, and gains acceptance criterion 7; 010-09 limits controllers in packet 0 to the shared case of a single-stream source; 010-18 and 010-20 cite C-205. The epic's constants table now carries the revised C-181 row and a C-205 row. No ticket added; effort unchanged.
- C-199 (default tempo 120 beats per minute) and C-200 (playhead does not advance through a gap while stopped): no subject in this send-side epic. They govern the transport state a consumer holds before any arrives and through a gap (C-184, C-141); the packetiser carries transport state in packet 0 without interpreting it, and gap slices are header-only. The content of "transport state" stays with epic 004 (remaining gap 6).

Settled in the final sweep (C-177 to C-212 and the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184; no entry of the "Remaining gaps" list is closed, the list keeps its 8 entries, none owned by the design owner):

- C-08 revised and C-209: the controller member counts towards the 32 members of a bundle, so a source with controllers has at most 31 audio members (186 channels). 010-04 now refuses to append a controller member to a layout with 32 audio members and gains acceptance criterion 8; 010-01 carries the revised C-08 row; the epic's constants table gains C-209 and the revised C-08, its Bundles section states the rule and its acceptance criteria gain 15 (covered by 010-04). The compile-time rejection of a wider source stays with the compiler.
- C-134 revised and C-207: an edge's resampler delay is counted only in D, never in L. 010-02 states it in its context and out-of-scope lines; the epic's C-105 row cites C-207 and the table gains a C-207 row. No change to `output_position`.
- C-208 (with C-186): a re-alignment after a changed A is announced afresh by each fan-in, one lead ahead, not relayed. 010-15 states that `relay_announcement` is not used for a module's own re-alignment; the epic's announcement section and constants table say the same. No new code: `announce` (010-13) already does it.
- 010-20 lists C-207, C-208 and C-209 among the constants to reconcile.
- C-181 (late revision): already applied; the epic and 010-18 match the table. C-141, C-184 (and C-199, C-200): no subject, the packetiser carries transport state without interpreting it and gap slices are header-only. C-153, C-169: no subject.
- No subject in this send-side epic: C-177 and C-201 (seqlock and pinning; the send path has no lock), C-178 to C-180, C-182, C-183, C-185 (refusal reasons are admission, epic 012), C-187 to C-191, C-193 to C-195, C-196 (this is not a specification epic), C-197, C-198 (the receiver report is not built here), C-202 to C-204, C-210 to C-212. C-192 is already met (010-19 uses the project's fuzz harness, the tool being the team's choice; 010-06 loads the epic 007 vectors wherever the vectors crate sits). C-206: the file name of the 010-10 phase evaluation report within `doc/design/reports/` is this epic's choice; no remaining gap depends on it.
- No ticket added or deleted; effort unchanged.

Settled after the ticket split (C-181 revised, C-215, C-219; no entry of the "Remaining gaps" list is closed, the list keeps its 8 entries, none owned by the design owner):

- C-181 revised (permissive): a single-stream source may always put its controllers in a second stream, must do so beyond the 100-byte cap (C-137), and may share the audio packet while they fit. 010-18 had implemented sharing as automatic whenever it fit; its constructor now takes the placement declared in the manifest, always builds the 2-member layout for a second stream, and refuses a shared placement beyond the cap without switching silently (Work and acceptance criteria 2, 3 and 5 corrected). 010-23 notes that sharing is optional. The epic's constants row and Bundles section match the table.
- C-215: a ratchet step is a re-alignment announced one lead (C-43) ahead in the fan-in's own output. The epic already treated a ratchet-triggered re-alignment this way through `announce`; the epic's announcement section and 010-15 now cite C-215. No new code.
- C-219: the wire cost of a controller stream is one packet per slice of headers (at most the 100-byte cap) + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead, at 2 000 packets/s, the figure admission and the commissioning tool use. 010-05 had left controller-member traffic out of scope; new ticket 010-24 (XS) adds it to `wire_bits_per_second` and `packets_per_second`, and 010-04 records the controller bytes per slice on the controller member. The 66 bytes equal the 28 bytes of C-100 plus the 38 Ethernet bytes 010-05 counts; remaining gap 2 (C-20 terms and their citations) stays with epic 004. The epic gains acceptance criterion 16 and the C-215 and C-219 rows; its scope stays L (24 tickets, about 23 days).
- 010-20 lists C-215 and C-219 among the constants to reconcile.

Other rulings applied: C-01 and C-80 (single wire rate; the rate parameter of `wire_bits_per_second` dropped), C-95 (fuzz budget, 010-19), C-116 (probe size, 010-03), C-120 (010-22), C-122 (degraded flag and sign convention, 010-02, 010-06), C-125 (DoD applicability note in the epic), C-126 (scope), C-130 and C-171 (output locations, 010-10, 010-12, 010-20, 010-21), C-134 (resampler only in edge modules; noted in 010-02), C-146 ("unprivileged" means per-change CI; noted in 010-20), C-158 (initial value of a dense controller only; placement now settled by C-181; noted in 010-18), C-159 (counter list; 010-21), C-165 (pacing override is operator policy; 010-11, 010-12). Rulings with no subject in this send-side epic: C-35 (owner change), C-45, C-46, C-94, C-96, C-101 to C-104, C-106 to C-115, C-117 to C-119, C-121, C-123, C-124, C-127 to C-129, C-131, C-133 (already met: all code in `hadronyche-core`), C-135, C-136, C-138 to C-143, C-145, C-147 to C-154, C-156, C-157, C-160 to C-164, C-166 to C-170, C-172 to C-176. C-181 is applied above (010-18, 010-23, 010-20); C-205 is applied above (010-04, 010-09, 010-18, 010-20); C-199 and C-200 have no subject here.
