# 022a-05 — DSCP marking through QoS flows and TOS-byte receive
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-03, 022a-04, epic 004

## Context
- Part of epic 022a (Windows edge backend), `net.rs`.
- DSCP values (C-13): sync **48**, media and one-way probes **46**, control **40**. Hellos and regime stamps are control messages (C-17, C-18), so discovery sockets use **40**.
- Windows `[memory]`: plain `IP_TOS` is typically ignored; marking goes through the QoS flow API, and arbitrary values may need administrator rights or policy. Check: the receiving peer reads the byte; otherwise the stamp says *unverified*.
- DSCP marking is **report-only** on Windows: failure to mark is reported, never fatal.
- How a peer reports what it received is settled (C-47): each producer and consumer publishes, per connection, the DSCP it observed in its own regime stamp. Receiver reports are not multicast and there is no query message. The regime stamp is multicast to the control group on change, refreshed every 10 s, and can be fetched by unicast request (C-18); its layout is epic 004's.
- Consequences: the **media** class can be peer-verified only when this edge is a producer (capture) and the consumer's stamp shows the observed value. The **sync** and **control** classes are peer-verified under C-152 (the holder reports the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos); that verification, and this edge's own report of the hello DSCP it observes from each peer, are ticket 022a-21. A sink-only edge is therefore verifiable on sync and control, not on media.
- Stamp states use C-129: *present* (marking requested and peer-verified), *unverified* (requested, no peer confirmation), *absent* (marking refused by the OS).
- As a consumer, this edge must itself publish the DSCP it observes on each incoming media connection (C-47); the TOS-byte receive provides it.
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- Open sub-question owned by epic 022 (answered by the hardware run in 022b-06): can DSCP be set without administrator rights on a standalone laptop? Falsifier: no → marking stays *unverified* and the commissioning checklist (task 028) documents the policy route.

## Work
1. Marking: for each socket class (sync / media+probe / control) create a QoS flow and request the DSCP value from C-13. Record per class `DscpMarking::{Requested, Refused(reason)}`; refusal (e.g. access denied) is logged and reported, not fatal.
2. Receive of the TOS byte: request the received TOS byte as control data and extend `parse_rx_cmsgs` (022a-04) to return it in `RxMeta`. Extend the fuzz target. Expose the observed DSCP per incoming connection for this edge's own stamp (C-47).
3. DSCP verification per class: `CapState` (C-129) per class. Media becomes *present* only when a consumer's regime stamp (C-47) shows, for its connection from this edge, the value 46; a mismatch or absence leaves *unverified*; a refused marking is *absent*. Sync and control use the same `CapState` slot and the same *absent* rule for a refused marking; their peer verification (C-152) is filled in by 022a-21.
4. Unit tests for the verification logic with synthetic peer stamps.

## Acceptance criteria
1. With marking refused (simulated access denied), the socket still opens and sends, and the class state is *absent*.
2. A synthetic consumer stamp showing 46 for its connection from this edge yields media *present*; any other value (e.g. 0) or no stamp yields *unverified*.
3. On loopback or a two-host test, the TOS byte of a received datagram is extracted correctly for at least values 0, 40<<2, 46<<2, 48<<2, and the observed DSCP appears in the per-connection data for the stamp.
4. The extended control-message parser fuzz target runs 60 s on every change and 1 hour nightly without panic, with the corpus kept (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Parsing the peer's regime stamp (core, epic 004 layout).
- The hardware run that answers the administrator-rights question and checks AC-34 against a Linux peer (022b-06).
- The stamp layout (022a-17).
- Peer verification of sync and control DSCP and the per-peer hello DSCP report (022a-21, C-152).
- The policy-route checklist itself (epic 028).
