# 005-07 — Discovery map transition table
**Epic:** 005-Control-state-machines-specification · **Effort:** M · **Depends on:** 005-03, 005-04, 005-06, 005-23

## Context
- Machine 1 (discovery map) of `doc/design/spec/control.md`; states, data and inputs were defined in 005-06 and mirrored in `doc/design/spec/control.toml` (machine `discovery`); stamp-version inputs came from 005-23. The checker from 005-03 verifies totality.
- Rules to encode:
  - capacity 63 peer entries (C-217: at most 64 modules, C-34, and no entry for itself); a 65th module (a 64th peer) is not admitted and raises a named event (`ModuleCap`, with the event decided in 005-04);
  - a hello inserts or refreshes an entry (`PeerJoined` on insert); a non-live peer expires 3 s after its last hello (C-31, `PeerLeft`); a peer with a live connection is **not** removed by expiry (L-1);
  - foreign-domain hellos (a different non-zero domain) are dropped and counted (`ForeignDomainSeen`); a hello with domain 0 is ignored and counted, not refused, and raises no refusal (C-173); a version mismatch raises `VersionMismatchSeen` and stops the module joining, with no map entry (C-185); `VersionMismatch`, `NameTaken` and `ModuleCap` are discovery refusals, never admission ones (C-185);
  - each hello updates the entry's observed DSCP, reported in the stamp (C-152);
  - id collision (same 64-bit id, different 64-bit nonce, C-11): detected by the pair within one hello period, 1 s (005-AC-5, system AC-22); the later joiner redraws its id and rejoins; third parties quarantine the id (ignore both) until only one nonce has been seen for 3 s (C-46); the redrawn module is a new entry; both log it (`IdCollision`);
  - a newcomer whose name is already present refuses to join after listen-before-claim 1.5 s (C-32, `NameTaken`);
  - liveness counter (the C-115 authentication counter) in the hello; stall visible within 2 hello periods (C-42);
  - a hello with a newer stamp version leads to a unicast stamp request (C-18, rows from 005-23);
  - a hello with a new address updates the map entry but never redirects a live stream (L-5); destinations are pinned at admission.
- Invariants to annotate: L-1 to L-8 (see 005-01). Relevant here: L-1, L-3, L-4, L-5, L-6, L-8.

## Work
1. Write the complete transition table of machine 1 in `control.md`: every (state, input) pair has a row, including `ignore`, with guard, new state, output messages, events, invariants preserved.
2. Encode the same rows in `control.toml`; the checker of 005-03 must pass.
3. Write "Worst-case work per input": bound the operations per input (e.g. one lookup over at most 63 entries), stating the number.
4. Show in a short paragraph that collision detection happens at the first hello of the second module seen by its peer, i.e. within one hello period (1 s).
5. Show by a timeline that the C-46 quarantine never blocks the rejoin: redraw, then listen-before-claim (1.5 s), then first hello (≤ 1 s) puts the new id in every third party's map within 2.5 s, independent of the old id's quarantine; and state when the old id is usable again (3 s after only one nonce is seen).
6. Emitted event names must be `ControlEvent` / `RefuseReason` variants from 005-04.

## Acceptance criteria
1. The checker reports 0 missing (state, input) pairs for machine `discovery`.
2. No row whose input is the hello-expiry timer removes an entry in the Live-pinned state (L-1).
3. No row changes a pinned destination (L-5); a hello with a new address on a live peer is a row whose outputs contain no connection change.
4. Collision is detected within 1 s (one hello period, C-30) and the later joiner is the one that redraws.
5. The quarantine ends exactly when one nonce has been seen for 3 s (C-46), and the rejoin timeline shows the new id admitted within 2.5 s of the redraw.
6. A 64th peer insert (the 65th module) yields `ModuleCap` and the map size stays 63 (C-217).
7. Worst-case work per input is stated as a number with 63 peers (C-217).
8. A domain 0 hello increments the domain-0 counter and changes no entry; a hello from a different non-zero domain yields `ForeignDomainSeen` (C-173).

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- The full cross-machine invariant analysis (005-17).
- Discovery scenarios (005-19, 005-20, 005-21).
- Implementation (epic 011).
