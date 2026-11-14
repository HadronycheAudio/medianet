# 019-21 — Loss alarm and redundancy verdict
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-20

## Context
Epic 019 provides "the go/no-go on v1 redundancy". v1 has no redundancy (no retransmission, no forward error correction); it detects, classifies, conceals and accounts. That stance assumes clean-medianet loss sits near the floor.

- **C-90 loss floor** `[memory]`: at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire), one loss per stream per link about every 13 hours at 2 000 packets/s. Measured by `loss-soak` (019-20).
- **C-118 loss soak**: 24 hours at the two-bundle load (64 000 packets/s, about 5.5e9 packets); the statistic is aggregate loss over all streams, where the floor predicts about 58 losses.
- **C-91 loss alarm threshold**: set from the clean-network loss measured at L1; alarm when exceeded. Its value is 10 × the measured clean-wire loss (C-167).
- Open sub-question: "Is clean-wire loss near the floor?" Falsifier, settled by **C-179**: clean-medianet loss above 10 × the published floor (C-90), about 580 losses in the 24-hour soak (C-118), reopens the redundancy decision. Between 1 × and 10 × it is recorded and investigated as a cabling or host problem and does not.
- Criterion **019-B**: the report states, for the redundancy question, measured clean-wire aggregate loss against C-90 and a recommendation.

Evidence tags: C-90's bit-error-rate figure is `[memory]` until its citation is checked; then it becomes `[standard]` with the named standard and clause (C-132). The redundancy section uses whichever tag the constants table carries at the time.

The decision rule is settled (C-179) and the C-91 margin is settled (C-167); this ticket proposes no rule of its own.

## Work
- In the `loss-soak` report and in `doc/design/reports/l1-results.md`, add a "Redundancy recommendation" section with: measured aggregate clean-wire loss count and rate (from 019-20) with Poisson 95 % interval, C-118's about 58 expected losses and C-90's 1.1e-8, their ratio, and a recommendation: keep "detect and conceal, no redundancy" for v1, or return redundancy to v1.
- Apply the C-179 rule to the measured aggregate clean-wire loss: above 10 × C-90 (about 580 losses over a full soak, scaled to the packets actually sent for a partial one) → recommend reopening the redundancy decision; between 1 × and 10 × → recommend keeping v1's stance, and record the excess and open an investigation of cabling and hosts (owner, falsifier); at or below 1 × → keep v1's stance. Cite C-179 in the section.
- Compute C-91 as 10 × the measured clean-wire loss (C-167), stated per packet and as losses per stream per hour, with the measured basis tagged `[measured]` and the rule cited as C-167.
- If the recommendation is to reopen the redundancy decision, open an item with owner and falsifier.

## Acceptance criteria
1. The `loss-soak` report and `l1-results.md` both state measured clean-wire aggregate loss against C-90 and C-118's prediction, and one explicit recommendation (criterion 019-B).
2. C-91 has a numeric value with unit, equal to 10 × the measured clean-wire loss (C-167), with evidence tags; or a stated reason why not.
3. The recommendation follows C-179: it reopens the redundancy decision if and only if measured aggregate loss exceeds 10 × C-90 (about 580 losses in the 24-hour soak); a result between 1 × and 10 × is recorded with an investigation item (cabling or host problem) and does not reopen it.

## Definition of done
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- Running the soak (019-20).
- Implementing the C-91 alarm in the runtime, or implementing redundancy.
- Writing C-91 to the constants table (019-22).
