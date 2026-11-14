# 027-13 — Device power-cycle check
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-12

## Context
- Open sub-question owned by epic 027: "Does the device's declared D hold across power cycles (not only soft restarts)?" Falsifier: skew changes after a power cycle beyond the limit.
- Epic acceptance criterion **027-AC-11:** after each of 20 power cycles of the device, the 99.9th percentile of |skew| is at most the sum of the E values, or the device's D or E is revised with a stated reason. The epic's scenario list names `power-cycle-device`.
- The embedded reference device (epic 026) declares D (plumbing delay: driver buffer + resampler + converter) and E. Its hardware runs at 48 kHz and converts 2:1 through the kit's resampler node at the edge (C-80); the wire carries only 96 kHz (C-01). The conversion delay is part of the device's D (C-134), so a power cycle that changes the resampler's start-up delay shows up as a change in D.
- L3 rig: Linux sink on capture channel 1, device on channel 2; skew = ch2 − ch1, inter-channel skew subtracted. Pass rule (C-79): p99.9 |skew| ≤ E_linux + E_device, max reported; E_linux target 50 µs (C-70).
- Soft restarts are covered by `restart-device` (027-12). The measurement window after each cycle is 60 s and the spread of D is max − min (C-166), as in 027-12.

## Work
1. `RigControl::power_cycle_device()` — via the switchable power outlet or PoE port listed in the rig parts list (027-09).
2. Scenario `power-cycle-device`: 20 power cycles (same count as the restart scenarios), the device's stamp re-fetched after each (027-22), skew measured after each for the same window as 027-12.
3. Compare the per-cycle median skew distribution and the stamped D (spread as max − min, C-166) with the `restart-device` distribution.
4. Record the answer to the sub-question in the report and in `l3-results.md` (027-20): either "D holds across power cycles `[measured]`" or a revised D/E for the device with stated reason.

## Acceptance criteria
1. 20 power cycles completed; each has a `SkewReport`, median skew and stamped D.
2. **027-AC-11.** Every cycle has `p999_abs_ns <= limit_ns`, or the falsifier is recorded as triggered with the measured values and a revised D or E with its reason.
3. The answer to the open sub-question is written with an evidence tag.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — reads stamps only through 027-07/027-22; "Design documents updated" — 027-20; "The rig is photographed or diagrammed" — 027-09 lists the power switch; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Soft restarts (027-12).
- Changing the device firmware if D does not hold (epic 026).
