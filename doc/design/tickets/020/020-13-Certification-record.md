# 020-13 — Certification record
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-06, 020-07, 020-09

## Context
`report.rs` produces a **certification record** with these fields `[decision]`: platform, OS build, audio device, driver, device rate, buffer sizes, the audition offset A used and its inputs (computed with the commissioning rule, C-148), D and its method, E (99.9th percentile and maximum), restart spread, pass/fail per criterion, both stamps.

Certification is two-tier: *self-monitored* and *commissioned* `[decision]` (a sink self-monitors clock bound and rate-tracking error; the analogue part of D needs the external measurement). Only the commissioned state gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140). "Both stamps" are the regime stamps of the sink and of the holder it followed (C-191); the tier the sink reached is a separate field of the record, not one of the two stamps. A commissioned sink whose clock later becomes unlocked stays commissioned: its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180), so the record is not withdrawn or re-issued on an unlock.

The certified E is what lets the sink declare an E: a sink with no certified E declares E as unknown and is refused with reason RequiredEExceeded from any presentation group that sets a required E (C-103). Every injection used by a run is stamped like any injection (C-120), and the copied stamps show it.

**Epic criterion 020-A7: the record carries complete stamps and names the reference hardware; a run on other hardware is marked *unverified*.** Things to be stamped, not tested `[decision]`: any bound on Windows scheduling latency; a device whose driver misreports latency without this measurement; macOS aggregate devices; Bluetooth and network audio outputs; any OS, driver and device combination outside the reference list (stamped *unverified*).

Pass criteria to carry, with Linux values `[decision]`; every criterion naming E uses the platform's target E, never the E measured in the same run (C-149): E ≤ 50 µs (C-70, E per C-73); restart spread of D (max − min, C-166) ≤ 25 µs over 20 restarts (C-74); marker offset within ±2 samples of the median of the first 60 s (C-75, C-166); zero slip within ±1 sample after loss, 50 ppm handover (C-78) and announced step (C-76), a one-sample slip passing and being reported as a slip event (C-166); MTIE ≤ 10 µs up to 1 s and ≤ E overall (C-77); during the handover ratio change ≤ 1 ppm/s (C-61) and no marker offset step beyond ±1 sample (AC-5, hardware part; C-166). Drift ramp: E held throughout the ±100 ppm sweep at no more than 0.5 ppm/s, correction term within C-60 and C-61 (C-119, criterion 020-A9). Platform targets and E budget splits: Linux 50 µs = 20 clock + 20 tracking + 10 D (C-70); macOS E 100 µs = 40 + 40 + 20 (C-71, C-150), spread 50 µs (C-74); Windows E 1 ms = 100 + 100 + 800 (C-72, C-150), spread 0.5 ms (C-74). On macOS and Windows the handover no-step tolerance is the platform restart spread (C-166).

## Work
- In `report.rs`, define `CertificationRecord` with all fields above plus: rig (from 020-09, including capture delay and its method), reference-hardware identifier, harness version, scenario list with per-scenario results, a `certification_state` field (self-monitored or commissioned, the tier the run establishes) with `unverified` (with reason), and a `regime_stamps` block holding the two stamps of the record: the regime stamps of the sink and of the holder it followed (C-191), as read at the end of the run (declared D, declared E or unknown per C-103 and C-140, the per-connection counters of C-47 as completed by C-159, the lateness percentiles of C-160, the capability states of C-129, injection stamps per C-120), a `strict_mode` block: whether the run ran in strict mode and the state of each strict-mode capability (C-127, C-147, ticket 020-27), and for a handover run, a `handover_difference` block with the measured natural difference between the two holders' crystals and the injected drift that makes up the 50 ppm (C-223), an optional `commissioning_window` field naming the window the measurement was taken in (C-178; empty for the isolated-rig runs of this epic, filled when epic 028 packages the procedure).
- Reference list: a data table in the crate of OS / driver / device combinations that are reference hardware; any other combination sets `unverified = true` with reason "outside reference list". The table mirrors the reference hardware recorded in the constants table after approval by the product owner (C-172). Leave the macOS / Windows entries to epics 021 / 022.
- Output both a machine-readable form (JSON) and a Markdown rendering suitable for `doc/design/reports/l2-results-linux.md` (C-130). Their layout and file names are decided by this ticket and documented in `report.rs` (C-206); only the field list above, which epics 021, 022 and 028 depend on, is fixed by the epic.
- Gather OS build, driver and device information automatically on Linux where possible.
- Document every stamp field in the module doc.

Note: the epic makes this task the creator of the reference list, starting with the Linux rig of 020-21; epics 021 and 022 add their entries. The regime stamp's own field layout belongs to the wire specification (epic 004) and is only copied here.

## Acceptance criteria
1. A record built from a synthetic analysis result contains every field listed in Context; a test fails if any is empty. Its `regime_stamps` block holds exactly two stamps, the sink's and that of the holder it followed (C-191), and a record carrying any other pair is rejected.
2. A run whose (OS, driver, device) is not in the reference list is stamped *unverified* with a reason; a run on the reference entry is not.
3. Pass/fail is reported separately for each criterion (E, restart spread, C-75, C-76 per disturbance with its slip events, C-77, AC-5 ratio and step, drift-ramp per C-119).
4. JSON output round-trips through the record type.
5. The Markdown rendering of a synthetic record is committed as a golden test.
6. A handover record carries the measured natural crystal difference and the injected drift, summing to 50 ppm (C-223); a handover record lacking either fails the test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented — not applicable (C-125): this ticket adds no regime stamp field; it copies the stamps of the sink and the reference holder into the certification record, whose fields are documented in `report.rs`
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Producing the real Linux record and committing it (020-22, 020-23, 020-24).
- Cross-sink skew fields for macOS and Windows (epics 021, 022).
