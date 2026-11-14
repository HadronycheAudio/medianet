# 021-21 — Interface pinning and socket options verified on the wire
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-04, 021-15, 021-16
## Context
Epic criterion 021-AC3: with Wi-Fi up and default-routed, every multicast and unicast packet leaves by the medianet interface (verified by capture on both interfaces); every packet is IPv4 and multicast uses TTL 1 (C-100).

- Porting trap [memory]: multicast follows the default route unless every socket is pinned to the medianet interface.
- Platform mapping checks [memory]: don't-fragment is checked by "capture on the wire"; DSCP via `IP_TOS` is "verified by the receiving peer" (receiver report).
- Peer verification of sync and control DSCP [decision, C-152]: the holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos.
- DSCP sync 48, media and one-way probes 46, control 40 (C-13); don't-fragment on every packet (C-06).
- Control messages, including regime stamp sections, are at most 1 252 bytes of UDP payload (C-17, C-18); the path-MTU probe pair is exempt and is sent at the media packet size (C-116). A default media packet is about 1 320 bytes on the wire (C-20).
## Work
- Manual hardware procedure, documented: laptop with Wi-Fi up as default route plus a wired medianet NIC; run sink and capture for 10 minutes; capture on both interfaces.
- Analyse captures: count medianet-runtime packets on Wi-Fi; check IP version, TTL, DF bit, DSCP and size of each packet on the medianet interface; check peer receiver reports for DSCP verification.
## Acceptance criteria
1. 0 runtime packets (multicast or unicast) on the Wi-Fi interface over 10 minutes.
2. 100 % of runtime packets on the medianet interface are IPv4, have DF set and the DSCP of their role (48/46/40); every multicast packet has TTL 1.
3. Every control packet other than the path-MTU probe pair carries at most 1 252 bytes of UDP payload.
4. The Linux peer's receiver report marks DSCP verified; the edge stamp shows DSCP *peer-verified*. The holder's stamp shows DSCP 48 observed on the edge's sync requests, and the Linux peer's stamp shows DSCP 40 observed on the edge's hellos (C-152).
5. The don't-fragment and DSCP `[memory]` rows are recorded as `[measured]` (or corrected) for 021-25.
## Definition of done
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Discovery-chatter suppression and firewall checklist (epic 028).
