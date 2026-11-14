# 021-20 — Regime stamp `macos-std`
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-05, 021-06, 021-12, 021-14, 021-27, 021-30
## Context
Epic criterion 021-AC7 (AC-31): the stamp carries profile `macos-std`, OS build, audio device, driver, receive-timestamp source (*kernel* or *userspace*), drop accounting *absent*, DSCP *peer-verified* or *unverified*, D and its method, E, certification state, the cost calibration factor, the commissioning window when open, the per-connection counters, and the DSCP observed on each peer's hellos.

- **Transport** [decision, C-18]: the stamp is a dedicated control message split into sections that each fit 1 252 bytes of UDP payload (C-17); multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only the stamp version number. The hosted backend (epic 015) sends the stamp message that epic 013 defines (C-144); this ticket supplies the macOS fields and must keep each section within the bound.
- **E and certification state** [decision, C-103, C-140]: the certification state is *uncertified*, *self-monitored* or *commissioned*; only *commissioned* gives a numeric E, the others declare E as *unknown*.
- **Per-connection counters** [decision, C-47, C-159]: producer and consumer each publish, per connection, received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead, plus the observed DSCP. Lateness percentiles are the 50th, 99th, 99.9th and the maximum (C-160), over a rolling 60 s window with the maximum since the connection went live beside them (C-182). Core keeps the counters; the macOS stamp must carry them and the observed DSCP read through 021-03.
- **DSCP of peers** [decision, C-152]: every module reports the DSCP observed on each peer's hellos (read through 021-03); the holder reports the DSCP observed on each follower's sync requests, which verifies this edge's sync DSCP from the holder's side.
- **Commissioning window** [decision, C-107, C-153]: the tool announces the window on the control group with an expiry time; every module stamps it while open; it closes at expiry or on explicit close. While it is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178).
- **Cost calibration factor** [decision, C-106, C-176]: each target records its time-per-operation factor in the stamp (obtained in 021-27 from the epic 025 conformance suite).
- **Capability states** [decision, C-129]: every capability is stamped as present, absent, unverified or not requested. The receive-timestamp source is *kernel* or *userspace* (C-131) with such a state; drop accounting is *absent*; DSCP *peer-verified* is *present*, otherwise *unverified*; real-time scheduling, memory lock, CPU idle-state restriction and control isolation (*present* by construction on a hosted platform, C-161) come from 021-30 (C-127).
- Any OS, driver and device combination outside the short reference list of certified combinations is stamped *unverified*.
- A *commissioned* sink whose clock is unlocked stays *commissioned*, with its alignment claim suspended until the clock is locked again (C-180).
- If the interface is held by another process (C-175): `InterfaceHeld` is reported and the certification state is *uncertified* (021-12).
- DSCP is set with `IP_TOS` and verified by the receiving peer through its receiver report.
- The certification-state values are those of C-140. The full enumeration of allowed values for "D method" belongs to the regime stamp message defined by epic 013 (C-144); until it exists this ticket uses `DeviceProperties` / `Commissioned` and documents them as provisional.
## Work
- Assemble the stamp fields from: OS build (sysctl), audio device and driver names (Core Audio), timestamp source (021-05), drop accounting (021-06), DSCP state from peer receiver reports, D and method (021-14/021-22), E (numeric only when *commissioned*, C-140), certification state, calibration factor (021-27), commissioning window state (from core), per-connection counters, lateness percentiles and observed DSCP (from core), DSCP observed on each peer's hellos (C-152).
- Reference-list lookup: a compiled-in table of certified (OS build, device, driver) combinations, initially empty, filled by 021-26.
- Lay the macOS fields out so that each stamp section stays within 1 252 bytes with the maximum number of connections the edge can hold.
- Document every field (meaning and allowed values) in the stamp documentation.
## Acceptance criteria
1. Unit test: every listed field is present and non-empty; profile = `macos-std`; drop accounting = *absent*; E = *unknown* unless the certification state is *commissioned* (C-140).
2. With an empty reference list, the platform combination is stamped *unverified*; with `InterfaceHeld`, the certification state is *uncertified* and E *unknown* (C-140, C-175).
3. DSCP is *peer-verified* only after a receiver report confirms DSCP 46 on media; otherwise *unverified*.
4. Unit test: every encoded section is at most 1 252 bytes (C-17, C-18) at the largest supported connection count.
5. Unit test: every capability field carries exactly one of the four C-129 states.
6. With a commissioning window open, the stamp shows it with its expiry; with none, or after expiry or explicit close, the field is absent or *closed* (C-107, C-153).
7. Unit test: the stamp carries the DSCP observed on each peer's hellos (C-152) and all eleven per-connection counters of C-159.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] A short reference list of certified OS build / device combinations is published; everything else stamps *unverified*
## Out of scope
Filling the reference list (021-26). Measuring E (021-23). Sending the stamp message (epic 015, C-144) and keeping the counters (core). The stamp's field encoding (epic 013).
