# 026b-20 — Declare restart spread in stamp
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** XS · **Depends on:** 026b-14, 026a-11, 026a-12

## Context

The restart spread of D is how far a sink's plumbing delay D moves from one restart to the next. Hosted platforms have target values (C-74, over 20 restarts), and the L2 tolerances for the round-trip marker and for slip after a disturbance are stated against the platform's restart spread (C-75, C-76). A device has no platform target: **an embedded device declares its own restart spread, measured at L3** (C-191, proposal pending first runs). The restart spread of D is max − min of D over the restarts (C-166). A measurement taken by the commissioning tool inside a commissioning window is a valid commissioning result, recorded with the window it was taken in (C-178). When the device is paired with a different sink at L2 or L3, the tolerance is the larger of the two sinks' own no-step and recovery tolerances, and both sinks' figures are reported beside each verdict (C-203); the device's side of that comparison is stated against the restart spread declared here.

D itself, its components and its method are declared in 026b-14 (C-170). The L3 rig belongs to task 027; this ticket gives the device a declared restart-spread figure, carries it in the stamp and makes the device restartable by the rig.

Still open: none created here. How epic 013 encodes a figure that has not been measured yet is epic 013's choice.

## Work

1. Add a declared restart-spread field to the firmware profile (026a-04) next to D and E, published in the device stamp (026a-11) using epic 013's field for it.
2. Until L3 has measured it, the field states that no value is declared yet; after the L3 run of task 027, set it to the measured max − min (C-166, C-191) with the run it came from.
3. Make sure a restart driven by the rig (power cycle or the reset path of 026b-17) recomputes D from its components (026a-10) and re-publishes it, so each restart yields one D sample for the spread.
4. Record the declared value in `doc/design/reports/` (C-130) once it exists.

## Acceptance criteria

1. Host test: the restart-spread field appears in the stamp decoded by the hosted collector (026a-12), both before a value is declared and with a declared value.
2. After each restart on the board, the stamp carries a freshly computed D, so the L3 rig can compute max − min over the restarts (C-166).
3. Once task 027 has run L3, the declared restart spread in the collector's record equals the L3 figure, recorded `[measured]` with the run (C-191).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- The L3 rig and the restart-spread measurement itself (task 027).
- Measuring D and certifying E (026b-14).
