# 027-04 — Synthetic file generator and harness self-check
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-02, 027-03

## Context
- The L3 tool `hadronyche-l3` measures skew (channel 2 minus channel 1) between the Linux reference sink (channel 1) and the embedded reference device (channel 2) from one two-channel capture.
- Epic acceptance criterion **027-AC-8, Harness self-check:** "On a synthetic two-channel file with a known 37 µs skew the tool reports 37 µs within 2 µs."
- The tool must be trusted before it judges sinks; this ticket provides the synthetic input and the check.
- Capture rate: 96 kHz, the domain rate and the only rate on the wire (C-01); one sample = 10.4 µs, so 37 µs is about 3.55 samples (sub-sample).

## Work
1. Subcommand `hadronyche-l3 synth --skew-ns <n> --rate <hz> --seconds <s> --out <file>` writing a two-channel file (via `write_capture` from 027-02) containing the marker stream described by a `MarkerPlan`, channel 2 delayed by exactly `n` ns (fractional delay applied by band-limited interpolation, not by sample shift).
2. Optional parameters for later tests: linear drift (ns per hour), a step at a given position, missing markers in one channel, additive noise level.
3. Subcommand `hadronyche-l3 selfcheck` that generates the 37 µs file in memory at 96 kHz, runs `measure_skew`, and passes if the median skew is within 37 µs ± 2 µs.
4. Commit one generated 37 µs file (short, for example 10 s) as a CI fixture.
5. `selfcheck` result and skew value are printed and also included in every scenario report (027-07).

## Acceptance criteria
1. **027-AC-8.** `hadronyche-l3 selfcheck` reports a skew in [35 µs, 39 µs] on the 37 µs file at 96 kHz and exits 0.
2. With `--skew-ns 37000` and a drift option of 0, every individual `SkewSample` is within 37 µs ± 2 µs (not only the median).
3. `selfcheck` exits non-zero if the reported skew is outside the ±2 µs window (tested by temporarily injecting a 5 µs offset in a test).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The skew analysis runs in CI on recorded two-channel files
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — uses the 027-02 reader, adds no parser; "Regime stamp fields touched" — none; "Design documents updated" — no design behaviour; "No new open item" — opens none; "Every new factual claim carries an evidence tag" — no claim beyond the test result; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- CI job wiring for real recorded files (027-08).
- Statistics beyond the median (027-05).
- Self-check at other capture rates: the rig captures at 96 kHz only (C-01).
