# 027-20 — L3 results document and constants
**Epic:** 027-L3-cross-sink-rig · **Effort:** M · **Depends on:** 027-09, 027-10, 027-11, 027-12, 027-13, 027-14, 027-15, 027-16, 027-17, 027-18, 027-19, 027-22, 027-23, 027-24, 027-25

## Context
- Epic 027 deliverable: document `doc/design/reports/l3-results.md` (location fixed by C-130), containing the measured skew, the rig, and the values written back to the constants table `doc/design/02-constants.md` (C-79, the device's E).
- C-79 (pending measurement, owner L3): the 99.9th percentile of absolute skew between the two sinks' outputs is at most the sum of their two E values, for 1 hour and across restarts of either; the maximum is reported beside it. E (C-73) = 99.9th percentile of |actual emission − (T(p)+A)| over 1 h, max beside. Linux sink target E = 50 µs (C-70); device E declared per device (epic 026).
- Open sub-question owned by this epic: "Is the skew within the sum of the two E values?" Falsifier: skew exceeds the sum — one sink's D or E is dishonest; find which with L2 (epic 020).
- The cross-sink evidence and group-E certification procedure produced here are what the commissioning tool (epic 028) packages for operators; when packaged it runs inside a declared commissioning window (C-107), and its group-certification results are valid commissioning results, each recorded with the window it was taken in, while ordinary production measurements are unqualified during that window (C-178).
- This is a final, consolidating ticket: it does not re-run scenarios unless a full clean campaign is needed for the record.

## Work
1. Run a full campaign on one rig configuration and one software version: calibration (027-10), `steady`, `restart-linux`, `restart-device`, `power-cycle-device`, `handover`, `loss`, `interrupt`, `epoch`, `holdover`, `admission`; archive all reports.
2. Write `doc/design/reports/l3-results.md`:
   - rig summary and link to the parts list / diagram (027-09); capture inter-channel skew `[measured]`;
   - harness self-check result (37 µs ± 2 µs, 027-AC-8);
   - one table per scenario: p50, p99.9 |skew|, max |skew|, limit, pass, plus drift slope / step / recovery values where applicable — all `[measured]`;
   - both sinks' complete stamps (profile, timestamp source with capability state, D and method, E, certification state) and the C-159 counters per scenario;
   - the admission test as run under C-194 (required E below the device's own declared E), with both E values and the required E used;
   - the A used in each run with its inputs (C-148), and the statement that every run was strict (C-147) with both sinks commissioned (C-140);
   - the limit computed from target E values (C-149);
   - the test parameters used — handover tolerance, 10 s comparison windows, 60 s baseline and post-restart window, loss pattern, D spread as max − min (C-166); burst spacing 1 s on the media path only, announced step 5 ms, comparison windows starting at the event end, handover 60 s into the run as a holder loss with holdover observed, each sink's rate change at takeover within C-61 and no phase step, measured per channel by 027-25 with its stated resolution (C-212), the skew within the sum of the two E values through holdover and takeover as the L3 handover assertion, with no per-sink emission-error line (C-220), the measured natural frequency difference between the two holders' crystals (no drift injected at L3; the L2 test's 50 ppm is that natural difference plus an injected drift, C-223), the takeover by a holder with usable time and the epoch kept (C-213), calibration repeatability against the mean of 10 calibrations (C-191) — with whether each held in practice, since C-166 and C-191 are proposals pending first runs;
   - the device's restart spread measured at L3 (027-12) beside the spread it declares (C-191), `[measured]`;
   - answers to the epic's open sub-questions (skew within sum of E; D across power cycles; capture inter-channel skew at 96 kHz; macOS/Windows inclusion — link to 027-21 if still open);
   - the no-step and recovery tolerance used for the Linux–device pair, the larger of the two sinks' own tolerances (C-203), with both sinks' figures beside each verdict;
   - the working assumptions used where the epic is still open (recording file format), each with its owner, so a later ruling can be checked against the data.
3. Constants table first: update C-79 (confirmed or revised, status moved from pending-measurement) and the device's E entry; if the device's E is revised, state the reason.
4. Update other design documents where measured behaviour differs.
5. Any failing criterion becomes an open item with owner and falsifier.

## Acceptance criteria
1. `l3-results.md` is committed in `doc/design/reports/` and contains a `[measured]` skew figure (p99.9 and max) for `steady` and for each restart scenario, or the device's E is revised with a stated reason.
2. Constants table updated for C-79 and the device's E in the same change, before any other doc edit.
3. Every epic acceptance criterion 027-AC-1 to 027-AC-11 has a row in the results with pass/fail and a link to its report.
4. Every factual claim in the document carries an evidence tag.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented
- [ ] The rig is photographed or diagrammed and its parts listed
- [ ] `l3-results.md` is committed with a `[measured]` skew figure, or the device's E is revised with a stated reason
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Tests green in CI, unprivileged" — no code; "Every parser touched is fuzzed" — no parser; "The skew analysis runs in CI on recorded two-channel files" — no analysis code (fixtures come from 027-08 to 027-18).

## Out of scope
- Operator-facing commissioning tool (epic 028).
- macOS / Windows sinks (027-21).
