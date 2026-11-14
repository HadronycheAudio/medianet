# 005-18 — Worst-case memory and work summary
**Epic:** 005-Control-state-machines-specification · **Effort:** XS · **Depends on:** 005-07, 005-10, 005-14, 005-16

## Context
- 005-AC-3 of epic 005: each machine states its worst-case memory with 63 peers (C-217) and 64 followers and its worst-case work per input.
- Bounds: active modules per medianet at most 64 (C-34); followers per holder at most 64 (C-35, pending measurement at L1, task 019; with 64 modules at most 63 are real, the 64th is synthetic in tests, C-114); the map has a fixed capacity of 63 peer entries, since a module holds no entry for itself (C-217). Addresses are IPv4 (C-100). Machines run in the `no_std` core `hadronyche-core` and must have a fixed memory bound (no heap).
- Per-machine numbers are written in each machine section of `doc/design/spec/control.md` (by 005-06/07, 005-09/10, 005-13/14, 005-15/16).

## Work
1. Write the "Worst-case memory and work" section of `control.md`: one table, one row per machine (discovery, election, admission, connection consumer, connection producer), columns: worst-case bytes with 63 peers (64 followers for election), worst-case operations per input, the input that reaches it.
2. Add `memory_bound_bytes` totals to `control.toml` and a check that the per-state bounds sum consistently with the summary.
3. Give the total control-plane memory for one module.

## Acceptance criteria
1. The table has 5 rows, each with a number of bytes and a number of operations per input.
2. Every number matches the one in its machine section (checked by the CI test).
3. The total for one module is stated in bytes.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- Measuring the implementation (epics 011, 012).
