# 012-24 — Cancel a Pending request
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-14

## Context
C-45 `[decision]`: a request whose target is not present has no timeout. It stays Pending, shown as unsatisfied, until the target appears or the operator cancels it. The graph is a desired set (epic 012), so a cancelled request leaves the desired set. The epic adds `Admission::cancel(&mut self, conn: ConnId) -> Option<ConnEvent>` for this; nothing else cancels a Pending request.

## Work
- Implement `cancel`: for a Pending slot, apply the `Cancelled` input of 012-11, free the slot, remove it from the graph view, and return an event saying the connection left the desired set (`ConnEvent::State` of the last state followed by slot release; document the exact form).
- For any other state `cancel` returns `None` and changes nothing; tearing down an admitted connection is not part of C-45.

## Acceptance criteria
1. 012-A10: cancelling a Pending request frees its slot, removes it from the graph view, and sends nothing on the wire.
2. A target appearing after the cancel does not revive the request.
3. `cancel` on Requested, Admitted, Live or a terminal slot returns `None` and changes nothing.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-45
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: acts on Pending only
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- The operator interface that calls `cancel` (epics 015, 028). Removing admitted connections.
