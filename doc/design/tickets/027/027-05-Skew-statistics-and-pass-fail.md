# 027-05 — Skew statistics and pass/fail
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-01

## Context
- `report.rs` of binary `hadronyche-l3` (crate `hadronyche-tools`) turns a skew series into a verdict:
  ```rust
  pub struct SkewReport { pub p50_ns: i64, pub p999_abs_ns: i64, pub max_abs_ns: i64, pub limit_ns: i64, pub pass: bool }
  ```
- Each sink publishes **E**: the 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, with the maximum reported beside it (C-73). The Linux reference sink's target is 50 µs (C-70); the embedded device's E is declared per device (epic 026).
- **Pass criterion (C-79, settled statistic):** the 99.9th percentile of the absolute skew is at most the sum of the two sinks' E values, for 1 hour and across restarts of either; the maximum is reported beside it and not asserted. So `limit_ns = E_linux_ns + E_device_ns` and `pass = p999_abs_ns <= limit_ns`.
- The E values are the targets, never the E measured in the same run (C-149): E_linux is the C-70 target, E_device the device's declared E (epic 026).
- A sink with no certified E declares E as unknown (C-103); only the certification state commissioned gives a numeric E (C-140). No limit can be computed for an unknown E, so the verdict cannot pass.
- `skew_ns` is channel 2 minus channel 1 (device minus Linux).

## Work
1. `pub fn summarize(series: &[SkewSample], e_linux_ns: Option<i64>, e_device_ns: Option<i64>) -> SkewReport` (`None` = E unknown, C-103, C-140; the values passed are target E values, C-149), plus a diagnostic reason when `pass` is false for a reason other than the limit.
2. `p50_ns` = median of signed `skew_ns`; `p999_abs_ns` = 99.9th percentile of `|skew_ns|`; `max_abs_ns` = maximum of `|skew_ns|`; `limit_ns = e_linux_ns + e_device_ns`.
3. `pass = p999_abs_ns <= limit_ns` (C-79). State the rule and its C-id in the doc comment.
4. Use a documented percentile definition (nearest-rank). An empty series ⇒ `pass = false` with reason "empty series"; an unknown E ⇒ `pass = false`, `limit_ns = 0`, with reason "E unknown (C-103)".
5. A helper to combine several series (for the restart and power-cycle scenarios: one verdict per restart and one overall verdict that passes only if every restart passes).

## Acceptance criteria
1. On the series 1..=1000 ns, `p50_ns` is 500 or 501 (nearest-rank documented), `p999_abs_ns` = 999, `max_abs_ns` = 1000.
2. Negative values: on −1000..=−1 the absolute percentiles equal those of criterion 1 and `p50_ns` is negative.
3. With E_linux = 50 000 ns (C-70) and E_device = 30 000 ns (a test value), `limit_ns` = 80 000; a series with p99.9 |skew| = 79 999 passes and 80 001 fails, even when `max_abs_ns` exceeds 80 000.
4. Empty series ⇒ `pass == false`.
5. Either E unknown ⇒ `pass == false` with the C-103 reason.
6. Combined verdict over 20 restart series fails if exactly one series fails.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none (E values arrive as numbers); "Design documents updated" — implements C-79 as written; "No new open item" — opens none; "The skew analysis runs in CI on recorded two-channel files" — 027-08; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Drift slope, step and return-to-baseline checks (027-06).
- Rendering the report with stamps (027-07).
- Subtracting the capture device's inter-channel skew (027-10) — `summarize` receives an already corrected series.
