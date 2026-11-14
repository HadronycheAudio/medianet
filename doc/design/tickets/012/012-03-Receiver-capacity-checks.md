# 012-03 — Receiver capacity checks
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-01, 012-02

## Context
In epic 012 the receiver (consumer side) checks every declared unit of a request before accepting `[decision]`:

| Unit | Checked by | Refusal reason |
|---|---|---|
| bit/s on the wire, including headers | receiver against its interface, ceiling 70 % (C-23) | `PortCeiling` |
| packets/s | receiver | `PacketRate` |
| slice rate | receiver | `SliceRate` |
| input count | receiver | `InputCount` |
| buffer memory (A × rate of the subscribed members, C-96) | receiver; decisive on embedded | `BufferMemory` |
| compiler cost bound per slice | receiver, for DSL modules | `CostBound` |

CPU for non-DSL modules is a diagnostic (overrun counter and alarm), **not** admission. `Declared` is `{ wire_bits_per_s: u64, packets_per_s: u32, slices_per_s: u32, buffer_bytes: u32, cost_bound_ops: Option<u32> }`.

Settled rulings:
- Cost bound unit (C-106): counted abstract operations per slice, identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in its regime stamp; admission converts the declared operations to time with the receiver's factor and compares with its remaining CPU time per slice.
- Skew budget (C-96): the consumer's presentation group's A; outside any group, the inherited downstream A or the configured budget (C-142, resolved by 012-26). The buffer figure is computed by 012-04.
- A consumer with no free connection slot refuses with `InputCount` (epic 012 handshake rules).
- Wire cost of a controller stream (C-219): computed like an audio stream, one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. The declared bit/s and packets/s of a request that includes a controller member count it at this figure (supplied by epic 010's `wire_bits_per_second`), the same figure the commissioning tool uses; the port and packet-rate checks treat it like any other member.

The per-platform values of the limits are still open (epic open sub-question owned by L1 measurement, C-92/C-93, and platform tasks 015, 021, 022, 026): they are configuration inputs here.

## Work
In `hadronyche-core` (admission module):
- Add `pub struct ReceiverBudget` holding the receiver's interface `PortBudget` and its remaining capacity for packets/s, slices/s, inputs, buffer bytes, and CPU time per slice with the calibration factor in nanoseconds per operation (both `None` for non-DSL consumers). Limits are configuration inputs supplied by the platform or device layer.
- Add `pub fn check_receiver(budget: &ReceiverBudget, d: &Declared) -> Result<(), RefuseReason>` checking in the fixed order of the table above and returning the first failing reason.
- Cost check: `cost_bound_ops × ns_per_op ≤ remaining CPU ns per slice`, in integer arithmetic without overflow. Checked only when the consumer is a DSL module; a DSL consumer receiving `None` is refused with `CostBound`.
- Add `commit` / `release` on `ReceiverBudget` that add or remove one admitted connection's declared amounts (and one input).

## Acceptance criteria
1. Each of the six reasons is returned by a unit test where only that unit exceeds the remaining budget.
2. When several units exceed, the first in table order is returned (one test with all six exceeded returns `PortCeiling`).
3. A request exactly at every remaining limit is accepted; one unit above in any field is refused.
4. `commit` then `release` of the same `Declared` restores the budget exactly (property test).
5. Non-DSL consumer: any `cost_bound_ops` value is ignored (accepted).
6. Cost conversion: the same `cost_bound_ops` is accepted on a target with a small calibration factor and refused with `CostBound` on one with a factor large enough to exceed the remaining time (C-106).
7. GbE consumer with two 96-channel bundles admitted (676 Mbit/s, 64 000 packets/s, C-92): a third is refused with `PortCeiling`.
8. C-219: a request of audio members plus the controller member is checked with the controller member's bit/s computed as (headers + controller bytes per slice + 66) × 8 × 2 000 and its 2 000 packets/s added to the audio members' figures; a remaining budget that holds the audio members but not the controller member's figure refuses it with `PortCeiling` (or `PacketRate` when only the packet budget is short).
9. No allocation; builds in the 012-01 `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: the calibration factor is read from the stamp, not written here
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-23, C-96, C-106, C-219
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Buffer memory figure for a bundle subset (012-04).
- Manifest, identity and presentation-group checks (012-05, 012-06, 012-07).
- Wiring into the responder path (012-13). CPU overrun diagnostics (not admission). Measuring the calibration factor (compiler and platform epics).
