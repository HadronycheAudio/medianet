# 012-28 — Changed A carried in receiver reports
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-15, 012-26

## Context
C-142 has a fan-in outside any presentation group learn its inherited A at admission, from `Accept.group_a_ns` (012-26). C-186 has every fan-in that inherited the old A re-align when A changes while its inputs are live. C-198 settles how a change reaches an upstream fan-in after admission `[decision]`:
- the receiver report (C-40, one per bundle every 100 ms) carries the consumer's current A, its own group's A or the one it inherited;
- a change propagates one hop per report, on the data plane;
- each fan-in that inherited the old value re-aligns (C-186), through the 012-26 path.

The value a report carries is the one an accept carries (012-26): the consumer's own group A, else its inherited A, else none. A configured budget is local and is not passed upstream (C-142). Because the carrier is the data plane, not gossip, L-1 and the 012-21 invariant hold: a reported A only updates a stored figure and queues new budgets; the connection state still changes only on `AnnouncedRealign` evidence (012-11).

Sending and parsing receiver reports are outside this epic (data plane, epic 009); this ticket supplies the value to send and takes the value received.

## Work
In `hadronyche-core` (admission module), no allocation:
- Consumer side: `fn report_a_ns(&self) -> Option<u32>` (suggested name), the consumer's current A to place in each receiver report it sends: own group A, else `inherited_a_ns()` (012-26), else `None`. It is the same value that fills `Accept.group_a_ns`, and it follows every change of the inherited A at once.
- Producer side: `fn on_report_a(&mut self, conn: ConnId, a_ns: Option<u32>)` (suggested name), called with the A of each receiver report received on a producer-side connection. For a non-terminal producer-side slot, replace the stored `group_a_ns` (012-26) when it differs; ignore an unknown, consumer-side or terminal slot.
- When the stored value changes and `inherited_a_ns()` changes as a result, run the 012-26 C-186 path unchanged: update the stored A of each non-terminal aligned input whose A was inherited and queue `(ConnId, new A)` for `next_rebudget`. Inputs sized to their own group's A or to a configured budget are not affected.
- `on_report_a` emits no `ConnEvent` and moves no state.
- Add both methods to the epic's deliverables (names may be refined).

## Acceptance criteria
1. C-198, one hop per report: chain P1 → P2 → sink, P1 and P2 outside any group, the sink in a group with A = 10 ms, all Live. The sink's group A becomes 20 ms: the sink's `report_a_ns` returns 20 ms; after `on_report_a(conn, 20 ms)` on P2's output connection, P2 queues `(ConnId, 20 ms)` for each inherited aligned input and its `report_a_ns` returns 20 ms; P1 queues nothing until it receives P2's report carrying 20 ms, then queues its own inputs.
2. Reports repeating the stored A queue nothing.
3. A fan-in feeding two groups with A = 10 ms and A = 20 ms: a report lowering the 20 ms connection to 5 ms makes the inherited A 10 ms (the largest), and the affected inputs are queued with 10 ms.
4. `on_report_a` changes no connection state and returns no event; each queued input stays Live until `AnnouncedRealign` evidence moves it to Aligning (C-186).
5. A report on a terminal, unknown or consumer-side connection changes nothing.
6. A report carrying no A (a downstream consumer with only a configured budget or no budget) stores `None`, as an accept carrying `group_a_ns: None` does.
7. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: the receiver report is parsed on the data plane (epic 009), not in this ticket
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field is added
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — the reported A arrives on the data plane and only queues new budgets (criterion 4)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- The receiver report layout, sending a report every 100 ms (C-40) and parsing it (data plane, epic 009). Announcing the re-alignment position, picked by each fan-in one lead ahead (C-208), and resizing rings and the ratchet ceiling (epic 009, C-53, C-186). A reported A that the fan-in's buffer memory cannot hold (C-211, 012-29). Choosing or changing a group's A (commissioning, C-55).
