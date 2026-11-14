# 005-11 — Election model check
**Epic:** 005-Control-state-machines-specification · **Effort:** M · **Depends on:** 005-10

## Context
- Definition of done of epic 005: "A model check or exhaustive small-scope exploration shows no reachable state with two holders after convergence".
- 005-AC-4 (system AC-36): partition heal converges to one holder within election window plus hello period: 400 ms (C-33) + 1 s (C-30) = 1.4 s, counted from the moment connectivity is restored.
- The election table lives in `doc/design/spec/control.toml` (machine `election`, 005-10). Rules: trigger on 1.5 s sync-reply silence (C-37), never on hello expiry (L-6); window 400 ms; deterministic winner (005-08: a Locked candidate before one in holdover, then the lowest module id, C-221); candidates need usable time (Locked, or in holdover with the claim not lapsed, C-213); with no such candidate the lowest eligible id wins and bumps the timeline epoch (C-113), otherwise the epoch is kept (C-213); higher election epoch wins on merge, rank not compared across epochs, equal-epoch rule from 005-10; no rotation.
- Falsifier of the winner function (005-08, C-221): two modules compute different winners in any scenario.

## Work
1. Build an exhaustive small-scope explorer, as a tool in the `hadronyche-tools` crate (C-133), driven directly by `control.toml` (so it checks the table, not a re-implementation): N modules (at least 2 to 4, all holder-eligible or mixed, each Locked, in holdover with the claim not lapsed, or without usable time), discrete time in steps no larger than 100 ms, nondeterministic message loss, partition and heal events, holder loss, pressure, operator force.
2. Property P1: after convergence (no further partition/loss events for ≥ 1.4 s), exactly one module is in a holder state, and all followers name it.
3. Property P2: from restored connectivity, a single holder is reached within 1.4 s.
4. Property P3: every module computes the same winner from the same candidate set.
5. Property P4: when a window concludes with no candidate with usable time, the winner is the lowest eligible id and the timeline epoch is bumped exactly once (C-113); when at least one candidate has usable time, the timeline epoch is not bumped (C-213), and the winner is the lowest-id Locked candidate if any is Locked, otherwise the lowest-id candidate in holdover (C-221).
6. Record the explored scope (N, time horizon, number of states explored) in `control.md` with a `[measured]` tag; run it in CI unprivileged. The explorer's output format and file names are recorded in its own documentation in `hadronyche-tools`, not in `control.md` (C-206).

## Acceptance criteria
1. The explorer reads the election rows from `control.toml`; changing one row changes its result (shown by a mutation fixture that injects a rotation row and makes P1 or P3 fail).
2. P1, P2, P3, P4 hold for N = 2, 3, 4 with the stated time horizon, including equal-epoch partitions.
3. P2's bound used is 1.4 s exactly, measured from restored connectivity.
4. The run completes in CI within the job's time limit and reports the number of explored states.

## Definition of done
- [ ] A model check or exhaustive small-scope exploration shows no reachable state with two holders after convergence
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Model checking the other three machines.
- Prose scenarios (005-19).
- Real-time behaviour of the implementation (epic 011).
