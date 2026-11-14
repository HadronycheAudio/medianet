# Epic 017a — Reference modules: tickets

## Summary
Epic 017a (phase P4, scope L) is the first part of epic 017, split under C-143. It ports the POC's test instrument — sender, echo (synth) and mixer — onto the new runtime as permanent hand-written Rust **reference modules** in crate `hadronyche-reference`: instrument, analyzer and transport ports, the fixed-point sine table shipped as a constant, no libm in any signal path, a whole-sample structural latency and no resampler (C-105, C-134). The mixer becomes a position-aligned fan-in that zero-fills a gap, marks the output degraded and takes its ceiling from the downstream presentation group or an explicit budget, refusing with NoSkewBudget otherwise (C-142, C-185), and follows a changed inherited A (C-186, C-198, C-208, C-211). The part ends with fuzzing of the two payload decoders and a bit-exactness proof between an x86-64 and a 64-bit ARM machine (C-151). Epic 017b (the L0 harness and scenarios) depends on this part.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 017a-01 | Port voices with a constant sine table | S | epic 002 |
| 017a-02 | Port the phrase generator | S | 017a-01, epic 002 |
| 017a-03 | Port the transport state machine | S | epic 002 |
| 017a-04 | Analyzer windows and per-family sample counts | M | epic 002 |
| 017a-05 | Analyzer exact tier and loss exclusion | S | 017a-04, epic 002 |
| 017a-06 | Analyzer tolerance tier and per-voice projections | S | 017a-04, 017a-01, epic 002 |
| 017a-07 | Reference echo module | M | 017a-01, 017a-03, epic 009, epic 015 |
| 017a-08 | Reference mixer as position-aligned fan-in | M | 017a-03, epic 009, epic 015 |
| 017a-09 | Reference sender with replicas and comparison | M | 017a-02, 017a-03, 017a-05, 017a-06, 017a-07, 017a-08, epic 010, epic 015 |
| 017a-10 | Cross-machine bit-exactness of the ported modules | M | 017a-09, epic 002, epic 013 |
| 017a-11 | Fuzz the reference modules' payload decoders (new, from the split) | S | 017a-07, 017a-08 |
| 017a-12 | Design documents, constants and provenance for the reference modules (new, from the split) | S | 017a-01 to 017a-11, 017a-13 |
| 017a-13 | Reference mixer follows a changed inherited A (new, C-186, C-198, C-208, C-211) | S | 017a-08, epic 009, epic 015 |

Origin in the former epic 017: 017a-01 to 017a-09 were 017-06 to 017-14; 017a-10 was 017-31; 017a-11 and 017a-12 take the reference-module half of 017-29 (fuzzing) and 017-32 (design documents and provenance). 017a-10 no longer depends on the L0 runner: it records the regime stamps of both machines directly (epic 013). 017a-08 now depends on 017a-03 for the hold-and-advance transport step (C-141).

No ticket was deleted: nothing in this part was written for the withdrawn rational-L rule (C-105 as revised), so there was no rational-number handling, rounding of L or fractional delay line to remove.

## Effort tally
- XS: 0
- S: 8
- M: 5
- Total: 13 tickets, about 21 days (20.5; XS 0.25, S 1, M 2.5 days) — scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `hadronyche-reference/sender.rs` (controllers placed per C-181) | 017a-09 |
| `hadronyche-reference/echo.rs` | 017a-07 |
| `hadronyche-reference/mixer.rs` (with fan-in budget and NoSkewBudget refusal, C-142, C-185; changed inherited A, C-186, C-198, C-208, C-211) | 017a-08, 017a-13 |
| `instrument/voices.rs` (constant fixed-point table) | 017a-01 |
| `instrument/phrase.rs` | 017a-02 |
| `analyzer/` (windows, projections, exact and tolerance comparison, per-family counts) | 017a-04, 017a-05, 017a-06 |
| `transport/` (state machine, two existing tests, hold-and-advance on a gap, C-141, C-200) | 017a-03 |
| Fuzz targets for the echo and mixer payload decoders | 017a-11 |
| Cross-machine bit-exactness report (file name and layout decided here, C-206) | 017a-10 |
| Open sub-questions: voices exact vs tolerance, epsilons, finding ids, transport test names (owner: epic 002) | 017a-05, 017a-06, 017a-01 to 017a-09, 017a-03 |
| Open sub-question: received-positions bitmap (deferred) | note in 017a-05 |
| Open sub-question: echo degraded timing (epic 009) | 017a-07 |
| Dense controller: the mixer's volume (settled, C-192) | 017a-08 |
| Initial transport state: stopped, playhead at zero, 120 beats per minute (settled, C-184, C-199) | 017a-03 (used by 017a-07, 017a-08) |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 1. AC-017a.1 echo | 017a-01, 017a-07 |
| 2. AC-017a.2 mixer | 017a-08, 017a-13 |
| 3. AC-017a.3 sender and replicas | 017a-02, 017a-09 |
| 4. AC-017a.4 analyzer | 017a-04, 017a-05, 017a-06 |
| 5. AC-017a.5 transport | 017a-03 |
| 6. AC-017a.6 cross-machine (C-151) | 017a-10 |

### Definition of done

| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all |
| Every parser touched is fuzzed | 017a-11 |
| Regime stamp fields touched are complete and documented | not applicable (C-125): this part adds no stamp field |
| Design documents updated, constants table first | 017a-12 |
| No new open item without owner and falsifier | 017a-12 (and where raised: 017a-05, 017a-07, 017a-13) |
| Frozen POC not modified | 017a-01 to 017a-09, 017a-12 |
| Evidence tags on new claims | all; audit in 017a-12 |
| Bit-exact with replicas on two machines (float and fixed) | 017a-10 |
| Ported pieces cite source file and archaeology finding | 017a-01 to 017a-09, 017a-12 |

## Remaining gaps

1. **Tolerance-tier epsilons.** Affects 017a-06. Answer: epic 002 (reading of `analyse`).
2. **Voice windows exact vs tolerance** decides whether voices are tested in 017a-05 or 017a-06 (bounded class, C-123). Answer: epic 002.
3. **Archaeology finding ids** for provenance headers (location settled by C-130: `doc/design/reports/`). Affects 017a-01 to 017a-09, 017a-12. Answer: epic 002.
4. **Names of the two transport tests at `transport.rs:864`.** Affects 017a-03. Answer: epic 002.
5. **`ModuleHandle`, `StreamHandle` and the module API.** Affects 017a-07, 017a-08, 017a-09. Answer: epic 015.
6. **When an arrival-driven module emits the degraded output for a position that never arrives** (raised by applying C-122 to the echo; C-144 covers only a module missing its own deadline). Affects 017a-07. Answer: epic 009.
7. **Resynchronisation bound after loss** (inherit 3 s?). Affects 017a-07 (AC 4 uses 3 s meanwhile). Answer: a measurement in 017b-11, informed by epic 002.

## Settled since the split

- Former gap 8 of epic 017, which two machines prove bit-exactness → C-151 (x86-64 and 64-bit ARM), applied in 017a-10.
- C-104, C-105 (revised) and C-134 → structural latency is a whole number of samples and resamplers exist only in edge modules; stated in 017a-07 and 017a-08. Nothing here depended on the withdrawn rational-L rule.
- C-122 as now worded (stamps denote content position) → echo, mixer and exact-tier tickets restated: the output stamped q carries the content of input q and is degraded if that input was gapped.
- C-137 (header cap) → sender acceptance criterion in 017a-09.
- C-141 (transport state on a gap) → hold-and-advance in 017a-03, used by 017a-07 and 017a-08.
- C-142 (fan-in A) → the mixer's ceiling comes from the downstream presentation group or an explicit budget, else its aligned inputs are refused; 017a-08, 017a-10.
- C-143 → epic 017 split into 017a and 017b; fuzzing and design-document tickets split between the parts (017a-11, 017a-12).
- C-158 (dense-controller initial value) → criterion in 017a-08, now unconditional under C-192.
- C-192 → former gap 8 closed: the reference dense controller is the mixer's volume; 017a-08 declares it dense with its manifest initial value and AC 8 is no longer conditional. The fuzzing tool is the implementing team's choice and the vectors crate sits at the workspace root (017a-11). Its other points (float and fixed variants of a corpus program count as one program; the L0 baseline scenario is defined by 017b) have no effect on this part.
- C-184 (transport state before any has arrived: stopped, playhead at zero, default tempo) → initial state of the read-only view and a new criterion in 017a-03; stated in 017a-07 and 017a-08. Raised gap 8 (tempo value), now closed by C-199.
- C-199 (default tempo 120 beats per minute) and C-200 (playhead does not advance through a gap while stopped) → gap 8 closed: 017a-03 initialises the view to 120 beats per minute instead of the ported default and its `hold_and_advance` advances the playhead only while playing (Work 4 and 5 corrected, acceptance criteria 4 to 6 restated); stated in 017a-07, 017a-08 and 017a-12, and in the epic's AC-017a.5. No new ticket.
- C-135 (foreign-epoch slices restart the dead clock) → stated in the epic background; asserted in 017b.
- C-181 (where controllers travel) → the reference sender is a single-audio-stream source: the mixer's volume rides in the audio feed packet within the 100-byte cap, the parts in a second controller stream; checked by the sender's own tests (no compiler for a hand-written module). Stated in the epic, 017a-08 and 017a-09 (Work 1, AC 3).
- C-185 (NoSkewBudget) → the mixer's refusal with no downstream group and no budget now names its reason; epic AC-017a.2, 017a-08 (Work 2, AC 7), 017a-10.
- C-186, C-198, C-208, C-211 (changed A at a fan-in, carried by the receiver report; re-alignment position picked and announced by the fan-in; A beyond buffer memory) → new ticket 017a-13 (S); stated in the epic and AC-017a.2.
- C-206 (presentation details of tools) → the analyzer report layout and the cross-machine report file name are this part's to decide and record (017a-04, 017a-10); only the per-family counts, inert families and mismatches that 017b depends on are a contract.
- C-08 as revised with C-209 (controller member counts towards the 32) → constants list of the epic updated; no bundle source in this part.
- C-35's owner change (L1, task 019) and C-127 to C-133 have no new effect on this part; C-136, C-138, C-140, C-144 to C-150, C-152 to C-157, C-159 to C-176, C-177 to C-180, C-182, C-183, C-187 to C-191, C-193 to C-197, C-201 to C-205, C-207, C-210 and C-212 concern the harness (017b) or other epics: the reference modules are hand-written (no compiler checks, no FIR), contain no resampler, publish no stamp lateness figures and report no one-way figures. C-134 and C-141 as revised and C-184 were already stated.
