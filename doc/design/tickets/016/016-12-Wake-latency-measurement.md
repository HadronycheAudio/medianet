# 016-12 — Wake latency measurement
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-01, 016-02

## Context
- Signature: `pub fn measure_wake_latency(duration: core::time::Duration) -> WakeLatency;` — samples wake latency for `duration`; part of the self-test.
- The self-test that precedes any reported number checks that real-time scheduling was actually granted and measures wake latency. Numbers produced before the self-test passes are labelled *unqualified* `[decision]`. Any run whose numbers enter the constants table runs in strict mode; L0 does not (C-147).
- `WakeLatency` carries mean, 99.9th percentile and maximum (016-01). These go into the regime stamp as plain fields (016-13).
- Epic AC 7: over 1 s, returns mean, 99.9th percentile and maximum; values in the stamp. Contributes to AC-8 (trustworthy loopback run). The figure is used by L1 and L2 (tasks 019, 020).
- Method and period are an open sub-question owned by this ticket, confirmed at L1 (task 019); falsifier: latency measured at 100 µs differs materially from latency at the slice period (0.5 ms, C-02), or a 1 s run yields fewer than 1 000 samples. The epic's proposal, implemented here: `clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME)` on a fixed 100 µs period, latency = actual wake time − target, on the calling thread (so it measures that thread's scheduling class). At 100 µs, 1 s gives about 10 000 samples, enough for a 99.9th percentile with about 10 samples above it.

## Work
1. Implement `measure_wake_latency` in `hadronyche-os-linux::rt`: preallocate the sample buffer before the loop (no allocation inside), collect `duration / period` samples, compute mean, p99.9 (nearest-rank) and max.
2. Pure helper `fn summarize(samples: &mut [u32 /* ns */]) -> WakeLatency` for unit tests.
3. Make the period a named constant `WAKE_PROBE_PERIOD = 100 µs`, and propose it to the design owner for the constants table (`doc/design/02-constants.md`, C-130) with the owner and falsifier above.

## Acceptance criteria
1. `summarize` on 1..=1000 ns returns mean 500.5 ns (±1 ns), p99.9 = 999 ns, max = 1000 ns (unit test).
2. `measure_wake_latency(1 s)` returns within 1.2 s, with `samples` ≥ 9 000, and `mean ≤ p999 ≤ max` (unprivileged CI test; no absolute bound asserted).
3. No heap allocation happens inside the sampling loop (checked with a counting allocator in a test).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Pass/fail thresholds of the self-test and *unqualified* labelling (callers: epics 017, 019, 020).
- Writing the values into the stamp (016-13).
