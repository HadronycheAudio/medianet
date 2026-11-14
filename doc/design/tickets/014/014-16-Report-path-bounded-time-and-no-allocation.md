# 014-16 — Report path bounded time and no allocation
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-03, 014-06
## Context
Receiver reports are generated on the consumer's **data thread** (invariant L-1: an admitted, healthy connection becomes dead only from data-plane evidence; the producer stops after 2 s without reports, C-41, so reports must not depend on housekeeping) `[decision]`. Definition of done of the epic: "Report generation takes bounded time on the data thread and is shown not to allocate or lock." Worst case: a 32-member bundle (C-08), at most 1 252 bytes encoded (C-17, C-100). The data thread cycles once per slice (48 frames at 96 kHz = 0.5 ms, C-01, C-02).

Settled locations: harness and tool binaries live in the `hadronyche-tools` crate (C-133); measured results go to `doc/design/reports/` (C-130). Any run whose numbers enter the constants table runs in strict mode (C-147); the benchmark here goes to a report only, so it need not, unless its figure is proposed for the table. The data thread's cycle is one slice period (C-156).

Still open (README remaining gaps, owner a measurement here, then epic 015 which owns the data thread's cycle budget): no numeric time budget for report generation is set. This ticket supplies the measurement.

## Work
1. Add a test harness (std, test-only) with a global allocator that panics on any allocation during a marked region; run `ReportTimer::due` + `encode_report` for a 32-member bundle inside it.
2. Show no locking: the module `measure` contains no mutex, spin lock or atomic read-modify-write loop; enforce with a CI grep or `clippy` lint.
3. Show bounded time: the code path has no loops other than over members (≤ 32) and fixed-size fields; add a micro-benchmark in `hadronyche-tools` (C-133) reporting worst-case duration for 32 members, and record the measured figure with `[measured]` in `doc/design/reports/` (C-130).

## Acceptance criteria
1. The no-allocation test passes for 0, 16 and 32 members.
2. The no-lock check runs in CI and passes.
3. The benchmark result for 32 members is recorded with `[measured]` in `doc/design/reports/`, and the per-call work depends only on the member count (≤ 32), not on counter values.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Report generation takes bounded time on the data thread and is shown not to allocate or lock
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Real-time scheduling of the data thread and strict mode (C-127; epic 015).
- The bad-input time bound of the parsers (C-164, 014-25).
