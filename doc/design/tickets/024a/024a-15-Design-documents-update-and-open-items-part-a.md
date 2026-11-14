# 024a-15 — Design documents update and open items (part a)
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** XS · **Depends on:** 024a-04, 024a-06, 024a-11, 024a-12, 024a-13

## Context
Closing ticket for the documentation duties of part 024a (it takes over the part-a share of the former ticket 024-31; part 024b's share is 024b-20). The epic's definition of done requires design documents to be updated where behaviour differs (constants table first), every new open item to have an owner and a falsifier, and every new factual claim to carry an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, and `[standard]` for a figure cited from a named published standard with its clause, C-132). Specifications go in `doc/design/spec/`, reports and results in `doc/design/reports/`, the constants table is `doc/design/02-constants.md` (C-130); the architecture specification is `doc/design/01-architecture.md` (C-171).

Constants and rules this part applies: slice 48 frames = 0.5 ms at 96 kHz, the only rate on the wire (C-01, C-02); 4-byte samples, f32 or Q1.31 (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); bundles of up to 32 members, the controller member included (C-08, C-209); edge hardware at 48 kHz converts 2:1 through the kit's resampler (C-80); L always a whole number of domain-rate samples, resampler only at 2:1 and 1:2 (C-105); resampler only inside an edge module, conversion delay counted only in D and never in L, reported separately in the manifest on the device-facing port (C-134, C-170, C-207); this part is the single builder of both resamplers for every edge, in phase P4 (C-197); two kinds of resampler, fixed-ratio DSL node and separate steerable rate actuator, and the frequency-dependent delay property on outputs for recursive filters (C-189); even-tap linear-phase FIR rejected in v1, resampler filters odd-tap (C-183); cost in counted operations (C-106); degraded flag follows the content (C-122); where the bounded class arises (C-123); laptop sleep resets the edge loops (C-124); the rate-actuator trait defined in this part with a ratio range of ±1 000 ppm (C-210); drift-ramp bounds for the rate actuator (C-119, C-68); machine pair x86-64 and 64-bit ARM (C-151); definition-of-done applicability (C-125); output and tool locations (C-130, C-133). New constants from this part: `ABI_VERSION = 1`; resampler group delays for 2:1 and 1:2 as whole numbers of domain-rate samples (024a-11, 024a-12); the listed-target set (024a-06).

Open sub-questions of this part and their owners:
- Resampler length versus device cost — 024a-11, then task 026; falsifier: does not fit the reference device.
- 32-bit device target — 024a-06 with task 026; falsifier: cannot be emulated unprivileged, or differs from 026's reference device.
- Variable-ratio format and resolution of the rate-actuator resampler (C-189; range settled at ±1 000 ppm, C-210) — 024a-13; falsifier: quantisation alone produces slip or exceeds E in the L2 drift ramp (C-76, C-119).
- ABI auxiliary types and non-audio fingerprint layout — epic 010; falsifier: the types cannot be expressed in a `no_std` const context.

## Work
- Update the constants table: add `ABI_VERSION`, the resampler group delays for 2:1 and 1:2 (whole samples, C-105) and the listed-target set (if 024a-06 has not already added it); confirm C-01 to C-04, C-08, C-80, C-105, C-106, C-122, C-123, C-134, C-183, C-189, C-197, C-207, C-209, C-210 are applied as written or record differences.
- Update design documents where implemented behaviour differs from the epic (for example refined ABI names, the name of the frequency-dependent delay property, C-189, the rate-actuator ratio format).
- Ensure the open-items list carries every item above with owner and falsifier, and remove items that have been answered.
- Review all docs written in this part for evidence tags.

## Acceptance criteria
1. The constants table lists `ABI_VERSION = 1`, the resampler group delays for 2:1 and 1:2 as whole numbers of domain-rate samples, and the listed-target set, each with an evidence tag.
2. Each behavioural difference from the epic text found in tickets 024a-01 to 024a-14 is recorded in the design documents.
3. Every open sub-question above has an owner, a falsifier and a status.
4. A review of new documents finds no untagged factual claim, and no document of this part is outside the locations of C-130 and C-171.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- New implementation work; any missing behaviour is filed against the ticket that owns it.
- Documentation duties of part 024b (024b-20).
