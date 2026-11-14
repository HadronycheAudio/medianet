# 017a-04 — Analyzer windows and per-family sample counts
**Epic:** 017a-Reference-modules · **Effort:** M · **Depends on:** epic 002

## Context
- Deliverable: `hadronyche-reference/src/analyzer/` — ported windows, projections and comparison, **with per-family sample counts**. This ticket ports the windowing and bookkeeping; comparison is 017a-05 (exact) and 017a-06 (tolerance).
- POC analyzer `[code]` (`src/audio.rs`): 1 024-sample windows (`:26`). A window is compared only when complete on both sides; a window still incomplete 256 windows later (`MAX_LAG_WINDOWS`, `:31`, about 2.7 s) is counted *lost* or as a *sender gap* and never analysed. A synth silent for over a second gets a 3-second settling exclusion (`SETTLE_SECS`, `:34`).
- POC bugs `[measured]`: in a 25 s loopback that looked green, the "lost" counter random-walked around 21.7 million with zero actual loss, and the whole music and transport check families reported "0 of 0 windows". Epic 002 diagnoses these; the port must fix them, not copy them.
- L0 rule `[decision]`: **a check family with zero samples is a failed run** (AC-9). The analyzer must therefore expose a sample count per family.
- **Report presentation (settled, C-206):** the layout of `analyzer::Report` output is decided by this part and recorded in its documentation; only what 017b's runner depends on (per-family counts and the inert-family list) is a contract.
- Exact-tier loss rule `[decision]`: windows touched by loss or by a degraded slice are excluded and counted.

## Work
1. Port window assembly from POC `audio.rs` into `analyzer/window.rs`: window size 1 024 samples, `MAX_LAG_WINDOWS = 256`, `SETTLE_SECS = 3`.
2. Window states: compared, excluded-loss, excluded-degraded, excluded-settling, lost, sender-gap. Every window ends in exactly one state.
3. Define check families (at least: `audio`, `music`, `transport`, `transport-integrity`) and a `FamilyCounts { compared, excluded_by_reason, lost, sender_gap }` per family.
4. Implement `analyzer::Report` with all counts and a function `inert_families(&Report) -> Vec<Family>` returning families with `compared == 0`.
5. Fix the lost-counter bug identified by epic 002 and add a regression test.
6. Header comment citing POC file/lines and the epic 002 finding.

## Acceptance criteria
1. Feeding two identical complete streams of 100 windows gives `compared = 100` for that family and zero in every other state.
2. A window incomplete for 256 windows becomes `lost` (or `sender-gap` when the sender side is missing) and is never compared; at 255 it is still pending.
3. A synth silent for > 1 s has the following 3 s of windows marked `excluded-settling`.
4. With zero injected loss over 25 s, the lost count is exactly 0 (regression for the 21.7 million bug).
5. For every family, `compared + Σexcluded + lost + sender_gap` equals the number of windows closed.
6. `inert_families` returns `music` when the music family has no compared window.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- Bit-exact comparison (017a-05); tolerance-tier projections (017a-06).
- Failing the run on an inert family (017b-09).
