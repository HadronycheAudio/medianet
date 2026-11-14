# 014-19 — Linux drop counter and DSCP tests (AC-37, AC-34)
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-03, 014-04, epic 013
## Context
- Drop accounting `[decision]`: the kernel's per-socket overflow counter, cumulative, read on every receive (`SO_RXQ_OVFL`, Linux, since 2.6.33) `[kernel-doc]`; interface drop counters as deltas. macOS and Windows have no per-socket counter `[memory]`: there the field is absent and loss is inferred from position gaps only.
- Observed DSCP: the receiver reads the received IPv4 type-of-service byte (IPv4 only in v1, C-100) and reports it, because marking is verified by the peer and never self-reported. Expected: sync 48, media and probes 46, control 40 (C-13).
- Capability states (C-129): whether each drop counter and the type-of-service byte can be read is published in the regime stamp as present, absent, unverified or not requested. A report field left absent corresponds to a stamp state other than present.
- System AC-37: an induced socket overflow appears in the report's drop counters on Linux. System AC-34: the report carries the observed DSCP of received media; a sender marking 46 is reported as 46.
- Epic 013 defines where drop counters, observed DSCP and capability states enter the regime stamp.
- The socket tests live in the `hadronyche-tools` crate (C-133), not in `hadronyche-core`, which stays `no_std`.

## Work
1. Add Linux-only, unprivileged, std integration tests in `hadronyche-tools` (C-133) using UDP over IPv4 loopback.
2. AC-37: set a small `SO_RCVBUF` on the receiver, enable `SO_RXQ_OVFL`, burst packets without reading, then receive; pass the cumulative counter as `DropCounters::socket_overflow` into `encode_report`, parse, and check.
3. AC-34: sender sets the TOS byte for DSCP 46 (`46 << 2`); receiver enables `IP_RECVTOS`, reads the TOS byte, passes `tos >> 2` as `observed_dscp`; parse and check. Repeat with 40 and 48.
4. Check that the regime-stamp fields from epic 013 for drop counters, DSCP and their capability states (C-129) are filled from these values and documented: present when read, absent when the platform has no counter.
5. Non-Linux: a unit test asserts that `socket_overflow: None` is encoded as absent and parses as `None`.

## Acceptance criteria
1. After an induced overflow, the parsed report's `drops().socket_overflow` is `Some(n)` with `n >= 1` (AC-37), and the stamp shows the socket counter as present.
2. A sender marking 46 is reported as `observed_dscp() == Some(46)`; 40 → 40; 48 → 48 (AC-34).
3. Both tests run unprivileged in CI on Linux.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Reading interface drop counters and whether they need privilege per platform (epic open sub-question 3, owned by tasks 015, 021, 022; falsifier: not readable, the field is absent there and stamped absent, C-129).
- Production socket code (epic 015).
- Peer verification of sync and control DSCP (the holder's report of followers' sync-request DSCP and each module's report of peers' hello DSCP, C-152): stamp fields outside the reverse path.
