# 028b-16 — `soak` command and loss alarm threshold
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-01, 028b-08, 028b-15, epic 019

## Context
- **Loss floor (C-90).** At a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire): one loss per stream per link about every 13 hours at 2 000 packets/s. Tagged `[memory]`, pending a pre-implementation check and L1; it becomes `[standard]` once the citation is checked (C-132).
- **Soak definition (C-118).** 24 hours at the two-bundle load: 64 000 packets/s (2 × 96 channels, 32 streams), about 5.5e9 packets. The statistic is aggregate loss over all streams, where the floor predicts about 58 losses.
- **Threshold (C-91).** Set from the clean-network loss measured at L1 (epic 019): 10 × the measured clean-wire loss (proposal, C-167); alarm when exceeded. Loss above it is a cable-health alarm. Its value is pending that measurement.
- **Counting.** Loss is the per-connection `missing` counter in the stamps (C-47, C-159). A slice counted missing and later reclassified as late (missing −1, late +1, C-108) is not a loss; nor is a slice on another timeline epoch, which has its own counter (C-135, C-159).
- **Window (C-107, C-153).** The soak generates test traffic, so it runs only inside a commissioning window stamped by every module (guard 028b-08), announced with an expiry long enough for the run (24 h by default). The soak is the tool's own measurement, so its result is a valid commissioning result, recorded with the window it was taken in (C-178); this includes the L1 run that proposes C-91.
- **Redundancy decision (C-179).** Clean-medianet loss above 10 × the published floor (C-90), about 580 losses in the 24-hour soak, reopens the redundancy decision. Between 1 × and 10 × the floor it is recorded and investigated as a cabling or host problem and does not.
- **Strict mode (C-147).** A run whose numbers enter the constants table runs in strict mode, so the L1 run that proposes C-91 does.

## Work
- `soak [--duration <h>]` (default 24 h, C-118): check the window and that it will not expire before the run ends (C-153), generate the two-bundle load with the generator of 028b-15, and read `missing` counters from stamps; report aggregate loss over all streams and a per-link breakdown, with the expected count from C-90 for the duration and load.
- Report pass/fail against C-91 once it has a value; before that, report the aggregate against the floor only.
- Classify the aggregate against the published floor for the run's duration and load (C-179): at most 1 × the floor; between 1 × and 10 × (recorded, to be investigated as a cabling or host problem); above 10 × (reopens the redundancy decision). Record the window the run was taken in (C-178).
- When run on the L1 rig, in strict mode (C-147), write the measured clean-network loss as a report in `doc/design/reports/` (C-130) and propose the C-91 value, 10 × that loss (C-167), for the constants table.

## Acceptance criteria
1. A default soak prints an expected aggregate count of about 58 losses (64 000 packets/s × 86 400 s at the C-90 floor, C-118).
2. A link with losses above C-91 is reported by name.
3. A fixture with a missing slice later reclassified as late counts zero losses.
4. Outside a commissioning window the command refuses to start (epic AC-10).
5. After an L1 run in strict mode, a report with `[measured]` evidence gives the clean-network loss and the C-91 value derived from it (C-167).
6. The soak refuses to start when the window expires before the requested duration ends.
7. For a default soak, a fixture aggregate of about 300 losses is reported as between 1 × and 10 × the floor (cabling or host investigation, redundancy decision not reopened), and one above about 580 as reopening the redundancy decision (C-179). The result names the window it was taken in (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Continuous post-hoc loss alarms (028b-18).

## Notes
- C-91's value is pending the L1 measurement (remaining gap, owner epic 019).
