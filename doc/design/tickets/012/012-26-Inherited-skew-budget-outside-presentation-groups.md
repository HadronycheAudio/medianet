# 012-26 — Inherited skew budget outside presentation groups
**Epic:** 012-Admission-handshake · **Effort:** M · **Depends on:** 012-04, 012-08, 012-12, 012-13, 012-17

## Context
The skew budget of a consumer in a presentation group is the group's A (C-55, C-96); its rings, its buffer-memory check (012-04) and its socket buffer (012-17) are sized to A. C-142 settles the case of a fan-in outside any presentation group `[decision]`:
- it inherits the A of the downstream presentation group it feeds, the largest if several, learned at admission;
- with no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused, with the reason `NoSkewBudget` (C-185).

Inputs that operator policy opts out of alignment (C-165) are not aligned inputs and are not refused under this rule.

How the A is learned at admission `[decision]`, as stated in the epic: every accept carries `group_a_ns`, the A of the group that applies at the consumer (its own group's A, or the A it inherited itself). A module takes as its inherited A the largest `group_a_ns` over its admitted, non-terminal producer-side connections. This lets the value pass through a chain of processors outside any group. A configured budget is local and is not passed upstream: C-142 makes only a group's A inheritable.

Settled since this ticket was written `[decision]`:
- Refusal reason (C-185): an aligned input with neither an inherited A nor a configured budget is refused with `NoSkewBudget` (the 18th reason; 012-01). This replaces the provisional `BufferMemory`.
- A changed while inputs are live (C-186): every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do. This replaces the provisional "keep the A used at admission". The re-alignment position is picked by each fan-in itself, one lead (C-43) ahead, and announced in its own output (C-208). To keep L-1 and the 012-21 invariant (no state change out of Live from `on_message`), the engine does not move the input itself: it publishes the new A of each affected input, and the transition Live → Aligning arrives as `AnnouncedRealign` evidence from the data plane (epic 009, which owns the ratchet and its ceiling, C-53), as for a sink.

- Carrier of a changed A upstream (C-198): the receiver report (C-40) carries the consumer's current A, its own or the one it inherited; a change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186). This replaces the open question of the carrier. This ticket handles the changes seen at admission and on connection end (a downstream accept carrying a different `group_a_ns`, a downstream connection that ends); 012-28 feeds the reported A into the same stored `group_a_ns` and the same re-budget path.

## Work
In `hadronyche-core` (admission module), no allocation:
- Consumer side (012-13): fill `Accept.group_a_ns` with the consumer's own group A when it belongs to a group (from its configuration, C-55), otherwise with its inherited A, otherwise `None`.
- Requester side (012-12): store `group_a_ns` from each accept on the producer-side slot.
- `fn inherited_a_ns(&self) -> Option<u32>`: the largest stored `group_a_ns` over producer-side slots from Admitted to Live, Interrupted or Demoted; terminal slots do not count.
- `set_configured_skew_budget(ns: Option<u32>)` (epic deliverable), set from configuration.
- `fn skew_budget_for(&self, input: PortRef) -> Result<u32, RefuseReason>`: own group A, else inherited A, else configured budget, else `Err(RefuseReason::NoSkewBudget)` (C-185); an input opted out of alignment (C-165) needs no budget. 012-04 and 012-17 take the result as their A.
- Store the A in force in the consumer-side slot of each aligned input.
- When `inherited_a_ns` changes (C-186): for every non-terminal aligned input whose A was inherited, update its stored A and queue `(ConnId, new A)` in a fixed-size list drained one per call, as the 012-14 draining rule requires (suggested `fn next_rebudget(&mut self) -> Option<(ConnId, u32)>`). Housekeeping (epic 015) passes each entry to the data plane, which announces the re-alignment position and resizes the rings and the ratchet ceiling; 012-04 and 012-17 recompute buffer and socket-buffer figures from the new A. The connection state changes only on the resulting `AnnouncedRealign` evidence (012-11). Inputs sized to their own group's A or to the configured budget are not affected.

## Acceptance criteria
1. 012-A17: a fan-in outside any group, feeding two groups with A = 10 ms and A = 20 ms, admits an aligned input with a buffer and socket buffer sized to 20 ms (C-142).
2. A chain source → processor P1 → processor P2 → sink in a group with A = 15 ms: once P2's output and then P1's output are admitted, P1 sizes its aligned inputs to 15 ms.
3. With no downstream group and a configured budget of 5 ms, aligned inputs are sized to 5 ms, and P1's accepts carry `group_a_ns: None`.
4. With no downstream group and no configured budget, an aligned input is refused with `NoSkewBudget` (C-185), on the wire and as `Refused(NoSkewBudget)`; an input opted out of alignment (C-165) is admitted.
5. C-186: a fan-in with a Live aligned input inheriting 10 ms admits a further output whose accept carries 20 ms; one `(ConnId, 20 ms)` entry is queued for that input, its buffer and socket-buffer figures follow 20 ms, its state stays Live until `AnnouncedRealign` evidence moves it to Aligning, and no state changes from `on_message`. When that downstream connection then ends (Dead, Ended), the inherited A falls back to 10 ms and the input is queued again with 10 ms.
6. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — the `group_a_ns` field is decoded by the 012-08 codec and covered by the 012-21 target
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field is added
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — a change of the inherited A only queues a new budget; the state changes on data-plane evidence (criterion 5)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Ring allocation and resizing (outside this epic); announcing the re-alignment position and resizing the ratchet ceiling (epic 009, C-53, C-186). The per-input alignment opt-out policy itself (commissioning configuration, C-165). Choosing A (commissioning, C-55). Taking a changed A from receiver reports after admission (C-198, 012-28). A new A that the fan-in's buffer memory cannot hold (C-211, 012-29). A change of the consumer's own group A (C-214, 012-30).
