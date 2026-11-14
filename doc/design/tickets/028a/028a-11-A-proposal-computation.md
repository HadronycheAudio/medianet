# 028a-11 — A proposal computation library
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** XS · **Depends on:** 028a-02

## Context
The audition offset A is proposed, **never set live**. A is operator configuration per presentation group with no default value (C-55); changing it is a domain-wide discontinuity. Every sink emits sample p at `emit(p) = T(p) + A` and submits it at `emit(p) − D`. A is also the skew budget cap; alignment rings are sized to A (C-96).

Proposed **minimum feasible A** follows the commissioning rule (C-148): max over the group's sinks of (maximum lateness + D + headroom), where:
- maximum lateness = what each consumer measures in its aligning state: 4 s observation (C-50), maximum observed (C-51); the stamp publishes it beside the 50th, 99th and 99.9th percentiles (C-160), over a rolling 60 s window, with the maximum since the connection went live beside them (C-182). The proposal uses the maximum since the connection went live, never the rolling 60 s figure (C-148); 028b-11 supplies that value;
- D = the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170); typical Linux ~1 ms, macOS a few ms, Windows 6 to 20 ms (C-81);
- headroom = the consumer rule C-52: the largest of 25 % of the maximum lateness, one slice period (0.5 ms, C-02) and twice the published clock bound (published by the servo core, epic 008a, C-187).

The ladder harnesses compute A for L1, L2 and L3 runs with the same rule (C-148); if `hadronyche-tools` already holds that function when this ticket starts, reuse it instead of writing a second one.

Measurements are taken under a provisional A (028b-11). A measurement in which an input reached that ceiling or was Demoted (C-109) is truncated and must not be used.

## Work
- Pure function `propose_a(samples: &[SinkMeasurement]) -> Result<Proposal, ProposalError>` where `SinkMeasurement { sink, lateness_max_us /* since-live maximum, C-148 */, d_us, clock_bound_us, hit_ceiling: bool }` and `Proposal { a_us, per_sink: Vec<{sink, lateness, d, headroom, total}>, limiting_sink }`.
- Return `ProposalError::Truncated { sinks }` if any measurement has `hit_ceiling`.
- Rounding: round A up to the configuration's resolution (`a_ms` with 0.1 ms).
- Unit tests.

## Acceptance criteria
1. For two sinks (lateness 2.0 ms, D 1 ms, clock bound 10 µs) and (lateness 1.0 ms, D 15 ms, clock bound 10 µs), the proposal is driven by the second sink and equals 1.0 + 15 + max(0.25, 0.5, 0.02) = 16.5 ms.
2. Each per-sink row shows lateness, D and headroom (epic AC-4, part).
3. A measurement flagged `hit_ceiling` yields `Truncated` naming the sink, and no A.
4. Function performs no I/O.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Collecting measurements and writing configuration (028b-11).
