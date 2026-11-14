# 021-04 — Interface-pinned multicast and unicast sockets
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** M · **Depends on:** 021-03, epic 016
## Context
`MacNet` implements `OsNet { open_media, open_discovery, iface_stats }` (trait from epic 016). This ticket covers `open_media` and `open_discovery`.

- Porting trap [memory]: a laptop has Wi-Fi up beside the medianet port. **Multicast follows the default route unless every socket is pinned to the medianet interface.** Pin every socket, for both join and send, by interface.
- IP is **IPv4 only** in v1 and multicast is sent with **TTL 1** (C-100).
- Epic criterion 021-AC3: with Wi-Fi up and default-routed, every multicast and unicast packet leaves by the medianet interface (verified by capture on both interfaces), as IPv4 with multicast TTL 1. On-hardware verification is 021-21; this ticket provides the mechanism and unit tests.
- All socket options of 021-03 (DF, `IP_TOS` per role, TOS receive) apply to every socket opened here.
## Work
- `open_media` / `open_discovery` take the configured medianet interface and:
  - bind the socket to that interface (interface-scoped bound socket option);
  - set the multicast outgoing interface explicitly;
  - set the multicast TTL to 1;
  - join multicast groups on that interface index only;
  - apply 021-03's socket builder.
- Refuse to open (named error) if the configured interface does not exist or has no IPv4 address.
- Unit tests using the loopback interface plus a second interface where available.
## Acceptance criteria
1. Every socket returned by `open_media` and `open_discovery` has the interface binding and the multicast outgoing interface set to the medianet interface index (checked by `getsockopt`).
2. Multicast joins are made on the medianet interface index only (unit test inspects membership or join call arguments through a seam).
3. Opening with a nonexistent interface, or an interface with no IPv4 address, returns a named error, not a socket bound to the default route.
4. Loopback multicast send/receive round-trip works through the pinned sockets.
5. Every returned socket reads back multicast TTL 1 (C-100).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
## Out of scope
`iface_stats` (021-06). Packet capture on real hardware with Wi-Fi up (021-21). Receive timestamps (021-05). Discovery-chatter suppression and firewall (epic 028 commissioning checklist).
