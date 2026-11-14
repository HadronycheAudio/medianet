# 011-26 — Module cap at discovery (new, C-185)
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-06, 011-11

## Context
- C-34 (decided): at most 64 active modules per medianet; commissioning refuses more.
- C-217 (decided): the peer map holds 63 peers, never an entry for the module itself (011-06). A newcomer therefore cannot store 64 peers: it learns of the 64th as a further distinct id refused with `MapFull` while its map is full.
- C-185 (decided): VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not at admission. VersionMismatch is 011-08 and NameTaken is 011-11; no ticket raised ModuleCap.
- The listening phase of 011-11 (C-32, 1.5 s) hears every present module at least once (hello period 1 s, C-30), so a newcomer knows at its end how many active modules are present.
- Rule `[decision]` (epic): a newcomer that has heard 64 distinct active module ids by the end of listen-before-claim would be the 65th; it raises `MapEvent::ModuleCap` and stays silent exactly as for `NameTaken`. An established module whose map is already full does not insert a further id and raises `MapFull` (011-06, 011-07).

## Work
1. Add `ModuleCap` to `MapEvent` (011-06 defines the enum; the epic lists the variant).
2. During the listening phase, record when a hello from a distinct id not in the map is refused with `MapFull` (011-07) while the map holds `MAX_PEERS` = 63 non-quarantined peers (C-217). At the end of the listening phase of 011-11, before the first hello: if that was recorded (64 distinct active ids heard), raise `MapEvent::ModuleCap` and enter the same refused state as `NameTaken`: no hello is ever sent.
3. If both refusals apply, raise `NameTaken` first and queue `ModuleCap` (drained with `next_event()`).

## Acceptance criteria
1. With 64 established modules sending hellos, a newcomer raises `ModuleCap` at the end of listen-before-claim and sends zero hellos over the following 10 s (C-185, C-34).
2. With 63 established modules (the newcomer's map full at 63 peers, C-217, and no further id heard), the newcomer joins with its first hello at t = 1.5 s (± one tick).
3. A newcomer that is refused for ModuleCap leaves the established modules' maps unchanged (it never sent a hello).
4. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (behaviour follows the epic and C-185; 011-22 records the constant); "The frozen POC is not modified" (the POC had no module cap; no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Commissioning's refusal of more than 64 modules (C-34): not part of this epic.
- Admission refusals (epic 012): ModuleCap is not one of them (C-185).
- How the platform reports `ModuleCap` to the operator: epics 015, 026.
