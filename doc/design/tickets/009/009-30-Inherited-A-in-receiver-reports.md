# 009-30 — Inherited A carried in receiver reports
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-11, 009-29, epic 014

## Context
A fan-in outside any presentation group inherits the A of the downstream group it feeds, the largest if several (C-142, `SkewBudget::InheritedA`, 009-11). When that A changes while inputs are live, the fan-in re-aligns at an announced position (C-186, `FanIn::change_inherited_a`, 009-29). A ruling now fixes how the change travels `[decision]`:

- C-198: the receiver report (C-40) carries the consumer's current A, its own or the one it inherited. A change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186).
- C-40: one receiver report per bundle every 100 ms.
- The receiver report wire format and its parsing are epic 014's; this ticket provides the value a consumer puts in its outgoing report and the fan-in's handling of the A read from incoming ones.
- C-208: the fan-in picks the re-alignment position itself, one lead (C-43) ahead, and announces it in its own output; `change_inherited_a` (009-29) does this, so no position travels with the reported A.
- C-211: a new A deeper than the fan-in's compiled ring is applied with the ceiling clamped to that ring and the cannot-meet-A alarm raised (009-29); the change is never refused at the fan-in, so the inherited A in force is the new A.
- C-216: the fan-in's receiver report carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need.

## Work
Extend `FanIn`:
```rust
impl<..> FanIn<..> {
    pub fn current_a_ns(&self) -> Option<i64>;                                     // C-198
    pub fn on_downstream_a(&mut self, downstream: usize, a_ns: i64) -> Option<i64>; // C-198, C-142
}
```
- `current_a_ns`: the A this consumer carries in the receiver report it sends upstream: `GroupA` → that A; `InheritedA` → the inherited A currently in force; `Configured` or `None` → `None` (no A to carry; documented).
- `on_downstream_a`: for a fan-in with `InheritedA`, records the A read from the receiver report of downstream consumer `downstream` in a fixed-capacity table (const generic, no allocation; the downstream set is the one learned at admission, C-142). When the largest recorded A differs from the inherited A in force, returns `Some(largest)`; the caller passes it to `change_inherited_a` (009-29), which picks the position (C-208). An unchanged largest returns `None`. Any other budget returns `None` without recording (documented).
- After `change_inherited_a`, `current_a_ns` returns the new A, so the next report upstream carries it and the change moves one more hop (C-198). When the ceiling was clamped (C-211) it still returns the new inherited A, not the clamped ceiling (C-216), so each upstream fan-in applies C-211 against its own memory.

## Acceptance criteria
1. A fan-in with `GroupA(10 ms)` → `current_a_ns() == Some(10_000_000)`; `Configured` → `None`.
2. Inherited A 10 ms from two downstream consumers both at 10 ms; one reports 5 ms → `None` (largest still 10 ms); the other then reports 5 ms → `Some(5_000_000)`.
3. A downstream report raising the largest to 12 ms → `Some(12_000_000)`; a repeated report of 12 ms before the change is applied → `Some(12_000_000)` again, and `None` once `change_inherited_a` has applied it.
4. Chain of two fan-ins with inherited A: after the downstream one accepts a new A, its `current_a_ns` carries it, and feeding that value to the upstream one's `on_downstream_a` returns the new A (one hop per report, C-198).
5. After a change clamped by C-211 (`ceiling_short: true`, 009-29), `current_a_ns` returns the new inherited A, not the clamped ceiling (C-216).
6. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The receiver report wire format, its sending every 100 ms and its parsing (epic 014).
- Picking and announcing the re-alignment position (009-29, C-208).
- The fan-in's re-alignment itself (009-29).
