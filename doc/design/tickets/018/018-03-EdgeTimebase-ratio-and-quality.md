# 018-03 — EdgeTimebase ratio and quality
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-02, epic 008

## Context
- `EdgeTimebase` combines the published time view of loop 1 (`TimeView`, epic 008) with `DeviceLine` (loop 2, 018-02). The epic requires `TimeView` to expose the domain rate in host time, the clock bound, the lock and holdover state, and `T(p)`; the exact type is owned by epic 008 (remaining gap; this ticket uses a fake with those four items).
- **Resampler ratio** = quotient of the two slopes: domain rate against device rate, both expressed in host time `[decision]`. Direction: ratio = input (domain) samples per output (device) sample. A 48 kHz device in the 96 kHz domain (C-01, C-80) gives a ratio near 2.0 plus the ppm correction; 2:1 and 1:2 are the only conversion ratios in v1 (C-105). `ratio()` is this full slope quotient; the edge applies the factor 2 through the fixed-ratio resampler node and only the ppm part through the steerable rate actuator (C-189, wired in 018-14 and 018-16); both resamplers are the kit's, from epic 024a (C-197).
- The rate loop is a second servo cascaded on loop 1, bandwidth **well below 1 Hz** (C-68, pending measurement), so it filters servo jitter instead of passing it into pitch. Because the loop holds filter state, the epic's API has an explicit `update(&mut self, view, line)` step; `ratio(&self)` returns the filtered value.
- Servo properties consumed (epic 008): published time continuous while locked; slew ≤ 10 ppm (C-60); rate change ≤ 1 ppm/s (C-61); frequency-continuous handover; steps only as announced timeline-epoch changes. C-60 and C-61 bound the servo's correction term, not the drift it tracks (C-119): the edge ratio follows real device drift and is **not** capped at 1 ppm/s.
- Open sub-question of epic 018 **owned by this ticket**: does the ratio need a bounded phase correction beyond the slope quotient? Falsifier: with a slope-only ratio, emission error in a 10-minute fake run drifts beyond the 20 µs rate-tracking budget (C-70 decomposition).
- Published quality number = clock bound + regression residual `[decision]`.
- `no_std`, no allocation, no libm (epic criterion 018-A7).

## Work
1. Implement in `hadronyche-core/src/edge/`:
   ```rust
   pub struct EdgeTimebase { /* rate-loop filter state, … */ }
   impl EdgeTimebase {
       pub fn update(&mut self, view: &TimeView, line: &DeviceLine);
       pub fn ratio(&self) -> f64;
       pub fn quality_ns(&self, view: &TimeView, line: &DeviceLine) -> u32; // clock bound + residual
   }
   ```
2. `update`: raw ratio = (domain frames per host second from `view`) ÷ (device frames per host second from `line`), passed through the rate-loop low-pass. Expose the bandwidth as a parameter with a provisional default (owner of the final value: epic 020, C-68).
3. Phase term: run the falsifier above on a fake. If slope-only drifts beyond 20 µs, add a proportional phase correction driven by the emission error of 018-04, with its contribution bounded and the bound documented; otherwise document why it is not needed. Record the decision and evidence in the edge design document.
4. `quality_ns` = `view` clock bound (ns) + `line.residual_ns()`, saturating.
5. Unit tests reusing the synthetic pair generator of 018-02 and a fake `TimeView`.

## Acceptance criteria
1. **Epic criterion 018-A1 (core unit tests, CI):** with synthetic pairs from a device crystal offset by **±100 ppm** and pair jitter of **50 µs**, the ratio error stays below **1 ppm** after **30 s**.
2. **Epic criterion 018-A1 bandwidth check:** a step in servo jitter of **20 µs** on the fake `TimeView` changes the ratio by less than **0.1 ppm**.
3. With a 48 kHz device and the 96 kHz domain, the ratio converges to 2.0 × (1 + offset) within 1 ppm after 30 s.
4. A frequency-continuous servo handover (fake view changes holder, rate continuous) produces no ratio change larger than 1 ppm.
5. `quality_ns` equals clock bound + residual in a table-driven test, saturating at `u32::MAX`.
6. The phase-term decision is recorded in the edge design document with the fake-run result as evidence (`[measured]` on the simulation).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Emission error (018-04), E statistics (018-05), alignment-claim state (018-06), reset on departure (018-21).
- Applying the ratio to a real resampler (018-14; the resampler is the kit's, epic 024a, C-197).
- Final bandwidth value (epic 020).
