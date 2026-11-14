# 011-11 — Name taken: listen before claim
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-07

## Context
Rule `[decision]`: a newcomer whose **name** is already present refuses to join, using the listen-before-claim period.
- C-32 (inherited from the POC): listen-before-claim 1.5 s.
- C-30: hello period 1 s, so every present module is heard at least once during 1.5 s.
- The epic fixes the placement: from start, the module listens for 1.5 s and sends no hello; it then either raises `NameTaken` and stays silent, or joins by sending its first hello at t = 1.5 s (epic AC 15).
- Event: `MapEvent::NameTaken`. NameTaken is raised at discovery, where it stops the newcomer joining, not at admission (C-185).

`[code]` Retired POC behaviour: duplicate names were only warned (`module.rs:537-539`).

## Work
1. Add a listening phase to `Discovery`: from the first `on_tick`, for 1.5 s the module receives hellos and fills its map but sends no hello.
2. If during that phase a hello arrives whose name equals the own name (from a different id), raise `MapEvent::NameTaken` and stay in a refused state: no hello is ever sent.
3. At the end of the phase with no conflict, the module joins: it sends the first hello and continues with the 1 s cadence of 011-07 (whose tests are written relative to the join instant and need no change).

## Acceptance criteria
1. With an existing module named "X" sending hellos every 1 s, a newcomer named "X" raises `NameTaken` within 1.5 s and sends zero hellos over the following 10 s.
2. A newcomer with a unique name sends its first hello at t = 1.5 s (± one tick) and then every 1 s.
3. A hello with the same name arriving *after* the listening phase does not make the established module refuse (it is the newcomer's job to refuse).
4. The name comparison is exact byte equality (documented).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (behaviour follows the epic); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- How the platform reports `NameTaken` to the operator: epics 015, 026.
- Id collisions: 011-10.
- The ModuleCap refusal during the same listening phase (C-185): 011-26.
