# 007b-01 — Discovery scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007a-04, epic 004, epic 005, epic 006

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)` for one control state machine, stored in `vectors/control/*.json`, driven through `UnderTest::control_step(machine, now_ns, input)` (007a-04).
- Epic criterion 007-AC3: every transition row of every control state machine is exercised by at least one scenario vector. Epic 005 supplies the discovery state table and scenario scripts, and epic 004 the wire layouts and reject reasons (C-193).
- Discovery rules `[decision]`:
  - at most 64 active modules per medianet; commissioning refuses more (C-34). A 65th module arriving anyway is stopped from joining with ModuleCap, raised at discovery, not at admission (C-185);
  - hellos are sent every 1 s (C-30); a peer expires after 3 s without hello **only if it is not live** (C-31);
  - id collision is detected within one hello period (1 s); the later joiner redraws its id; third parties quarantine the colliding id until only one instance nonce has been seen for one hello-expiry period, 3 s (C-46). Module id and instance nonce are 64 bits each (C-11).
- VersionMismatch and NameTaken are likewise raised at discovery and stop a module joining (C-185).
- A hello with domain 0 (unconfigured, C-11) is ignored and counted, not refused; a hello from a different non-zero domain is refused with ForeignDomain (C-173).
- On a real implementation the collision is produced with the test-only hook that forces a module id and nonce (C-120).

## Work
1. Write scenarios in `vectors/control/discovery-*.json`:
   - fill the map to 64 entries, then a 65th arrival: ModuleCap, the 65th does not join, the 64 entries are unchanged (C-185);
   - a hello with a mismatched version: VersionMismatch, the module does not join (C-185);
   - a module whose name is already taken: NameTaken, the module does not join (C-185);
   - hello expiry at 3 s for a non-live peer (removed at 3 s, still present at 3 s − 1 ns);
   - no expiry at 3 s for a live peer;
   - id collision: detected within 1 s, later joiner redraws, third party quarantines;
   - quarantine lift: after the second nonce is last seen, the third party keeps the id quarantined at 3 s − 1 ns of single-nonce observation and lifts it at 3 s;
   - quarantine kept: a second nonce reappears before 3 s and the 3 s count restarts;
   - a hello with domain 0: no map entry, no refusal, its drop counter +1 (C-173).
2. Add one scenario per remaining discovery transition row not covered above.

## Acceptance criteria
1. A 64-entry map plus 65th arrival scenario exists and expects ModuleCap with the map unchanged.
2. Expiry boundary is tested at 3 s and just before 3 s, for live and non-live peers (at least 3 scenarios).
3. The collision scenario checks detection time ≤ 1 s, redraw by the later joiner, and quarantine at a third party.
4. The quarantine lift is tested at 3 s and at 3 s − 1 ns of single-nonce observation, and a reappearing nonce restarts the count.
5. The domain-0 hello leaves the map unchanged, emits no refusal and is counted once.
6. Every discovery transition row is listed as covered by the coverage report of 007b-13 (or, before 007b-13 lands, by a list in the vector file).
7. VersionMismatch and NameTaken each have a scenario expecting that reason and no new map entry.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Election (007b-02), holder follower cap (007b-03), admission (007b-05), connection lifecycle (007b-07).
- The mechanical coverage check (007b-13).
