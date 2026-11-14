# 008b-06 — Case 06: handover at 50 ppm
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-16, 008b-05
## Context
Simulation case 6: handover with a 50 ppm crystal difference (the L2 handover figure, C-78) — no phase step, rate change within the limit. On the L2 rig the 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report (C-223); in this simulation there is no natural difference, so the whole 50 ppm is injected through the two-clock model, and the results file says so. Epic criterion A4 (system AC-5): handover at 50 ppm difference stays within 1 ppm/s with no step.

Rule `[decision]`: handover is frequency-continuous; the new holder starts at the old holder's rate and relaxes to its own crystal within C-61 (1 ppm/s). At followers, C-60 and C-61 bound the correction term, not the drift estimate (C-119). In the POC the new holder switched to its own crystal at once, a rate step of up to the crystal difference `[code]`.

Trigger, settled by C-212 and C-202 `[decision]`: the L2 handover is a holder loss. The old holder stops answering, followers pass through about 2 s of holdover (C-37 election silence plus the C-33 election window) and the new holder takes over; emission error stays within the target E during that holdover, frequency is continuous at takeover, and a run in which holdover was not observed is failed. This case is the simulation forecast of that run, so it uses the same trigger. Followers enter holdover after the holdover entry delay (C-195) proposed by 008b-05, at or below C-37.

Every candidate is therefore in `Holdover` when the election runs. Candidacy, settled by C-213 `[decision]`: a holder-eligible module whose time is usable (Locked, or in `Holdover` with its alignment claim not lapsed, C-65, C-169) takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time. About 2 s into holdover the successor's claim is far from the ≈ 25 s lapse, so this case asserts a frequency-continuous takeover from `Holdover` with the epoch kept. With every candidate in `Holdover`, the successor is the one with the lowest module id (C-221: a Locked candidate would be preferred, but none is Locked after a holder loss). The other reading (a candidate in `Holdover` does not count and the C-113 bump is taken) is kept only as a labelled comparison run.

## Work
1. Named test `case_06_handover`: multi-holder model, old holder at 0 ppm, successor at ±50 ppm, the whole difference injected (C-223), two further followers at other offsets; successor Locked before handover and given the lowest module id among the holder-eligible nodes, so that C-221 elects it; `WIRED_PROFILE` (C-94); at least 20 seeds. The handover is triggered as a holder loss (C-212): the old holder stops answering at a scheduled time and the successor takes over after C-37 plus C-33; `holdover_after_ns` is the value proposed by 008b-05 (C-195). The verdict run uses the C-213 candidacy (the successor in `Holdover` with its claim not lapsed takes over, epoch kept). Repeat it once as a comparison, labelled as such in the test name and the results file, with the C-113 epoch bump forced; its figures do not decide the verdict.
2. Record at the successor and at every follower: phase step at and after handover, rate change per second at the successor, correction-term change per second at followers, honest-lock violations, and per follower the holdover interval between holders with its maximum `bound_ns` and |true error|, and the timeline epoch at every node before and after takeover; record the module id of the node that took over.

## Acceptance criteria
1. Phase step 0 ns at every update on every node across the handover, and the timeline epoch is unchanged at every node (C-213); the node that took over is the lowest-id holder-eligible node (C-221).
2. Successor rate change ≤ 1 ppm/s at all times after handover; follower correction-term change ≤ 1 ppm/s while Locked.
3. 0 honest-lock violations.
4. Followers that were Locked before handover are Locked again within 10 s (C-63) of their first exchange with the new holder.
5. Every follower is observed in `Holdover` between holders; a seed in which it is not fails the test (C-212).
6. During that holdover, |true error| ≤ `bound_ns` ≤ the target E (C-70, read from `ServoConfig`, C-149) at every sample (C-202).
7. The results file `doc/design/reports/008-simulation-results.md` states, tagged `[measured]` (simulation), that the C-213 takeover from `Holdover` is frequency-continuous with the epoch kept, or records the counter-example for the design owner; the comparison run's figures (epoch bump, re-lock time) appear beside it, labelled as a comparison. The file states that the 50 ppm difference was wholly injected in simulation (C-223).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Successor with a poor estimate of the old rate (008b-07).
