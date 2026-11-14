# 012-29 — A beyond fan-in buffer memory
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-03, 012-04, 012-26, 012-28

## Context
012-26 re-budgets every aligned input that inherited A when the inherited A changes (C-186), and 012-28 feeds a changed A from receiver reports into the same path (C-198). Neither says what happens when the new A needs more buffer memory than the fan-in holds. C-211 settles it `[decision]`:
- the commissioning tool (epic 028) checks declared buffer memory before applying a change of A and refuses one that does not fit;
- if such a change reaches a fan-in anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running.

C-222 `[decision]` applies the same clamp, and C-216, to any consumer that buffers to meet A, sinks included; the sink's own changed A reaches this capping through 012-30.

So a changed A is never a reason to refuse or tear down an admitted input. At admission the existing rule stands: an input whose buffer figure (012-04) exceeds the receiver's buffer-memory limit (012-03) is refused with `BufferMemory`.

Settled by C-216 `[decision]`: the cannot-meet-A alarm stays raised until a later change of A that fits; the fan-in's receiver report (C-198) carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need. `report_a_ns` (012-28) therefore keeps returning the inherited A, never the capped ceiling. This replaces the earlier reading of a single alarm event per capping: the alarm is a held state, cleared only by a fitting change, not by the end of `CannotMeetA` evidence.

## Work
In `hadronyche-core` (admission module), no allocation:
- In the 012-26 re-budget path, before queueing `(ConnId, new A)`: compute the input's buffer figure for the new A with the 012-04 rule and compare it with the buffer memory the receiver can give that input (its configured limit, 012-03, less the figures committed to its other inputs).
- If it fits, queue the entry as today. If not, queue it with a ceiling: the deepest A, in whole slices, whose buffer figure fits; commit that figure, not the full one, and keep the input's state unchanged.
- Mark each input capped this way as clamped in its slot and raise `ConnEvent::Alarm(conn)` when the mark is set (the cannot-meet-A trigger of C-173, C-174). While the mark is set the alarm stays raised (C-216): expose it as `fn a_clamped(&self, conn: ConnId) -> bool` (suggested name) so housekeeping keeps it in the hello's alarm field; a further capped change while already clamped raises no new event. `CannotMeetA` evidence from epic 009 raises the alarm as 012-15 already does and neither sets nor clears the mark. Because the re-budget path is also reached from `on_report_a`, which returns no event, the alarm is returned with the drained entry (suggested: `next_rebudget` yields the A, the ceiling and the alarm; names and shape may be refined).
- When a later change makes the full A fit again, queue the entry with ceiling equal to A, clear the mark and raise no alarm. Nothing else clears it.
- `report_a_ns` returns the A given (own or inherited), never the capped ceiling (C-216).

## Acceptance criteria
1. 012-A19 (C-211): a fan-in whose buffer limit holds 15 ms for an inherited input admitted at 10 ms receives a change to 20 ms; the entry is queued with a 15 ms ceiling, the committed buffer figure follows 15 ms, one alarm is raised for that connection, the input stays Live and no refuse message is sent.
2. The same change arriving through `on_report_a` (012-28) gives the same entry and alarm, raised when the entry is drained.
3. C-216: `a_clamped(conn)` is true from the capping until a later change back to 10 ms, which queues the entry with a 10 ms ceiling, clears it and raises no alarm. Repeated reports of 20 ms, a further capped change and the absence of `CannotMeetA` evidence leave it true and raise no new event.
4. C-216: `report_a_ns` keeps returning the inherited A (20 ms), not the 15 ms ceiling, while the ceiling is capped.
5. An admission (not a change) that exceeds the limit is still refused with `BufferMemory`.
6. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-211, C-216
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — a capped re-budget changes no state (criterion 1)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- The commissioning tool's check of declared buffer memory before applying a change of A (epic 028, C-211). Resizing the ring and the ratchet ceiling to the capped value, and judging cannot-meet-A from lateness (epic 009, C-174). Choosing A (commissioning, C-55).
