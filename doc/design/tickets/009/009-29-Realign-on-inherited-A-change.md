# 009-29 — Re-alignment on a change of inherited A
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-11, 009-13, 009-16, 009-26

## Context
A fan-in outside any presentation group takes its skew budget from the A of the downstream group it feeds (C-142, `SkewBudget::InheritedA`, 009-11). When that A changes while inputs are live, a ruling fixes what the fan-in does `[decision]`:

- C-186: every fan-in that inherited the old A (C-142) re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do.
- C-96: rings are sized to A; `check_depth` (009-11) refuses an A deeper than the compiled `DEPTH`.
- C-208: each fan-in that learns of the change picks its own position, one lead (C-43: 64 slices = 32 ms at defaults) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment (009-16). Fan-ins at different depths switch at different, announced positions.
- C-211: the commissioning tool checks declared buffer memory (with `check_depth`, 009-11) before applying a change of A and refuses one that does not fit. If one reaches a fan-in anyway, the fan-in keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174, 009-26) and keeps its streams running. This replaces the earlier reading of this ticket, which refused the change and kept the old A.
- C-216: the cannot-meet-A alarm stays raised until a later change of A that fits (the hold of 009-26); the receiver report carries the A given, not the clamped value (009-30).
- C-215: a ratchet step is itself a re-alignment at an announced position one lead ahead (009-16), so both kinds share the `announce` field.
- C-214: a sink switches to a changed A at its own position, one lead (C-43) ahead, as a fan-in does, with nobody to announce it to; that is the "as the sinks do" of C-186 and belongs to the sink epics.
- C-53: the ratchet ceiling is A (009-16).
- C-50, C-51, C-52: aligning lasts 4 s and sets the offset to maximum lateness plus headroom (009-13).
- C-198: the changed A reaches the fan-in in the receiver reports (C-40) of the consumers it feeds, each carrying that consumer's current A; detecting the change from those reports is 009-30, which yields the new A passed here. The position is not carried by C-198: the fan-in picks it (C-208).
- A fan-in with `GroupA` or `Configured` budget is not covered by C-186 and is unchanged by this ticket.

## Work
Extend `FanIn`:
```rust
impl<..> FanIn<..> {
    pub fn change_inherited_a(&mut self, new_a_ns: i64) -> Realign; // C-186, C-208, C-211
}
pub enum Realign { At { position: u64, ceiling_ns: i64, ceiling_short: bool }, NotInherited }
```
- Only for a fan-in whose budget is `SkewBudget::InheritedA`; any other budget returns `NotInherited` without change (documented).
- Position (C-208): `position` = the next round's position + 64 × 48 (one lead, C-43), picked by the fan-in; from the call until that position passes, every `Round` carries `announce = Some(position)` (the field of 009-16), so the output path (epic 015) announces it in the fan-in's output as for a ratchet step. A pending ratchet step (009-16) is superseded by the re-alignment.
- Depth (C-211): `check_depth(DEPTH, SkewBudget::InheritedA(new_a_ns), slice_ns)` (009-11). `Ok` → ceiling = the new A, `ceiling_short = false`. `Err(Exceeded { .. })` → the change is still applied, with the effective span = the compiled `DEPTH` (the deepest ring it can hold), ceiling = `DEPTH` × `slice_ns`, `ceiling_short = true`; the fan-in calls `set_ceiling_short(true)` (009-26), which raises `CannotMeetA` at once and holds it (C-216), and the streams keep running. A later change that fits calls `set_ceiling_short(false)`.
- Rounds for positions before `position` run unchanged at the current offset.
- At `position`: the effective ring span becomes `ring_depth_for_skew(new_a_ns, slice_ns)`, or the compiled `DEPTH` when short; slices beyond it are refused as `TooEarly` as for any span (009-10); the ratchet ceiling becomes `ceiling_ns` (009-16); the phase returns to `Aligning` with the estimator restarted (009-13), so no round is produced until `finish_aligning`.
- Per-input health, session and counters are kept; the rolling 60 s window and the maximum since the connection went live (C-182, 009-13) are kept, since the connection itself is not re-admitted (reading recorded in 009-22).

## Acceptance criteria
1. Inherited A 10 ms → 5 ms with the next round at q → `At { position: p, ceiling_ns: 5_000_000, ceiling_short: false }` with p = q + 3 072 (64 slices, C-43, C-208); rounds q to p − 48 carry `announce == Some(p)`; rounds for positions before p are produced at the old offset; from p the phase is `Aligning`, the effective span is `ring_depth_for_skew(5_000_000, 500_000) == 10` slices, and after `finish_aligning` a persistently late input steps the offset at most to 5 ms (C-163) and is then demoted at that ceiling (C-109).
2. Inherited A raised beyond the compiled `DEPTH` (DEPTH 20, slice 0.5 ms, new A 10.5 ms) → `At { ceiling_ns: 10_000_000, ceiling_short: true, .. }`; one `CannotMeetA` event at once (C-211); rounds keep being produced up to the announced position and, after aligning, at a span of 20 slices; a persistently late input steps the offset at most to 10 ms and is then demoted (C-109).
2a. No `MeetsAAgain` before a later change of A (C-216); a later change to an A that fits (for example 8 ms) → `ceiling_short: false`, the hold is released (009-26) and the ceiling becomes 8 ms.
3. A fan-in with `GroupA` or `Configured` budget → `NotInherited`, no change.
4. Counters, health and session of every input are identical before and after the change.
5. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Detecting the new A from downstream receiver reports (009-30, C-198).
- Encoding the announcement in the output slices and its re-issue (output path, epic 015; layout epic 004).
- Refusing a change of A that does not fit before it is applied (the commissioning tool, C-211).
- The sinks' own re-alignment, at their own position one lead ahead with no announcement (C-214, sink epics), likewise for a ratchet step (C-225, 009-16); a sink clamps a change of its own A that does not fit with the same `check_depth` and `set_ceiling_short` (C-222, 009-11, 009-26).
- Raising and displaying the connection alarm (epic 012, C-173); this ticket emits the `CannotMeetA` event of C-211.
