# 009-16 — Offset ratchet and demotion
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-13, 009-14, 009-15, epic 008

## Context
Offset `[decision]`: the offset is frozen once live. Ratchet: it grows by one slice when an input is persistently late; it never shrinks automatically; it is capped by A; beyond the cap the offending input is demoted. Parameters validated by simulation in task 008.

Constants and rules `[decision]`:
- C-53: ratchet step = one slice period (0.5 ms, C-02); ceiling = the presentation group's audition offset A (C-55), or for a fan-in outside any group the inherited A or configured budget (C-142). For a sink the caller passes A − D as the ceiling (C-226).
- C-163: when the gap between offset and A is less than one slice period, the offset steps to A exactly.
- C-54: dwell before automatic re-alignment = 10 s.
- C-101: "persistently late" = at least one late slice in each of 10 consecutive 1 s windows; this is the dwell C-54 expressed as a test.
- C-215 (with C-43, C-208): a ratchet step is a re-alignment like any other: it takes effect at a position the fan-in announces one in-band announcement lead ahead (64 slices = 32 ms at defaults, C-43) in its own output, never silently on the next round. The step therefore applies from the announced position, not from the next round.
- C-225 (with C-214): a sink has no output to announce in, so it takes the step at its own position one lead (C-43) ahead, as it does for a changed A. The same `Stepped { at_position }` serves it: `at_position` and the deadline move are identical, and the sink simply carries `announce` nowhere.
- C-109: an input that is persistently late with the offset already at A enters the health state `Demoted`: treated as gapped, counted separately (`demoted`, 009-14), cleared only by re-admission. There is no automatic retry, so the former question "should a demoted input be re-tried automatically?" is closed and has no ticket.
- All aligned inputs, including bundle members, share the consumer's one offset, so a step moves every input's deadline.
- Epic 012 consumes ratchet and demotion events as connection-state inputs.

## Work
Extend `FanIn`:
```rust
pub enum RatchetOutcome { Stepped { new_offset_ns: i64, at_position: u64 }, AtCeilingDemoted(InputIndex), None }
impl<..> FanIn<..> {
    pub fn ratchet(&mut self, now_ns: i64, ceiling_ns: i64) -> RatchetOutcome;
    pub fn pending_realign(&self) -> Option<(u64, i64)>; // announced position and offset in force from it (C-215)
}
// Round gains: pub announce: Option<u64>   // pending re-alignment position, carried in the output (C-43, C-215)
```
- Per input: a ring of 10 one-second "had a late slice" bits (C-101), fed by `Inserted::Late`, rolled by `now_ns`.
- `ratchet`: if some input in health `Live` or `Interrupted` is persistently late: if `offset < ceiling_ns` → new offset = min(offset + slice_ns, ceiling_ns) (C-53, C-163), `at_position` = the next round's position + 64 × 48 (one lead, C-43, C-215), clear all dwell bits, `Stepped`; rounds before `at_position` keep the old offset and `deadline_ns` uses the new one from `at_position` on; while a step is pending no further step is taken; else (offset already at A) set that input's health to `Demoted`, emit `HealthEvent::Demoted`, `AtCeilingDemoted(input)`. At most one outcome per call.
- A `Demoted` input: reported as `Demoted` in every round (009-12), counted in `demoted` (009-14), excluded from the ratchet and from the interrupted count; the dead threshold still applies (009-15).
- `readmit(input)` (009-15) is the only way out of `Demoted` (C-109); it clears the ring and the dwell bits and returns the input to `Live` at the current offset.
- From the `Stepped` decision until `at_position` has passed, every `Round` carries `announce = Some(at_position)`, so the output path (epic 015) carries the announcement in the fan-in's output; its re-issue cadence (C-44) and encoding are the output path's. A sink (C-225) ignores `announce`; nothing in `ratchet` depends on whether the consumer has an output.
- No code path decreases the offset.

## Acceptance criteria
1. Input late in each of 10 consecutive 1 s windows, offset 2.5 ms, ceiling 10 ms, next round at q → `Stepped { new_offset_ns: 3_000_000, at_position: q + 3 072 }` (64 slices, C-43, C-215); `deadline_ns` is unchanged for positions before q + 3 072 and moves by 500 000 for every input from q + 3 072 on (EC-11); every round from q to q + 3 024 has `announce == Some(q + 3 072)` and later rounds `None`.
2. Late in 9 of 10 windows (one clean second) → `None`.
3. After a step, the dwell restarts: no second step before another 10 consecutive late windows.
4. Offset 9.8 ms, ceiling 10 ms, persistent lateness → `Stepped { new_offset_ns: 10_000_000, .. }` (C-163, A exactly); another 10 consecutive late windows → `AtCeilingDemoted(i)`; offset unchanged at 10 ms; health of i is `Demoted`; input i is `Demoted` in every later round and `demoted` grows by one per round; other inputs unaffected (EC-11).
5. A demoted input whose path becomes clean stays `Demoted` for a 120 s simulation (no automatic retry, C-109).
6. After `readmit(i)`, input i is `Live`, its rounds return `Content` again, and the offset is unchanged.
7. Over a 60 s simulation with lateness then a clean path, the offset never decreases.
8. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Providing A (presentation group configuration and commissioning, C-55) and the parameter validation (task 008).
- Re-alignment of a whole consumer (not implemented automatically).
- The connection alarm raised on demotion (C-173, epic 012).
- The re-admission protocol that leads to `readmit` (epics 005 and 012).
