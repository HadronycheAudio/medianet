# 028a-10 — `validate`: declared buffer memory against A (new, C-211)
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** XS · **Depends on:** 028a-03, 028a-08, 028a-09

## Context
The commissioning tool checks declared buffer memory before applying a change of A and refuses one that does not fit (C-211). The check applies to any consumer that buffers to meet A, sinks included (C-222). If such an A reaches a consumer anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running (C-211, C-222); the alarm stays raised until a later change of A that fits, and the consumer's receiver report carries the A it was given, not the clamped value (C-216). `watch` reports that alarm (028b-18).
- A is the skew budget cap: alignment rings are sized to A (C-96). The design's sizing is about 370 KB per 16-member bundle at a 10 ms skew budget (C-27), that is channels × 96 000 samples/s (C-01) × 4 bytes (C-03) × A: 96 × 96 000 × 4 × 0.010 = 368 640 bytes.
- The A a module holds: a sink, its group's `a_ms` (C-55); a fan-in outside any group, the largest A of the downstream groups it feeds, or its `budget_ms` when it feeds none (C-142; the walk exists in 028a-09).
- A change of A reaches live fan-ins through the receiver reports, one hop per report (C-198), and each re-aligns with resized rings at its own announced position (C-186, C-208); each sink switches at its own position one lead (C-43) ahead (C-214). The check therefore covers every aligning module the change reaches, the group's sinks and the fan-ins upstream.
- A commissioned change of a sink's own A is applied by the tool inside a declared window: buffer memory is validated first (this check), then the new A is announced to the group's members (C-224, 028b-11).
- `buffer_kb` is optional (028a-03); absent means "not declared".

## Work
- For each module with at least one aligned input, sinks included (C-222) (connections not opted out, C-165), compute ring memory = Σ over its aligned inputs of the source output's channel count × 96 000 × 4 bytes × the A it holds.
- Check `memory.ring_exceeds`: refuse a module whose ring memory exceeds its declared `buffer_kb`, naming the module, the A, the group or budget it comes from, and both sizes (C-211).
- Notice `memory.undeclared`: list aligning modules with no `buffer_kb`, whose fit cannot be checked.
- Expose the check as a function that `propose-a --write` (028b-11) calls with the candidate A before writing it.
- Tests for the refusal and the notice.

## Acceptance criteria
1. A 96-channel aligned input with A = 10 ms needs 368 640 bytes (C-27); a module declaring 350 KB is refused, one declaring 400 KB passes (epic AC-11).
2. A fan-in feeding sinks of groups with A of 6 ms and 20 ms is checked at 20 ms (C-142); a sink is checked at its own group's A and refused like a fan-in (C-222).
3. An opted-out input adds nothing to ring memory.
4. An aligning module with no `buffer_kb` yields the notice, not a refusal.
5. All tests run with no network.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The runtime's behaviour when an oversized A arrives anyway (runtime epics; C-211). Writing the configuration (028b-11). Parsing `buffer_kb` (028a-03).
