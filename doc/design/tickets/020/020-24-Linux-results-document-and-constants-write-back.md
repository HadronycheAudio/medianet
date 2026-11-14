# 020-24 — Linux results document and constants write-back
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-19, 020-20, 020-21, 020-22, 020-23

## Context
Epic 020 deliverable `[decision]`: **document `l2-results-linux.md` in `doc/design/reports/` (C-130)**: the Linux certification record, and the measured values written back to the constants table for **C-68, C-70, C-74, C-75, C-76, C-77, C-81**.

Epic criterion 020-A8: results are written back to the constants table with the `[measured]` tag, or the target is revised with a stated reason.

Constants and their proposed values `[decision]`:
- C-68: rate loop bandwidth well below 1 Hz.
- C-70: Linux E target 50 µs = 20 clock + 20 tracking + 10 D.
- C-74: Linux restart spread of D at most 25 µs over 20 restarts.
- C-75: round-trip marker offset constant, Linux within ±2 samples.
- C-76: zero accumulated slip, Linux within ±1 sample after each disturbance.
- C-77: MTIE at most 10 µs for τ up to 1 s, at most E for any τ.
- C-81: typical Linux D about 1 ms.

At 96 kHz (C-01) one sample is 10.4 µs. The epic says "pass numbers (proposals until measured)" and "the numbers E and D stop being proposals" at L2.

The certified E is what the sink then declares; a sink without a certified E declares E as unknown and is refused by any presentation group that sets a required E (C-103).

This epic provides the certified Linux reference sink compared against by L3 (epic 027), the harness reused by epics 021 and 022, and the group-E certification procedure packaged by epic 028.

## Work
- Write `doc/design/reports/l2-results-linux.md` (C-130) containing: the complete certification record (Markdown rendering from 020-23); the rig reference (020-21); the stock vs real-time kernel answer (020-22); the calibration-method decision (020-11); the MTIE-mask decision (020-19); the spectral-criterion decision (020-20); the idle/active threshold measured in 020-22; the C-166 test parameters as applied (no-step tolerance, 10 s comparison windows, injected loss pattern, 60 s restart window, D spread as max − min, tolerances against the median of the first 60 s, one-sample slip passing as a slip event) and the C-191 parameters as applied (loss bursts 1 s apart on the media path only, 5 ms step, handover 60 s into the run, comparison window after an event starting when it ends, calibration repeatability against the mean of the 10 calibrations, "both stamps" as the regime stamps of the sink and of the holder it followed), and the C-202 holdover assertion of the handover run in place of a separate sync-loss scenario, with the handover triggered as a holder loss and the observed holdover interval (C-212); the value of A and its inputs for each run (C-148); the strict-mode state of each run (C-147); the certification procedure as executed, so epic 028 can package it (inside a declared commissioning window, C-107, where its measurements are valid commissioning results recorded with the window they were taken in, C-178).
- Update the constants table first, from strict-mode runs only (C-147): for each of C-68, C-70, C-74, C-75, C-76, C-77, C-81, either record the measured value with the `[measured]` tag and a reference to `l2-results-linux.md`, or revise the target with a stated reason.
- Update other design documents where measured behaviour differs.
- Commit the Linux certification record (JSON) next to the document.

## Acceptance criteria
1. `l2-results-linux.md` exists and contains every section listed in Work.
2. Each of the 7 constants C-68, C-70, C-74, C-75, C-76, C-77, C-81 has either a `[measured]` value or a revised target with a reason (epic criterion 020-A8).
3. Every factual claim in the document carries an evidence tag.
4. The Linux certification record is committed.
5. Every open sub-question of the epic (calibration method, stock kernel, MTIE mask, spectral criterion, idle/active threshold; the sync-loss question is settled by C-202) is answered in the document or carried as an open item with an owner and a falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The Linux certification record is committed
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- macOS and Windows results (epics 021, 022).
- Packaging the procedure in the commissioning tool (epic 028).
