# 018-18 — Regime stamp fields for ALSA edge
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-11, 018-14, 018-16, epic 013

## Context
- The stamp format is owned by epic 013. Epic 018 fills these fields: **audio device, driver, device rate, buffer and period sizes, D and its components and method, E self-monitored (99.9th percentile and maximum), regression residual, ratio statistics, certification state (*uncertified*, *self-monitored*, *commissioned*)**, plus the capability states of strict mode: real-time scheduling on the data and clock threads, memory lock, CPU idle restriction (C-127), each present, absent, unverified or not requested (C-129).
- Transport (C-18): the stamp travels in a dedicated control message split into sections that each fit one packet at the fallback MTU, at most 1 252 bytes of UDP payload (C-17); it is multicast on any regime change, refreshed every 10 s, and fetchable by unicast request; the hello carries only a stamp version number.
- Which of these fields are regime changes (immediate multicast, version bump) and which are values carried by the 10 s refresh is owned by epic 013 (remaining gap). Until 013 rules, this ticket assumes: configuration fields, D and its method, certification state and lacking features are regime changes; E, residual and ratio statistics are refreshed values.
- "Ratio statistics" is not defined by the epic or the constants table (remaining gap, owner epic 013). This ticket assumes minimum, maximum and mean ppm deviation from nominal over the 10 s refresh interval.
- Certification failures go into the regime stamp. Epic criterion 018-A8: the sink runs unprivileged in best-effort mode and **stamps what it lacks**.
- **C-178** (completes C-153): while a commissioning window is open, ordinary production measurements in the stamp (latency, loss, alignment figures) are unqualified; E values recorded while a window is open are marked as such. The commissioning tool's own measurements (D, restart spread) are valid commissioning results recorded with their window (018-11). The lateness and loss figures are flagged in 018-24 and 018-23.
- The hosted backend (epic 015) sends the stamp message (C-144); this ticket fills the edge's fields.
- E = 99.9th percentile with maximum beside it (C-73); blank until locked.
- A clock unlock does not change the certification state: a *commissioned* sink stays *commissioned* while its claim is suspended (C-180).
- Per-connection health counters (C-47) are ticket 018-23.

## Work
1. Populate all listed fields for `AlsaSink` and `AlsaCapture` from config, `DeclaredDelay` (018-11), `EmissionStats` (018-05), `DeviceLine::residual_ns`, ratio statistics as assumed above, and `CertificationState` (018-06).
2. Add the capability set from 018-12 and 018-25 (four states, C-129) and certification failures.
3. Flag E statistics recorded while a commissioning window is open as unqualified (C-153, C-178); a commissioning-measured D carries the window it was taken in, not an unqualified flag (C-178).
4. Mark each field as regime change or refreshed value per the assumption above, in one table, so the 013 ruling changes one place.
5. Check that the edge's fields fit within sections of at most 1 252 bytes (C-17, C-18).
6. Document every field (meaning, unit, source, change class) in the stamp documentation of epic 013.

## Acceptance criteria
1. A stamp produced by a fake-backend sink contains all 9 field groups listed above (device, driver, device rate, buffer and period sizes, D with components and method, E p99.9 and max, regression residual, ratio statistics, certification state) with units.
2. **Epic criterion 018-A8:** an unprivileged run's stamp shows each missing RT feature as absent, and a capability disabled by configuration as not requested (C-129).
3. Before lock, E fields are blank, not zero.
4. Certification state shows *commissioned* only when a commissioning D is loaded, and still shows *commissioned* while the clock is unlocked (C-180).
5. Every section carrying edge fields serialises to at most 1 252 bytes of UDP payload.
6. A change of certification state bumps the stamp version; a change of E alone does not.
7. Round-trip through the epic 013 stamp serializer is lossless.
8. E recorded while a commissioning window is open is flagged unqualified (C-153, C-178); a commissioning-measured D loaded from 018-11 is shown with its window and is not flagged unqualified (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every parser touched is fuzzed

## Out of scope
- The stamp format, sectioning and serializer (epic 013).
- C-47 and C-159 health counters (018-23); lateness percentiles and maximum since live (018-24).
- Sending the stamp message (epic 015, C-144).
