# 027-16 — Interrupt scenario
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-11

## Context
- L3 rig: Linux sink on capture channel 1, embedded device on channel 2; skew = ch2 − ch1, inter-channel skew subtracted. Pass rule (C-79): p99.9 |skew| ≤ E_linux + E_device, max reported.
- Scenario `interrupt` (fixed by the epic): one sink's link interrupted for under 2 s, then restored.
- Liveness: interrupted after 8 consecutive missed slices (C-38), dead after 2 s (C-39) counted from the last slice received with content or a marked gap (C-111). A missed slice is one absent at its deadline (C-102). Resume from interrupted is immediate at the frozen offset, so a short interruption must not shift a sink's output in time. Slice = 48 frames = 0.5 ms (C-02), so 8 slices = 4 ms.
- Each sink's stamp carries interrupted events, dead and the per-connection counters (C-159, completing C-47), fetched before and after each event (027-22).
- Interrupted and dead describe the stream a sink receives. The producer judges the liveness of a sink-only device by the producer timeout, 2 s without receiver reports (C-168, C-41); an interruption under 2 s therefore must not time out the source's connection to either sink. The source module's stamp carries its per-connection counters (C-47) and is fetched like a sink's (027-22).
- **027-AC-4 (interruption part):** after an interruption shorter than 2 s the skew returns to its prior value within the pair tolerance: the larger of the two sinks' own recovery tolerances, ±1 sample (10.4 µs) for the Linux sink (C-166) and the device's declared restart spread (C-191), with both figures reported beside each verdict (C-203). Checked with `recovered()` and `pair_tolerance_ns` (027-06).
- Recovery parameters (C-166, C-191): the median of the 10 s window that starts when the interruption ends (link restored) is compared with the median of the first 60 s; a one-sample slip passes and is reported as a slip event.

## Work
1. `RigControl::interrupt_link(sink, duration)` — switch port disable via the management interface or a controllable relay in the Ethernet cable; document the method in the rig parts list (027-09).
2. Scenario `interrupt`: interruptions of, for example, 100 ms, 500 ms and 1.5 s (all < 2 s, all > 4 ms) on each sink in turn, with baseline and recovery windows; log positions. Fetch both sinks' stamps and the source's stamp before and after each event.
3. Report per event: pre/post medians, `recovered` result with the pair tolerance and both sinks' own tolerances beside it (C-203; the device's declared restart spread read from its stamp or declaration, epic 026) and any slip event, the change in the affected sink's interrupted and dead indications, and whether the source's connection to that sink stayed up (C-168).

## Acceptance criteria
1. **027-AC-4.** For each interruption (< 2 s) on each sink, |median after − median before| ≤ the pair tolerance, the larger of 10.4 µs (Linux, C-166) and the device's declared restart spread (C-191) (C-203); both sinks' tolerances appear beside each verdict.
2. Interruption durations are verified (from the capture gap, link logs or the stamp counters) to be under 2 s; any event reaching 2 s since the last received slice is discarded and reported.
3. Each kept event shows exactly one new interrupted event and no dead in the affected sink's stamp.
4. Outside event windows `p999_abs_ns <= limit_ns`.
5. For each kept event the source's stamp shows no producer timeout on its connection to the interrupted sink (C-41, C-168).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — reads stamps only through 027-07/027-22; "Design documents updated" — 027-20; "No new open item" — opens none; "The rig is photographed or diagrammed" — 027-09 lists the port control; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Packet loss injection (027-15).
- Dead-sink handling beyond 2 s.
