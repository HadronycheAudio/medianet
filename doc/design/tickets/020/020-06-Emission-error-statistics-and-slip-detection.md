# 020-06 — Emission-error statistics and slip detection
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-05

## Context
From the per-marker emission-error series (020-05) the harness computes the certification figures `[decision]`:

- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a **one-hour** run, with the maximum reported beside it (C-73). On hosted platforms E is a percentile claim, not a bound.
- **Round-trip marker offset constant** over the run (C-75): on Linux within ±2 samples; on macOS and Windows within that platform's restart spread (C-74: 50 µs and 0.5 ms).
- **Zero accumulated slip** (C-76): after each disturbance the offset returns to its prior value, on Linux within ±1 sample; on macOS and Windows within that platform's restart spread (C-74: 50 µs and 0.5 ms). A one-sample slip passes on Linux and is reported as a slip event (C-166).

Settled test parameters (C-166): sample tolerances are measured against the median of the first 60 s of the run; offsets before and after a disturbance are compared over 10 s windows. The 10 s window after a disturbance starts when the injected event ends (C-191); there is no separate settling time.

At 96 kHz (C-01) one sample is 10.4 µs, so ±2 samples is about ±21 µs `[decision]`.

## Work
In `analysis.rs`:
- `emission_stats(series) -> { p999_abs_ns, max_abs_ns, count }` (99.9th percentile of absolute error and maximum, C-73).
- Marker-offset series: the round-trip offset per marker, in samples and in ns, and its deviation from the run's reference value. The reference is the median of the first 60 s of the run (C-166); document it in the module doc.
- `offset_constant(series, tolerance)` returning pass/fail and the worst deviation (C-75).
- Slip detection: given each disturbance's start and end timestamps, compare the offset over the 10 s window after it, starting at the end of the injected event (C-191), with the offset over the 10 s window before it (C-166); report the difference d. Two separate outputs:
  - **slip event**: flagged whenever |d| rounds to one whole sample or more (|d| ≥ 0.5 sample), so that a 1-sample slip is always flagged (needed by the epic's self-check, criterion 020-A5, ticket 020-08);
  - **C-76 verdict**: pass if |d| is within the platform tolerance (Linux ±1 sample).
  The tolerances are per platform with Linux values ±2 samples for C-75 and ±1 sample for C-76; the comparison window is 10 s (C-166) and the window after a disturbance starts when the injected event ends (C-191).
- The epic states this split as harness behaviour: a slip event is any settled-offset change that rounds to a non-zero whole number of samples, and the C-76 verdict is separate. A 1-sample slip on Linux passes the C-76 verdict and is reported as a slip event (C-166); the record carries both outputs.
- Per-platform tolerance table (Linux, macOS, Windows) as above.

## Acceptance criteria
1. On a series of 10,000 values with a known distribution, `p999_abs_ns` matches a reference implementation exactly, and `max_abs_ns` equals the true maximum.
2. A permanent post-disturbance step of exactly 1 sample (10.4 µs at 96 kHz) raises a slip event with size reported within ±0.1 sample and passes the Linux C-76 verdict (C-166); a step of 1.5 samples raises a slip event and fails the Linux C-76 verdict (±1 sample); a 0.3-sample transient that returns to the prior offset raises no slip event and passes.
3. A series whose offset wanders by 3 samples fails C-75 with Linux tolerance ±2 samples and reports a worst deviation of 3 samples ± 0.1.
4. The offset deviation is measured against the median of the first 60 s (C-166): a test series whose first 60 s sit 1 sample away from the rest of the run reports a worst deviation of 1 sample ± 0.1, and the reference is documented in the module doc.
5. The window after a disturbance starts at the event's end timestamp (C-191): a test series with a 2-sample offset excursion confined to the event, returning to the prior offset by its end, raises no slip event and passes C-76; the same excursion persisting 1 s past the event's end shifts the after-window mean and is reported.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Ratio series and MTIE (020-07).
- The end-to-end self-check with a synthetic 30 µs error and 1-sample slip (020-08).
- Cross-sink skew against the Linux reference sink (epics 021, 022).
