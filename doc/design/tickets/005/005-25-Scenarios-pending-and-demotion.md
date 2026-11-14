# 005-25 — Scenarios: Pending and demotion
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-14, 005-16

## Context
- Deliverable 4 of epic 005 (scenario scripts in prose, to become vectors in task 007) now includes two scenarios created by rulings:
  - **Pending (C-45):** "target not present" is not a refusal; Pending has no timeout, persists until the target appears or the operator cancels, and is shown as unsatisfied (only Live is satisfied, C-154). Pending → Requested when the target appears; Pending → Ended on operator cancel.
  - **Demotion (C-109):** an input persistently late (at least one late slice in each of 10 consecutive 1 s windows, C-101) steps its offset by one slice period (C-53) until the ceiling A (C-55), the last step landing on A exactly when less than one slice period remains (C-163); persistently late at A, it enters Demoted, is treated as gapped, counted separately, and only re-admission clears it.
- A laptop edge that sleeps has departed and returns through normal admission (C-124); this is scripted as a third case.
- Guards come from 005-24; tables from `doc/design/spec/control.toml`. Lateness can be produced in a vector with the raw-packet test hook (C-120).

## Work
1. In the "Scenarios" section of `control.md`, write as timed scripts (time ms, module, input, expected state, events):
   - S8 Pending persists: request to an absent target, Pending for 60 000 ms with no timer row firing and the request shown unsatisfied; then the target appears and the request proceeds to Requested;
   - S9 Pending cancelled: request Pending, operator cancel, Ended, no re-send;
   - S10 demotion: a Live input persistently late; ratchet steps of one slice period every 10 s, each taking effect at a position announced one lead (64 slices, 32 ms) after the C-101 test holds (C-215), and a variant in which the consumer is a sink, which takes each step at its own position 64 slices after the test holds with no announcement (C-225), until the offset reaches A, with A chosen so that the last step is less than one slice period and lands on A exactly (C-163); at A, persistently late again enters Demoted; re-admission returns it to Requested;
   - S11 laptop sleep: a Live edge connection stops sending; Interrupted after 8 absent slices, Dead 2 s after the last content; the woken edge returns through Requested, Locking, Aligning.
2. Walk each through the tables and record that results match.

## Acceptance criteria
1. Four scripts S8–S11 exist with timestamps in ms.
2. S8 shows 0 transitions out of Pending for 60 000 ms and the request shown unsatisfied throughout.
3. S10 shows each ratchet step no earlier than 10 000 ms after the previous one, each applied at its announced position 32 ms (64 slices, C-43) after the C-101 test holds and announced in-band (C-215), or in the sink variant at the same position with no announcement output (C-225), a final partial step to A exactly (C-163), Demoted entered only with the offset at A, and the demoted-rounds counter incremented once (C-159).
4. S11 enters Dead at 2 000 ms after the last slice with content (C-111) and resumes only through admission (C-124).
5. Every input used exists in the machine-readable tables.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- Refusal scenarios (005-21); election and gossip scenarios (005-19, 005-20).
- Vector generation (epic 007).
