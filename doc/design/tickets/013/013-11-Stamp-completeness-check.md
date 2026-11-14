# 013-11 — Stamp completeness check
**Epic:** 013-Regime-stamp-wire-object · **Effort:** XS · **Depends on:** 013-06, 013-07, 013-08, 013-09

## Context
- AC-31 (013-AC-2): **every record set carries a complete stamp (all eight groups present)**. `[decision]`
- The eight groups: Identity, Capabilities, Network, Clock, Alignment, Platform, Drops, Injection.
- The epic settles that completeness counts the eight groups only. The self-test result, per-connection entries and per-connection summaries (C-47) are not groups: a stamp without a self-test is complete but unqualified (013-10); a node with no connections has no entries or summaries.
- A group may be present while individual fields inside it are "absent" or "unverified" on a profile; completeness is about groups, not values.
- Because the stamp travels as sections (C-18), a receiver may hold a partial stamp; completeness is checked on the reassembled stamp (013-13).

## Work
1. Add `Stamp::is_complete(&self) -> bool` and `missing_groups()` returning a fixed-size bitset, in `hadronyche-core`.
2. Make `StampBuilder::finish` report which groups are missing (choose error or flag and document it).

## Acceptance criteria
1. A stamp with all 8 groups: `is_complete()` true.
2. For each of the 8 groups, a stamp missing only that group: `is_complete()` false and the missing group is named (8 test cases).
3. A complete stamp with extra unknown sections is still complete.
4. A stamp with all 8 groups and no self-test section is complete and `qualified()` is false.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" (no new parser; `is_complete()` is called by the 013-03 target); "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added); "Design documents updated where behaviour differs, constants table first" (no behaviour differs); "No new open item without an owner and a falsifier" (none opened); "Every new factual claim carries an evidence tag" (no factual claim); "The stamp format is versioned independently of the media format" (no framing change).

## Out of scope
- Enforcing completeness when writing record sets (013-15).
