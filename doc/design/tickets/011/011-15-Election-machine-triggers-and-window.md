# 011-15 — Election machine: triggers and window
**Epic:** 011-Gossip-and-discovery-core · **Effort:** M · **Depends on:** 011-13, 011-14, epic 005

## Context
`Election` is a pure state machine (no I/O, no allocation) owned by housekeeping (L-8). It agrees on one clock holder.

Rules `[decision]`:
- **Trigger:** 1.5 s without sync replies at a follower (C-37, delivered by the clock thread as `ElectionInput::ReplySilence`); the holder coming under pressure (`Pressure`); an operator force (`Forced(id)`). **Never** hello expiry.
- L-3: gossip is discovery, admission bootstrap and election input only. A follower that receives no sync replies for 1.5 s opens an election regardless of hellos.
- L-6: the holder is judged by sync replies only.
- Window C-33: 400 ms. Every module computes the same winner from the same candidate set, deterministically.
- Ordering C-221 (decided): among candidates with usable time a Locked candidate is preferred over one in holdover; among equals the lowest module id wins.
- Candidates: as in 011-14 (eligible bit, usable time per C-213, that is Locked or in holdover with the alignment claim not lapsed, not under pressure). The no-usable-time case (C-113: lowest-id holder-eligible candidate, timeline epoch bump) is finished in 011-17.
- Event delivery `[decision]` (epic): `step` returns the first event; further events of the same input wait in a fixed-capacity queue drained with `Election::next_event()`.

`[code]` POC behaviour to retire: the deterministic draw by hashing id with epoch (`module.rs:641`), replaced by the ordering of C-221; election re-requests every 10 s (`REQUEST_EVERY_NS`).

Shapes from the epic:
```rust
pub struct Election { /* epoch, candidates, deadline, event queue */ }
pub enum ElectionInput<'a> {
    ReplySilence,                       // 1.5 s without sync replies, from the clock thread
    Pressure, Forced(ModuleId),
    Message(&'a [u8]), Tick,
}
pub enum ElectionEvent {
    Opened { epoch: u32 }, HolderChanged { holder: ModuleId, epoch: u32, continuous: bool },
    TimelineEpochBump, FollowerRefused(ModuleId),
}
impl Election {
    pub fn step(&mut self, now_ns: i64, input: ElectionInput<'_>, map: &PeerMap,
                me: &Candidacy, out: &mut Outbox) -> Option<ElectionEvent>;
    pub fn next_event(&mut self) -> Option<ElectionEvent>;
}
```

## Work
1. Implement `Election::step` per epic 005's transition table: on `ReplySilence`, `Pressure` (when I am holder) or `Forced`, increment the election epoch, emit `Opened { epoch }`, broadcast an election message (011-13), set a deadline at now + 400 ms.
2. On `Message`, collect candidates for the current epoch. On `Tick` past the deadline, compute the winner by the ordering of C-221 (Locked before holdover, then lowest module id) and emit `HolderChanged { holder, epoch, continuous }`.
3. Implement the event queue and `next_event()`.
4. No input path from `Discovery` expiry reaches `step`; there is no `ElectionInput` variant for hello expiry.
5. No periodic re-request timer.

## Acceptance criteria
1. `ReplySilence` opens an election (`Opened`) and the result is decided at 400 ms (± one tick) after opening.
2. `Forced(id)` and `Pressure` (at the holder) each open an election.
3. Five simulated modules given the same candidate set compute the same winner, over 1 000 random candidate sets; the winner is a Locked candidate whenever one exists, otherwise a candidate in holdover, and in either class the one with the lowest module id (C-221).
4. No `Opened` occurs in a 10-minute run with replies flowing and no pressure/force inputs, even with periodic `Tick`s (no 10 s re-request).
5. `ElectionInput` has no variant derived from hello expiry (checked by inspection in review and noted in the transition table).
6. When one input produces two events, `step` returns the first and `next_event()` the second, then `None`.
7. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] No election path is reachable from hello expiry (shown by test and by inspection of the transition table)

Not applicable (C-125): "Every parser touched is fuzzed" (uses the election codec of 011-13, fuzzed there); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (behaviour follows the epic and epic 005's table).

## Out of scope
- Epoch merge and partition heal: 011-16. Handover continuity and the no-usable-time case: 011-17. Follower cap: 011-18.
- Detecting reply silence (the clock thread): epics 008, 015.
