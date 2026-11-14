# 020-18 — Drift-ramp scenario
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-14

## Context
The L2 scenario list includes **`drift-ramp`**. The sink follows an in-process reference holder whose clock drift is injected (epic 017 ground-truth injection API); the injected values are the ground truth `[decision]`.

The scenario is settled by C-119 `[decision]`: **the injected crystal drift sweeps ±100 ppm at no more than 0.5 ppm/s; pass is E held throughout.** The slew limits C-60 (at most 10 ppm from the holder's rate) and C-61 (at most 1 ppm/s) bound the servo's **correction term**, not the drift estimate it tracks. A ±100 ppm sweep is therefore compatible with a 10 ppm slew limit: the drift estimate follows the holder, and the correction term stays small. This is epic criterion 020-A9.

Other figures `[decision]`: Linux E target 50 µs (C-70), E = 99.9th percentile of absolute emission error with the maximum reported (C-73); lock bound at most 25 µs (C-62); rate loop bandwidth well below 1 Hz (C-68). At 96 kHz, the domain rate and only rate on the wire (C-01), one sample is 10.4 µs.

## Work
- Scenario `drift-ramp` in `run.rs`: injected holder drift swept from −100 ppm to +100 ppm and back, repeated to fill the run duration. The sweep rate is configurable up to 0.5 ppm/s (default 0.5 ppm/s, so one 200 ppm leg takes 400 s); a requested rate above 0.5 ppm/s is refused (C-119).
- Record injected drift (ground truth), steered ratio, the servo's drift estimate and correction term (separate series, read through the epic 018 sink interface), and emission error.
- Pass verdict: E over the whole sweep within the platform target (Linux 50 µs), the target and never an E measured in another run (C-149). Also report E per drift bin so that a local excursion at one end of the sweep is visible.
- Report: correction-term verdicts against C-60 and C-61 (020-07), tracking error (drift estimate vs injected drift) as a function of drift, and the C-75 and C-77 figures for information.

## Acceptance criteria
1. `hadronyche-l2 run drift-ramp` sweeps the injected drift across the full ±100 ppm range, verified from the recorded ground truth (min ≤ −100 ppm, max ≥ +100 ppm), with no sweep segment faster than 0.5 ppm/s.
2. A requested sweep rate of 1 ppm/s is refused with a message naming C-119.
3. The report contains the E pass verdict over the sweep, E per drift bin, tracking error vs drift and the C-60 and C-61 correction-term verdicts.
4. On a simulated servo whose correction term stays within 10 ppm and 1 ppm/s while its drift estimate follows the ±100 ppm sweep, C-60 and C-61 pass; on one whose correction term reaches 12 ppm, C-60 fails.
5. A shortened CI version passes unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Handover (020-17), loss and step (020-16).
- Servo tuning (epics 017, 018; servo simulation in epic 008).
- The real Linux drift-ramp run (020-23).
