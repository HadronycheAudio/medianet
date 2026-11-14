# 018-05 — E statistics accumulator
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-01

## Context
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a **one-hour** certification run, with the **maximum** reported beside it (C-73). On hosted platforms E is a percentile claim, not a bound `[decision]`.
- Linux reference sink target E = **50 µs** (C-70, pending measurement) = 20 µs clock bound + 20 µs rate-tracking error + 10 µs uncertainty of D.
- Self-monitored E is computed for every submitted period and published (epic criterion 018-A2). At the 96 kHz domain rate (C-01) with 0.5 ms slices (C-02), one hour at one error per 0.5 ms period is 7.2 million values; the accumulator must not store them.
- Lives in `hadronyche-core` `edge/` so all edges reuse it: `no_std`, no allocation, no libm.

## Work
1. Add `EmissionStats` in `edge/`: fixed-size histogram of |error| in ns (log-linear bins), plus exact maximum and sample count.
2. API: `record(error_ns: i64)`, `p999_ns() -> Option<u32>`, `max_ns() -> Option<u32>`, `count() -> u64`, `reset()`, and a windowed mode (one-hour window, or rolling by sub-histograms) for continuous self-monitoring.
3. Document the quantisation error of the percentile (bin width at 50 µs must be at most 1 µs).

## Acceptance criteria
1. For 10⁶ synthetic errors from a known distribution, `p999_ns` is within **1 µs** of the exact 99.9th percentile in the 0–200 µs range.
2. `max_ns` is exact.
3. With zero records, both values are `None` (blank).
4. `size_of::<EmissionStats>()` is a compile-time constant below 16 KiB.
5. Recording 7.2 × 10⁶ values (one hour at 2 000 periods/s) takes under 1 s on CI hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Computing individual errors (018-04).
- Publishing to the regime stamp (018-18).
- The one-hour certification run itself (epic 020).
