# 012-18 — Graph view
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-14, 012-15

## Context
Epic 012 deliverable: a published "graph view" structure listing every wanted connection with its state and refusal reason. The graph is a desired set; the runtime publishes satisfied versus unsatisfied `[decision]`. There are no all-or-nothing groups in v1. "Target not present" shows as Pending, visible and unsatisfied, with no timeout (012-A10, C-45). A refused connection shows as unsatisfied with its reason (012-A2, AC-12). A cancelled Pending request leaves the desired set and the view (C-45, 012-24). Consumers: epic 017 (reference graph) and epic 028 (commissioning tool).

`ConnState { Requested, Pending, Refused(RefuseReason), Admitted, Locking, Aligning, Live, Interrupted, Demoted, Dead, Ended }`.

Settled by C-154 (with C-45): only Live is satisfied. Pending, Locking, Aligning, Interrupted and Demoted are shown by name as unsatisfied; Requested, Admitted, Refused, Dead and Ended are unsatisfied too.

## Work
- `pub struct GraphEntry { pub conn: ConnId, pub source: ModuleId, pub output: PortRef, pub consumer: ModuleId, pub input: PortRef, pub state: ConnState, pub satisfied: bool }` (refusal reason is inside `state`).
- `fn graph_view(&self) -> impl Iterator<Item = GraphEntry>` on `Admission`, no allocation; members list available by reference.
- `const fn is_satisfied(state: ConnState) -> bool`, true for Live only (C-154).
- Document entries for operators (feeds 012-22).

## Acceptance criteria
1. 012-A2 (AC-12): after a `PortCeiling` refusal, the view shows that connection with `state == Refused(PortCeiling)` and `satisfied == false`.
2. 012-A10: a Pending connection appears with `state == Pending` and `satisfied == false`; after its target appears it shows `Requested`; after an operator cancel it is absent.
3. A Demoted connection shows `state == Demoted` (012-A12).
4. 012-A16 (C-154): `is_satisfied` is true for Live and false for each of the other ten states (exhaustive test).
5. Every slot ever requested and not cancelled appears exactly once, including terminal ones until the slot is reused.
6. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: read-only view
- [ ] Refusal reasons and connection states are documented for operators

## Out of scope
- Transport/publication of the view to other hosts and its UI (epics 017, 028).
