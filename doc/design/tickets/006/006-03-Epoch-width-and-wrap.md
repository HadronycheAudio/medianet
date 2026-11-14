# 006-03 — Timeline epoch width and wrap analysis
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-01, epic 004
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Epoch width and wrap". The width is settled: the timeline epoch is 16 bits (C-10, decided). C-10 carries a falsifier, "spec drafting shows a 16-bit epoch wraps within a plausible uptime"; this ticket is the spec drafting that checks it, and specifies the comparison rule that makes a wrap harmless.

Facts (`[decision]`):
- A **timeline epoch** is a 16-bit counter (C-10: stream number / session / timeline epoch widths are 16 / 32 / 16 bits). It is distinct from the *election* epoch: a continuous clock handover does not change it; a discontinuity does.
- The epoch is carried in **every sync reply** and in **every slice header** (both formats owned by epic 004).
- Bump causes: partition merge (losing side); new holder that was not locked (C-113); holder that detects its own stall; step larger than 2 ms (C-67).

Arithmetic to put in the document: 2^16 = 65 536 values. A wrap within one year (8 766 h) requires more than 65 536 / 8 766 ≈ 7.5 bumps per hour, that is one bump every ≈ 8 minutes sustained for a year. At one bump per minute the counter wraps in ≈ 45.5 days; at one per hour, in ≈ 7.5 years.

## Work
1. Write the wrap computation above, with the bump-rate assumption made explicit and tagged (`[decision]` for an assumed rate; `[measured]` only if a measurement exists).
2. Specify how two epochs are compared so that a wrap is harmless: modular (serial-number) comparison over 16 bits, where `a` is newer than `b` iff `(a − b) mod 2^16` is in `1..=32 767`; state the window within which the comparison is valid (fewer than 32 768 bumps between two observations).
3. Specify that equality, not ordering, is what fan-ins and sinks need (they compare an input's epoch to the module's current epoch), and where ordering is needed (rejecting a stale sync reply).
4. Record the check of C-10's falsifier: "falsifier not met at assumed rate R". If the analysis shows the falsifier is met, do not change C-10 here: record it as an open item with owner the design owner (constants table) and epic 004 (field layout), and falsifier as in C-10.

## Acceptance criteria
1. The section states the number of bumps per hour that would wrap the counter in under a year (≈ 7.5/h) and the assumed realistic bump rate with its tag.
2. The comparison rule is given as a formula and tested by three examples, including one across the wrap (e.g. 65 535 → 0 is "newer").
3. The section cites C-10 as decided and states whether its falsifier is met.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Changing C-10.
- The slice-header and sync-reply field layout (epic 004).
- The `TimelineEpoch` Rust type (006-08), which implements the comparison rule written here.
