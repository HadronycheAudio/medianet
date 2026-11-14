# 021-05 — Receive timestamps, kernel or userspace
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-02, 021-04, epic 001
## Context
Each received datagram needs a receive timestamp on the host clock H (mach absolute time, 021-02).

- Mechanism [memory]: kernel receive stamps on a monotonic basis via the socket option `SO_TIMESTAMP_MONOTONIC`; would be cleaner than Linux's wall-clock basis. Epic 001's receive-stamp probe decides whether it is available for UDP.
- If absent: read the host clock immediately after receive and mark the source *userspace*.
- The regime stamp records the receive-timestamp source as *kernel* or *userspace* (epic criterion 021-AC7, AC-31; filled by 021-20). These are two of the four source values of C-131 (mac-hardware and interrupt are embedded only), and each is reported with one of the capability states present, absent, unverified, not requested (C-129).
- Falsifier of the open sub-question (owner epic 001): if kernel stamps are absent, the clock bound and the 100 µs E target rise.
- The backend that builds sync exchanges from these stamps is the home of wall-clock step detection (C-121); that is 021-28. This ticket must expose, with every stamp, which basis it is on.
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
## Work
- In `net.rs`: if epic 001 confirmed kernel monotonic stamps, enable `SO_TIMESTAMP_MONOTONIC` and parse the ancillary timestamp into H nanoseconds (same timebase conversion as `MachClock`). Otherwise, stamp with `MachClock::now_ns` immediately after `recvmsg` returns.
- Keep a runtime fallback: if the option fails to set or a datagram arrives without the control message, use userspace stamping for that socket and record the source as *userspace*.
- Expose the active source (`Kernel` | `Userspace`) per datagram (for 021-28) and per socket (for the regime stamp), with its C-129 state: *kernel* is *present* only when epic 001 measured it and the control message arrives, *unverified* while epic 001's result is missing.
- Extend the cmsg fuzz target to the timestamp control message.
## Acceptance criteria
1. Each received datagram carries a timestamp on the H basis; on loopback the stamp is within 1 ms before the userspace read time.
2. The reported source matches what was actually used; a forced setsockopt failure (test seam) yields *userspace*.
3. The fuzz target covering the timestamp cmsg runs 60 s on every change and 1 hour nightly without a crash, corpus kept (C-95).
4. The epic 001 result used is cited with its `[measured]` tag in the module doc.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Building exchanges and step detection (021-28). Replacing the epic's receive-timestamp claim with the measured statement (021-25). Regime stamp assembly (021-20).
