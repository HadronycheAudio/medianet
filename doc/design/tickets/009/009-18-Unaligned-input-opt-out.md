# 009-18 — Unaligned input opt-out
**Epic:** 009-Gap-and-reassembly-core · **Effort:** XS · **Depends on:** 009-13, 009-16

## Context
Fan-in alignment `[decision]`: a per-input "unaligned" opt-out exists (mix on arrival); a return edge in a cycle is declared unaligned. Single-input modules are arrival-driven and forward immediately; deadlines exist only at fan-in points and at sinks. All *aligned* inputs of a consumer share its one offset.

The epic's deliverables name the API: `FanIn::set_unaligned(&mut self, input: InputIndex, unaligned: bool)`, set per input before aligning. An unaligned input has no ring, no lateness sample, no ratchet participation and is never late `[decision]`.

Relevant constants `[decision]`: C-51 (offset = maximum observed lateness plus headroom), C-52 (headroom = largest of 25 % of the maximum, one slice period, twice the clock bound), C-101 / C-109 (ratchet and demotion apply to aligned inputs).

## Work
- `FanIn::set_unaligned(&mut self, input: InputIndex, unaligned: bool)` at construction time (before aligning).
- An unaligned input: no ring storage use, no lateness observation (excluded from `finish_aligning`'s maximum, C-51), no ratchet participation and never demoted (009-16), never `Late`.
- `on_deadline` reports for it the latest slice received since the previous round (`Content`), or `Missing` if none; it does not set `degraded` for being absent unless the caller asks (documented).
- The flag is set from the operator policy in the commissioning configuration (C-165) or by the graph builder for a return edge in a cycle; this ticket only provides the flag.

## Acceptance criteria
1. An unaligned input with lateness 50 ms during aligning does not change the offset (other inputs' max 2 ms, slice 0.5 ms, clock bound 0.1 ms → offset 2.5 ms, C-52).
2. A slice arriving on an unaligned input after the aligned deadline is reported in the next round as `Content`, never counted `late`.
3. An unaligned input never triggers `ratchet` and is never `Demoted`.
4. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Cycle detection and return-edge declaration (graph / DSL epics, e.g. epic 024); reading the operator policy from the commissioning configuration (C-165).
- Arrival-driven forwarding of single-input modules (data thread, epic 015).
