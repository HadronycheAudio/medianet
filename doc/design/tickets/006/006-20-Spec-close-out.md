# 006-20 — Timeline spec close-out: open items, tags, rendering
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-01, 006-02, 006-03, 006-04, 006-05, 006-06, 006-07, 006-08, 006-09, 006-10, 006-11, 006-12, 006-13, 006-14, 006-15, 006-16, 006-17, 006-18, 006-19, 006-21
## Context
Final ticket of epic 006. It checks that `doc/design/spec/timeline.md` and the Rust types meet the epic's definition of done as a whole, and closes the "Open items" section.

Settled before the tickets ran (list them as closed, with the constant that closes them):
- 16-bit epoch wrap: C-10 decided; 006-03 checks its falsifier.
- 64-slice lead in a deep graph: C-43 decided and flat; L0 task 017 confirms it.
- Rates other than 96 kHz: none on the wire (C-01, C-80); a resampler node only inside an edge module (C-134); L is a whole number of samples (C-105).
- Holder stall threshold (former Q-7): C-136.
- Counter class and dead-threshold effect of an other-epoch slice (former Q-8): C-135, C-159.
- Crate of the timeline types (former Q-9): `hadronyche-core`, C-133.
- Announcements on a fallback path: omitted if they do not fit the header cap (C-137); outcome stated in 006-12.
- Session numbering across a switch: C-19.
- Unlocked takeover winner: C-113.

The epic's open sub-questions and where they are handled:

| Id | Question | Owner | Falsifier | Handled in |
|---|---|---|---|---|
| Q-1 | Rounding rule off a slice boundary | This epic | `position_at(time_of(p)) ≠ p` for some `p` | 006-02 |
| Q-2 | `position_at` before the origin | This epic | Two implementations differ for the same time before the origin | 006-02 |
| Q-3 | Choice of `epoch_origin` at cold start and after a bump | This epic | Two modules reconstruct different lines from the same sync replies | 006-04 |
| Q-4 | `step_ns` sign and unknown-step value | This epic | Two modules report different `step_ns` for the same bump | 006-06 |
| Q-5 | Source in Locking or holdover at an epoch change | This epic, with epic 005 | It announces a switch computed from a stale line | 006-10 |
| Q-6 | Losing side of a partition merge | Epic 005 | Both sides keep their epoch, or both bump | 006-05, 006-13 |
| Q-7 | Holder stall threshold | Closed by C-136 | — | 006-05, 006-15 |
| Q-8 | Counter class and dead-threshold effect of an other-epoch slice | Closed by C-135, C-159 | — | 006-21, 006-18 |
| Q-9 | Crate of the timeline types | Closed by C-133 | — | 006-08 |

Also pending: C-61 (handover relaxation ≤ 1 ppm/s), owner epic 008 (006-07); C-195 (holdover entry delay, at or below C-37), owner epic 008b (006-07, 006-10, 006-14, 006-15).

## Work
1. Fill the "Open items" section: one row per item above that is still open, with owner and falsifier; closed items listed with the section or constant that closes them.
2. Remove every placeholder line left by 006-01.
3. Check every factual claim in the document carries an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, `[standard]` per C-132; a `[memory]` whose standard citation has been checked becomes `[standard]`).
4. Render the three mermaid diagrams (partition merge, unlocked holder takeover, holder stall) with a mermaid renderer and fix any error.
5. Check the constants table is still the first section, contains every C-nn used in the text, and matches `doc/design/02-constants.md` value for value.
6. Check no text mentions a wire rate other than 96 kHz, a per-channel id byte, IPv6, or a structural latency L that is not a whole number of samples (C-01, C-04, C-100, C-105).
7. Confirm the frozen POC is untouched in the epic's changes.
8. Mark not-applicable checklist lines with their reason (C-125): "Every parser touched is fuzzed" is not applicable, because the epic adds no parser (wire formats belong to epic 004, the regime stamp message to epic 013, C-144).

## Acceptance criteria
1. 0 placeholder lines remain; every section heading of 006-01 has content.
2. Every open item has an owner and a falsifier; Q-1 to Q-9 each appear as closed or open, and the nine settled items appear as closed with their C-id.
3. The 3 diagrams render without error.
4. Every C-nn id cited in the text appears in the constants table with the value of `doc/design/02-constants.md`.
5. The not-applicable line carries its reason.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable (C-125): epic 006 adds no parser; wire formats belong to epic 004, the regime stamp message to epic 013 (C-144)
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The three sequence diagrams render
- [ ] The epoch field of the regime stamp is specified (which epoch a record set belongs to; an epoch change starts a new record set)

## Out of scope
- New normative content (belongs to the ticket owning the section).
- Answering Q-6 (epic 005).
- Conformance vectors (epic 007), simulation (epic 008), L0 confirmation (task 017).
