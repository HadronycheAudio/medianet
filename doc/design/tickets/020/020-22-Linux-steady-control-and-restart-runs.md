# 020-22 — Linux steady, control and restart runs
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-14, 020-15, 020-21, 020-27

## Context
This ticket runs the first half of the Linux L2 certification on the reference rig (020-21). **AC-26 (Linux)** of the epic `[decision]`: E at most 50 µs over one hour (99.9th percentile; maximum reported) (C-70, C-73); restart spread (max − min, C-166) at most 25 µs over 20 restarts (C-74), D measured over a 60 s window after each restart (C-166); marker offset within ±2 samples of the median of the first 60 s (C-75, C-166); zero slip after loss, the 50 ppm handover and an announced step (C-76, covered by 020-23); MTIE at most 10 µs up to 1 s and at most the 50 µs target overall (C-77, C-149). E budget: 50 µs = 20 clock + 20 tracking + 10 D. Typical Linux D about 1 ms (C-81). 1 sample = 10.4 µs at 96 kHz (C-01).

**Epic criterion 020-A3, trap 1 demonstrably avoided:** a control run with no injection shows a servo that is idle (ratio tracks only the device crystal); the certified run shows the servo active.

Open sub-question owned here `[decision]`: *Is 50 µs reachable on a stock kernel, or does the Linux reference need a real-time kernel?* Falsifier: target met on a stock kernel over one hour.

Open sub-question settled here by measurement: the threshold separating an idle servo from an active one (criterion 020-A3), provisional in 020-14.

The audition offset A has no default (C-55); the harness computes it with the commissioning rule (C-148, 020-26) and the record carries it with its inputs. These runs feed the constants table, so they run in strict mode (C-147, 020-27): real-time scheduling on the data and clock threads and memory locked, an unverified capability refused (C-127, C-128).

Epic definition of done: the analysis code runs in CI on recorded capture files.

## Work
- On the rig, in strict mode (C-147), with a **stock kernel**: run `steady` (1 h), the control run (`steady --no-injection`, 1 h), and `restarts` (20).
- If the stock-kernel `steady` run misses E ≤ 50 µs, repeat `steady` and `restarts` on a real-time kernel and record both results; the answer to the sub-question is whichever kernel meets the target, with figures.
- Keep the full recordings as artefacts; commit a short excerpt (a few minutes) of the steady recording as a CI fixture and add a CI test that re-runs the analysis on it and compares with the committed numbers.
- From the control and injected runs, fix the idle/active threshold, with the two distributions of the servo-activity metric as evidence `[measured]`, and update 020-14's provisional value.
- Produce the record fragments (020-13) for these runs.

## Acceptance criteria
1. Steady run: E (99.9th percentile) and maximum reported; pass/fail against 50 µs `[measured]`.
2. Steady run: marker offset verdict (±2 samples against the median of the first 60 s, C-166) and MTIE verdict (10 µs up to 1 s, ≤ 50 µs overall, C-149) reported `[measured]`.
3. Restarts: 20 values of D and their spread, pass/fail against 25 µs `[measured]`.
4. Control vs injected runs show the servo idle vs active, and the measured threshold separating them is recorded with its evidence (epic criterion 020-A3).
5. The stock-kernel question is answered with figures from at least one 1-hour run.
6. The CI test on the committed excerpt passes unprivileged without audio hardware.
7. Every run's record shows strict mode with real-time scheduling and memory lock present (C-127, C-147).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Disturbance runs (020-23).
- Writing the results document and constants (020-24).
