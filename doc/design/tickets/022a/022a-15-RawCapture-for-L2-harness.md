# 022a-15 — `RawCapture` for the L2 harness
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-10, 022a-11, epic 020

## Context
- Part of epic 022a (Windows edge backend). Deliverable: `impl RawCapture for WasapiRawCapture { /* open, read */ }` — for the L2 harness (epic 020).
- The L2 harness measures emission timing (E, D, skew) on real hardware. E = 99.9th percentile of |actual emission − (T(p) + A)| over one hour, maximum reported (C-73).
- Code placement is settled (C-133): `WasapiRawCapture` lives in `hadronyche-os-windows`; the harness binary and its Windows build live in the `hadronyche-tools` crate.
- Capture uses WASAPI exclusive mode only (shared mode forbidden for certification) `[decision]`; each captured buffer is paired with (device position, QPC time) from the device clock (022a-11).

## Work
1. `WasapiRawCapture` wrapping an exclusive capture endpoint (022a-10).
2. `open`: open the endpoint the harness names; `read`: return raw frames plus the (device position, QPC ns) pair for the buffer, without resampling or slicing.
3. Integrate with the epic-020 harness in `hadronyche-tools` (C-133) so it builds and runs on Windows.
4. Test with a mock device.

## Acceptance criteria
1. `WasapiRawCapture` implements the epic-020 `RawCapture` trait and the `hadronyche-tools` harness binary builds on Windows.
2. `read` returns contiguous frames with monotonic device positions over 10 000 reads (mock).
3. Manual: the harness captures 60 s from a reference interface with no dropped buffer.

## Definition of done
- [ ] Tests green in CI, unprivileged

## Out of scope
- The L2 runs themselves (022b-02 to 022b-06).
