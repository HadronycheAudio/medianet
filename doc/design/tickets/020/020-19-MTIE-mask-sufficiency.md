# 020-19 — MTIE mask sufficiency
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-07, epic 008

## Context
The L2 wander criterion is a two-point MTIE mask (C-77) `[decision]`: **MTIE at most 10 µs for intervals up to 1 s, and at most E for any interval** (E is the platform target, never the E measured in the same run, C-149: Linux 50 µs, C-70). MTIE for an interval τ is the largest peak-to-peak variation of the time error within any window of length τ.

Open sub-question owned by epic 020, together with the servo simulation results of epic 008 `[decision]`:

| Question | Falsifier |
|---|---|
| MTIE mask: are the two proposed points sufficient, or is a full mask needed? | An audible or measurable artefact passes the two-point mask |

## Work
- Take the servo simulation outputs of epic 008 (time-error series under the simulated conditions) and, with the synthetic generator of 020-08, construct time-error series that pass the two-point mask but contain candidate artefacts (for example periodic wander at frequencies between the 1 s point and the run length, or slow wander just under E).
- For each, determine whether it is measurable (with the 020-07 analysis and the 020-20 spectral check if available) or audible (documented listening test or a stated psychoacoustic criterion with its source; a figure from a named, published standard carries `[standard]` with the clause, C-132).
- Write a short decision: two-point mask kept `[decision]`, or a full mask proposed with its points, and the falsifier result.
- If a full mask is proposed, extend `wander_verdict` in `analysis.rs` with the extra points and tests.

## Acceptance criteria
1. At least 3 artefact candidates that pass the two-point mask are evaluated, each with a measurable/audible verdict and evidence tag.
2. A decision is written (in the module doc of `analysis.rs` and carried into the results document by 020-24) stating whether the falsifier was met.
3. If a full mask is adopted, `wander_verdict` has a test where a series passing the two-point mask fails the full mask.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The base MTIE computation (020-07).
- Servo design changes (epic 008 and the servo epics).
- The spectral criterion itself (020-20).
