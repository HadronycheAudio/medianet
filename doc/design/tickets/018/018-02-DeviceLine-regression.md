# 018-02 — DeviceLine regression (loop 2)
**Epic:** 018-Linux-ALSA-edge · **Effort:** M · **Depends on:** 018-01

## Context
- Two-loop timebase `[decision]`: loop 1 (core servo, epic 008) maps host clock H to the domain line; **loop 2** (this ticket) maps the device sample counter S to H by regression over `DevicePair { frames: u64, host_ns: i64 }` observations. On ALSA these pairs come from the PCM status (hardware pointer, delay, timestamp) — produced by 018-10.
- The rate loop bandwidth is **well below 1 Hz** (C-68, pending measurement) so servo and pair jitter are filtered instead of passed into pitch. The exact bandwidth value is an open sub-question owned by epic 020 at L2 (falsifier: the edge passes servo jitter into the resampler ratio, or tracks too slowly after a handover).
- Published quality number of an edge = clock bound + **regression residual** `[decision]`. Which statistic the residual is (RMS, percentile, maximum) is an open sub-question of epic 018 **owned by this ticket** (falsifier: the published quality number is below the emission error observed at L2).
- `edge/` is `no_std`, no allocation, and must make **no libm call** (epic criterion 018-A7): use only `+ − × ÷` on `f64`/integers; if a square root is needed (e.g. RMS residual), implement it locally (Newton iteration) or choose a residual statistic that needs none.
- The device runs at the 96 kHz domain rate (C-01) or at 48 kHz (converted 2:1 at the edge, C-80); the line is in device frames either way.

## Work
1. In `hadronyche-core/src/edge/`, implement:
   ```rust
   /// Loop 2: S -> H. Fixed-memory regression with a bandwidth well below 1 Hz.
   pub struct DeviceLine { /* slope, intercept, residual */ }
   impl DeviceLine {
       pub fn update(&mut self, pair: DevicePair);
       pub fn frames_at(&self, host_ns: i64) -> f64;
       pub fn residual_ns(&self) -> u32;
   }
   ```
   plus a `const fn new(nominal_rate: u32, ...)` constructor (seeded from `DeviceClock::nominal_rate()`) and a `reset()` used on a backwards counter and on departure (018-21).
2. Fixed memory: either an exponentially weighted least-squares (constant-size accumulators) or a const-generic ring of N pairs. No heap.
3. Numerical stability: centre frames and host time on a reference pair (re-based periodically) so `f64` keeps sub-ns precision over a one-hour run at 96 kHz and over `u64` frame counts.
4. Expose the bandwidth (or time constant) as a constructor parameter with a provisional default chosen to pass the acceptance criteria below; record the provisional value in the code comment as provisional, owner epic 020 (C-68).
5. Residual: choose the statistic (e.g. RMS or a high percentile of |host_ns − predicted host_ns| in ns), document it, saturate into `u32`, and record the choice and its falsifier in the edge design document so the epic's open sub-question is closed with evidence.
6. Handle wrap/reset of the device counter (frames going backwards → reset the line, residual reported as `u32::MAX` until re-converged).
7. Unit-test harness: synthetic pair generator with a configurable crystal offset (ppm) and uniform/Gaussian pair jitter, deterministic seed, `no_std`-compatible.

## Acceptance criteria
1. Synthetic pairs from a device crystal offset by **+100 ppm** and by **−100 ppm**, with pair jitter of **50 µs**, at 96 kHz: after **30 s** of pairs, the slope error of `DeviceLine` is below **1 ppm** (contribution to epic criterion 018-A1).
2. With zero jitter and a constant offset, `frames_at(h)` predicts within **1 frame** of the true value 10 s ahead of the last pair.
3. `residual_ns()` with 50 µs jitter is within a factor 2 of the generated jitter statistic it is documented to estimate; with zero jitter it is below 1 µs.
4. A backwards frame count resets the line; convergence within 1 ppm is reached again within 30 s.
5. Memory footprint of `DeviceLine` is a compile-time constant (`core::mem::size_of` asserted in a test).
6. The residual statistic is named in the edge design document with its falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Provisional bandwidth value documented as provisional with owner epic 020

## Out of scope
- Ratio computation and the bandwidth check against servo jitter steps (018-03).
- Reading real ALSA status pairs (018-10).
- Choosing the final bandwidth value (epic 020).
