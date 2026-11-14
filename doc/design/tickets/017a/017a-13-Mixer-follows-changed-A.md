# 017a-13 — Reference mixer follows a changed inherited A
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** 017a-08, epic 009, epic 015

## Context
- The reference mixer is a fan-in outside any presentation group; it inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission, or uses an explicit budget (C-142, 017a-08). In L0 and in the cross-machine check it has an explicit budget, so the inherited path and its changes are exercised only here.
- **Carrier of a changed A (settled, C-198):** the receiver report (C-40) carries the consumer's current A, its own or the one it inherited. A change propagates one hop per report, on the data plane. The mixer's own receiver reports therefore carry its current A (the explicit budget, or the inherited value) to its upstreams.
- **Re-alignment on a changed A (settled, C-186):** every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do.
- **Re-alignment position (settled, C-208):** the mixer picks its own position, one lead (C-43: 64 slices) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment.
- **A larger than buffer memory (settled, C-211):** the commissioning tool checks declared buffer memory before applying a change of A. If one reaches the mixer anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running.
- The re-alignment mechanism, ring resizing and alarm are epic 009's; this ticket wires the mixer to them and tests the mixer's behaviour.

## Work
1. Have the mixer supply its current A to the runtime's receiver reports (C-198): the explicit budget when configured, otherwise the inherited value.
2. On a receiver report from downstream carrying a different A, when the mixer's A is inherited: trigger epic 009's re-alignment with the new ceiling, at a position one lead ahead announced in the mixer's output (C-186, C-208).
3. Declare the mixer's alignment buffer memory through the module API of epic 015, and when a new A exceeds it, keep the deepest ring it can hold as the ceiling and raise cannot-meet-A (C-211).
4. A test downstream consumer that reports a chosen A in its receiver reports, to drive the inherited path.

## Acceptance criteria
1. With no explicit budget and a test downstream reporting A₁, the mixer's ceiling and ring equal A₁, and its own receiver reports carry A₁ (C-142, C-198).
2. When the downstream report changes to A₂, the mixer announces a re-alignment position one lead (64 slices) ahead in its output, re-enters aligning there, and afterwards its ceiling and ring equal A₂; its receiver reports carry A₂ (C-186, C-198, C-208).
3. With an explicit budget configured, a downstream report of a different A does not change the mixer's ceiling.
4. An A₂ larger than the declared buffer memory leaves the ceiling at the deepest ring that fits, raises the cannot-meet-A alarm, and the mixer's output continues without a gap caused by the change (C-211).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The re-alignment, ring-resizing and alarm logic themselves (epic 009); the commissioning tool's buffer-memory check (commissioning epic).
- A changed A in L0 scenarios (017b).
