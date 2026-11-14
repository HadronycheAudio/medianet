# 011-07 — Discovery machine: hello cadence and expiry
**Epic:** 011-Gossip-and-discovery-core · **Effort:** M · **Depends on:** 011-02, 011-03, 011-06

## Context
`Discovery` is the pure state machine (no I/O, no allocation) that owns the peer map, the module's own identity and the hello timers. The platform calls it with the current time and received bytes; it writes outgoing hellos into an `Outbox`. It is owned by housekeeping (L-8).

Constants and rules:
- C-30 (inherited from the POC): hello period 1 s. A hello is sent at join and every 1 s. Join is the end of the 1.5 s listen-before-claim (C-32); the listening phase itself is 011-11. This ticket measures the cadence from the join instant.
- C-31 (inherited): hello expiry 3 s, **for non-live peers only**.
- L-1 `[decision]`: gossip silence alone never tears down a live stream; a peer with a live connection is not removed from the map by hello expiry.
- L-5 `[decision]`: a hello with a new address never redirects a live stream; destinations are pinned at admission (epic 012). The map may record the new address, but nothing here pushes it to an existing stream.
- Epic AC 3, AC-20 (a): with all hellos suppressed for 30 s, no peer with a live connection is removed from the map.
- C-34, C-217: at most 64 modules, so at most 63 peers in the map (no entry for self); a 64th peer raises `MapEvent::MapFull` (map from 011-06).
- Event delivery `[decision]` (epic): an input can produce several events. `on_hello` and `on_tick` return the first; the rest wait in a fixed-capacity queue drained with `next_event()` until it returns `None`.

## Work
1. Implement with the epic's signatures:
   ```rust
   pub struct Discovery { /* map + own identity + timers + event queue */ }
   impl Discovery {
       pub fn new(me: LocalInfo, rng: &mut dyn IdSource) -> Self;
       pub fn on_hello(&mut self, now_ns: i64, from: Addr, dscp: u8, bytes: &[u8], out: &mut Outbox) -> Option<MapEvent>;  // dscp recorded by 011-25 (C-152)
       pub fn on_tick(&mut self, now_ns: i64, out: &mut Outbox) -> Option<MapEvent>;
       pub fn next_event(&mut self) -> Option<MapEvent>;
       pub fn mark_live(&mut self, peer: ModuleId, live: bool);                         // from the connection machine
   }
   ```
   `new` draws id and nonce from `rng`.
2. `on_tick`: emit the own hello at join and then every 1 s; expire peers whose last hello is older than 3 s **and** that are not marked live, raising `Left(id)`. Until 011-11 lands, join is the first tick; 011-11 moves it to the end of the listening phase without changing this ticket's tests, which are written relative to the join instant.
3. `on_hello`: parse; on a new id insert and raise `Joined(id)`; on a known id update the entry (including address, per L-5 in the Context); on a full map raise `MapFull`.
4. `mark_live(peer, true)` exempts the peer from expiry; `mark_live(peer, false)` makes it subject to the 3 s rule again, timed from its last hello.
5. Event queue: fixed capacity sized for the worst case of one input (for example `MAX_PEERS` expiries in one tick); `next_event()` drains it.

## Acceptance criteria
1. Ticking every 10 ms for 10 s after the join instant J, exactly 11 hellos are written (J, then every 1 s ± one tick).
2. A non-live peer whose hellos stop at t is removed with `Left(id)` at t + 3 s (± one tick), not before.
3. AC-20 (a): a peer marked live, with all hellos suppressed for 30 s, is still in the map at 30 s; after `mark_live(peer, false)` it is removed within 3 s of the next tick.
4. A hello from a known id with a new `Addr` updates the stored address and raises no `Joined`/`Left`.
5. A 64th distinct peer id (the map holding 63, C-217) produces `MapFull` and no insertion.
6. Ten non-live peers silenced at the same instant expire in one tick: the tick returns one `Left`, and `next_event()` yields the other nine, then `None`.
7. No code path in this machine opens an election (it has no access to `Election`).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] No election path is reachable from hello expiry (shown by test and by inspection of the transition table)

Not applicable (C-125): "Every parser touched is fuzzed" (calls the hello parser of 011-03, fuzzed by 011-05; adds none); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (behaviour follows the epic).

## Out of scope
- Reject counters and version events: 011-08. Domain 0 / strict mode: 011-09.
- Id collision and `redraw_id`: 011-10. Name refusal and the listening phase: 011-11. Stall detection: 011-12. Stamp version tracking: 011-23. Recording the observed hello DSCP: 011-25.
- Pinning stream destinations at admission: epic 012.
- Driving `Discovery` from a thread and a socket: epics 015, 026.
