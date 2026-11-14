# 027-11 — Scenario runner and steady hour
**Epic:** 027-L3-cross-sink-rig · **Effort:** M · **Depends on:** 027-05, 027-06, 027-07, 027-08, 027-09, 027-10, 027-22, 027-23, 027-24

## Context
- **Claim under test:** every sink in a presentation group emits sample p at `emit(p) = T(p) + A` (T(p) = network time of timeline position p, `position = (network_time − epoch_origin) × rate`; A = the group's audition offset, one number per group, no default, C-55) and submits it to hardware at `submit(p) = emit(p) − D` (D = declared plumbing delay).
- Rig (027-09): Linux host (clock holder, reference marker source of epic 017, ALSA reference sink of epic 018) on capture channel 1; embedded reference device (epic 026) on channel 2; one two-channel capture device at 96 kHz; inter-channel skew calibrated and subtracted (027-10). IPv4 only, one subnet (C-100).
- Skew = channel 2 − channel 1 (device minus Linux). Pass rule (C-79): the 99.9th percentile of |skew| is at most E_linux + E_device, max reported beside it; E_linux target 50 µs (C-70); E_device declared per device; E as defined by C-73. The limit uses these target values, never the E measured in the same run (C-149). A sink whose E is unknown (C-103), which includes any sink not in certification state commissioned (C-140), cannot be judged.
- Every run feeds the constants table, so it runs in strict mode (C-147); the run eligibility gate (027-24) checks this and the certification states before capture. A is computed by the harness with the commissioning rule and recorded with the results (C-148, ticket 027-23).
- Both sinks follow the same clock holder with the core servo: slew ≤ 10 ppm (C-60), rate change ≤ 1 ppm/s (C-61), lock bound ≤ 25 µs (C-62, pending simulation). Default slice 48 frames = 0.5 ms (C-02); 1 sample at 96 kHz = 10.4 µs.
- Stamps are fetched by unicast request at scenario start and every multicast update during the run is logged (C-18, ticket 027-22). The stamps' per-connection counters (C-159, completing C-47) show whether any slice was missing or late during the run. On a clean network about one packet per stream per link is lost every 13 hours (C-90), so a clean hour normally shows none.
- **027-AC-1 / AC-29 (steady part):** the 99.9th percentile of |skew| between the hosted and the embedded sink is at most the sum of their E values for 1 hour (`steady`).
- **027-AC-2, No drift:** over the hour a linear fit of skew has a slope below 1 µs per hour (two crystals, one timeline).
- L3 does not repeat L2 (E, D restart spread C-74, marker offset ±2 samples C-75, zero slip C-76, wander C-77); it measures the difference between two certified sinks.

## Work
1. `run.rs`: scenario runner common to all scenarios: fetch both stamps (027-22), run the eligibility gate (027-24), wait for both sinks locked to the holder, compute and apply A (027-23), start capture, mark event times (in timeline positions) in a run log, log stamp updates, stop capture, fetch both stamps again, run calibrated `analyze`, write JSON + Markdown report (027-07) per scenario into a run directory.
2. Rig control abstraction (`trait RigControl`) with the operations later scenarios need (restart a sink, power-cycle the device, change holder, inject loss, interrupt link, announce epoch step, remove/restore holder, set group required E). This ticket implements only what `steady` needs and stubs the rest with "not implemented".
3. Scenario `steady`: 1 hour continuous play; report p50, p99.9 |skew|, max, limit, pass, the drift slope (027-06) with pass if |slope| < 1 µs/h, the A used (C-148), and the C-159 counters at start and end of the run.
4. Run it on the rig; keep the full capture outside the repository and commit a short excerpt as CI fixture (027-08).
5. If the steady run fails, record which sink is suspected per the falsifier "Skew exceeds the sum: one sink's D or E is dishonest; find which with L2", and open an item with owner and falsifier.

## Acceptance criteria
1. **027-AC-1 / AC-29 (steady).** On the rig, `hadronyche-l3 run steady` over 3600 s reports `p999_abs_ns <= limit_ns`, with `limit_ns` = the C-70 target plus the device's declared E (C-149); `max_abs_ns` is reported.
2. **027-AC-2.** The same run reports |drift slope| < 1 µs/h.
3. The report includes both sinks' stamps, inter-channel skew, self-check value, unpaired marker count, the A used and its inputs (C-148), the eligibility result (C-147, C-140), and the C-159 counters (any missing or late slice during the hour is listed).
4. Runner code paths (event log, stamp logging, report writing) are covered by unit tests with a fake `RigControl`, a fake stamp source and a synthetic capture, unprivileged in CI.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — reuses the parsers of 027-02, 027-07 and 027-22; "Design documents updated" — results written up by 027-20; "The rig is photographed or diagrammed" — 027-09; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Other scenarios (027-12 to 027-19).
- Writing `l3-results.md` (027-20).
