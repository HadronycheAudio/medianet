# 012-17 — Receive side enforcement parameters
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-13

## Context
The handshake of epic 012 sets up receive-side enforcement that epic 015 (housekeeping) applies `[decision]`:
- the **source filter**: only admitted peers are parsed;
- the **bounded drain per input**: per cycle, at most 2 × the admitted packets per slice (C-16), where the cycle is one slice period (C-156);
- the **socket buffer** sized to the declared rate times the skew budget, whose per-socket overflow counter is read on every receive `[kernel-doc]`.

Settled: the skew budget is the consumer's presentation group's A (C-96, C-55). Every stream runs at the domain rate (C-01) with 2 000 slices/s at the default slice length (C-02), one packet per stream per slice at defaults (C-05). A 96-channel source is 16 streams, 32 000 packets/s, 338 Mbit/s (C-22). The skew budget of a consumer outside any presentation group is settled by C-142 (the largest downstream group A, learned at admission, or a configured budget); 012-26 resolves it and the functions here take it as a parameter.

Inputs available from the admitted connection: `pinned_source`, `pinned_dest` (IPv4, C-100), `Declared { wire_bits_per_s, packets_per_s, slices_per_s, buffer_bytes, .. }`.

## Work
In `hadronyche-core`, pure functions/iterators over the `Admission` table, no allocation:
- `fn source_filter(&self) -> impl Iterator<Item = (Addr, u16)>`: one entry (pinned source address and port) per admitted, non-terminal consumer-side connection (Admitted, Locking, Aligning, Live, Interrupted, Demoted); removed when Dead, Ended or Refused.
- `fn drain_bound(d: &Declared) -> u32 = 2 × ceil(packets_per_s / slices_per_s)` packets per input per slice period (C-16, C-156).
- `fn socket_buffer_bytes(d: &Declared, skew_budget_ns: u64) -> u32 = ceil(wire_bits_per_s / 8 × skew_budget_ns / 1e9)`, saturating, integer arithmetic.

## Acceptance criteria
1. A 96-channel source (32 000 packets/s at 2 000 slices/s, C-22) → drain bound 32; a single stream (2 000 packets/s) → 2.
2. A refused or Pending connection never appears in the source filter; an admitted one appears from Admitted until Dead/Ended, including while Demoted.
3. `socket_buffer_bytes` for 338 Mbit/s and a 10 ms skew budget = 422 500 bytes; saturates without panic for `u64::MAX` inputs.
4. `slices_per_s == 0` is handled without division by zero (document the result).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-16 and C-96
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Installing the filter, setting `SO_RCVBUF`, reading the overflow counter, running the drain (epic 015 and platform epics).
