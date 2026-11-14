# 026b-11 — Per-connection counters in stamp
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026a-08, 026a-11, 026b-02

## Context

Third-party observability of stream health (C-47): **producer and consumer each publish per-connection summary counters in their own regime stamp**. The full list (C-159, completing C-47): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead; plus the observed DSCP (C-47). Receiver reports are not multicast and there is no query message. The device is a consumer, so its stamp must carry these counters for every input.

Counting rules the counters follow:
- At the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified late (missing −1, late +1) and discarded (C-108).
- Interrupted after 8 consecutive slices absent at the deadline; a marked gap or a slice on another timeline epoch resets the count (C-38, C-102).
- Dead 2 s after the last slice with content or a marked gap (C-39, C-111). A slice on another timeline epoch restarts the dead clock, is not a miss, and has its own counter (C-135).
- An input persistently late at the ratchet ceiling enters Demoted and is counted separately (C-109).

The stamp is sectioned, multicast on change and every 10 s, unicast on request (C-18). Observing the DSCP of received packets depends on the network stack (026b-02); when it cannot, the observed-DSCP field is stamped with the C-129 state (absent or unverified) instead of a value.

## Work

1. Per admitted input, keep the eleven C-159 counters (received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead) and the observed DSCP of media packets; fixed-size storage, no allocation.
2. Feed the counters from core's gap classification (026a-08); apply the C-108 reclassification.
3. Publish them in the device stamp (026a-11) using epic 013's per-connection fields; a counter change alone does not count as a regime change unless epic 013 says so, and is carried by the 10 s refresh.
4. Read received DSCP on the board if `rx_dscp_observation` is present (026b-02).

## Acceptance criteria

1. Host test: a scripted input with known losses, late arrivals within the ring span, marked gaps and an 8-slice outage yields exact counter values, including the missing-to-late reclassification (C-108) and one interrupted event.
2. Host test: a 2 s outage yields one dead event (C-111); a Demoted input is counted in demoted rounds (C-109); scripted other-epoch, too-early, duplicate and incomplete-reassembly packets each increment only their own counter (C-159, C-135).
3. Host test: the counters appear in the stamp decoded by the collector (026a-12) within one 10 s refresh.
4. On the board, the observed DSCP of an EF media stream reads 46 (C-13), or the field shows the C-129 state when the stack cannot read it.
5. Any new stamp parser path is fuzzed at the C-95 budget.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- Gap classification itself (epic 009, 026a-08).
- Lateness percentiles, reclaimable latency and peer DSCP observations (026b-12).
- Producer-side counters (the device produces no stream).
