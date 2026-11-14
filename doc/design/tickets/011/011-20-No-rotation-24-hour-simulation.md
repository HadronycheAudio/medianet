# 011-20 — No rotation: 24-hour simulation
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-19

## Context
Rule `[decision]`: **no rotation** — the holder changes only on loss or pressure (or operator force).

`[code]` POC behaviour retired: rotation after 2 mandates (`MANDATES = 2`, `module.rs:62`); election re-requests every 10 s (`REQUEST_EVERY_NS`).

Epic AC 6: in a 24-hour simulated run without loss or pressure, the holder never changes.

Timing: hello period 1 s (C-30), election window 400 ms (C-33), reply-silence trigger 1.5 s (C-37).

## Work
1. Using the simulation harness from 011-19 with a virtual clock (no real sleeping), run 8 modules (3 holder-eligible and locked) for 24 simulated hours, ticking at least every 10 ms, with replies always flowing and no `Pressure`/`Forced` inputs.
2. Count `HolderChanged` and `Opened` events after the initial election, draining `next_event()` after every input.
3. Keep the run time in CI bounded (target under 2 minutes); if too slow, run it nightly and keep a 1-hour variant per change.

## Acceptance criteria
1. After the initial election, zero `HolderChanged` and zero `Opened` events in 24 simulated hours on every module.
2. Grep/inspection confirms no mandate counter or periodic re-request timer exists in `Election`.
3. The test fails if a rotation rule is reintroduced (checked once by a deliberate mutation on a branch).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Every parser touched is fuzzed" (tests only); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (no behaviour change); "No new open item without an owner and a falsifier" (opens no item); "No election path reachable from hello expiry" (covered by 011-19; this run has no expiry).

## Out of scope
- Election triggers and window: 011-15.
