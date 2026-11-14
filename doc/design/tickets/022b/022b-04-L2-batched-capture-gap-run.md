# 022b-04 — L2 Windows: batched capture emission vs Linux consumer margins
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** M · **Depends on:** 022a-14, 022a-17, 022b-01

## Context
- Part of epic 022b (Windows edge L2 certification), L2 certification of the capture edge.
- Capture edges emit a batch of 6 to 20 slices (0.5 ms each, C-02) per device period (about 3 to 10 ms `[memory]`). Positions are exact, but lateness at consumers includes the period, so the group's A must cover it. A has no default; commissioning proposes it per presentation group (C-55).
- A consumer measures arrival lateness for 4 s (C-50) and sets its offset to the observed maximum plus headroom (C-51, C-52); the batch pattern is absorbed if stable.
- "No gaps" is settled by the counting rules: a slice absent at the deadline is counted missing, and reclassified late if it arrives within the ring span (C-108); an input with at least one late slice in each of 10 consecutive 1 s windows is persistently late and ratchets (C-101); persistently late at the ceiling A, it is Demoted (C-109).
- The consumer publishes these counters per connection in its regime stamp (C-47), so the result can be read from the Linux consumer's stamp as well as from its logs. These are ordinary production measurements, unqualified while a commissioning window is open (C-178), so the run is made with no window open.
- Interrupted after 8 consecutive missed slices (C-38, C-102).
- 022-AC-1 / AC-28 (part): **batched capture emission causes no gaps at the commissioned A** on a Linux consumer over one hour.
- Open sub-question owned here: does batched capture emission break consumer margins? Falsifier: the consumer sees late or missing slices, a ratchet step or a Demoted input at the commissioned A.

## Work
1. For a group containing the Windows capture edge and a Linux consumer, let the harness compute A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and record it with the results (C-148); this is the commissioned A of the criterion. The run is in strict mode (C-147).
2. With no commissioning window open (C-178), run one hour at the smallest and largest available device period; record the consumer's per-connection counters (C-159, including missing, late, marked gap, other-epoch, too-early and duplicates), ratchet steps, health state, lateness distribution and batch sizes. The stamp's lateness percentiles cover a rolling 60 s window, with the maximum since the connection went live beside them (C-182), so the one-hour distribution is collected by the harness over the whole run and the stamp's since-live maximum is recorded beside it.
3. Answer the sub-question; if falsified, record the consequence and an owner.

## Acceptance criteria
1. **(022-AC-1 / AC-28, part.)** Over one hour at the commissioned A, at each tested period, the Linux consumer counts zero missing and zero late slices (C-108), takes no ratchet step (C-101) and never enters Demoted (C-109).
2. Batch-size statistics match what the stamp reports (022a-17).
3. Sub-question answered `[measured]`; results in `doc/design/reports/` (C-130).
4. No commissioning window was open during either one-hour run (C-178).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Capture implementation (022a-14).
