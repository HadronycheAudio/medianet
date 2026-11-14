# 022a-02 — QPC host clock
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-01, epic 015

## Context
- Part of epic 022a (Windows edge backend, crate `hadronyche-os-windows`).
- The host clock H on Windows is `QueryPerformanceCounter` (QPC) with its published frequency `[memory]`. Check: unit test.
- H is the clock that loop 1 of the shared two-loop timebase (epic 018) maps to the domain line through sync exchanges; loop 2 maps the device sample counter S to H. Every receive timestamp and every device-position pair on Windows is in QPC units, so the conversion QPC ticks → ns must be the single shared one.
- Sync requests run at 50/s (20 ms period, C-36); a 1 ms timer is enough for them `[memory]`.
- Every `[memory]` fact must end up confirmed `[measured]` or corrected.

## Work
1. `clock.rs`: `pub struct QpcClock` holding the QPC frequency read once at construction.
2. `impl HostClock for QpcClock { /* now_ns, declared_jitter_ns */ }` (trait from epic 015):
   - `now_ns`: QPC ticks converted to ns without overflow (128-bit intermediate or split quotient/remainder).
   - `declared_jitter_ns`: a constant derived from the QPC resolution (one tick rounded up to ns); document the derivation in a comment.
3. A public helper `qpc_ticks_to_ns(ticks: i64) -> u64` (or equivalent) reused by `net.rs` (receive stamps) and `wasapi/device.rs` (device-position pairs).
4. Unit tests: monotonicity, conversion accuracy, overflow behaviour at large tick counts.

## Acceptance criteria
1. 1 000 000 consecutive `now_ns` calls are non-decreasing.
2. `qpc_ticks_to_ns` matches an exact 128-bit reference to within 1 ns for tick counts up to 2^62 and for frequencies 10 MHz and 3 579 545 Hz (unit test with injected frequency).
3. Over a 1 s sleep, `now_ns` advances by 1 s ± 20 ms (coarse sanity, unprivileged CI).
4. `declared_jitter_ns` is ≥ one QPC tick expressed in ns.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The QPC fact ("QPC with its published frequency") is recorded as `[measured]` from the unit test result, for 022b-08

## Out of scope
- Mapping H to the domain line (shared core, epic 018).
- Device clock S regression (022a-11).
