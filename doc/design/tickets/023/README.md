# Epic 023 — DSL compiler front end: tickets

## Summary
Epic 023 (phase P5, scope L — 32 tickets, about 38.75 days; depends on epics 004, 012, 013 and 024a; blocks 024 except its kit part 024a, and 025) builds the front half of the module-construction DSL compiler, crate `hadronyche-dsl`: the language surface (syntax decided here), lexer and parser with spans and coded diagnostics, type and rate checking over the v1 content domains (audio f32 / Q1.31, dense controllers, sparse keyed controllers, read-only transport state) with a single wire rate of 96 kHz and rate conversion only inside edge modules, at 2:1 or 1:2 (C-01, C-80, C-105, C-134), lowering into a dataflow IR built from a closed node set with no loops, recursion, allocation or out-of-kit calls, join padding with whole-sample delay lines (C-104), gap defaults including transport state (C-141, C-184, C-199, C-200) and dense-controller initial values (C-158), controller placement of a single-stream source and of a bundle source, whose controller member counts towards the 32-member cap (C-181, C-205, C-209), the kit's fixed 2:1 / 1:2 resampler node modelled from epic 024a (C-189, C-197), rejection of even-tap linear-phase FIRs (C-183) and of unpaddable device-side joins (C-190), and the static analyses the runtime relies on — structural latency L as a whole number of domain-rate samples, with an edge's resampler delay counted only in D, never in L, and reported separately (C-105, C-134, C-207), a cost bound in abstract operations per slice (C-106), state size and worst-case stack, determinism class (exact-fixed / exact-float; bounded never for DSL programs, C-123) and per-output input dependency sets for degraded marking (C-122). It ends in `check(source) -> Result<(Ir, Manifest), Vec<Diagnostic>>`, a manifest with a binary form and a readable dump, a declaration path for hand-written modules, and `LANGUAGE.md`. Code generation (024) and fingerprint evaluation / conformance (025) are out of scope. Epic acceptance criteria are numbered 023-AC-1 to 023-AC-9.

## Tickets

| Ticket | Title | Effort | Depends on |
|---|---|---|---|
| 023-01 | Concrete syntax decision | S | none |
| 023-02 | Crate skeleton, source spans and diagnostics | XS | 023-01 |
| 023-03 | Lexer | S | 023-01, 023-02 |
| 023-04 | AST and parser | M | 023-03 |
| 023-05 | Parser fuzz target | S | 023-04 |
| 023-06 | Content types, rates and channel counts model | S | 023-02, epic 004 |
| 023-07 | Bundle layout and channel map | S | 023-06 |
| 023-08 | Type and rate checker | M | 023-04, 023-06, 023-07 |
| 023-09 | IR: dataflow graph with the closed node set | S | 023-06 |
| 023-10 | Lowering AST to IR, with kit rejections | M | 023-08, 023-09 |
| 023-11 | 2:1 / 1:2 resampler node and its whole-sample delay | XS | 023-09, epic 024a |
| 023-12 | Gap-handling defaults and overrides | S | 023-10 |
| 023-13 | Analysis test corpus with hand-derived values | S | 023-10, 023-11 |
| 023-14 | Dataflow analysis for degraded marking | S | 023-10 |
| 023-15 | Structural latency analysis | M | 023-11, 023-13 |
| 023-16 | Cost bound analysis | S | 023-11, 023-13, 023-28 |
| 023-17 | State size and worst-case stack analysis | S | 023-11, 023-12, 023-28 |
| 023-18 | Determinism class assignment | S | 023-10, 023-11, epic 013 |
| 023-19 | Manifest type and human-readable dump | S | 023-07, 023-12, 023-14, 023-15, 023-16, 023-17, 023-18, 023-28, 023-29 |
| 023-20 | Admission figures | S | 023-07, epic 012 |
| 023-21 | Manifest binary form and round trip | M | 023-19, 023-20, epic 004, epic 013 |
| 023-22 | Manifest decoder fuzz target | XS | 023-05, 023-21 |
| 023-23 | `check()` pipeline and 023-AC-1 rejection suite | S | 023-05, 023-10, 023-19, 023-20, 023-28, 023-29, 023-31, 023-32 |
| 023-24 | Expressiveness: reference echo instrument and metronome type-check | S | 023-23, 023-30 |
| 023-25 | Manifest declaration for hand-written (non-DSL) modules | S | 023-20, 023-21, epic 012 |
| 023-26 | `LANGUAGE.md` language reference and node table | M | 023-11, 023-12, 023-15, 023-16, 023-18, 023-24, 023-28, 023-29, 023-30 |
| 023-27 | Design documents update and epic close-out | XS | 023-22, 023-24, 023-25, 023-26 |
| 023-28 | Join padding pass and per-join opt-out (new, C-104) | M | 023-11, 023-13, 023-15 |
| 023-29 | Edge modules, device-facing ports and the single wire rate (new, C-01, C-105, C-134) | S | 023-08 |
| 023-30 | Transport-state gap default (new, C-141) | S | 023-12 |
| 023-31 | Controller placement of a single-stream source (new, C-181) | S | 023-19, 023-20, epic 004 |
| 023-32 | Controller member stream of a bundle source (new, C-205; 31-audio-member limit, C-209) | XS | 023-07, 023-20, 023-31 |

No ticket was deleted: no ticket existed only for the withdrawn rational-L rule. Its traces were removed where they appeared: 023-11 lost the 147:320 ratio, the rational delay and the "any ratio" proof and dropped from M to S (C-105); 023-06 lost rational rates; 023-15 lost rational arithmetic; 023-28 lost the fractional-difference diagnostic; 023-13, 023-19, 023-21, 023-25 and 023-26 now carry L as whole samples; 023-29 now rejects 44.1 kHz instead of accepting it. Earlier drops: the synthetic non-deterministic node test of 023-18 (C-123), the device-rate constant of 023-06 (C-01); the provisional cost unit of 023-16 became the decided one (C-106). New ticket 023-30 implements the transport-state gap default (C-141); the dense-controller initial value (C-158) was added to 023-12 within its S effort. Latest round: new ticket 023-31 implements the controller placement check (C-181); C-183 (even-tap FIR rejection) was added to 023-10 and 023-23, C-184 (transport state before any has arrived) to 023-30, and C-190 (device-side join) corrected in 023-28, all within their efforts. Final round: new ticket 023-32 implements the controller member stream of a bundle source (C-205); C-199 (default tempo) and C-200 (stopped playhead) were applied to 023-30 — C-200 corrects its provisional "advance by the positions elapsed" reading with a stopped-transport test — and to 023-01, 023-12, 023-24 and 023-26; C-207 is cited where 023-11, 023-13, 023-15, 023-19, 023-26 and 023-29 keep the conversion delay out of L, all within their efforts. Closing round: C-209 adds the rejection of a source with controllers wider than 31 audio members to 023-32 (within its XS effort) and to 023-23, 023-25 and 023-26; C-197 moves the resampler's design and delay to the kit epic 024a, so 023-11 only models the kit's node and drops from S to XS. Last round: revised C-181 (permissive) corrects 023-31 — within the cap the second stream is the source's choice, not excluded — and adds the opt-in construct to 023-01 and 023-26; C-218 is cited in 023-23 and 023-27; all within their efforts, no new ticket.

## Effort tally
- XS: 5 (1.25 days)
- S: 20 (20 days)
- M: 7 (17.5 days)
- Total: 32 tickets, about 38.75 days → scope L (C-126); below the 40-day split threshold of C-143

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| Crate `hadronyche-dsl` | 023-02 |
| `syntax/` — lexer, parser, AST, source spans, coded diagnostics | 023-01, 023-02, 023-03, 023-04, 023-05 |
| `types/` — content types, rates, channel counts, bundle layouts; checking incl. single wire rate and edge modules | 023-06, 023-07, 023-08, 023-29 |
| `ir/` — dataflow graph IR, closed node set, no unbounded loops, no recursion; join padding; gap defaults | 023-09, 023-10, 023-11, 023-12, 023-28, 023-30 |
| `analysis/latency.rs` (whole-sample L, join path latencies, edge conversion delays) | 023-13, 023-15 |
| `analysis/cost.rs` (abstract operations) | 023-13, 023-16 |
| `analysis/memory.rs` | 023-17 |
| `analysis/determinism.rs` | 023-18 |
| `analysis/dataflow.rs` | 023-14 |
| `manifest.rs` — type, binary serialisation, readable dump, hand-written declaration | 023-19, 023-20, 023-21, 023-22, 023-25 |
| `LANGUAGE.md` incl. diagnostic codes | 023-26 |
| `check(source) -> Result<(Ir, Manifest), Vec<Diagnostic>>` | 023-02 (stub), 023-23 |
| Open sub-question: concrete syntax (owner: this epic) | 023-01, verified by 023-24 |
| Open sub-question: hand-written module manifests (owner: this epic with 012) | 023-25 |
| Open sub-question: edge-module device-facing side (owner: this epic) | 023-01, 023-29 |
| Settled rule: transport-state gap default; a stopped playhead does not advance (C-141, C-200) | 023-30, 023-24, 023-26 |
| Settled rule: dense-controller initial value in the manifest (C-158) | 023-01, 023-12, 023-19, 023-21 |
| Settled rule: frequency-dependent join path padded on its structural part and flagged (C-104) | 023-13, 023-15, 023-28, 023-19 |
| Settled rule: resampler only in edge modules, 2:1 / 1:2 only, conversion delay only in D, never in L, reported on the device-facing port (C-105, C-134, C-207) | 023-11, 023-15, 023-29, 023-23 |
| Settled rule: a single-stream source may always put its controllers in a second stream and must beyond the 100-byte cap; within it, it may share its packet (C-181, revised) | 023-31, 023-01, 023-20, 023-23, 023-26 |
| Settled rule: every compile-time rejection is implemented in this epic; 024b only asserts, 025a tests (C-218) | 023-23, 023-27 |
| Settled rule: controllers of a bundle source always in their own member stream (C-205, C-181) | 023-32, 023-20, 023-23 |
| Settled rule: the controller member counts towards the 32 of C-08; a source with controllers has at most 31 audio members (186 channels), wider rejected (C-209) | 023-32, 023-07, 023-08, 023-23, 023-25, 023-26 |
| Settled rule: the fixed 2:1 / 1:2 resampler node is built by the kit epic 024a and modelled here; the rate actuator is not a DSL node (C-197, C-189) | 023-11, 023-09 |
| Settled rule: even-tap linear-phase FIR rejected, odd-tap design suggested (C-183) | 023-10, 023-15, 023-23, 023-26 |
| Settled rule: transport state before any has arrived — stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199) | 023-30, 023-26 |
| Settled rule: device-side join with an odd domain-rate difference rejected, opt-out suggested (C-190) | 023-28, 023-23, 023-26 |
| Definition of done (all items; none not applicable, C-125) | per-ticket DoD; consolidated in 023-27 |

### Acceptance criteria

| Epic criterion | Tickets |
|---|---|
| 023-AC-1 Rejections (loop/recursion, out-of-kit call, allocation, cross-rate without resampler, network port not at 96 kHz, resampler outside an edge module, resampler ratio other than 2:1 / 1:2, transport authoring, even-tap linear-phase FIR (C-183), unpaddable device-side join (C-190), port > 192 channels or bad bundle layout, source with controllers wider than 31 audio members (C-209)) | 023-04, 023-07, 023-08, 023-10, 023-28, 023-29, 023-32, 023-23 |
| 023-AC-2 Latency as whole domain-rate samples over ≥ 20-program corpus; edge conversion delay reported outside L; recursive filter sets `non_constant_delay` | 023-11, 023-13, 023-15 |
| 023-AC-3 Join padding, opt-out and frequency-dependent flag (C-104) | 023-13, 023-15, 023-28, 023-19 |
| 023-AC-4 Cost bound finite and monotone, abstract operations (C-106) | 023-13, 023-16 |
| 023-AC-5 Determinism class; never bounded for DSL; stable under reformatting | 023-03, 023-09, 023-10, 023-18 |
| 023-AC-6 Dataflow sets {1} and {1, 2} | 023-14 |
| 023-AC-7 Manifest round trip; 96 ch → 16-member bundle with channel map; ~21.1 Mbit/s and 2 000 packets/s; 32 000 packets/s and ~338 Mbit/s for 96 ch; second controller stream beyond the header cap, or by the source's choice within it (C-181); controller member stream of a bundle source (C-205) | 023-07, 023-19, 023-20, 023-21, 023-31, 023-32 |
| 023-AC-8 Reference echo instrument and metronome type-check | 023-01, 023-24, 023-30 |
| 023-AC-9 Parser and manifest decoder fuzzed at the C-95 budget | 023-05, 023-22 |

## Remaining gaps

Only what neither the epic nor the constants table answers. None needs the design owner.

1. **Manifest binary layout** — field order, widths, endianness, width of L (now a whole number, C-105). Affects 023-21, 023-22, 023-25. Owner: the wire specification epic 004, with epic 013 for the stamp-related fields.
2. **Per-packet header breakdown and admission units for input count and buffer memory** — C-20 fixes ~1 320 bytes per packet and C-137 caps all headers and the checksum at 100 bytes; the field breakdown inside that cap and the units are not settled, and buffer size depends on A (C-96), unknown at compile time. Affects 023-20 and 023-31 (headers plus controllers against the cap, beyond which C-181 requires a second stream). Owner: epic 012 (admission) with epic 004.
3. **Bundle members wider than 6 channels** — whether a non-default slice length or the multi-packet regime (C-05) lets a member carry more than 6 channels and a port exceed 192. Affects 023-07, 023-08, 023-25. Owner: the wire specification epic 004.
4. **Stack model** — how the IR live-value bound relates to the generated code's stack. Affects 023-17. Owner: the code generator epic 024.
5. **`abi_version` value and versioning policy.** Affects 023-19, 023-21, 023-25. Owner: epic 012 (manifest exchange).
6. **Verification of declared cost, L, state and stack for hand-written modules.** Affects 023-25. Owner: epic 025 (conformance) with epic 012.
7. **Delay of the 2:1 / 1:2 resampler design** — fixed by the kit epic 024a (C-197) and imported by 023-11; confirmed by measurement. Owner: epic 025's probe suite (falsifier: the probe disagrees with the compiler's conversion delay).

Open sub-questions this epic answers itself (work, not gaps): concrete syntax (023-01, falsified by 023-24); hand-written module manifests (023-25, with epic 012); edge-module device-facing side in the DSL and the manifest (023-01, 023-29).

## Settled since the split

Last round (C-181 revised, C-218):
- Controllers of a single-stream source — revised C-181 (permissive): the source may always put its controllers in a second stream, and must when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. Corrects the reading of 023-31 that computed the placement from the byte sum alone: 023-31 now honours an opt-in second stream within the cap (Context, Work 1 and 3, new acceptance criterion 4); 023-01 adds the opt-in construct to the grammar outline; 023-20 and 023-26 restate the rule; epic 023-AC-7 clause updated. Within the S effort of 023-31.
- Who implements a compile-time rejection — C-218: this epic implements every one; the code generator 024b only asserts it never receives such a program; the conformance corpus 025a tests it. Confirms what 023-23 already did (every 023-AC-1 case rejected by `check()`); 023-23, 023-27 and the epic cite it. No work change.

Closing round (C-209, C-197, C-189, C-192, C-206; C-08 revised):
- Former gap 8, controller member stream and the 32-member cap — C-209 (C-08 revised): the controller member counts towards the 32; a source with controllers has at most 31 audio members (186 channels) and a wider one is rejected at compile time. 023-32 gains the rejection (Work 4, acceptance criteria 4 and 5) and loses its "do not add a rejection" instruction; new 023-AC-1 case (l) in 023-23; 023-06, 023-07, 023-08, 023-20, 023-25 and 023-26 cite it.
- Who builds the resampler — C-197: the kit epic 024a builds the fixed 2:1 / 1:2 node, ahead of this epic. 023-11 no longer specifies a filter; it imports the kit's delay, cost and state (S → XS); the epic now depends on 024a. C-189 and C-210: the edge's rate actuator is not a DSL node (023-09, 023-11). Gap 7 now names 024a as the source of the delay.
- C-192: float and fixed corpus variants count as one program (023-13); the fuzzing tool is the team's choice (023-05). C-206: the readable dump's format is this epic's presentation detail (023-19).
- C-177 to C-180, C-182, C-185 to C-188, C-191, C-193 to C-196, C-198, C-201 to C-204, C-208, C-211, C-212, and late revisions C-153 and C-169, have no subject here; revised C-134, C-141, C-181 and C-184 were already applied.

Final round (C-199, C-200, C-205, C-207, C-181 revised):
- Former gap 9, value of the default tempo — C-199: 120 beats per minute (C-184 now cites it). 023-30 replaces its placeholder with that constant; 023-26 states it.
- Playhead through a gap while stopped — C-200: does not advance (see C-141). Corrects the provisional reading "advance the playhead by the positions elapsed" in 023-30 (Work 2, new acceptance criterion 6), 023-01, 023-12, 023-24 and 023-26.
- Former gap 8, controllers of a multi-stream source — C-205 and revised C-181: a bundle source's controllers always ride in their own member stream. New ticket 023-32 (XS); 023-20, 023-23 and 023-31 point to it; new 023-AC-7 clause. Whether that member counts toward C-08 was gap 8, closed by C-209 in the closing round.
- Resampler delay and L — C-207: counted only in D, never in L; L is in the domain-rate timeline; the conversion delay is reported separately on the device-facing port. Confirms what 023-11, 023-13, 023-15, 023-19, 023-26 and 023-29 already did under C-134; they now cite C-207. No work change.

Latest round (C-181, C-183, C-184, C-190):
- Controllers of a single-stream source — C-181: they share the audio packet while headers plus controllers stay within the 100-byte cap (C-137), else a second stream; the compiler checks it from the manifest. New ticket 023-31; 023-20 no longer counts such a controller port as a stream of its own; new 023-AC-7 clause.
- Even-tap linear-phase FIR — C-183: rejected in v1 with a diagnostic suggesting an odd-tap design, so L stays exact. New 023-AC-1 rejection (023-10, 023-23, 023-26; 023-15 notes it).
- Transport state before any has arrived — C-184: stopped, playhead at zero, default tempo (023-30, 023-26); the tempo's value, then gap 9, is now C-199.
- Device-side join — C-190 settles the provisional diagnostic of 023-28: the compiler rejects the join and suggests the per-join opt-out (the alternative suggestion of moving the delay to the 96 kHz side is dropped). New 023-AC-1 rejection (023-28, 023-23, 023-26).

Previous round (C-104 and C-105 revised, C-127 to C-176):
- Former gap 9, padding a fractional latency difference at a join — closed: L is always a whole number of domain-rate samples (C-105, rational-L rule withdrawn) and padding uses a whole-sample delay line (C-104). 023-28 keeps only a diagnostic for an odd domain-rate difference at a 48 kHz device-side join.
- Former gap 5, gap default for read-only transport state — C-141: hold the last known state, advance the playhead by the positions elapsed, mark the output degraded; new ticket 023-30; the override requirement in 023-12, 023-24 and 023-26 is removed.
- Resampler ratios — C-105: only 2:1 and 1:2, other ratios out of v1 (new 023-AC-1 rejection, 023-29, 023-23); C-134: resampler only inside an edge module, and its conversion delay is part of D (defined by C-170), not of L (023-11, 023-15, 023-19, 023-29).
- Frequency-dependent path at a join — C-104: padded on its structural part only, join flagged in the manifest (023-13, 023-15, 023-19, 023-21, 023-28).
- Dense controller with no prior value — C-158: initial value declared in the manifest, default 0 (023-01, 023-04, 023-12, 023-19, 023-21, 023-25).
- Calibration factor — C-176: measured by epic 025 with a reference workload per listed target (023-16).
- Header cap — C-137 narrows gap 2 (023-20).
- C-146: "unprivileged" applies to per-change CI; this epic has no privileged test (023-27). C-143: 38 days is below the 40-day split threshold. C-35, C-144 and the other new rules have no subject here.

Earlier rounds:
- Former gap 2, manifest shape vs contents mismatch — epic change: `Manifest` now carries slice frames, per-output cost, join padding and gap overrides.
- Former gap 5 (old list), "source-rate samples" ambiguous — C-105: L is in domain-rate samples (now always whole).
- Former gap 6, latency join rule — C-104: pad the shorter path, report it, L is the longest path, per-join opt-out (ticket 023-28).
- Former gap 7, resampler delay and non-integer ratios — now C-105 (2:1 and 1:2 only, whole-sample delay) and C-80 (2:1 at 48 kHz edges).
- Former gap 8, slice size at rates other than 96 kHz — C-01 (single wire rate), C-02, C-80: wire slices are 48 frames; a 48 kHz device side has 24 frames (derived).
- Former gap 9 (old list), "outside the kit ⇒ bounded" vs rejection — C-123: no non-deterministic DSL node; bounded only from a fingerprint mismatch or for hand-written modules.
- Former gap 11, cost unit owned by epic 025 — C-106: abstract operations per slice; per-target calibration factor in the regime stamp.
- Former gap 13, fuzz budget — C-95: 60 s per change, 1 hour nightly, corpus kept.
- Former gap 16, diagnostic format and codes — epic change: span, stable code and rule-naming message; codes defined in 023-02 and listed in `LANGUAGE.md`.
- Narrowed, not closed: former gap 3 (C-04, C-20 and now C-137; breakdown and units remain, gap 2 above); former gap 15 (C-123 fixes the hand-written default class; verification remains, gap 6 above); former gap 4 (members capped at 6 channels, ports at 192; wider members remain, gap 3 above).
- Also applied: C-04, C-20/C-21/C-22, C-100, C-122, C-125, C-126, C-130 and C-133, C-132; criteria carry ids 023-AC-1 to 023-AC-9.
