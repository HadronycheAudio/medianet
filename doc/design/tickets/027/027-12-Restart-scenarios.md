# 027-12 — Restart scenarios
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-11

## Context
- L3 rig: Linux ALSA reference sink (epic 018) on capture channel 1, embedded reference device (epic 026) on channel 2, one capture clock at 96 kHz; skew = channel 2 − channel 1, inter-channel skew subtracted (027-10).
- Pass rule (C-79): the 99.9th percentile of |skew| is at most E_linux + E_device, "for 1 hour, and across restarts of either"; the maximum is reported beside it. E_linux target 50 µs (C-70); E_device declared per device; E as defined by C-73.
- Each sink submits sample p at `submit(p) = T(p) + A − D`; if D is honest after every restart, the skew is unchanged. L2 already measured the Linux restart spread of D: at most 25 µs over 20 restarts (C-74).
- **027-AC-1 / AC-29 (restart part):** the 99.9th percentile of |skew| is at most the sum of the E values after every one of 20 restarts of each sink.
- Scenario names fixed by the epic: `restart-linux`, `restart-device` — 20 restarts of each, skew re-measured after each.
- The restarted sink's stamp is re-fetched after each restart (C-18, 027-22).
- Parameters (C-166, proposals pending first runs): the measurement window after a restart is 60 s; the restart spread of D is max − min over the restarts.
- The embedded device declares its own restart spread, measured at L3 (C-191, proposal pending first runs): `restart-device` is that measurement. The Linux sink's spread was measured at L2 (C-74), so the spread of the per-restart median skew in `restart-device` is attributed to the device.
- A restarted sink must come back eligible: strict mode on the Linux host (C-147) and certification state commissioned (C-140); the gate of 027-24 is re-run on the re-fetched stamp.

## Work
1. Implement `RigControl::restart_sink(Linux|Device)` (soft restart of the sink process / device firmware, not a power cycle; power cycles are 027-13).
2. Scenarios `restart-linux` and `restart-device`: for i in 1..=20, restart, wait for lock and play, re-fetch the restarted sink's stamp and re-run the eligibility gate (027-24), measure skew for the configured window (default 60 s, C-166), produce one `SkewReport` per restart plus the combined verdict (pass only if all 20 pass, 027-05).
3. The report also lists the per-restart median skew, so the spread of the skew across restarts is visible, and the restarted sink's D after each restart, with the spread of the stamped D as max − min over the 20 restarts (C-166).
4. For `restart-device`, report the device's measured restart spread as max − min of the 20 per-restart median skews (C-166), beside the restart spread the device declares (C-191), read from its stamp or its declaration (epic 026), and hand both to 027-20.
5. Commit one restart excerpt as CI fixture.

## Acceptance criteria
1. **027-AC-1 / AC-29 (restarts).** `restart-linux`: all 20 per-restart reports have `p999_abs_ns <= limit_ns`.
2. **027-AC-1 / AC-29 (restarts).** `restart-device`: all 20 per-restart reports have `p999_abs_ns <= limit_ns`.
3. The report shows 20 entries per scenario, each with restart time, median skew, p99.9 |skew|, max |skew|, the window used and the stamped D, plus the D spread (max − min, C-166).
4. The `restart-device` report states the device's measured restart spread (max − min of the per-restart median skews) beside its declared restart spread, tagged `[measured]` and citing C-191.
5. Restart control code unit-tested with a fake `RigControl`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — reads stamps only through 027-07/027-22; "Design documents updated" — 027-20; "The rig is photographed or diagrammed" — 027-09; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Power cycles of the device (027-13).
- The steady hour (027-11).
