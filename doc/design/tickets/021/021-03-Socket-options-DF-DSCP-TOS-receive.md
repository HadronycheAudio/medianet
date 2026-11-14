# 021-03 — Socket options: don't-fragment, DSCP, TOS receive
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-01, epic 016
## Context
Part of `net.rs`, the macOS `OsNet` / `DatagramSocket` implementation. Rules that apply unchanged on macOS:

- IP is **IPv4 only** in v1; multicast is sent with TTL 1 (C-100). All options below are IPv4 options.
- Don't-fragment on **every** packet; path-MTU fallback 1 280 bytes, which leaves 1 252 bytes of UDP payload (C-06, C-100). The path-MTU probe pair is sent at the intended media packet size, not limited to 1 252 bytes (C-116); a default media packet is about 1 320 bytes on the wire (C-20).
- DSCP: sync CS6 (48), media and one-way probes EF (46), control CS5 (40) (C-13).
- macOS mechanisms [memory]: don't-fragment uses a dedicated IPv4 socket option, **not** Linux's path-MTU-discovery option; DSCP is set with `IP_TOS` on the socket and verified by the receiving peer (receiver report). Receive of the TOS byte is required so this host can verify peers and publish in its own regime stamp the observed DSCP of each connection (C-47) and of each peer's hellos (C-152).
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
## Work
- In `net.rs`, a socket-builder helper applied to every `DatagramSocket` created by `MacNet`: create IPv4 sockets only; set the macOS don't-fragment option; set `IP_TOS` = DSCP << 2 by socket role (sync 48, media/probe 46, control 40); enable reception of the TOS byte as ancillary data.
- Parse the received TOS ancillary data into the DSCP value exposed by the `DatagramSocket` receive API of epic 016.
- Fuzz target for the ancillary-data (cmsg) parser, wired into the per-change and nightly fuzz jobs with a kept corpus.
## Acceptance criteria
1. Unit test (loopback, unprivileged): a socket built for each role reports back via `getsockopt` the TOS values 0xC0 (48), 0xB8 (46), 0xA0 (40).
2. Loopback send/receive: the receiver reads the sender's DSCP correctly for all three roles.
3. A send larger than the path MTU fails with an error instead of fragmenting (loopback with a lowered MTU; if that needs privilege, the test runs in the privileged nightly job, C-146); a send at the media packet size on a 1 500-byte MTU succeeds with DF set.
4. The cmsg parser fuzz target runs 60 s on every change and 1 hour nightly without a crash, and its corpus is stored (C-95).
5. Every socket the builder creates is `AF_INET`; a request for another family is refused with a named error.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Interface pinning and multicast TTL (021-04). Receive timestamps (021-05). On-the-wire verification of DF and peer-verified DSCP (021-21). The path-MTU probe logic itself (core).
