# 023-27 — Design documents update and epic close-out
**Epic:** 023-DSL-compiler-front-end · **Effort:** XS · **Depends on:** 023-22, 023-24, 023-25, 023-26

## Context
- Epic 023 definition of done, items that span the whole epic:
  - Tests green in CI, unprivileged (C-146: the line applies to per-change CI; this epic has no test needing a privilege).
  - Every parser touched is fuzzed (DSL parser, manifest decoder; budget C-95).
  - Regime stamp fields touched are complete and documented (determinism class stamp, epic 013).
  - Design documents updated where behaviour differs, constants table first (`doc/design/02-constants.md`, C-130). Constants used: C-01 96 kHz only wire rate, C-02 48 frames = 0.5 ms, C-03 4-byte samples, C-04 6 channels/stream and no per-channel id byte, C-08 ≤ 32 bundle members, C-09 ≤ 256 streams/source, C-20 ~1 320 bytes per packet, C-21 ~21.1 Mbit/s and 2 000 packets/s, C-22 96 ch = 32 000 packets/s and 338 Mbit/s, C-80 2:1 at 48 kHz edges, C-95 fuzz budget; rules C-104, C-105 (L always whole; the rational-L rule is withdrawn), C-106, C-122, C-123, C-134, C-137, C-141, C-158, C-176, C-181, C-183, C-184, C-190, C-199, C-200, C-205, C-207, C-209 (the controller member counts towards the 32 of C-08), C-197 (the resampler node is built by the kit epic 024a), C-218 (this epic implements every compile-time rejection).
  - No new open item without an owner and a falsifier.
  - The frozen POC is not modified (task 003 is the only exception).
  - Every new factual claim carries an evidence tag (`[standard]` with clause for published standards, C-132).
  - Not applicable lines are marked with the reason, never skipped (C-125); the epic records none.
- Open sub-questions of this epic: concrete syntax (023-01), hand-written module manifests (023-25), edge-module device-facing side (023-01, 023-29). Decided since the split: cost unit (C-106, factor measured by 025 per C-176), resampler ratios and whole-sample L, conversion delay counted only in D (C-105, C-134, C-207), join rule (C-104), fractional join padding (cannot arise, C-104, C-105), hand-written default class (C-123), transport-state gap default, a stopped playhead not advancing (C-141, C-200), dense initial value (C-158), transport state before any has arrived and the default tempo of 120 beats per minute (C-184, C-199), controllers of a single-stream source (C-181, revised: a second stream is always allowed and required beyond the header cap), controllers of a bundle source in their own member stream (C-205), the controller member counting towards the 32-member cap so a source with controllers has at most 31 audio members (C-209), the resampler design and delay supplied by the kit epic 024a (C-197), even-tap linear-phase FIR rejected (C-183), device-side join with an odd domain-rate difference rejected (C-190), every compile-time rejection implemented here, asserted by 024b and tested by 025a (C-218). Owned elsewhere: manifest layout (004 with 013), header breakdown and admission units (012 with 004), member width > 6 (004), stack model (024), `abi_version` (012), verification of hand-written figures (025 with 012), confirmation of the resampler delay (025 probe suite).
- Specifications produced by this epic live in `doc/design/spec/` (C-130).

## Work
1. Review all design documents touched; update the constants table first if any constant changed or was added.
2. Mark the sub-questions owned by this task as resolved, linking the decision; confirm each sub-question owned elsewhere still has its owner and falsifier.
3. Verify the regime-stamp determinism-class field documentation (from 023-18/023-21) is complete.
4. Verify no file of the frozen POC changed (diff against its frozen revision).
5. Confirm the CI jobs run all tests unprivileged and both fuzz targets at the C-95 budget.

## Acceptance criteria
1. Every DoD box of epic 023 is ticked with a link to the evidence (PR, test, document).
2. 0 open items without owner and falsifier in the documents touched.
3. 0 changes to the frozen POC.
4. Every specification written by the epic is under `doc/design/spec/`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] `LANGUAGE.md` covers every construct with one example
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution

## Out of scope
- New functionality; any work of epics 024/025.
