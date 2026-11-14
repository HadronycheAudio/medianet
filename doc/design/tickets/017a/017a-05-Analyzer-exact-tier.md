# 017a-05 — Analyzer exact tier and loss exclusion
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** 017a-04, epic 002

## Context
- Verification tiers `[decision]`:

| Tier | Proves | Mechanism |
|---|---|---|
| Transport integrity | Bytes delivered equal bytes sent, per slice | Slice checksum |
| Exact | Deterministic modules produce identical bits | The sender's replica runs the same code; compare |
| Tolerance | Non-exact modules are within stated epsilons | Analyzer windows and per-voice projections |

- Exact tier and loss `[decision]`: windows touched by loss or by a degraded slice are excluded and counted. The sender's replica never applies the echo's gaps (POC behaviour `[code]`); a received-positions bitmap that would let it do so is **deferred past v1**.
- Do not cite the POC as the exact-tier baseline until epic 002 is done.
- Open sub-question (owner: epic 002): are voice windows compared bit for bit in the POC, or only within tolerance? Falsifier: reading shows tolerance-only, and the exact-tier description for voices changes. If epic 002 finds tolerance-only, voices move to 017a-06 and this ticket's voice test is dropped.
- Settled: a hand-written module that claims exactness is checked here; one that does not falls in the bounded class (C-123) and is checked on the tolerance tier (017a-06).
- Settled: the degraded flag follows the content through a module with latency L (C-122). Stamps denote content position: while a module processes input position p it emits the output stamped p − L, whose content came from input position p − L, and that output is degraded if the input at p − L was gapped. Exclusion therefore uses the degraded flag carried by the received output slice, not a guess from the input side.

## Work
1. `analyzer/exact.rs`: compare a received window with the replica's window bit for bit; record `mismatch` with the first differing sample index and position.
2. Mark windows overlapping any lost slice or any slice carrying the degraded flag as `excluded-loss` / `excluded-degraded` before comparison.
3. Transport-integrity tier: per-slice checksum comparison, counted in the `transport-integrity` family.
4. Expose `exact_mismatches` per family in `analyzer::Report`.

## Acceptance criteria
1. Two identical streams of 100 windows: 0 mismatches, 100 compared.
2. Flipping one bit in sample 500 of window 42: exactly 1 mismatch, reported at window 42, sample 500.
3. A lost slice inside window 10: window 10 is `excluded-loss`, not a mismatch; a degraded slice inside window 11: `excluded-degraded`.
4. A corrupted slice payload with a correct header is detected by the checksum family.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Tolerance tier (017a-06); resynchronisation bound after loss (017b-11).
- Received-positions bitmap (deferred past v1).
