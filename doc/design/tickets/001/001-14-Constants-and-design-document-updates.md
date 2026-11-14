# 001-14 — Constants and design document updates
**Epic:** 001-Receive-stamp-probes · **Effort:** XS · **Depends on:** 001-13

## Context
Epic 001 provides to the constants table a confirmed or corrected C-90 (expected loss floor; currently BER 1e-12, about 1.1e-8 per default packet of about 1 320 bytes on the wire, one loss per stream per link about every 13 hours at 2 000 packets/s `[memory]`), the matching C-118 soak prediction (about 58 losses in 24 hours at 64 000 packets/s) if C-90 changes, the C-179 reopen count (10 × the floor, about 580 losses in the soak) with it, evidence for the clock shares of C-150 (macOS 40 µs, Windows 100 µs) and so for the E targets C-71 (macOS, 100 µs) and C-72 (Windows WASAPI exclusive, 1 ms), with a proposed revision if the strict-mode 99.9th percentile exceeds the share or kernel stamps are absent, and the input from which the supported OS versions of C-138 are set. C-71, C-72 and C-150 stay pending measurement, owned by L2 (task 020). A value cited from a published standard is tagged `[standard]` with its clause, replacing `[memory]` (C-132). The findings are in `doc/design/reports/REPORT-0a.md` (001-13, C-130).

Locations: the constants table is `doc/design/02-constants.md`; design documents are the files under `doc/design/`, including the architecture specification `doc/design/01-architecture.md` (C-171) and this epic, `doc/design/implementation/001-Receive-stamp-probes.md`, whose Background lists the `[memory]` claims.

Design rule: design documents are updated where behaviour differs, constants table first.

## Work
- Update the constants table entries C-90 (and C-118 and the C-179 reopen count if C-90 changes) to the values in `REPORT-0a.md`, or mark C-90 confirmed, with the citation, the tag `[standard]` in place of `[memory]` (C-132) and a link to the report. Add the REPORT-0a evidence to C-150, C-71 and C-72 and, if proposed, the revised value, leaving them pending measurement.
- Add to C-138 a proposed supported range per OS, derived from the report's supported-versions input (the oldest tested version on which every facility the backend needs works), marked as a proposal with a link to the report.
- Update any design-document statement that relied on the struck or corrected `[memory]` claims (receive-stamp facilities, don't-fragment options, drop accounting on macOS / Windows), including epic 001's Background, and the clock-bound assumption behind AC-27 / AC-28 if a revision is proposed.
- Record the remaining open item "Does NIC interrupt coalescing bias the stamps measured here?" with owner L1 (task 019) and falsifier "offset shifts when coalescing is toggled", if not already recorded.

## Acceptance criteria
1. C-90 shows either "confirmed by REPORT-0a" or a new value, with citation, `[standard]` tag and link; C-118 and the C-179 reopen count match it.
2. C-150, C-71 and C-72 reference REPORT-0a and show any proposed value.
3. C-138 shows a proposed supported range per OS with a link to the report's supported-versions input.
4. No design document still states a claim struck in `REPORT-0a.md`.
5. Every open item touched has an owner and a falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

Not applicable (C-125): "Tests green in CI, unprivileged", "Every parser touched is fuzzed" and "Probe sources committed" — no code; "Regime stamp fields touched" — none (task 013 copies the table); "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Writing the regime stamp specification (task 013) or wire specification (task 004).
- Measuring interrupt-coalescing bias (task 019).
- Closing C-71 / C-72 / C-150 (L2, task 020).
- Probing versions older than the oldest obtainable one (C-138 limits the probes to the current and the oldest obtainable release).
