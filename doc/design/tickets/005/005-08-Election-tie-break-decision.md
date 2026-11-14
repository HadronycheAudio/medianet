# 005-08 — Election tie-break decision
**Epic:** 005-Control-state-machines-specification · **Effort:** XS · **Depends on:** 005-01

## Context
- Machine 2 (election): the holder's monotonic clock defines network time; one holder per medianet. Within the election window (C-33 = 400 ms) every module computes the same winner from the same candidate set, deterministically.
- Candidates: holder-eligible capability bit set, usable time, and on a host not under I/O pressure. Usable time means Locked, or in holdover with the alignment claim not lapsed (C-213, C-65, C-169). Embedded, macOS and Windows nodes do not set the bit in v1. Linux hosts are the only clock holders in v1. A module whose holdover claim has lapsed is Unlocked with time unusable (C-162) and has no usable time.
- Decided: when no candidate has usable time, the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch (C-113); otherwise a candidate with usable time wins and the timeline epoch is kept (C-213).
- **Decided (C-221):** among several candidates with usable time, a Locked candidate is preferred over one in holdover; among equals the lowest module id wins. This ticket no longer decides the order; it writes the C-221 order as the winner function and its test table. The tie-break sub-question is closed by C-221.
- Merge: the higher election epoch wins; rank is not compared across epochs.
- `[code]` POC behaviour to keep: epoch comparison on hello (`module.rs:547-557`), listen-then-claim, deterministic draw.
- The lock state and bound come from the servo core (epic 008a) through the published time view (C-187); that epic defines the Locked state.
- Falsifier kept for the winner function: two modules compute different winners in any scenario vector.
- Module id is 64 bits and unique after collision handling (C-11, C-46).
- The I/O-pressure eligibility rule's inheritance is owned by task 002; it enters as a candidacy flag either way.

## Work
1. Define the winner function `winner(candidate_set) -> module id` as a total order: if no candidate has usable time, the lowest module id among holder-eligible candidates (C-113); otherwise, among the candidates with usable time (Locked, or in holdover with the claim not lapsed, C-213), the key (clock class, module id) with Locked before holdover and the lowest module id first (C-221). No other field (rank, load) enters the key.
2. State that the clock class enters the key as carried in the hello candidacy fields of the current election epoch (field from epic 004), so that every module computes the same result from the same set.
3. Write the rule in `control.md` Machine 2 citing C-113, C-213 and C-221; restate the falsifier.
4. Add a property test or table of at least 10 hand-made candidate sets with expected winners, at least 2 of them with no candidate with usable time, at least 2 in which the only candidates with usable time are in holdover with the claim not lapsed, and at least 2 mixing a Locked candidate with a lower-id candidate in holdover, reused by 005-11.

## Acceptance criteria
1. The order is total: two distinct candidates never compare equal (the last key is the unique module id).
2. The function is a pure function of the candidate set, independent of arrival order (test permutes each of the 10 sets and gets the same winner).
3. With no candidate with usable time, the winner is the lowest eligible module id (C-113); a candidate in holdover with its claim not lapsed is a candidate with usable time and enters the ordered key (C-213).
4. A Locked candidate wins over every candidate in holdover whatever their ids; among Locked candidates, or among candidates in holdover, the lowest module id wins (C-221).
5. The falsifier is restated in `control.md`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The election state list and table (005-09, 005-10).
- The I/O-pressure eligibility rule's provenance (owned by task 002; only noted here).
- The definition of the Locked state (servo core, epic 008a, C-187; C-62 is pending simulation).
