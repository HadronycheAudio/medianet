# 020-16 — Loss and step disturbance scenarios
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-14

## Context
The L2 procedure applies disturbances, each followed by a check that the marker offset returns to its prior value `[decision]`. This ticket covers two of the three: **injected packet loss** and **an announced timeline step** (the third, the 50 ppm handover, is 020-17).

Rules `[decision]`:
- A step larger than 2 ms (C-67) is never slewed but announced as a timeline-epoch change; `position = (network_time − epoch_origin) × rate`, so an announced step changes `epoch_origin`.
- **Zero accumulated slip (C-76):** after each disturbance the offset returns to its prior value, on Linux within ±1 sample (10.4 µs at 96 kHz, C-01); on macOS and Windows within that platform's restart spread (C-74). A one-sample slip passes on Linux and is reported as a slip event (C-166). Offsets before and after are compared over 10 s windows (C-166).
- Run for one hour per condition.
- Loss classes (C-108): at the deadline an absent slice is counted missing; producer and consumer publish per-connection counters in their regime stamps: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-47, C-159). A slice on another timeline epoch restarts the dead clock, is not a miss and has its own counter (C-135).
- Interruption: 8 consecutive missed slices (4 ms at defaults) raise an interrupted event (C-38); only slices absent at the deadline count as misses (C-102). The dead threshold is 2 s from the last slice received with content or a marked gap (C-39, C-111).

Injected loss is 1 % random plus bursts of 8 slices (C-166); a burst of 8 slices equals the interrupted threshold (C-38). Settled by C-191: the bursts are spaced 1 s apart and hit the media path only; L2 has no separate sync-loss scenario: sync loss is covered by the holdover assertion of the handover run (C-202, ticket 020-17), and `loss` stays on the media path; the announced step is 5 ms; the 10 s comparison window after an event starts when the injected event ends.

## Work
- Scenario `loss`: during a 1-hour run, inject media packet loss on the path from the reference holder to the sink using the injection API of epic 017: 1 % random loss plus bursts of 8 consecutive slices (C-166), the bursts spaced 1 s apart, on the media path only; sync exchanges are never dropped by this scenario (C-191). All episodes stay well below the dead threshold (C-39). Record each episode's timestamps and slice counts as disturbance events.
- Cross-check each loss episode against the sink's per-connection counters (C-47): the lost slices appear as missing. Markers that fell in lost slices are counted as expected-absent, not as detection failures.
- Scenario `step`: during a 1-hour run, inject a timeline step of 5 ms (C-191; larger than C-67's 2 ms) through the holder (public injection API of epic 017), which the runtime announces as an epoch change (C-67). Record the event.
- After each event, run the slip detection of 020-06 (offset over the 10 s windows before and after, C-166; the window after starts when the injected event ends, C-191) and report slip events and the C-76 verdict.
- Cross-check the step against the counters: slices on the old epoch appear in the other-epoch counter, not as missing, and raise no interrupted event (C-135, C-102).
- Verify that the announced step is not slewed: the sink must not show a ramped offset over the step.

## Acceptance criteria
1. `hadronyche-l2 run loss` and `hadronyche-l2 run step` each run 1 hour and report, per disturbance, the offset before and after and the C-76 verdict (Linux ±1 sample).
2. On a simulated device with a deliberate 2-sample slip injected after one loss episode, the run reports exactly that event as a slip and fails C-76.
3. The `step` scenario injects 5 ms (C-191); a step of 1 ms (below C-67's 2 ms) is refused with a message, since it would be slewed rather than announced.
4. In the `loss` recording the 8-slice bursts start 1 s apart, and every injected drop recorded in the stamped injection log (C-120) is on the media path, none on the sync path (C-191).
5. The sink's missing-slice counter (C-47) increases by exactly the number of injected lost slices per episode; each burst of 8 consecutive lost slices raises exactly one interrupted event and isolated random losses raise none (C-38, C-102, C-166). Across the step, the other-epoch counter increases and the missing counter does not (C-135).
6. CI tests for both scenarios (shortened duration) pass unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The 50 ppm handover (020-17) and the drift ramp (020-18).
- The certification runs on the rig (020-22).
