# 027-15 — Loss scenario
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-11, epic 017

## Context
- L3 rig: Linux sink on capture channel 1, embedded device on channel 2; skew = ch2 − ch1, inter-channel skew subtracted. Pass rule (C-79): p99.9 |skew| ≤ E_linux + E_device, max reported.
- Packet loss is injected through the public injection API of the reference source (epic 017), on **one sink's path only**. Every injection is stamped (C-120).
- Liveness rules: a sink is interrupted after 8 consecutive missed slices, 4 ms at defaults (C-38), and dead after 2 s (C-39) counted from the last slice received with content or a marked gap (C-111). Only slices absent at the deadline count as misses — missing, or arriving late; a marked gap or a slice on another timeline epoch resets the count (C-102). So the injection must drop packets, not mark gaps, to drive the interrupted threshold. An absent slice counts as missing at the deadline and is reclassified as late if it arrives later within the ring span (C-108). Resume from interrupted is immediate at the frozen offset, so a short gap must not shift a sink's output in time. Default slice: 48 frames = 0.5 ms (C-02).
- Each sink's stamp carries per-connection counters — received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159, completing C-47); fetched before and after each event (027-22), they confirm what the injection did.
- **027-AC-4 (loss part):** after injected loss on one path the skew returns to its prior value within the pair tolerance: the larger of the two sinks' own recovery tolerances, ±1 sample (10.4 µs) for the Linux sink (C-166) and the device's declared restart spread (C-191), with both figures reported beside each verdict (C-203). Checked with `recovered()` from 027-06 (samples during the loss excluded): the median of the 10 s window after the event against the median of the first 60 s; a one-sample slip passes and is reported as a slip event (C-166).
- Loss pattern (C-166, proposal pending first runs): 1 % random loss plus bursts of 8 slices. A burst of 8 consecutive missing slices reaches the interrupted threshold (C-38) and stays far below the dead threshold; at 1 % the random part alone practically never produces 8 consecutive misses.
- Completed by C-191 (proposal pending first runs): the bursts are spaced 1 s apart and hit the media path only; sync loss is its own scenario (holdover, 027-18). The 10 s recovery window starts when the injected loss ends.

## Work
1. `RigControl::inject_loss(sink, pattern, duration)` using the epic-017 injection API, on the sink's media path only (C-191); the pattern is a parameter whose default is the C-166 pattern (1 % random plus bursts of 8 slices) with bursts 1 s apart (C-191), and the number of bursts per event recorded.
2. Scenario `loss`: baseline window, loss on the Linux path, recovery window, then the same on the device path. Log event positions. Fetch both stamps before and after each event.
3. Report the `recovered` result (with the pair tolerance from `pair_tolerance_ns`, 027-06, the device's declared restart spread read from its stamp or declaration, epic 026) and both sinks' own tolerances beside it (C-203), and pre/post medians per event, p99.9 |skew| outside event windows, and per event the change in the C-159 counters of the affected sink (missing, late, interrupted events, dead) and of the other sink (expected unchanged).

## Acceptance criteria
1. **027-AC-4.** For each loss event on each path, |median after − median before| ≤ the pair tolerance, the larger of 10.4 µs (Linux, C-166) and the device's declared restart spread (C-191) (C-203); both sinks' tolerances appear beside each verdict.
2. Outside event windows `p999_abs_ns <= limit_ns`.
3. The loss pattern used (random rate, burst length in slices, burst spacing 1 s, bursts per event, media path only) is listed in the report, citing C-166 and C-191, and the recovery window starts at the end of the injected loss.
4. The affected sink's stamp shows missing slices and exactly one new interrupted event per injected burst, and no dead; the other sink's counters show no loss.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — reads stamps only through 027-07/027-22; "Design documents updated" — 027-20; "No new open item" — opens none (the pattern is C-166 and C-191; the tolerance C-203); "The rig is photographed or diagrammed" — 027-09; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Physical link interruption (027-16).
- The injection API itself (epic 017).
