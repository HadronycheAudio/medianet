# 008b-07 — Handover without a good rate estimate
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008b-06
## Context
Open sub-question owned by this epic: "At handover, can a successor with usable time still have a rate estimate poor enough to break C-61 at followers?" Falsifier: a successor with usable time (C-213) causes a correction-rate change above 1 ppm/s at a follower; then epic 011 must tighten eligibility.

Settled part, C-213 `[decision]`: holder candidates are holder-eligible modules whose time is usable, that is `Locked`, or in `Holdover` with the alignment claim not lapsed (C-65, C-169); such a candidate takes over with frequency continuity and the timeline epoch is kept (eligibility consumed by epic 011). Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113); followers then re-acquire as after any epoch change, so a successor in `Acquiring`, or in `Holdover` past the lapse, never performs a frequency-continuous handover. After a holder loss every candidate is in `Holdover` when the election runs (C-195, at most C-37), so the successor takes over from `Holdover` (C-212, C-213). Handover is frequency-continuous: the new holder starts at the old holder's rate and relaxes to its own crystal within C-61 `[decision]`. A step > 2 ms (C-67) is an epoch change, never slewed.

## Work
1. Simulation study `case_06b_handover_poor_estimate` (multi-holder model): Locked successors with a deliberately degraded rate estimate (just Locked; rate error injected at 5, 20 and 50 ppm where the bound still allows Locked); 50 ppm crystal difference (C-78). Also run the C-113 path (no candidate with usable time: candidates `Acquiring`, or in `Holdover` past the lapse) to confirm it leads to an epoch bump and normal re-acquisition. Repeat the degraded-estimate runs with the handover triggered as a holder loss (C-212), the successor taking over from `Holdover` with the epoch kept (C-213), so its rate estimate has aged through the holdover between holders. Any run with the C-113 bump forced on a usable candidate is labelled a comparison and does not decide the conclusion.
2. Measure the worst correction-rate change at followers and any phase step.
3. Conclude: either frequency-continuous handover holds for every successor with usable time (so the C-213 eligibility suffices), or state the tighter eligibility condition for epic 011. Write the answer with evidence tag `[measured]` (simulation) into `doc/design/reports/008-simulation-results.md` under "Open questions".

## Acceptance criteria
1. The study runs as a named test and asserts the documented conclusion (e.g. "a successor with usable time, Locked or in Holdover before the lapse, never causes a correction-rate change > 1 ppm/s at followers").
2. If a counter-example exists among successors with usable time, the results file states the proposed tighter eligibility condition, and the test asserts that under it the successor is not eligible and the C-113 path (epoch bump) is taken instead.
3. The C-113 path shows an epoch bump, no phase step under the old epoch, and re-lock within C-63. A candidate in `Holdover` past the lapse (C-65, or C-169 with E unset) takes the C-113 path; one before the lapse takes over with the epoch kept (C-213).
4. The answer names its owner (epic 011 for the eligibility rule) and its falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Implementing the election rule (epic 011).
