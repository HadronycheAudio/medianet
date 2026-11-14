# 022b-05 — L2 Windows: zero slip after loss, handover and step
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022b-02, epic 020

## Context
- Part of epic 022b (Windows edge L2 certification), L2 certification.
- 022-AC-2 — zero slip: after **injected loss**, a **50 ppm handover** (C-78) and an **announced step**, the marker offset returns to its prior value within the platform's restart spread (C-76; Windows target ≤ 0.5 ms, C-74; the measured value from 022b-02 applies).
- Disturbances are driven through the public injection API and, where needed, the test-only hooks of C-120 (force a stream onto a timeline epoch, send a raw packet on a media or control socket); every injection is stamped.
- Test parameters (C-166, proposals pending first runs): injected loss is 1 % random plus bursts of 8 slices; the no-step tolerance at handover is the platform restart spread on Windows; comparisons use 10 s windows; sample tolerances are measured against the median of the first 60 s. Completed by C-191 (proposals pending first runs): loss bursts are spaced 1 s apart and hit the media path only; the announced step is 5 ms; a 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run. Sync loss has no separate scenario (C-202, superseding C-191's "its own scenario"): the handover run covers it.
- The handover is triggered as a holder loss (C-212): the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. The run asserts emission error within the target E (1 ms, C-72, C-149) while the sink is in holdover between holders (C-202) and frequency continuity at takeover: rate change within C-61 (at most 1 ppm per second) and no phase step; a run in which holdover was not observed is failed, not passed. The sink's clock state is read from its published time view (C-187).
- Who takes over (C-213): a holder-eligible module whose time is usable — Locked, or in holdover with its alignment claim not lapsed (C-65, C-169). Such a candidate takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time. The rig's second holder-eligible Linux node has usable time at the trigger, so the epoch must not change. After the loss every candidate is in holdover and the lowest module id wins (C-221); the epic 020 harness gives the second node the lowest id among the rig's holder-eligible modules (020-17).
- The 50 ppm (C-223): the measured natural difference between the two holders' crystals plus an injected drift, totalling 50 ppm; both parts are recorded in the run report. Per-sink emission error in holdover is an L2 assertion only; the L3 rig cannot observe it (C-220).
- The run is in strict mode (C-147) with A computed and recorded by the harness (C-148).
- Two-loop timebase (epic 018): loop 1 H→domain, loop 2 S→H; rate-loop bandwidth well below 1 Hz (C-68).
- Interrupted after 8 consecutive missed slices (C-38, C-102); dead after 2 s (C-39, C-111).

## Work
1. With the Windows sink live on the rig, measure the steady marker offset against the reference as the median of the first 60 s (C-166).
2. Inject (a) packet loss on the media path only, 1 % random plus bursts of 8 slices spaced 1 s apart (C-166, C-191); (b) a clock-holder handover with a 50 ppm rate difference, made up of the measured natural crystal difference plus an injected drift, both recorded (C-223), triggered 60 s into the run (C-191) as a holder loss: the old holder stops answering and the new holder takes over after the sink's holdover (C-212); record the sink's clock state through the handover, emission error during holdover, the sink's rate change and phase at takeover (C-61, C-212), the new holder's clock state at the trigger and the timeline epoch before and after the handover (C-213); (c) an announced step of 5 ms (C-191). After each, wait for reconvergence and measure the marker offset over a 10 s window starting when the injected event ends (C-191).

## Acceptance criteria
1. **(022-AC-2.)** After each of the three disturbances, the marker offset returns to its prior value within the restart spread measured in 022b-02 (≤ 0.5 ms), compared over a 10 s window starting when the injected event ends (C-166, C-191); across the handover no step larger than that spread appears (C-166).
2. Reconvergence time per disturbance and the disturbance sizes are recorded `[measured]` in `doc/design/reports/` (C-130).
3. Each injection appears in the stamp as an injection (C-120).
4. **(022-AC-2, handover.)** Holdover between holders is observed in the handover run, emission error stays within the target E of 1 ms throughout it (C-202), and frequency is continuous at takeover: rate change within C-61 (at most 1 ppm per second) and no phase step (C-212). A run in which holdover was not observed is recorded as failed, never passed.
5. **(022-AC-2, handover.)** The new holder's time was usable at the trigger (Locked, or in holdover with its alignment claim not lapsed) and the timeline epoch is unchanged across the handover (C-213); an epoch bump in this run is a failure.
6. **(022-AC-2, handover.)** The run report states the measured natural crystal difference and the injected drift, summing to 50 ppm (C-223).

## Definition of done
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Disturbance injection tooling (L2 harness, epic 020).
- The drift-ramp scenario (022b-10).
