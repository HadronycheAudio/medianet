# 015a-14 — Drop accounting and peer-verified DSCP in stamp and report
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-07, 015a-13, 015a-18, epic 013
## Context
- Per-socket overflow counter: `SO_RXQ_OVFL` returns a cumulative drop count in a control message `[kernel-doc]`, delivered as `RxMeta::dropped_total`. It goes into the regime stamp (format: epic 013) and the receiver report (epic 014).
- Interface drop counters `rx_dropped` and `tx_dropped` (015a-07) are sampled each time a stamp is built — on every regime change and at every 10 s refresh (C-18) — and stamped. Housekeeping samples them; it is not a real-time thread.
- DSCP classes (C-13): sync CS6 (48); media and one-way probes EF (46); control (hello, election, admission, receiver reports, regime stamp) CS5 (40). Marking is verified by the receiving peer, which reads the received TOS byte (`IP_RECVTOS`, `RxMeta::tos`).
- Where observations go (C-47): the receiver report of a bundle carries the media DSCP observed on that bundle; each observer publishes, per connection, the DSCP it observed for every class in its own regime stamp. Peer verification of sync and control DSCP (C-152): the holder reports the DSCP observed on each follower's sync requests (observed by its clock thread, which answers them, C-188, 015a-10), and every module reports the DSCP observed on each peer's hellos (observed by housekeeping). Media and probe DSCP are observed by the data thread. All reach the stamp through the lock-free counter board (015a-18). A class not yet observed is "unverified".
- AC-37: an induced socket overflow appears in the stamp's drop counters and in the receiver report.
- AC-34 (015a-A2): the receiver report of a bundle shows media 46; the observer's stamp shows, per connection, sync 48, media and probes 46, control 40, or "unverified".
## Work
1. On each receive, keep the last `dropped_total` per socket; compute deltas per report period; write the delta into the receiver report; publish the cumulative value to the counter board.
2. Housekeeping: sample interface drop counters when a stamp is built and put them in the stamp.
3. Data thread: record the observed media and probe DSCP (`tos >> 2`) per bundle; put the media DSCP in the receiver report; publish per connection to the board. Housekeeping records the DSCP observed on each peer's hellos, for up to 63 peers (C-217), and publishes it per peer (C-152); the holder's sync-request DSCP per follower comes from 015a-10.
4. Publish the effective socket buffer and the clamp flag from 015a-04 per bundle.
5. Document every regime stamp field touched (name, unit, source, evidence tag).
## Acceptance criteria
1. AC-37: with an induced overflow (small `SO_RCVBUF`, burst), the stamp's socket drop counter and the next receiver report both show a non-zero count equal to the `SO_RXQ_OVFL` delta.
2. AC-34: on loopback, the receiver report shows 46; the consumer's stamp shows, for the producer connection, 46 for media and probes and 40 for the producer's hellos; the holder's stamp shows 48 for each follower's sync requests (C-152).
3. When no TOS has been observed for a class, the stamp field for that class and connection reads "unverified".
4. Interface drop counters appear in the stamp and match sysfs at sampling time.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser (sysfs parser fuzzed in 015a-07); "Design documents updated" — no behaviour differs from the epic; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — measured in 015b-10.
## Out of scope
- Stamp and report wire formats (epics 004, 013, 014).
- Stamp transport (015a-18); per-connection gap-class counters (015a-19).
