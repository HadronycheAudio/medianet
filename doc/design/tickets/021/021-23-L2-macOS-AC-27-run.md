# 021-23 — L2 macOS run: E, D restart spread, cross-sink skew (AC-27)
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** M · **Depends on:** 021-17, 021-18, 021-20, 021-21, 021-22, 021-31, epic 020
## Context
Epic criterion 021-AC1 (AC-27): E at most **100 µs** (C-71) (99.9th percentile over one hour, maximum reported, C-73); restart spread of D at most **50 µs** over 20 restarts (C-74); cross-sink skew against the Linux reference sink at most **150 µs**, the sum of the two E targets (C-82).

- E = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum reported beside it (C-73). A percentile claim, not a bound.
- Only the certification state *commissioned* gives a numeric E; otherwise the sink declares E as *unknown* and is refused by groups that set a required E (C-103, C-140).
- Pass criteria use the target E of 100 µs, never the E measured in this run (C-149). The proposed macOS budget is 40 µs clock + 40 µs tracking + 20 µs uncertainty of D, pending this run (C-150).
- The run is in strict mode, since its numbers may enter the constants table (C-147). The harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results (C-148).
- Test parameters (C-166): measurement window after a restart 60 s; restart spread of D is max − min; sample tolerances are measured against the median of the first 60 s; comparison windows of 10 s.
- Typical D a few ms (C-81).
- Open sub-question owned here: is E = 100 µs reachable, and for which device classes (built-in, bus-attached, USB class)? Falsifier: measured E above target on the reference device. If kernel receive stamps are absent (epic 001), the clock bound and the 100 µs target rise.
- The statistic for the L2 skew limit (C-82) is not stated; C-79 uses the 99.9th percentile with the maximum beside it for L3. Open sub-question owned by epic 020; this run reports both and the verdict follows its answer.
- The reference hardware is proposed in 021-31 and approved by the product owner (C-172); the run uses the supported macOS versions (C-138).
- Real-time wake latency from the self-test (021-09) is recorded alongside; it has no pass threshold.
## Work
- Run the epic 020 harness on the reference Mac, in strict mode (C-147): one-hour E run per device class available (built-in, bus-attached, USB class); record the A used (C-148).
- 20 stream restarts; measure D each time; report spread (max − min).
- Simultaneous Linux reference sink; report skew 99.9th percentile and max.
- Record raw results, wake-latency figures and conditions for 021-26 in `doc/design/reports/` (C-130); any run tooling beyond epic 020's harness goes in the `hadronyche-tools` crate (C-133).
## Acceptance criteria
1. E (99.9th percentile, 1 h) ≤ 100 µs on the reference device, maximum reported beside it.
2. D restart spread ≤ 50 µs over 20 restarts.
3. Cross-sink skew vs the Linux reference sink ≤ 150 µs (C-82), both 99.9th percentile and maximum reported.
4. E reported per device class tested; the sub-question row is closed or its falsifier recorded with a new owner.
5. Wake-latency median, 99.9th percentile and maximum recorded `[measured]` beside E.
6. The stamp recorded with the results shows strict mode with real-time scheduling and memory lock *present* (C-147), and the results record the A used (C-148).
## Definition of done
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
## Out of scope
L2 common criteria (021-24). Drift ramp (021-29). Writing the certification record (021-26).
