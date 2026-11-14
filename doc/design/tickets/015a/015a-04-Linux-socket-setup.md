# 015a-04 — Linux socket setup (open_media, open_discovery)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-01
## Context
- File: `hadronyche-os-linux/src/net.rs`. Implements `OsNet::open_media(iface, class, rcvbuf)` and `OsNet::open_discovery(iface, group)`.
- IPv4 only in v1; multicast with TTL 1 (C-100).
- Send rules `[decision]`: don't-fragment on every packet; IP fragmentation is forbidden (`IP_MTU_DISCOVER` set to forbid fragmentation). By default a slice is one packet (C-05) at the path MTU; fallback path MTU is 1 280 bytes, which gives 1 252 bytes of UDP payload (C-06, C-100).
- DSCP classes (C-13), set with `IP_TOS`: sync CS6 (48); media and one-way probes EF (46); control (hello, election, admission, receiver reports, regime stamp) CS5 (40). The TOS byte is DSCP << 2.
- Receive options: `SO_TIMESTAMPNS`, `SO_RXQ_OVFL`, `IP_RECVTOS` enabled on every socket.
- Socket buffer `[decision]`: sized to the declared rate times the skew budget. The skew budget is capped by the presentation group's audition offset A (C-96), which has no default and is commissioned per group (C-55). The caller computes declared rate × A at admission and passes it as `rcvbuf`; the kernel may clamp it (`net.core.rmem_max`), so the effective value is reported.
- Multicast join pinned to the medianet interface, TTL 1.
- Medianets are wired, isolated LANs whose only traffic is this runtime's.
## Work
1. `open_media`: UDP/IPv4 socket, non-blocking, `IP_MTU_DISCOVER = IP_PMTUDISC_DO`, `IP_TOS = class << 2`, `SO_RCVBUF = rcvbuf` (report the effective value the kernel granted), receive options enabled, bound to the medianet interface.
2. `open_discovery`: same, class control (40), joined to `group` on the medianet interface only (`IP_ADD_MEMBERSHIP` with the interface address, `IP_MULTICAST_IF`), `IP_MULTICAST_TTL = 1`, loopback behaviour documented.
3. Expose the effective `SO_RCVBUF` and a "clamped" flag so callers can count and warn when the kernel grants less than declared rate × A.
4. Report whether `SO_TIMESTAMPNS` was accepted, so the receive-timestamp capability can be stamped (C-131, 015b-13).
5. Unit and integration tests on loopback, unprivileged, reading options back with `getsockopt`.
## Acceptance criteria
1. On both socket kinds, `getsockopt(IP_MTU_DISCOVER)` returns `IP_PMTUDISC_DO`.
2. `getsockopt(IP_TOS)` returns 192 (48<<2) for sync, 184 (46<<2) for media/probe, 160 (40<<2) for control and discovery.
3. The discovery socket has multicast TTL 1 and its membership is on the configured interface only.
4. `SO_TIMESTAMPNS`, `SO_RXQ_OVFL` and `IP_RECVTOS` read back as enabled.
5. Sockets are non-blocking (a receive on an empty socket returns immediately).
6. Requesting an `rcvbuf` above `rmem_max` sets the clamped flag and reports the granted value.
7. No IPv6 socket is ever opened.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — the clamp flag is published by 015a-14; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — no receive path (015b-10).
## Out of scope
- `send_to` (015a-05) and `recv_batch` (015a-06).
- Computing declared rate × A (caller, at admission, 015a-11).
- Packet capture verification of DF (015b-07).
