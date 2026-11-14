# 007b-03 — Holder follower cap and sync request rate vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007b-02

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- Settled rules `[decision]`:
  - a clock holder serves at most 64 followers; the excess is refused with a named event (C-35). The event name comes from the tables of epic 005;
  - with at most 64 modules per medianet (C-34) a holder has at most 63 real followers, so the 64-follower load and the refusal of a 65th (system criterion AC-25) are tested with synthetic followers (C-114);
  - the accepted sync request rate is 10 to 200 per second per follower; system criterion AC-7 is checked over this range (C-112).
  - requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's stamp (C-204).

## Work
1. Write scenarios in `vectors/control/holder-*.json`:
   - 64 synthetic followers each send sync requests at 50 per second (C-36) for at least 1 s: every request is answered;
   - a 65th synthetic follower: refused with the named event; the 64 others keep being answered;
   - one follower at 10 requests per second and one at 200 per second, each over at least 1 s: every request is answered;
   - one follower sending above 200 per second over at least 1 s: the requests beyond 200 per second get no reply and no other output, and the holder's next stamp carries that follower's dropped count equal to the number of requests beyond 200 per second; the other followers keep being answered (C-204).
2. Add one scenario per remaining follower-admission row in the tables.

## Acceptance criteria
1. The 64-follower scenario expects an answer to every request.
2. The 65th follower is refused with exactly one named event and no change to the other 64.
3. The 10 and 200 per second scenarios expect an answer to every request.
4. In the over-rate scenario, every request beyond 200 per second is dropped with no reply or other output, the holder's stamp counts them for that follower only, and the other followers are unaffected (C-204).
5. All files carry `spec_version`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Election (007b-02). Holder serving lateness under load: the C-35 falsifier is owned by L1, task 019 (C-35), with serving lateness defined as reply send time minus request receive time at the 99.9th percentile (C-167).
