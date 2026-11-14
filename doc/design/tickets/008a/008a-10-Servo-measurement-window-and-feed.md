# 008a-10 — Servo measurement window and feed
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-01
## Context
First part of the new servo in `hadronyche-core`. Fixed size, no allocation, `no_std` (epic criterion A9).

Facts:
- Each reply is a self-contained four-timestamp exchange `Exchange { t1, t2, t3, t4, epoch }`. A stale or duplicate reply is a valid exchange with an honestly large round trip. Every reply carries the timeline epoch `[decision]`.
- Exchanges arrive already built by the platform backend, which has discarded samples corrupted by a wall-clock step (C-121); `feed` parses nothing.
- Inherited starting point, to be confirmed by case 1: measurement window 200 ms using the minimum-round-trip exchange (C-66) `[decision]`.
- The POC estimator kept a 30 s history of minimum delay (`DMIN_HISTORY_NS`) and weighted measurements by delay above the floor `[code]`; the new design may keep or drop that.
- Formulas (epic Background): round trip = (t4 − t1) − (t3 − t2); offset = ((t2 − t1) + (t3 − t4)) / 2. A forward delay larger than the reverse delay by a biases the offset by +a/2 `[decision]`.
- Case 7: stale, duplicate and reordered replies leave the estimate unchanged.

## Work
1. `Servo::new(cfg: ServoConfig)` and `Servo::feed(&mut self, ex: Exchange)`.
2. Per window of `cfg.window_ns` (default 200 ms, C-66), keep only the exchange with the smallest round trip; on window close produce one measurement (offset, delay, local time).
3. Reject at feed: exchanges whose epoch differs from the current epoch (handled by 008a-13), and exchanges with negative round trip; count rejects by reason.
4. Duplicates and stale replies are not specially detected: they enter as valid exchanges and lose the min-RTT selection by their larger round trip. Document this.
5. Add `window_ns` (and any delay-floor parameters) to `ServoConfig`.

## Acceptance criteria
1. Feeding 10 exchanges in one 200 ms window yields one measurement equal to the min-RTT exchange's offset and delay exactly.
2. Feeding the same exchange twice, or a set of exchanges in reversed order, produces the identical measurement.
3. An exchange with round trip < 0 is rejected and counted.
4. `Servo` contains no heap types; `size_of::<Servo>()` is a compile-time constant asserted in a test; module builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

Not applicable (C-125): "Every parser touched is fuzzed" — this ticket touches no parser; `feed` takes an already-built `Exchange` (C-121).

## Out of scope
- Control loop, lock and bound (008a-11); holdover (008a-12); epoch change (008a-13); handover (008a-14).
