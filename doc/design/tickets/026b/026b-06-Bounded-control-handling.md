# 026b-06 — Bounded control handling
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03, 026a-12

## Context

On hosted machines, housekeeping is isolated from the data plane with no shared lock. On no_std this is not enforced by an OS: **the implementer must show bounded control handling; otherwise the device is stamped report-only** `[decision]`.

Open sub-question owned by this epic: can bounded control handling be shown (worst-case parse and state-machine time)? Falsifier: no bound can be shown → stays report-only. The stamp's control-isolation capability (housekeeping cannot delay the data path) follows: present only if the bound is shown, else absent or unverified (C-161). The proposed bound on processing one bad input is linear in packet length and at most 10 µs per packet (C-164, pending L1).

Epic criterion 026-AC-1 (AC-35): the device joins as a follower with a 63-entry peer map (C-217); control handling is bounded (**worst-case `poll` time measured and stated**) or stamped report-only; its stamp is published over the network and recorded by the hosted collector.

`Device::poll` returns `PollStats` including worst-case time per call. Control messages are at most 1 252 bytes (C-17), except the path-MTU probe pair sent at media size (C-116); the peer map has 63 entries, a medianet having at most 64 modules (C-34) and a module holding no entry for itself (C-217), so at most 63 other modules exist; slices arrive every 0.5 ms (C-02). Stamp sections are multicast on change and every 10 s, and requested by unicast (C-18).

## Work

1. Stress the board from a host traffic generator (a tool binary in `hadronyche-tools`, C-133): 63 peers' hellos with changing stamp versions, sync at 50/s (C-36), one 6-channel stream, receiver reports, stamp requests, path-MTU probes at media size, malformed control packets of up to 1 252 bytes.
2. Collect `PollStats.worst_case_ns` over at least 1 hour; also measure worst-case parse and state-machine time per message type with a cycle counter.
3. Decide: if the worst case is below the slice period budget (0.5 ms minus audio work), set `control_handling = Bounded { worst_case_poll_ns }` and `control_isolation = Present`; else `ReportOnly` with `control_isolation` absent or unverified (C-161).
4. Publish the result in the stamp; verify the collector recorded it. Store the run log in `doc/design/reports/` (C-130).

## Acceptance criteria

1. Worst-case `poll` time stated in ns `[measured]`, with the load conditions.
2. The stamp recorded by the hosted collector shows either `Bounded` with that number or `ReportOnly`.
3. No audio underrun during the 1-hour stress run if `Bounded` is claimed.
4. Peer map stays at 63 entries max during the run (C-217).
5. The `control_isolation` state in the recorded stamp matches the decision (C-161).
6. Worst-case processing time of one malformed packet is recorded against packet length and compared with C-164 (linear, at most 10 µs per packet; a proposal pending L1); the comparison is reported, not a pass condition.

## Definition of done

- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The table "what weakens on no_std" is reflected field by field in the device's stamp

## Out of scope

- Conformance vectors (026b-08).
