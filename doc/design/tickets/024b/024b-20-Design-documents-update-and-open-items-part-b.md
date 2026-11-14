# 024b-20 — Design documents update and open items (part b)
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-01, 024b-02, 024b-06, 024b-08, 024b-16, 024b-17, 024b-19, 024b-21

## Context
Closing ticket for the documentation duties of part 024b (it takes over the part-b share of the former ticket 024-31; part 024a's share is 024a-15). The epic's definition of done requires design documents to be updated where behaviour differs (constants table first), every new open item to have an owner and a falsifier, every new factual claim to carry an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, and `[standard]` for a figure cited from a named published standard with its clause, C-132), and regime stamp fields touched to be complete and documented. Specifications go in `doc/design/spec/`, reports and results in `doc/design/reports/`, the constants table is `doc/design/02-constants.md` (C-130); the architecture specification is `doc/design/01-architecture.md` (C-171).

Constants and rules this part applies: slice 48 frames = 0.5 ms at 96 kHz, the only rate on the wire (C-01, C-02); 4-byte samples, f32 or Q1.31 (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); bundles of up to 32 members, the controller member included (C-08, C-209); controllers of a bundle source in their own member stream, a single-stream source's within the 100-byte cap or in a second stream, a second stream always allowed (C-181 revised, permissive), checked from the manifest (C-137, C-181, C-205); even-tap linear-phase FIR rejected (C-183); every compile-time rejection implemented by the front end, the code generator only asserting it never receives such a program (C-218); float and fixed variants of a corpus program count as one program, the mix's volume is the reference dense controller, the fuzzing tool is the implementing team's choice (C-192); edge hardware at 48 kHz converts 2:1 (C-80); regime stamp transport (C-18), defined by epic 013 and sent by epic 015 (C-144); fuzz budget (C-95); join padding with whole-sample delay lines and frequency-dependent-join flags (C-104); L always a whole number of domain-rate samples (C-105); cost in counted operations with a per-target calibration factor (C-106) measured by epic 025 (C-176); degraded flag follows the content (C-122); where the bounded class arises (C-123); resampler only inside an edge module, conversion delay part of D (C-134, C-170), counted only in D and reported separately on the device-facing port (C-207); transport hold on a gap (C-141), a stopped playhead not advancing through a gap (C-200); transport state before any has arrived (C-184), default tempo 120 beats per minute (C-199); two kinds of resampler and the "frequency-dependent delay" output property (C-189); device-side join rejection in edge modules (C-190); dense initial values (C-158); machine pair x86-64 and 64-bit ARM (C-151); definition-of-done applicability (C-125); output and tool locations (C-130, C-133).

Open sub-questions of this part and their owners:
- Fingerprint coverage — task 025; falsifier: a target passes the fingerprint and differs on a conformance vector.
- Subnormal policy — 024b-02; falsifier: a real program produces subnormals in normal operation.
- Output form — 024b-01; falsifier: Rust source cannot guarantee evaluation order under optimisation.
- `stack_bytes` method — 024b-16; falsifier: measured stack exceeds the declared value.
- Non-audio fingerprint layout — epic 010.
- DSL syntax of overrides, join opt-out and edge modules; front-end rejection of a misplaced resampler, of an even-tap linear-phase FIR (C-183), of an odd-difference device-side join (C-190) and of controller-placement and bundle-cap violations (C-181, C-209), all compile-time rejections being the front end's (C-218); which nodes have frequency-dependent delay — epic 023.
- Manifest binary and readable format — epic 023.
- Exact-fixed vectors file content — task 025.
- Regime stamp field for the effective determinism class — epic 013 (C-144).

## Work
- Confirm in the constants table that C-01 to C-04, C-08, C-80, C-104, C-105, C-106, C-122, C-123, C-134, C-137, C-141, C-158, C-176, C-181, C-183, C-184, C-189, C-190, C-192, C-199, C-200, C-205, C-207, C-209 and C-218 are applied as written, or record differences.
- Update design documents where implemented behaviour differs from the epic (for example the `stack_bytes` method, the provisional calibration run of 024b-19, the provisional exact-fixed vectors content).
- Ensure the open-items list carries every item above with owner and falsifier, and remove items that have been answered.
- Check that the regime-stamp determinism-class field (024b-17) is documented in epic 013's specification.
- Review all docs written in this part for evidence tags.

## Acceptance criteria
1. Each behavioural difference from the epic text found in tickets 024b-01 to 024b-19 and 024b-21 is recorded in the design documents.
2. Every open sub-question above has an owner, a falsifier and a status.
3. The determinism-class stamp field is documented with every value it can take.
4. A review of new documents finds no untagged factual claim, and no document of this part is outside the locations of C-130 and C-171.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- New implementation work; any missing behaviour is filed against the ticket that owns it.
- Documentation duties of part 024a (024a-15).
