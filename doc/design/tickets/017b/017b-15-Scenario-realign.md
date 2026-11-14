# 017b-15 — Scenario `realign`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-02, 017b-03, 017a-08, 017b-09, epic 009, epic 010

## Context
- AC-17: descendants switch at the **announced position** (announcement lead 64 slices, C-43; re-issue every 16 slices, C-44); after persistent lateness the **ratchet steps one slice period** and **stops at A**; an input persistently late with its offset at A is **Demoted**.
- Settled rules:
  - **Persistently late (C-101):** at least one late slice in each of 10 consecutive 1 s windows; this is the 10 s dwell (C-54) expressed as a test.
  - **Ratchet (C-53):** one slice period per step; the ceiling is A.
  - **A (C-55, C-142):** the audition offset, one per presentation group, no default. The mixer feeds no presentation group, so the scenario configures an explicit budget that serves as its A (C-142); it appears in the stamp. The alignment ring is sized to A (C-96).
  - **Last ratchet step (C-163):** when the gap between offset and A is less than one slice period, the offset steps to A exactly.
  - **Demotion at the ceiling (C-109):** an input persistently late with the offset already at A enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission. (This is what the epic's earlier wording "a flapping input" meant.) Demoted rounds are a per-connection counter in the stamp (C-159); a connection alarm is raised on demoted (C-173); a Demoted connection is shown by name as unsatisfied (C-154).
- Aligning period 4 s (C-50). Slice = 0.5 ms (C-01, C-02).
- Announcements come from epic 010; ratchet and health states from epic 009.

## Work
1. Configure the mixer's budget A so that the ceiling is reached a small number of ratchet steps above the offset found by aligning, with A − offset not a whole number of slice periods, so the last step exercises C-163.
2. Scenario `realign`, phases:
   - (a) control: delay one input so that late slices occur in only 9 of 10 consecutive 1 s windows; assert no step.
   - (b) delay the input so it is persistently late (C-101); assert one-slice steps, each announced at least 64 slices ahead and re-issued every 16 slices, every descendant switching at exactly the announced position; repeat until the offset equals A, the last step being shorter than one slice period and landing on A exactly (C-163).
   - (c) keep the input persistently late with the offset at A; assert it is Demoted: gapped, counted separately, not cleared when the delay is removed.
   - (d) re-admit the input; assert Demoted is cleared.

## Acceptance criteria
1. `hadronyche-l0 realign --duration 120` exits 0.
2. Phase (a): no ratchet step.
3. Every descendant switch position equals the announced position; first announcement lead ≥ 64 slices; re-issues every 16 slices (± 0).
4. Every ratchet step but the last is exactly 1 slice period; the last brings the offset to A exactly although less than one slice period remained (C-163); each step is preceded by 10 consecutive 1 s windows with a late slice.
5. The persistently late input at A is Demoted and counted in its own counter (demoted rounds, C-159); a demoted alarm is raised (C-173) and the connection is reported unsatisfied (C-154); removing the delay does not clear it; re-admission does.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Epoch changes (017b-16, 017b-20).
