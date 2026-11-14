# 018-24 — Sink input lateness statistics and "cannot meet A" alarm
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-11, 018-14, 018-22, epic 009, epic 013, epic 015

## Context
- **C-174:** the "cannot meet A" alarm is raised when maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows. This replaces the provisional condition (emission error above declared E) that 018-14 carried before the ruling.
- **Headroom (C-52):** the largest of 25 % of the observed maximum lateness, one slice period (0.5 ms, C-02), and twice the published clock bound.
- **C-173:** "cannot meet A" is one of the connection alarms; it uses the alarm path of the hosted runtime (epic 015).
- **C-160:** the stamp carries lateness percentiles: 50th, 99th, 99.9th and the maximum. The sink, as a consumer, publishes them for its input.
- **C-182:** the percentiles and the "cannot meet A" alarm use a rolling 60 s window; the maximum since the connection went live is reported beside them. So the maximum lateness in the C-174 test is the maximum over the last 60 s, evaluated at each 1 s window. (Reclaimable latency, C-157, uses the same window but is not published by this epic.)
- **C-178:** lateness figures in the stamp are ordinary production measurements, unqualified while a commissioning window is open (C-153).
- Lateness of each arriving slice is measured by the alignment ring of epic 009; D comes from `DeclaredDelay` (018-11); A is the sink's configured audition offset (C-55); the clock bound comes from `TimeView` (epic 008).
- `AlsaSink` is a hosted module; the statistics may use fixed-size histograms like `EmissionStats` (018-05), with no allocation on the data path.

## Work
1. In `alsa/sink.rs`, record the lateness of each arriving slice (from the epic 009 ring) into a fixed-memory accumulator giving the 50th, 99th, 99.9th percentiles and the maximum over a rolling 60 s window (C-160, C-182), plus a separate maximum since the connection went live (C-182), reset when the connection goes live again.
2. At the end of each 1 s window, take the maximum lateness over the rolling 60 s window (C-182), compute headroom per C-52 and test maximum lateness + D + headroom > A.
3. Raise the "cannot meet A" connection alarm via the epic 015 alarm path when the test holds in each of 10 consecutive 1 s windows; clear it when a window passes (C-173, C-174).
4. Expose the percentiles, the rolling-window maximum and the maximum since live to the stamp fields of epic 013 (alongside 018-18), and document each (meaning, unit, window, source); flag them unqualified while a commissioning window is open (C-178).

## Acceptance criteria
1. **Epic criterion 018-A12:** on fake inputs with A = 2 ms and D = 1 ms, a maximum lateness making maximum lateness + D + headroom exceed A in each of 10 consecutive 1 s windows raises the alarm; 9 such windows followed by a passing window do not. The maximum lateness tested is the rolling 60 s maximum (C-182): a single large lateness keeps the test failing for 60 s, so the alarm is raised 10 s after it and clears once it leaves the window.
2. Headroom equals the largest of its three terms in a table-driven test covering each term being the largest (C-52).
3. For 10⁶ synthetic lateness values, the published 50th, 99th and 99.9th percentiles are within one histogram bin of the exact values over the last 60 s and the maximum is exact (C-160, C-182).
4. **C-182:** a lateness spike older than 60 s no longer appears in the percentiles or the rolling maximum but is still reported as the maximum since the connection went live.
5. The percentiles, the rolling maximum and the maximum since live appear in the sink's stamp, round-trip through the epic 013 serializer losslessly, and fit within a section of at most 1 252 bytes (C-17); figures published while a commissioning window is open are flagged unqualified (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Measuring lateness itself and the alignment ring (epic 009).
- The alarm path (epic 015).
- Choosing A (commissioning tooling, C-55; the L2 harness computes it for ladder runs, C-148).
