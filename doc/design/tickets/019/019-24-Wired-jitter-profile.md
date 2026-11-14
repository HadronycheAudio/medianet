# 019-24 — Wired jitter profile
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07, 019-08

## Context
The servo and alignment simulation uses a provisional, synthetic wired jitter profile (C-94): base one-way delay 100 µs; exponential jitter with 20 µs mean; 1 % of packets in bursts delayed by 1 ms. C-94 names L1 (task 019) as the owner that replaces it with the measured profile. Its falsifier: the measured profile differs enough to change a simulation verdict.

L1 is two Linux hosts on one wire through one switch, IPv4 only (C-100). One-way figures come from origin-to-arrival lateness in the record files; they are accurate to half the path asymmetry, and the runtime publishes a worst-case asymmetry bound with each figure. Receive stamps are kernel software stamps (C-131 source `kernel`). The `baseline` scenario (019-07) carries 2 000 packets/s per direction; `two-bundles` (019-08) carries 64 000 packets/s to one consumer port.

Epic 019 criterion **019-C**: a measured wired jitter profile is recorded in the same terms as C-94 (base delay, jitter distribution, burst fraction and delay) together with the empirical distribution, and handed to the simulation owner.

## Work
- From the records of a 1-hour `baseline` run and a 1-hour `two-bundles` run on the hardware pair, extract per-packet one-way delay with its asymmetry bound.
- Compute: base delay (the minimum and the 1st percentile), the distribution of excess delay over the base, a fit of an exponential to the bulk with its mean, and the fraction and delay of packets in the tail beyond that fit (the burst component). State the tail cut-off used.
- Export the empirical distribution (histogram or CDF) as a data file next to `doc/design/reports/l1-results.md` (C-130) in a form the simulation can load, with the stamp of the runs.
- Report a goodness-of-fit statement: whether the C-94 form (exponential plus bursts) describes the measurement, and where it does not.
- Record under C-94 in `doc/design/reports/l1-results.md`, tagged `[measured]`, for each load separately.

## Acceptance criteria
1. Dry-run of the analysis on synthetic records generated from the C-94 profile recovers base delay, jitter mean and burst fraction within 10 % of the generating values.
2. `l1-results.md` has a C-94 section with base delay, jitter mean, burst fraction and burst delay for both loads, each with the asymmetry bound and stamp, tagged `[measured]`.
3. The empirical distribution file exists and is referenced from that section.
4. The section states whether the C-94 form fits, and the hand-off to the simulation owner is listed for 019-22.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Re-running the simulation with the measured profile (simulation owner).
- Writing C-94 to the constants table (019-22).
