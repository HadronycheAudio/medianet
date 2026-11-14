# 018-04 — Emission error computation
**Epic:** 018-Linux-ALSA-edge · **Effort:** XS · **Depends on:** 018-03, epic 008

## Context
- Edge contract `[decision]`, for a sink in a presentation group with audition offset A:
  - Emission: `emit(p) = T(p) + A` — the instant sample p leaves the converter; `T(p)` is the network time of position p, with `position = (network_time − epoch_origin) × rate` at the single 96 kHz domain rate (C-01).
  - Hand-off to hardware: `submit(p) = emit(p) − D`, where D = driver buffer + resampler group delay + converter (declared total plumbing delay from hand-off to emission, C-170).
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| (C-73). The sink self-monitors the clock-bound and rate-tracking components continuously.
- The self-monitored E value is **blank until the clock is locked** (epic criterion 018-A2).
- The epic fixes the signature: the method takes the `TimeView` (for `T(p)` and the lock state) and returns `Option<i64>`, `None` while not locked.

## Work
1. Implement in `EdgeTimebase`:
   ```rust
   /// Self-monitored emission error for the frame about to be submitted; None while the clock is not locked.
   pub fn emission_error_ns(&self, view: &TimeView, position: u64, a_ns: i64, d_ns: i64, submit_host_ns: i64) -> Option<i64>;
   ```
   Result = (`submit_host_ns` + `d_ns`) − (T(position) + `a_ns`), all in host time.
2. Return `None` whenever `view` is not locked.
3. Document the sign convention (positive = late).

## Acceptance criteria
1. With a fake locked view, `a_ns = 2_000_000`, `d_ns = 1_000_000` (typical D on Linux about 1 ms, C-81), and `submit_host_ns = T(p) + 1_000_000`, the result is `Some(0)`; offsetting `submit_host_ns` by +37 µs gives `Some(37_000)`.
2. Sign convention (positive = late) documented and tested.
3. With an unlocked view, the result is `None` (no number published).
4. Positions near `u64` limits and epoch origins far from zero give no overflow (property test, 10 000 cases).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation

## Out of scope
- Percentile/maximum statistics (018-05).
- Calling it per submitted period and publishing (018-14).
