# 029-03 — Live read-only collector
**Epic:** 029-Monitor · **Effort:** M · **Depends on:** 029-01, 029-02, epic 015

## Context
- `hadronyche-monitor` (epic 029) must never affect a stream. `collect.rs` joins the multicast groups **read-only** and **never sends anything except the membership traffic that joining requires**. `[decision]`
- The network is IPv4 only, with multicast at TTL 1 (C-100): the only outbound traffic allowed from the monitor is IGMP membership; the monitor runs on a host attached to the medianet.
- Inputs: hellos on the discovery group (period 1 s, C-30: module id and instance nonce, who holds the clock, alarm field, control-degraded mark, stamp version number) and regime stamp messages multicast on the control group on any regime change and every 10 s, in sections of at most 1 252 bytes (C-17, C-18). If the discovery and control groups are the same address, one join covers both. The stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144).
- Receiver reports (one per bundle every 100 ms, C-40) are unicast consumer→producer and are **not** visible to the monitor; there is no multicast copy and no query message (C-47). Stream health comes only from per-connection summary counters in stamps.
- Epic criterion 029-AC-1: a capture on the medianet interface shows the monitor sends no control, sync, media or measurement packets and no stamp request.

## Work
1. `LiveSource` implementing `ObservationSource` (029-01): uses the hosted discovery socket from epic 015 in a receive-only configuration; joins the discovery and control IPv4 multicast groups (C-100).
2. Decode hellos → `Observation::Hello`; feed stamp sections to `StampAssembler` (029-02) → `Observation::Stamp` on completion.
3. Socket opened without any send path in the monitor's code: no call to `send`/`send_to` exists in `hadronyche-tools`' monitor code (enforce by a grep-based CI check or a lint). In particular the unicast stamp request of C-18 is never sent.
4. Run receive I/O on its own thread with a bounded channel to the consumer; count and expose any dropped observations (`dropped_observations: u64`), which must be 0 in normal operation (used by 029-22).
5. Until a module's first complete stamp arrives (at most 10 s, C-18), expose it as "no stamp yet" (029-02 `NoStampYet`).
6. Unprivileged: no capability or root requirement.

## Acceptance criteria
1. Against a two-module test medianet (epic 015 test harness), within 3 s of start the source yields at least one `Hello` per module (period 1 s, C-30), and within 10 s of start (C-18) at least one complete `Stamp` per module.
2. A regime change on one module yields a new `Stamp` observation with a higher stamp version without waiting for the 10 s refresh.
3. `grep` CI check: zero occurrences of `send`/`send_to`/`write` on network sockets in the monitor code.
4. A packet capture during a 60 s run shows only IGMP membership traffic attributable to the monitor; zero control, sync, media or measurement packets and zero stamp requests. (Partial 029-AC-1; full scenario test in 029-20.)
5. Runs as an unprivileged user in CI.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Reassembly logic (029-02), file replay (029-04), L0/L1 scenario test (029-20), scale test (029-22).
- Changes to the discovery socket or collector (epic 015). IPv6 (excluded by C-100).
