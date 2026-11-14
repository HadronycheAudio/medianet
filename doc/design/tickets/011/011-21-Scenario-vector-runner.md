# 011-21 — Scenario vector runner
**Epic:** 011-Gossip-and-discovery-core · **Effort:** M · **Depends on:** 011-10, 011-11, 011-12, 011-16, 011-17, 011-18, 011-23, 011-24, 011-26, epic 007

## Context
Epic 007 provides scenario vectors for discovery and election, in a format epic 007 defines; epic 005 supplies their state tables and scenario scripts and epic 004 the wire layouts and reject reasons (C-193). The vectors crate sits at the workspace root (C-192). Epic AC 1: all discovery and election scenario vectors pass (system criterion AC-1). The vectors are also the falsifiers of the open questions owned by epic 005: the later-joiner rule, the stall rule, the equal-epoch merge and the follower-attach input.

Machines under test (all in `hadronyche-core`, `no_std`, no allocation): `Discovery` (`on_hello`, `on_tick`, `next_event`, `mark_live`, `redraw_id`), `Election::step` with `ElectionInput::{ReplySilence, Pressure, Forced, Message, Tick}` and `Election::next_event`. Collision vectors set identities through the test-only forced-identity hook (C-120, 011-24).

## Work
1. A test runner that loads each epic 007 discovery and election vector, drives the machines with the given inputs and timestamps, drains `next_event()` after every input, and compares emitted `MapEvent`/`ElectionEvent` values (in order) and `Outbox` bytes with the expected ones.
2. The runner runs unprivileged in CI on the host; the machines are the same `no_std` code.
3. For each failing vector: fix the machine, or file the discrepancy against 005/007 with an owner and a falsifier.

## Acceptance criteria
1. Every discovery vector and every election vector of epic 007 passes; the runner prints the count (e.g. "N/N passed").
2. For every multi-module election vector, all modules report the same winner.
3. Adding a new vector file requires no code change.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] No election path is reachable from hello expiry (shown by test and by inspection of the transition table)

Not applicable (C-125): "Every parser touched is fuzzed" (the vector loader reads trusted test files from epic 007, not network input); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (discrepancies are filed against 005/007); "The frozen POC is not modified" (tests only).

## Out of scope
- Writing the vectors: epic 007.
