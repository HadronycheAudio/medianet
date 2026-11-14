# 014-21 — Design documents and constants update
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-01, 014-02, 014-03, 014-04, 014-05, 014-06, 014-07, 014-08, 014-09, 014-10, 014-11, 014-12, 014-13, 014-14, 014-15, 014-16, 014-17, 014-18, 014-19, 014-20, 014-22, 014-23, 014-25, 014-26
## Context
Closing ticket for epic 014. Behaviour introduced by the earlier tickets that differs from or adds to the design documents must be written back, constants table (`doc/design/02-constants.md`) first. Specifications go to `doc/design/spec/`, reports and measured results to `doc/design/reports/`, evidence to `doc/design/06-evidence-appendix.md` (C-130); the architecture specification is `doc/design/01-architecture.md` (C-171). Constants involved: C-01, C-02, C-05, C-06, C-08, C-10, C-13, C-17, C-19, C-22, C-23, C-38, C-39, C-40, C-41, C-47, C-90, C-91, C-95, C-96, C-100, C-102, C-104, C-105, C-107, C-108, C-109, C-111, C-118, C-129, C-131, C-132, C-134, C-135, C-137, C-142, C-144, C-147, C-153, C-159, C-160, C-162, C-164, C-167, C-168, C-169, C-178, C-179, C-186, C-187, C-192, C-193, C-198, C-205, C-209, plus the new probe default constant from 014-11.

Open sub-questions of the epic and their status:
| # | Question | Owner | Falsifier | Ticket |
|---|---|---|---|---|
| 1 | Counter semantics, widths and wrap handling | Task 004 with this task | A counter wraps within one report interval at 2 000/s per member or 64 000/s per socket, or a reclassification cannot be expressed | 014-02 |
| 2 | Is 100 ms the right report interval for the commissioning tool's A proposal? | — | — | settled: the tool uses the stamp's lateness maximum and percentiles (C-160) with the commissioning rule (C-148) |
| 3 | Can interface drop counters be read without privilege on each platform? | Tasks 015, 021, 022 | Not readable; the field is absent there and stamped absent (C-129) | stays open |
| 4 | Who initiates one-way probes, and how often by default? | This task, with task 012 | Probes at the default rate change an admission decision against the 70 % ceiling (C-23) | 014-11 |
| 5 | Start of the producer's 2 s window before the first report | Task 005 | A producer stops a live but not-yet-reporting consumer, or keeps sending to one that never reports | stays open |
| 6 | Is the loss floor figure right? | Task 001, then L1 (task 019) | Published figure differs; clean-network loss well above the floor | stays open |
| 7 | Minimum sample size for the loss-floor comparator | L1 (task 019) | Flag fires on a clean network, or stays silent over a 24 h soak with loss well above C-91 | stays open |
| 8 | Representation of the accumulated declared delay | — | — | settled: L is a whole number of samples (C-105 as revised), so the sum is an integer; width joins question 10 |
| 9 | Which epic publishes clock lock state and clock bound | — | — | settled: the servo core, epic 008a, through the published time view (C-187) |
| 10 | Range and saturation of hop count, residence and declared delay within the C-137 header cap; negative residence | Task 004 with this task | A real path exceeds the range, the extension does not fit the cap, or negative residence is seen on a locked module | 014-13 implements the provisional rule; stays open |

## Work
1. Update the constants table: new probe default constant (014-11), `MAX_REPORT_LEN` (014-03), the counter widths including every C-159 count (014-02). A measured figure entering the table comes from a strict-mode run (C-147); the C-164 parser figure (014-25) goes to `doc/design/reports/` and stays a proposal pending L1.
2. Document the provisional rules still waiting on an owner (lineage saturation and negative residence, omission of the extension under the C-137 cap, `ProducerWatch` start at admission) with `[decision]`, each linked to its open sub-question.
3. Document the settled behaviour introduced here with `[decision]`: per-member session in the report (C-19), encoder error path, `Lineage::critical` returning no value for a bad index, asymmetry bound of half the round trip, loss statistic aggregated over all streams (C-118) using missing counts after reclassification (C-108), declared delay as an integer sum of whole samples (C-105), one-way figures withheld once the holdover claim lapses (C-162, with the unknown-E lapse of C-169), tool-driven probe figures valid as commissioning results recorded with their window and production loss figures unqualified while a window is open (C-178), the consumer's current A carried in every report (C-198, 014-26).
4. Document, with epic 013, the regime stamp fields for drop counters, observed DSCP, their capability states (C-129) and the per-connection summaries with every C-159 counter (C-47, 014-22).
5. Update the open sub-question table: close numbers 1 and 4 (2, 8 and 9 are already settled by C-160, C-105 and C-187); keep the others with owner and falsifier.
6. Check every new factual claim has an evidence tag; a figure checked against a published standard uses `[standard]` (C-132).

## Acceptance criteria
1. The constants table lists every constant used by the module `measure`, each with an id and value.
2. The open-items table has no item without owner and falsifier.
3. A reviewer finds no untagged factual claim in the changed text.
4. Specifications are under `doc/design/spec/` and measured results under `doc/design/reports/` (C-130).

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Code changes (014-01 to 014-20, 014-22, 014-23, 014-25, 014-26).
- Answering the sub-questions owned by other tasks or the design owner.
