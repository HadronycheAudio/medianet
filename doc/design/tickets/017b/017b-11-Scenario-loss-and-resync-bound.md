# 017b-11 — Scenario `loss` and the resynchronisation bound
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-03, 017a-05, 017b-10

## Context
- AC-8, loss half: with injected offset, drift **and packet loss**: zero exact-tier mismatches outside excluded windows; estimated minus true offset within the lock bound (C-62, 25 µs, pending simulation); **packet and slice loss, as published in the producer and consumer stamps (C-47), equal to the injector's logged drops**; no assertion changes when the sync rate changes from 10 to 200/s (C-112).
- **Loss accounting (settled):** the injector is seeded and deterministic and logs every drop with its stream and position (017b-03), so loss is asserted as exact equality with that log, not as a statistical tolerance. A slice is missing if any of its packets was dropped; at defaults one slice is one packet (C-05), so slice loss equals packet loss. Lost slices are counted missing, never late (C-108), and drops must not produce eight consecutive misses on a stream, or the input goes interrupted (C-38).
- Exact tier and loss `[decision]`: windows touched by loss or by a degraded slice are excluded and counted; the reference instrument must **resynchronise within a stated bound (inherit: 3 s settling)**.
- **Open sub-question owned by this ticket**, from epic 002 findings and this scenario's measurement: the resynchronisation bound after loss (inherit 3 s?). Falsifier: a loss scenario shows divergence lasting longer. The measurement report goes to `doc/design/reports/` and the decided value to `doc/design/02-constants.md` (C-130).
- Deferred past v1: a received-positions bitmap letting the replica apply the echo's gaps. Falsifier: excluding loss-touched windows hides a real defect class.
- Link injection: `LinkInjection { seed, loss, delay_ns, reorder, drop_above_bytes }` (017b-03).

## Work
1. Scenario `loss`: `baseline` injections plus `set_link` with `loss = 0.01` (and a second run at 0.05), fixed seed, on sender→echo and sender→mixer.
2. Assertions: exact mismatches = 0 outside excluded windows; offset error ≤ C-62; the missing count in the consumer's C-47 counters equals the number of slices with at least one logged drop, exactly; late = 0; every divergence after a loss ends within the resync bound.
3. Measure the longest divergence after loss over the run; decide the bound (3 s unless epic 002 findings or measurement say otherwise); write the measurement to `doc/design/reports/` and the bound to the constants table with an evidence tag; close the open sub-question.
4. Sync-rate invariance test at 10/s and 200/s as in 017b-10.

## Acceptance criteria
1. `hadronyche-l0 loss --duration 120` exits 0 at `loss = 0.01` and `loss = 0.05`.
2. Runtime-counted missing slices equal the injector's logged drops exactly; lost windows > 0 and all are excluded, none counted as mismatch.
3. Longest post-loss divergence ≤ the resync bound (default 3 s); the run fails if any divergence exceeds it.
4. Identical assertion outcomes at sync rates 10/s and 200/s.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Received-positions bitmap (deferred past v1).
- Late vs lost classification (017b-13).
