# 017b-13 — Scenario `late-vs-lost`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-03, 017a-08, 017b-09, epic 009

## Context
- AC-15: injected delay beyond the offset is counted **late** only; injected drop is counted **missing** only.
- **Late versus missing (settled, C-108):** at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span, it is reclassified late (missing −1, late +1) and discarded. The ring is sized to A (C-96); the mixer feeds no presentation group, so the scenario configures an explicit budget that serves as its A (C-142; C-55 gives A no default). A slice delayed beyond the ring span therefore stays missing.
- Persistent lateness (C-101: at least one late slice in each of 10 consecutive 1 s windows) triggers a ratchet step. Late phases here must stay shorter than that, or the offset moves (that is 017b-15).
- Counts are read from the consumer's per-connection counters in its stamp (C-47) and compared with the injector's log (017b-03).
- The reference mixer fires at `T(p) + offset` (017a-08).

## Work
1. Scenario `late-vs-lost`, three phases on the sender→mixer link, each at most 5 s: (a) delay range entirely beyond the mixer offset but within the ring span, `loss = 0`; (b) `loss = 0.02`, delay 0; (c) delay beyond the ring span.
2. Assertions: phase (a) final late count = number of delayed slices, missing = 0, and each late slice was first counted missing then reclassified; phase (b) missing = number of slices touched by drops (injector log), late = 0; phase (c) the delayed slices remain missing; no ratchet step in any phase.

## Acceptance criteria
1. `hadronyche-l0 late-vs-lost --duration 30` exits 0.
2. Phase (a): `late > 0`, `missing == 0` at the end of the phase; the transient missing count equals the late count (C-108).
3. Phase (b): `missing > 0`, `late == 0`; phase (c): `missing` equals the number of delayed slices, `late == 0`.
4. Counts equal the injector's ground truth exactly; the offset is unchanged across the run.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Interrupted / dead classification (017b-14); ratchet (017b-15).
