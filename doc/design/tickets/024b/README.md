# Epic 024b — DSL code generator: tickets

## Summary
Epic 024 was split into two parts by rule C-143 (it exceeded 40 days by ticket tally). Part 024b (phase P5, scope L; depends on 024a, epics 004, 009, 010, 013, 015, 023; blocks 025, 026) builds the code generator in `hadronyche-dsl::codegen` and the `hadronyche-dsl build` command. It turns checked IR into Rust source implementing the `Module` trait of part 024a, with: whole-sample output latency and the edge conversion delay reported as part of D; join padding with whole-sample delay lines and frequency-dependent-join flags, and odd-difference device-side joins in edge modules asserted absent (the front end rejects them, C-218); the "frequency-dependent delay" output property; gap defaults (dense initial values, transport hold with playhead advance as the held state dictates, the before-first-state transport at 120 beats per minute); degraded marking that follows the content; an operation-counting build; and, for exact-float modules, a determinism fingerprint evaluated on the processing thread whose result sets the regime stamp field; the build command records controller placement in the manifest and asserts that no program breaking the placement rules or the bundle cap reaches it; every compile-time rejection is the front end's (epic 023, C-218). It depends on part 024a as a whole.

Acceptance criteria are numbered 024b/AC-1 … 024b/AC-10 in the epic and in the tickets; AC-32 and AC-33 are project-level criteria the part contributes to.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 024b-01 | Decide output form: Rust source versus object code | S | 024a-08 |
| 024b-02 | Decide subnormal policy for exact-float | S | 024a-08 |
| 024b-03 | Codegen: emit module scaffold and `INFO` from manifest | M | 024a-01, 024b-01, epic 023 |
| 024b-04 | Codegen: lower IR operations to kit calls (defines the shared corpus) | M | 024a-07, 024a-08, 024a-09, 024a-10, 024a-12, 024b-02, 024b-03 |
| 024b-05 | Codegen: whole-sample output latency and edge conversion delay | S | 024a-12, 024b-04, epic 023 |
| 024b-06 | Codegen: join padding delay lines | M | 024b-04, 024b-05, epic 023 |
| 024b-07 | Codegen: default gap concealment | S | 024b-03, 024b-04, epic 009, epic 010, epic 023 |
| 024b-08 | Codegen: transport-state gap default (new, C-141) | S | 024b-07, 024b-10, epic 010 |
| 024b-09 | Codegen: DSL gap overrides | S | 024b-07, epic 023 |
| 024b-10 | Codegen: degraded marking from dataflow | M | 024b-03, 024b-05, epic 023 |
| 024b-11 | Codegen: fingerprint vector synthesis | M | 024a-03, 024b-02, 024b-04 |
| 024b-12 | `hadronyche-dsl build` command producing module, manifest and vectors (now also controller placement, C-181, C-218) | S | 024b-03 … 024b-11, 024b-21, epic 004, epic 010, epic 023 |
| 024b-13 | Codegen: operation-counting build for the cost bound | S | 024b-04, 024b-12, epic 023 |
| 024b-14 | Fuzz manifest and vectors decoders | S | 024b-12 |
| 024b-15 | Generated modules: no_std, no-alloc, no-libm check | S | 024a-05, 024a-06, 024b-12 |
| 024b-16 | Stack bound emission and measurement | M | 024a-06, 024b-12 |
| 024b-17 | Fingerprint on processing thread and bounded downgrade | M | 024a-03, 024b-11, 024b-12, epic 009, epic 013, epic 015 |
| 024b-18 | Cross-target bit identity of generated programs | M | 024a-06, 024b-12, 024b-17 |
| 024b-19 | Six-channel gain module cost-bound benchmark | S | 024b-12, 024b-13 |
| 024b-20 | Design documents update and open items (part b) | S | 024b-01, 024b-02, 024b-06, 024b-08, 024b-16, 024b-17, 024b-19, 024b-21 |
| 024b-21 | Codegen: frequency-dependent-delay property on outputs (new, C-189) | XS | 024a-01, 024b-03, 024b-05, epic 023 |
| ~~024b-22~~ | Deleted (C-218): its whole purpose was a check duplicating the front end's rejections; placement recording and the assertion moved to 024b-12 | — | — |

**Renumbering from epic 024.** 024-15 → 024b-01, 024-16 → 024b-02, 024-17 → 024b-03, 024-18 → 024b-04, 024-19 → 024b-05, 024-32 → 024b-06, 024-20 → 024b-07, (new) → 024b-08, 024-21 → 024b-09, 024-22 → 024b-10, 024-23 → 024b-11, 024-24 → 024b-12, 024-33 → 024b-13, 024-25 → 024b-14, 024-26 → 024b-15, 024-27 → 024b-16, 024-28 → 024b-17, 024-29 → 024b-18, 024-30 → 024b-19; 024-31 was split into 024b-20 (this part's share) and 024a-15.

**Changes in this refinement.** 024b-05 (former 024-19) was rewritten and renamed: L is a whole number of samples (C-105), the edge conversion delay is reported as part of D and not added to L (C-134), and a misplaced resampler is rejected; the 147:160 and series-resampler cases and `non_constant_delay` are gone. 024b-06 (former 024-32) no longer rejects a non-integral join and now pads frequency-dependent paths on their structural part only, flagging the join (C-104). 024b-07 gained dense initial values (C-158). 024b-08 is new (C-141). 024b-17 now names epic 013 for the stamp field and epic 015 as sender (C-144). 024b-19 follows the C-176 calibration procedure. No ticket was deleted: none existed only for the withdrawn rational-L rule.

**Changes after rules C-184, C-189, C-190.** 024b-08: a transport input gapped before any state has arrived now reads stopped, playhead at zero, default tempo (C-184) instead of the provisional `None`. 024b-05: the DSL resampler node is fixed-ratio, and the edge's steerable rate actuator is separate, not emitted, its delay part of D (C-189). 024b-06: a device-side join of an edge module whose paths differ by an odd number of domain-rate samples is rejected with a diagnostic suggesting the opt-out (C-190). 024b-21 is new: the "frequency-dependent delay" output property (C-189). 024b-12 and 024b-20 follow.

**Changes after rules C-199, C-200, C-207.** 024b-08: the default tempo is 120 beats per minute (C-199), no longer taken from `TransportState`; the provisional "playhead advanced by the positions elapsed" for every held state is corrected: a stopped transport's playhead does not advance through a gap, a playing one moves on at its tempo (C-141, C-200); Work and acceptance criteria updated, with a new criterion for a gap after a stopped state. 024b-05 and 024b-12: the edge conversion delay is reported in the manifest separately, on the device-facing port, counted only in D and never in L (C-207). 024b-20 follows. No ticket added; the tally is unchanged.

**Changes after rules C-181, C-183, C-192, C-205, C-209 (and C-08 revised).** 024b-22 is new: the build command checks from the manifest that a bundle source's controllers ride in their own member stream, counted towards the 32 members (at most 31 audio members, a wider source rejected at compile time), and that a single-stream source's controllers share the audio packet within the 100-byte cap or move to a second stream; epic criterion 024b/AC-10 added, epic 004 added as a dependency for header sizes. 024b-05: defensive rejection of an even-tap linear-phase FIR with a diagnostic suggesting an odd-tap design (C-183). 024b-04 and the epic: float and fixed variants of a corpus program count as one program and the mix's volume is the reference dense controller (C-192), bundles of 32 members include the controller member (C-209); 024b-07 tests dense defaults on that volume; 024b-14 leaves the fuzzing tool to the implementing team (C-192). 024b-12 and 024b-20 follow. No ticket deleted.

**Changes after rules C-218 and C-181 (revised).** The front end (epic 023) implements every compile-time rejection; the code generator only asserts it never receives such a program, and the conformance corpus (epic 025a) tests it (C-218). 024b-22 is deleted: its whole purpose was the controller-placement and bundle-cap check, a duplicate of the front end's rejections; recording the placement in the manifest and the assertion moved to 024b-12 (new criteria 6 and 7, epics 004 and 010 added as dependencies). 024b-05: the defensive rejections of a misplaced resampler and of an even-tap linear-phase FIR become an assertion. 024b-06: the device-side odd-difference join rejection (C-190) becomes an assertion. A single-stream source may always put its controllers in a second stream and must above the 100-byte cap (C-181 revised, permissive); the former reading that controllers which fit must share the audio packet is corrected (024b-12, epic criterion 024b/AC-10). 024b-04 and 024b-20 follow. One ticket fewer: 21 tickets, about 32 days, scope L unchanged.

## Effort tally
- XS: 1
- S: 12
- M: 8
- Total: 21 tickets, about 32 days (XS 0.25 + S 12 + M 20 = 32.25); scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `codegen/emit.rs` — IR → Rust source implementing `Module`, `INFO` from manifest; lowering to kit calls | 024b-03, 024b-04 |
| Output latency (whole samples), edge conversion delay on the device-facing port (C-207), misplaced-resampler assertion (C-218) | 024b-05 |
| "Frequency-dependent delay" property on outputs (C-189) | 024b-21 |
| `codegen/joins.rs` — whole-sample padding, inserted delays, frequency-dependent-join flags, device-side odd-difference assertion (C-190, C-218) | 024b-06 |
| `codegen/gaps.rs` — default concealment (incl. dense initial values), transport hold and before-first-state transport, overrides | 024b-07, 024b-08, 024b-09 |
| `codegen/degraded.rs` — per-output flag, delayed by L | 024b-10 |
| `codegen/fingerprint.rs` — vector synthesis for exact-float | 024b-11 |
| Operation-counting build | 024b-13 |
| Even-tap linear-phase FIR assertion (C-183, C-218) | 024b-05 |
| Controller placement recorded in the manifest, placement and bundle-cap assertion (C-181, C-205, C-209, C-218) | 024b-12 |
| `hadronyche-dsl build <source> --out <dir>` (module, binary + readable manifest, vectors file) and decoders | 024b-12, 024b-14 |
| Fingerprint evaluation on the processing thread and stamp field | 024b-17 |
| Output-form and subnormal decisions | 024b-01, 024b-02 |
| Shared test corpus | 024b-04, 024b-06 |

### Acceptance criteria

| Criterion | Tickets |
|---|---|
| 024b/AC-1 No allocation, no libm | 024b-15 |
| 024b/AC-2 Bounded stack | 024b-16 |
| 024b/AC-3 Contributes to AC-32 | 024b-03, 024b-11 (never bounded), 024b-17 (forced mismatch, stamp), 024b-18 (bit identity) |
| 024b/AC-4 Fingerprint placement | 024b-17 |
| 024b/AC-5 Gap defaults | 024b-07, 024b-08, 024b-09 |
| 024b/AC-6 Degraded marking | 024b-10 |
| 024b/AC-7 Cost bound | 024b-13, 024b-19 |
| 024b/AC-8 Join padding | 024b-06, 024b-05, 024b-21, 024b-12 |
| 024b/AC-9 Output latency (contributes to AC-33) | 024b-05 |
| 024b/AC-10 Controller placement | 024b-12 |

### Definition of done

| Item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all except 024b-20 |
| Every parser touched is fuzzed | 024b-14 |
| Regime stamp fields touched are complete and documented | 024b-17, 024b-20 |
| Design documents updated where behaviour differs, constants table first | 024b-20 |
| No new open item without an owner and a falsifier | 024b-01, 024b-02, 024b-06, 024b-08, 024b-16, 024b-20 |
| The frozen POC is not modified (task 003 is the only exception) | not applicable (C-125): no ticket reads or ports POC code |
| Every new factual claim carries an evidence tag | all |
| ABI documented in `hadronyche-core` with one hand-written example module | not applicable (C-125): done in 024a-04; this part changes no ABI type |
| Every shipped table has a checksum test and a generation note | not applicable (C-125): this part ships no table |
| CI target list recorded as the listed-target set for exact-float | not applicable (C-125): recorded in 024a-06, reused here |

### Open sub-questions

| Question | Owner | Handling |
|---|---|---|
| Fingerprint vector covers every differing path? | Task 025 | Note in 024b-11 and 024b-20 (not a ticket here) |
| Subnormal policy for exact-float | This part | Ticket 024b-02 |
| Output form: Rust source versus object code | This part | Ticket 024b-01 |
| `stack_bytes` method | This part | Ticket 024b-16 |
| Items owned outside the part | See Remaining gaps | Tracked in 024b-20 |

## Remaining gaps

1. **Non-audio fingerprint layout.** Word layout of dense, sparse and transport content in the fingerprint vector. Affects 024b-11. Answer: content layouts epic 010.
2. **DSL constructs and analyses.** Syntax of gap overrides, of the join opt-out (C-104), and of an edge module with its non-domain-rate hardware port; the front-end rejection of a resampler outside an edge module or at another ratio (rule settled by C-105, C-134), of an even-tap linear-phase FIR (rule settled by C-183), of an odd-difference device-side join (C-190) and of controller-placement and bundle-cap violations (C-181, C-209), all compile-time rejections being the front end's (C-218); which nodes have frequency-dependent delay (recursive filters, C-189) and their structural part (C-104). Affects 024b-04, 024b-05, 024b-06, 024b-09, 024b-21. Answer: DSL front-end epic 023.
3. **Manifest format.** Binary and readable encoding (fields now settled: whole-sample L, inserted join delays, frequency-dependent join flags, edge conversion delay on the device-facing port (C-207), dense initial values, cost in operations, channel map, controller placement (C-181, C-205, C-209)). Affects 024b-12, 024b-14. Answer: epic 023.
4. **Exact-fixed vectors file content.** Every build writes a vectors file (C-123), but its content for exact-fixed modules is undefined; 024b-12 writes a provisional, versioned format. Affects 024b-12, 024b-14. Answer: conformance suite task 025.
5. **Regime stamp field.** Name and encoding of the effective determinism class field in the stamp message. Affects 024b-17, 024b-20. Answer: epic 013, which defines the message (C-144).
6. **32-bit device target.** Used by 024b-15, 024b-16, 024b-18. Answer: part 024a (024a-06), confirmed by epic 026.

## Settled since the split

- Who implements a compile-time rejection (C-218): the front end (epic 023) implements every one; the code generator only asserts it never receives such a program; the conformance corpus (epic 025a) tests it. The defensive rejections in 024b-05 (misplaced resampler, even-tap FIR) and 024b-06 (odd-difference device-side join) become assertions; 024b-22 is deleted and its placement recording and assertion moved to 024b-12.
- Controller placement of a single-stream source (C-181 revised, permissive): it may always use a second stream, must above the 100-byte cap, and may share the audio packet while it fits; the former "shares when it fits" reading is corrected (024b-12).
- Controller placement and the bundle cap (C-181, C-205, C-209, C-08 revised): a bundle source's controllers ride in their own member stream, which counts towards the 32 members, so at most 31 audio members (186 channels) and a wider source with controllers is rejected at compile time; a single-stream source's controllers may share the audio packet within the 100-byte cap (C-137), else a second stream; the compiler checks this from the manifest. Ticket 024b-22 was added, then deleted under C-218 (see above).
- Even-tap linear-phase FIR (C-183): rejected in v1 with a diagnostic suggesting an odd-tap design, so L stays exact; front end in epic 023; 024b-05 only asserts its absence (C-218).
- Smaller settled points (C-192): float and fixed variants of a corpus program count as one program; the mix's volume is the reference dense controller (024b-04, 024b-07); the fuzzing tool is the implementing team's choice (024b-14).
- Former gap "default tempo value": C-199 (120 beats per minute). 024b-08 initialises the before-first-state transport with it instead of taking it from `TransportState`.
- Playhead through a gap while stopped (C-200, with C-141 revised): a stopped transport's playhead does not advance; a playing one moves on at its tempo. 024b-08's provisional advance by the positions elapsed for every held state is corrected.
- Resampler delay and L (C-207): the edge's resampler delay is counted only in D, never in L; the manifest reports the conversion delay separately, on the device-facing port (024b-05, 024b-12).
- Former gap "transport gap before any received state": C-184 (stopped, playhead at zero, default tempo). 024b-08 no longer presents `None`; the default tempo value is C-199.
- Two kinds of resampler (C-189): the DSL node is fixed-ratio 2:1 or 1:2 with whole-sample delay; the edge's steerable rate actuator is separate and its delay is part of D (024b-05). The ABI keeps the "frequency-dependent delay" property on outputs, for recursive filters; new ticket 024b-21.
- Device-side join in an edge module (C-190): paths differing by an odd number of domain-rate samples are rejected with a diagnostic suggesting the per-join opt-out; the front end rejects, 024b-06 only asserts (C-218).
- Former gap "join with a non-integral latency difference": answered by C-105 (L is always a whole number of samples, so differences are whole) and C-104 revised (whole-sample delay line; a path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest). 024b-06 no longer rejects such joins.
- Former gap "gap default for read-only transport state": C-141 (last known state held, playhead advanced by the positions elapsed, output degraded); new ticket 024b-08. The before-first-state case is C-184 and C-199; the stopped case is refined by C-200.
- Former gap "calibration-factor procedure": C-176 (epic 025 runs a reference workload once per listed target and records time per operation, carried in the regime stamp). 024b-19 uses a provisional run of that shape until epic 025's value replaces it.
- Rational structural latency withdrawn (C-105 revised): L is a whole number of domain-rate samples in the ABI and manifest; rational sums, reduced fractions, the 147:160 ratio and resamplers in series are removed from 024b-05, 024b-10 and 024b-12.
- Resampler placement (C-134, C-170): only inside an edge module, rejected elsewhere by the front end, 024b-05 only asserting it (C-218); its conversion delay is part of D and is reported as such, not added to L (024b-05, 024b-12). `non_constant_delay` is dropped.
- Dense controller with no prior value (C-158): initial value from the manifest, default 0 (024b-07, 024b-12).
- Orphan duties (C-144): the hosted backend (epic 015) emits the gap slice on a missed deadline, raises the processing-overrun event and sends the regime stamp; epic 013 defines the stamp message, so the stamp field is now owned by epic 013 instead of epic 004 (024b-07, 024b-17).
- Machine pair for cross-machine bit-exactness: x86-64 and 64-bit ARM (C-151) (024b-18).
- Epic split (C-143): this part and 024a; 024-31 split into 024b-20 and 024a-15.
- Carried from earlier refinements: degraded flag follows the content (C-122); bounded class only from a fingerprint mismatch, so every build writes a vectors file (C-123); cost in counted operations with a per-target calibration factor (C-106, operation-counting build 024b-13); `stack_bytes` produced from the compiled artefact by this generator; shared test corpus built in 024b-04; fuzz budget (C-95, 024b-14); regime stamp transport (C-18); `[standard]` tag (C-132); output and tool locations (C-130, C-133, C-171); definition-of-done applicability and scope labels (C-125, C-126).
