# 010-20 — Send-path audit and design document update
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-01 to 010-19, 010-21 to 010-24
## Context
Closing ticket for epic 010 (packetiser and bundles core in `hadronyche-core`). It checks the epic-wide definition-of-done items that no single ticket can close:
- No allocation and no lock anywhere on the send path.
- Compiles for a `no_std` target without an allocator (epic AC-10).
- Design documents updated where behaviour differs, constants table first. Constants concerned: C-01 to C-10, C-14, C-18, C-19, C-20 to C-22, C-43, C-44, C-47, C-95, C-100, C-105 (whole-sample L), C-116, C-120, C-122, C-137, C-144, C-155, C-159, C-165, C-181, C-205, C-207 (resampler delay in D, never in L), C-208 (a fan-in announces its own re-alignment after a changed A), C-209 (controller member counts towards the 32 of C-08), C-215 (a ratchet step is an announced re-alignment), C-219 (wire cost of a controller stream).
- Every new factual claim carries an evidence tag (`[code]`, `[decision]`, `[memory]`, `[measured]`, `[kernel-doc]`, `[standard]` per C-132). A `[memory]` figure that comes from a published standard is checked and re-tagged `[standard]` with its clause.
- C-130, C-171: design updates go to `doc/design/` (constants table `doc/design/02-constants.md`, architecture specification `doc/design/01-architecture.md`, specifications in `doc/design/spec/`), reports to `doc/design/reports/`; nothing goes into the frozen POC tree.
- No new open item without an owner and a falsifier. Epic open items: spread plus phase vs queue drops (L1, task 019); phase derivation and sharing threshold (010-10, task 019); header budget within the C-137 cap, Ethernet terms, multi-packet placement and announcement edge rules (epic 004); epoch-switch boundary (epic 006); `no_std` target (epic 026). Settled and no longer open: phase range (C-155), rounding of L and the sign convention (C-105, C-122), owner of runtime gap emission (C-144), controllers of a single-stream source (C-181, revised: second stream always allowed, sharing allowed while within the cap), controllers of a bundle source always in their own member (C-205).
- C-125: a definition-of-done line with no subject is marked not applicable with the reason, never silently skipped.
- C-146: "unprivileged" applies to per-change CI; no test of this epic needs a privilege.

## Work
- Audit every send-path module (`output_position`, sizing, layouts, wire rate including the controller member, packetiser, bundle packetiser, slice writer, gap slice, pacing, announcements, header-cap omission, epoch switch, counters, test-only epoch hook): no `alloc`, no `Mutex`/`RwLock`/spin lock, no blocking call. Add a CI check that greps for these or a `#![deny]` lint set, and document it.
- Reconcile the constants table with what the code measures (C-04 bytes left, C-20, C-21, C-22, C-137 header cap, fallback channel count at 1 280); update the table first, then the text.
- Check that every open item of the epic has an owner and a falsifier, and that every provisional choice in the code names its open item.
- Confirm all epic 007 media-packet conformance vectors pass together (epic AC-1).
- Check the epic's "Not applicable in this epic" note still matches the delivered work (C-125).

## Acceptance criteria
1. The `no_std`, no-allocator CI build of 010-01 covers every module listed above and passes (epic AC-10).
2. The no-lock / no-alloc check runs in CI and passes; an intentional `Mutex` on a throwaway branch makes it fail.
3. All epic 007 media-packet conformance vectors encode byte for byte in one test run (epic AC-1).
4. The constants table in the design document matches the figures produced by the code, or each difference is explained.
5. Every open item of the epic has an owner and a falsifier in the design document, and every provisional behaviour in the code points to one.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No allocation and no lock anywhere on the send path
- [ ] Packets per slice and pacing mode are exposed for the regime stamp

## Out of scope
- New functionality; any defect found is fixed in the ticket that owns the code.
- Backend execution (epic 015) and device port (epic 026).
