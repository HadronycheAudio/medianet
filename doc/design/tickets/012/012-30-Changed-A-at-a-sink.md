# 012-30 — Changed A at a sink
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-26, 012-28, 012-29

## Context
012-26 re-budgets the aligned inputs that inherited A when the inherited A changes (C-186), and 012-28 has a sink's `report_a_ns` follow a change of its own group's A (C-198). No ticket says what the engine does with the sink's own aligned inputs when its group's A changes: 012-26 leaves inputs sized to their own group's A untouched. C-214 settles where the sink switches `[decision]`:
- at its own position, one lead (C-43) ahead, as a fan-in does (C-208);
- it has nobody to announce it to.

How the change reaches the sink is settled by C-224 `[decision]`: through the commissioning tool, inside a declared window (C-153); the tool validates buffer memory first (C-211) and then announces the new A to the group's members. The engine receives the announced value through `set_group_a_ns`; the announcement message and its receipt are not this epic's.

If a change reaches the sink anyway that needs more buffer memory than it holds, C-222 settles it `[decision]`: the clamp of C-211 applies to any consumer that buffers to meet A, sinks included, and C-216 applies to a sink in the same way. The sink keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running; the alarm stays raised until a later change of A that fits, and its receiver report carries the A it was given, not the clamped value. This is the 012-29 capping, reached from this path.

Picking the position and resizing the rings and the ratchet ceiling are the data plane's (epic 009, C-53). This epic supplies the new A of each affected input, as for a fan-in, and keeps L-1 and the 012-21 invariant: the engine does not move the input itself; the transition Live → Aligning arrives as re-alignment evidence from the data plane (the `AnnouncedRealign` variant, here for a switch the sink does not announce; names may be refined).

## Work
In `hadronyche-core` (admission module), no allocation:
- `fn set_group_a_ns(&mut self, ns: Option<u32>)` (suggested name; epic deliverable): the A of the consumer's own presentation group (C-55), set at start-up from configuration and on each change announced by the commissioning tool inside a commissioning window (C-224), read by 012-26 for `Accept.group_a_ns` and `skew_budget_for`, and by 012-28 for `report_a_ns`.
- When the value changes: for every non-terminal aligned input whose A was its own group's A, update its stored A and queue `(ConnId, new A)` on the 012-26 `next_rebudget` list through the 012-29 capping check; 012-04 and 012-17 recompute buffer and socket-buffer figures from the new A. An input whose new figure does not fit is queued with the 012-29 ceiling, marked clamped and alarmed exactly as for a fan-in (C-222, C-211, C-216). Inputs sized to an inherited A or a configured budget are not affected by this call.
- Move no state. The only event is the cannot-meet-A alarm of a newly clamped input, returned with the drained entry as in 012-29.

## Acceptance criteria
1. 012-A20 (C-214): a sink in a group with A = 10 ms, two Live aligned inputs and buffer memory for 20 ms; `set_group_a_ns(Some(20 ms))` queues one `(ConnId, 20 ms)` entry per input, their buffer and socket-buffer figures follow 20 ms, both stay Live until re-alignment evidence moves each to Aligning, and no event is raised.
2. `report_a_ns` returns 20 ms after the call (012-28).
3. Setting the same value again queues nothing.
4. 012-A20 (C-222): a sink whose buffer limit holds 15 ms for an input admitted at 10 ms receives `set_group_a_ns(Some(20 ms))`; the entry is queued with a 15 ms ceiling, the committed figure follows 15 ms, one alarm is raised, the input stays Live and no refuse message is sent; `a_clamped(conn)` stays true and `report_a_ns` returns 20 ms until a later change back to 10 ms, which clears the mark with no alarm (C-216).
5. An input sized to an inherited A or a configured budget is not queued by the call.
6. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-214, C-222, C-224
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created (the sink case of a changed A beyond buffer memory is C-222)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — the call only queues new budgets; the state changes on data-plane evidence (criterion 1)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Picking the switch position one lead ahead and resizing rings and the ratchet ceiling (epic 009, C-53, C-214). Choosing or changing a group's A (commissioning, C-55), checking declared buffer memory before applying it and announcing it to the group's members inside a commissioning window (epic 028, C-224, C-211, C-153).
