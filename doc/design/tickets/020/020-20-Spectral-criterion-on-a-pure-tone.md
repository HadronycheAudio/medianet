# 020-20 — Spectral criterion on a pure tone
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-02, 020-08

## Context
The L2 pass numbers are time-domain: E (99.9th percentile of emission error over 1 h, C-73; Linux 50 µs, C-70), marker offset constancy (C-75), zero slip (C-76), MTIE (C-77) `[decision]`. Resampler wander can also show up as pitch artefacts — sidebands around a pure tone — that the time-domain criteria might miss.

Open sub-question owned by epic 020 `[decision]`:

| Question | Falsifier |
|---|---|
| A spectral criterion (sidebands on a pure tone) | A pitch artefact is audible while all criteria pass |

At 96 kHz (C-01) one sample is 10.4 µs `[decision]`.

## Work
- Add to `analysis.rs` a sideband measurement: given a raw capture of a pure tone (frequency configurable) played by the sink, compute the spectrum with a long window and report the level of the largest sideband relative to the carrier in dBc, and its offset frequency.
- Add an option to the marker generator / reference source (020-03) or a scenario flag to play a pure tone instead of (or alongside) the markers.
- Evaluate with synthetic recordings (020-08 generator, extended with a phase-modulated tone) the sideband level produced by wander that passes all time-domain criteria.
- Decide: either propose a spectral criterion (threshold in dBc with its basis; a figure from a published standard is cited with `[standard]` and the clause, C-132) `[decision]`, or record that the falsifier was not met and no criterion is needed, with evidence.

## Acceptance criteria
1. On a synthetic tone phase-modulated with a known modulation index, the measured sideband level matches the theoretical value within 1 dB.
2. A decision note exists (in the `analysis.rs` module doc and carried into the results document by 020-24) with the proposed threshold or the reason none is needed.
3. The measurement runs in CI on a recorded file without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Making the spectral criterion a certification pass/fail for macOS or Windows (epics 021, 022).
- MTIE mask changes (020-19).
