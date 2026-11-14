# 011-16 — Epoch merge and partition heal
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-15, epic 005

## Context
When two partitions of a medianet rejoin, each may have its own holder and election epoch. Rules `[decision]`:
- **Merge:** higher election epoch wins; rank is a stale snapshot and is not compared across epochs.
- Partition heal: one holder within election window plus hello period: 400 ms (C-33) + 1 s (C-30) = 1.4 s.
- Every module computes the same winner from the same candidate set, deterministically.

`[code]` POC behaviour to keep: epoch adoption on hello (`module.rs:547-557`). The hello carries election epoch and believed holder, stored in the `PeerMap` (011-06); `Election::step` reads them from `map`.

Epic AC 4, AC-36: after a healed partition, one holder within 400 ms + 1 s = 1.4 s; every module computes the same winner.

Still open (epic open sub-question, owner epic 005): what resolves a heal where both partitions carry the **same** epoch with different holders. Falsifier: a heal vector in which two holders persist at the same epoch past 1.4 s. Implement epic 005's rule; do not invent one.

## Work
1. In `Election::step` (on `Tick` and `Message`), compare the own epoch with the epochs and believed holders found in `map`; adopt any strictly higher epoch and its holder, emitting `HolderChanged`.
2. Never compare rank across epochs.
3. Equal-epoch, different-holder conflict: handle per epic 005's table.
4. Simulation test harness: N modules, a partition switch, deterministic clock.

## Acceptance criteria
1. Two partitions of 4 modules each with holders at epochs 7 and 9: after heal, every module reports the epoch-9 holder within 1.4 s.
2. Over 200 randomized heal scenarios (including equal-epoch ones once epic 005 defines the rule), every module's final holder is the same and is reached within 1.4 s of the heal.
3. A module never adopts a lower epoch than its own.
4. Rank values are never compared between different epochs (asserted in the implementation, covered by a test that would fail if a lower-epoch better rank won).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (behaviour follows the epic and epic 005); "No election path reachable from hello expiry" (adoption reads epochs from hellos but never opens an election; covered by 011-19).

## Out of scope
- Opening an election: 011-15. Handover continuity: 011-17.
