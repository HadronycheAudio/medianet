# Epic 024a — Runtime ABI and arithmetic kit: tickets

## Summary
Epic 024 was split into two parts by rule C-143 (it exceeded 40 days by ticket tally). Part 024a (phase P4, C-197; scope L; depends on epics 009, 010; blocks 018, 021, 022a, 024b, 025, 026) builds the contract every module is compiled against: (1) the runtime ABI in `hadronyche-core::abi` (`ABI_VERSION = 1`, `ModuleInfo` with each output's latency as a whole number of domain-rate samples and its frequency-dependent delay property, `InputSlice` / `OutputSlice`, the `Module` trait, `check_module_info`, `Fingerprint`, `fingerprint_matches` and `effective_class`), documented with a hand-written example module; and (2) the `no_std`, no-alloc, no-libm arithmetic kit `hadronyche-kit` (Q1.31 / Q0.32 fixed point with `mul_high`, strict f32 with an add/multiply `sine`, shipped checksummed tables, linear ramps, the fixed-ratio 2:1 / 1:2 resampler node with a whole-sample group delay used inside edge modules, and a separate steerable resampler as the edge software rate actuator, C-189, with the rate-actuator trait every edge consumes and a ratio range of ±1 000 ppm, C-210; the kit builds both once for every edge, C-197), with the three-target CI matrix recorded as the listed-target set. Part 024b (the code generator) depends on this part as a whole.

Acceptance criteria are numbered 024a/AC-1 … 024a/AC-5 in the epic and in the tickets.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 024a-01 | ABI core types and Module trait | S | epic 009, epic 010 |
| 024a-02 | ABI version check with named reason | XS | 024a-01 |
| 024a-03 | ABI `fingerprint_matches` | S | 024a-01 |
| 024a-04 | ABI documentation and hand-written example module | S | 024a-01, 024a-02, 024a-03 |
| 024a-05 | Kit crate skeleton and no-libm / no-alloc check | S | none |
| 024a-06 | CI target matrix and listed-target set | S | 024a-05 |
| 024a-07 | Kit fixed point: Q1.31, Q0.32 and `mul_high` | S | 024a-05 |
| 024a-08 | Kit strict f32 helpers and `sine` | S | 024a-05 |
| 024a-09 | Kit shipped fixed-point sine table with checksum | S | 024a-05, 024a-07 |
| 024a-10 | Kit linear ramp, float and fixed | S | 024a-07, 024a-08 |
| 024a-11 | Resampler filter length versus device cost, and coefficient table | M | 024a-05, 024a-06, 024a-07 |
| 024a-12 | Kit 2:1 and 1:2 resampler node with whole-sample group delay | S | 024a-07, 024a-08, 024a-11 |
| 024a-13 | Kit rate-actuator trait and resampler (separate, variable ratio) | M | 024a-12 |
| 024a-14 | Kit cross-target determinism suite | S | 024a-06, 024a-07, 024a-08, 024a-09, 024a-10, 024a-11, 024a-12 |
| 024a-15 | Design documents update and open items (part a) | XS | 024a-04, 024a-06, 024a-11, 024a-12, 024a-13 |

**Renumbering from epic 024.** 024-01 … 024-14 became 024a-01 … 024a-14 (same order). 024a-15 is new: it carries the part-a share of the former 024-31, which was split between the two parts (part b's share is 024b-20).

**Changes in this refinement.** 024a-12 (former 024-12) shrank from M to S and was renamed: the withdrawn rational-L rule (C-105 revised) removes the 147:160 test ratio, fractional phase and rational delays; only 2:1 and 1:2 remain. No ticket was deleted: none existed only for the rational-L rule. 024a-01, 024a-04, 024a-11 and 024a-13 dropped rational latency (C-105, C-134).

**Changes after the last rulings (C-183, C-189).** No ticket added or removed; efforts unchanged. 024a-01 and 024a-04 restore the frequency-dependent delay property on `OutputInfo` (C-189; 024a-01 gains an acceptance criterion). 024a-13 was retitled and reworked: the rate actuator is a separate steerable resampler, not a mode of the DSL node (C-189); its file name is unchanged. 024a-11 constrains the resampler filters to odd-tap linear-phase designs (C-183). 024a-12 and 024a-15 updated accordingly.

**Changes after the last rulings (C-197, C-207).** No ticket added or removed; efforts unchanged. Epic moved to phase P4 and now blocks 018, 021, 022a and 026 as the single builder of both resamplers (C-197); 024a-11, 024a-12, 024a-13 and 024a-15 state it, and 024a-13 drops the reading that edges are only macOS and Windows. Resampler delay is counted only in D and never in L, and the manifest reports it on the device-facing port (C-207): 024a-01, 024a-04, 024a-11, 024a-12 (`group_delay` documentation), 024a-13 and 024a-15 updated.

**Changes after the last rulings (C-209, C-210).** No ticket added or removed; efforts unchanged. 024a-13 now defines the rate-actuator trait that every edge consumes and sets the ratio range to ±1 000 ppm around nominal, replacing the provisional "at least ±100 ppm" (C-210); it gains an acceptance criterion for the range ends. The bundle cap of 32 members includes the controller member (C-08, C-209): epic, 024a-01 and 024a-15 updated.

## Effort tally
- XS: 2
- S: 11
- M: 2
- Total: 15 tickets, about 17 days (XS 0.5 + S 11 + M 5 = 16.5); scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `hadronyche-core::abi`: `ABI_VERSION`, `DeterminismClass`, `Latency` (whole samples), `OutputInfo` frequency-dependent delay property (C-189), `ModuleInfo`, `InputSlice`, `OutputSlice`, `Module`, `Fingerprint` | 024a-01 |
| `check_module_info` / `AbiError` | 024a-02 |
| `fingerprint_matches<M>()`, `effective_class<M>()` | 024a-03 |
| ABI documentation and hand-written example module | 024a-04 |
| `hadronyche-kit` crate, no_std / no alloc / no libm | 024a-05 |
| `fixed.rs` — Q1.31, Q0.32, saturating / wrapping, `mul_high` | 024a-07 |
| `float.rs` — strict f32 helpers, `sine` | 024a-08 |
| `tables.rs` — fixed-path sine table with checksum | 024a-09 |
| `tables.rs` — resampler coefficients with checksum | 024a-11 |
| `ramp.rs` — linear ramp, float and fixed | 024a-10 |
| `resample.rs` — fixed-ratio 2:1 / 1:2 node, odd-tap, whole-sample group delay | 024a-11, 024a-12 |
| `resample.rs` — rate-actuator trait (C-210) and separate steerable rate-actuator resampler with variable ratio, range ±1 000 ppm (C-189, C-210) | 024a-13 |
| `hadronyche-tools`: `check-symbols`, `gen-tables`, aborting allocator | 024a-05, 024a-09, 024a-11 |
| CI targets and listed-target set; kit determinism suite | 024a-06, 024a-14 |

### Acceptance criteria

| Criterion | Tickets |
|---|---|
| 024a/AC-1 No allocation, no libm (kit and ABI) | 024a-05, 024a-01 |
| 024a/AC-2 Kit determinism | 024a-07, 024a-08, 024a-09, 024a-10, 024a-11, 024a-12, 024a-14 |
| 024a/AC-3 Resampler node | 024a-11, 024a-12 |
| 024a/AC-4 ABI stability | 024a-02 |
| 024a/AC-5 Fingerprint check | 024a-03 |

### Definition of done

| Item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all except 024a-15; CI set-up in 024a-06 |
| Every parser touched is fuzzed | not applicable (C-125): no parser in this part |
| Regime stamp fields touched are complete and documented | not applicable (C-125): no stamp field touched in this part (set in 024b-17) |
| Design documents updated where behaviour differs, constants table first | 024a-15 (and 024a-06 for the listed-target row) |
| No new open item without an owner and a falsifier | 024a-06, 024a-11, 024a-13, 024a-15 |
| The frozen POC is not modified (task 003 is the only exception) | 024a-07, 024a-08, 024a-09, 024a-10, 024a-15 |
| Every new factual claim carries an evidence tag | all |
| ABI documented in `hadronyche-core` with one hand-written example module | 024a-04 |
| Every shipped table has a checksum test and a generation note | 024a-09, 024a-11 |
| CI target list recorded as the listed-target set for exact-float | 024a-06 |

### Open sub-questions

| Question | Owner | Handling |
|---|---|---|
| Resampler filter length versus device cost (2:1, 1:2) | This part, then task 026 | Ticket 024a-11 (open item for 026) |
| 32-bit device CI target | This part, with task 026 | Ticket 024a-06 |
| Variable-ratio format and resolution | This part | Ticket 024a-13 |
| ABI auxiliary types and non-audio fingerprint layout | Epic 010 | See Remaining gaps; tracked in 024a-15 |

## Remaining gaps

1. **ABI auxiliary types.** `PortInfo`, the rest of `OutputInfo`, `SparsePart`, `SparseOut`, `TransportState`, and the fingerprint word layout for non-audio content are not defined. Affects 024a-01, 024a-03 (and 024b-11). Answer: content layouts epic 010.
2. **32-bit device target.** Not yet named. Affects 024a-06, 024a-14 (and 024b-15, 024b-16, 024b-18). Answer: proposed by 024a-06, confirmed by epic 026 against its reference device.
3. **Resampler cost on the reference device.** Affects 024a-11. Answer: measurement on the 32-bit CI target in 024a-11, then epic 026 on its reference device.

None of these needs the design owner.

## Settled since the split

- Rational structural latency withdrawn (C-105 revised): L is always a whole number of domain-rate samples; `Latency` is a sample count; the resampler is offered only at 2:1 and 1:2 and other ratios are out of v1. Removed: the 147:160 test ratio, rational group delays, fractional-phase handling, and the former settled item "L is an exact rational". 024a-12 shrank from M to S.
- Resampler placement and its delay (C-134, C-170, C-207): the node exists only inside an edge module, and its conversion delay is counted only in the edge's plumbing delay D, never in any L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (the reporting is 024b-05) (024a-01, 024a-04, 024a-12, 024a-13).
- Who builds the resamplers (C-197): this part builds both the fixed 2:1 / 1:2 node and the steerable rate actuator, once; every edge (Linux, macOS, Windows, embedded) uses the kit's and builds none of its own. The epic moves to phase P4, ahead of the Linux edge, and blocks epics 018, 021, 022a and 026 (024a-11, 024a-12, 024a-13, 024a-15).
- Two kinds of resampler (C-189): the DSL resampler node is fixed-ratio, 2:1 or 1:2, with whole-sample delay; the edge rate actuator is a separate steerable resampler whose ratio varies by parts per million around nominal, and its delay is part of D (024a-12, 024a-13). The ABI keeps the "frequency-dependent delay" property on outputs, for recursive filters; it is restored on `OutputInfo` after the earlier drop of `non_constant_delay` (024a-01, 024a-04).
- Even-tap linear-phase FIR rejected by the compiler in v1 so that L stays exact (C-183); the kit's resampler filters are odd-tap linear-phase (024a-11). The compiler rejection itself is outside this part.
- Home and range of the rate actuator (C-210): the rate-actuator trait is defined in this part together with its implementation, with a ratio range of ±1 000 ppm around nominal (covering the C-119 drift ramp); every edge consumes it. Only the ratio's format and resolution stay open, owned by 024a-13 (024a-13, 024a-15).
- Bundle cap includes the controller member (C-08, C-209) (epic Constants, 024a-01, 024a-15).
- Gap-slice emission on a missed deadline belongs to the hosted backend, epic 015 (C-144) (epic Background).
- Machine pair for cross-machine bit-exactness: x86-64 and 64-bit ARM (C-151), both in the CI matrix (024a-06, 024a-14).
- Epic split (C-143): this part and 024b; 024-31 split into 024a-15 and 024b-20.
- Carried from earlier refinements: the version check (`check_module_info`, `AbiError::VersionMismatch`); fingerprint layout, fresh `init` state and "not eligible" without a fingerprint, consulted only for exact-float (C-123); Q0.32 unsigned [0, 1) and a documented Q1.31 multiply rounding; table parameters chosen by the kit and recorded in each generation note; variable-ratio dynamics bounded by C-119 and C-68 (range now C-210), reset on sleep by C-124; `[standard]` tag (C-132); output and tool locations (C-130, C-133, C-171); definition-of-done applicability and scope labels (C-125, C-126).
