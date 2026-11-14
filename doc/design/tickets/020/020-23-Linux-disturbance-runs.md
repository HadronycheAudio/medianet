# 020-23 — Linux disturbance runs
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-16, 020-17, 020-18, 020-22

## Context
Second half of the Linux L2 certification, on the reference rig (020-21), with the kernel chosen in 020-22, in strict mode (C-147) and with A computed by the harness (C-148). Each condition runs one hour `[decision]`.

Criteria `[decision]`:
- **AC-26 (Linux), slip part:** zero slip after loss, the 50 ppm handover (C-78) and an announced step: after each disturbance the offset returns to its prior value within ±1 sample (C-76; 1 sample = 10.4 µs at 96 kHz, C-01); a one-sample slip passes and is reported as a slip event (C-166); offsets compared over 10 s windows (C-166). Marker offset within ±2 samples of the median of the first 60 s over each run (C-75, C-166); E ≤ 50 µs (C-70, C-73); MTIE ≤ 10 µs up to 1 s and ≤ the 50 µs target overall (C-77, C-149). Injected loss: 1 % random plus bursts of 8 slices (C-166), spaced 1 s apart, on the media path only (C-191).
- **AC-5 (hardware part):** during the 50 ppm handover the steered ratio changes by at most 1 ppm/s (C-61) and the marker offset shows no step beyond ±1 sample (C-166); emission error stays within the 50 µs target while the sink's clock is in holdover between holders (C-202, C-149). The handover is triggered as a holder loss (the old holder stops answering, about 2 s of holdover, the new holder takes over); a run in which holdover was not observed is failed, not passed (C-212).
- A step larger than 2 ms (C-67) is announced as a timeline-epoch change, never slewed.
- `drift-ramp` (C-119, criterion 020-A9): injected crystal drift swept ±100 ppm at no more than 0.5 ppm/s; pass is E held throughout; C-60 and C-61 checked on the servo's correction term, not on the drift estimate.
- Disturbance parameters completed by C-191: loss bursts spaced 1 s apart on the media path only (no separate sync-loss scenario at L2; it is covered by the handover run's holdover assertion, C-202); announced step 5 ms; handover triggered 60 s into the run (as a holder loss, C-212); each 10 s comparison window after an event starts when the injected event ends.

## Work
- Run `loss`, `handover-50ppm`, `step` and `drift-ramp`, 1 hour each.
- Produce record fragments (020-13) with per-disturbance offsets, C-76 verdicts, AC-5 ratio and step results, E, C-75 and C-77.
- Commit a short excerpt around the handover as a CI fixture and a CI test reproducing the AC-5 numbers from it.
- Merge all fragments from 020-22 and this ticket into the complete Linux certification record (JSON + Markdown).

## Acceptance criteria
1. For each of loss, handover and step: offset before and after, and the C-76 verdict (±1 sample) `[measured]`.
2. Handover: maximum ratio rate of change vs 1 ppm/s, offset change across the handover, and the observed holdover interval with the largest emission error in it vs 50 µs (C-202), with the AC-5 verdict; a handover run with no observed holdover is failed and repeated, not passed (C-212) `[measured]`.
3. Drift-ramp: E over the sweep against 50 µs as the pass verdict (C-119), E per drift bin, and the correction-term verdicts for C-60 and C-61 `[measured]`.
4. A complete certification record exists containing every field of 020-13 and all scenario results, with the regime stamps of the sink and of the holder it followed (C-191), stamped (commissioned, the only state that gives a numeric E, C-140; not *unverified* on the reference rig; strict mode present, C-147).
5. The handover CI fixture test passes unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Regime stamp fields touched are complete and documented — not applicable (C-125): this ticket adds no regime stamp field; it copies the stamps of the sink and the reference holder into the certification record, whose fields are documented in `report.rs`
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Writing `l2-results-linux.md` and the constants table (020-24).
- Cross-sink skew (epics 021, 022).
