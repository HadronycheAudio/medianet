# 021-24 — L2 common criteria on macOS
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** M · **Depends on:** 021-23
## Context
Epic criterion 021-AC2: L2 common criteria on the reference device:

- marker offset constant within the macOS restart spread of 50 µs (C-75, C-74);
- zero accumulated slip, offset returns to its prior value within 50 µs (C-76), after injected loss, a handover between holders whose crystals differ by 50 ppm (C-78) and an announced step of 5 ms (C-191);
- MTIE at most 10 µs for intervals up to 1 s and at most E for any interval (C-77); macOS E target 100 µs (C-71).

- **"At most E"** is the platform target, 100 µs (C-149), never the E measured in the same run.
- Test parameters (C-166): no-step tolerance at handover is the macOS restart spread (50 µs); comparison windows of 10 s; injected loss 1 % random plus bursts of 8 slices; sample tolerances are measured against the median of the first 60 s.
- Test parameters, completed (C-191): loss bursts are spaced 1 s apart and hit the media path only; the announced step is 5 ms; each 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run.
- **Handover as holder loss** [decision, C-202, C-212]: the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. The run asserts emission error within the 100 µs target E during holdover and frequency continuity at takeover, meaning a rate change within C-61 (at most 1 ppm per second) and no phase step; a run in which holdover was not observed is failed. Sync loss has no separate scenario (C-202). The rig therefore needs a second holder-eligible Linux node beside the old holder (C-212). Per-sink emission error in holdover is an L2 assertion only; the L3 rig cannot observe it (C-220).
- **The 50 ppm** [decision, C-223]: the measured natural difference between the two holders' crystals plus an injected drift, totalling 50 ppm; both parts are recorded in the run report (the epic 020 harness does this, 020-17, 020-28).
- **Who takes over** [decision, C-213]: a holder-eligible module whose time is usable (Locked, or in holdover with its alignment claim not lapsed, C-65, C-169) takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time. The second Linux node is Locked when the old holder stops, so the epoch must not change across the handover. After the loss every candidate is in holdover, so the lowest module id wins (C-221); the epic 020 harness gives the second node the lowest id among the rig's holder-eligible modules (020-17). The macOS edge is never a candidate (edge only).
- Runs are in strict mode (C-147); the harness computes A with the commissioning rule and records it with the results (C-148).
- The step scenario, including how the step is announced, is run as the epic 020 harness defines it.

Fault injection is expected to be available on macOS. The drift-ramp scenario of C-119 is a separate ticket (021-29).
## Work
- Run the epic 020 harness scenarios on the reference Mac in strict mode: marker offset across restarts; injected loss on the media path (1 % random plus bursts of 8 slices spaced 1 s apart); 50 ppm clock-holder handover triggered as a holder loss 60 s into the run, on a rig with a second holder-eligible Linux node (C-212), with emission error, the sink's clock state and the timeline epoch logged through the holdover and takeover; announced 5 ms step; MTIE computation. Record the A used.
## Acceptance criteria
1. Marker offset across restarts constant within 50 µs (C-75).
2. After each of injected loss, 50 ppm handover and announced 5 ms step: zero accumulated slip and offset back within 50 µs of its prior value (C-76, C-78), judged over a 10 s comparison window starting when the injected event ends (C-191).
3. Handover run: the run report states the measured natural crystal difference and the injected drift, summing to 50 ppm (C-223); holdover is observed between holders (otherwise the run fails), emission error stays within the 100 µs target E throughout it, and frequency is continuous at takeover: rate change within C-61 and no phase step (C-202, C-212). The takeover is by the second holder-eligible Linux node and the timeline epoch is unchanged across it (C-212, C-213).
4. MTIE ≤ 10 µs for observation intervals up to 1 s (C-77); MTIE over all intervals ≤ the 100 µs target E (C-77, C-149).
5. The results record strict mode (C-147) and the A used (C-148).
## Definition of done
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
AC-27 run (021-23). Drift ramp (021-29). The harness and fault injectors (epic 020).
