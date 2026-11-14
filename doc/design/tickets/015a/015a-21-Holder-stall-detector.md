# 015a-21 — Holder stall detector (C-136)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-03, 015a-10
## Context
- C-136: the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch.
- Reading those clocks is platform work, like wall-clock step detection (C-121), so the detector lives in the backend; the epoch bump itself is core's (timeline epoch and election state, epics 006, 008, 011). On Linux the monotonic clock is `CLOCK_MONOTONIC`, which does not count time in suspend, and `CLOCK_BOOTTIME` keeps running through it `[kernel-doc]`.
- The trait surface must stay implementable by the stub, macOS (021) and Windows (022) backends, so the suspend-inclusive read is a `HostClock` method that may report itself unavailable.
- The detector runs where holder timing runs: the clock thread (015a-10), checked once per loop; the outcome reaches core and the stamp lock-free (L-8, 015a-18).
## Work
1. Extend `HostClock` with a suspend-inclusive read (for example `fn suspend_inclusive_ns(&self) -> Option<i64>`); implement it with `CLOCK_BOOTTIME` on Linux and as `None` in the stub; update the crate docs.
2. In the clock thread, when this module is holder, track the difference between the two clocks; when it grows by more than 2 ms (C-67) since the last check, hand a stall to the core timeline so it bumps the epoch.
3. Count stalls and publish the count to the counter board.
4. Unit tests with an injectable suspend-inclusive clock.
## Acceptance criteria
1. A simulated divergence of more than 2 ms on a holder produces exactly one stall hand-off to core and one counted stall.
2. A divergence of 2 ms or less produces none.
3. A non-holder never reports a stall.
4. The stub backend compiles with the extended trait and reports the read as unavailable; the detector then stays idle.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The trait surface compiles with a stub backend (proves macOS and Windows can implement it)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — the stall count rides the board; its stamp field is not specified (epic 004); "Design documents updated" — follows the epic; "Per-packet receive cost measured" — not a receive path.
## Out of scope
- The epoch bump and its propagation (core, epics 006, 008, 011).
- A real suspend and resume of the test machine.
