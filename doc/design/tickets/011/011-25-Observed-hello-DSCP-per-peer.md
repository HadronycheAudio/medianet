# 011-25 — Observed hello DSCP per peer
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-06, 011-07

## Context
C-152 (decided): peer verification of sync and control DSCP. The holder reports in its stamp the DSCP observed on each follower's sync requests; **every module reports the DSCP observed on each peer's hellos**. Hellos are control messages, sent with DSCP CS5 (40) per C-13; the marking itself is applied by the platform transport (epics 015, 026). Reporting what actually arrives lets a third party see a switch or host that re-marks control traffic.

Core is I/O-free: it cannot read the IP header. The platform reads the DSCP of each received hello from the socket and passes it to `Discovery::on_hello` (the `dscp: u8` parameter in the epic's signature, added in 011-07). Core stores it per peer and exposes it read-only; the stamp field that carries it is part of the regime stamp message defined by epic 013 and sent by the hosted backend, epic 015 (C-144).

## Work
1. In `Discovery::on_hello`, for an accepted hello (not rejected by 011-03/011-08), store the `dscp` argument in the peer's entry (field added to `Peer` in 011-06), overwriting the previous value.
2. Add `pub fn observed_dscp(&self, peer: ModuleId) -> Option<u8>`, returning `None` for an unknown or quarantined peer.
3. Document, next to `on_hello`, that the platform must pass the DSCP of the received datagram (the 6 high bits of the IPv4 TOS byte).

## Acceptance criteria
1. A peer whose hellos arrive with DSCP 40 reports `observed_dscp(peer) == Some(40)`; when a later hello arrives with DSCP 0, it reports `Some(0)`.
2. A rejected hello (foreign domain, domain 0, malformed, wrong version) does not change any stored value.
3. An unknown id returns `None`; a peer removed by expiry returns `None`.
4. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no parser; the value arrives as an argument); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (core exports the value; the stamp field is epic 013's and is documented by 011-22); "Design documents updated" (follows C-152); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Reading the DSCP from the socket: platform epics 015, 026.
- The stamp field layout and sending the stamp: epics 013 and 015 (C-144).
- The holder's report of DSCP on followers' sync requests: the clock side, epic 008.
