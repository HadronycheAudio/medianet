# 015a-19 — Per-connection stream-health counters in the stamp (C-47, C-159, C-160, C-157, C-182)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-13, 015a-18
## Context
- Third-party observability (C-47): producer and consumer each publish per-connection summary counters in their own regime stamp. Receiver reports are not multicast and there is no query message beyond the stamp request (C-18).
- The full per-connection counter set (C-159, completes C-47): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead. A slice on another timeline epoch has its own counter (C-135). Demoted is the health state of C-109.
- Lateness percentiles in the stamp (C-160): 50th, 99th, 99.9th and the maximum.
- Window of the lateness figures (C-182): the percentiles, the reclaimable latency and the cannot-meet-A alarm (C-174) use a rolling 60 s window; the maximum since the connection went live is reported beside them. The windowed statistics are core's (epic 009). A cannot-meet-A alarm raised because A was clamped to the buffer memory (C-211) stays raised until a later change of A that fits (C-216); that case is published by 015a-24.
- Reclaimable latency (C-157): offset − (current maximum lateness + headroom), when positive; published in the stamp's alignment group.
- Late versus missing (C-108): at the deadline an absent slice is counted missing; if it arrives later while still within the ring span it is reclassified as late (missing −1, late +1) and discarded. The classification is core's (epic 009).
- Interrupted and dead are defined by C-38, C-102, C-39 and C-111 and detected on the data thread (015a-13).
- Consumer counters, lateness statistics, offset and headroom come from the core alignment ring and reassembler (epic 009); producer counters come from the receiver reports it receives (epic 014). The observed DSCP is handled in 015a-14.
- **L-8:** counters reach housekeeping only through the lock-free counter board (015a-18).
## Work
1. Consumer data thread: per connection, publish every C-159 counter (received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead) to the board, applying reclassifications from core.
2. Consumer data thread: per connection, publish the lateness 50th, 99th, 99.9th percentiles and maximum over the rolling 60 s window, the maximum since the connection went live beside them (C-160, C-182), and the reclaimable latency when positive (C-157, from the windowed maximum), from core's alignment state.
3. Producer data thread: per connection, publish the counters the consumer's receiver report carries.
4. Document each stamp field (name, unit, source).
## Acceptance criteria
1. On loopback with injected loss and delay, the consumer's stamp counters for the connection equal the core ring's counts.
2. A slice that arrives late within the ring span moves exactly one count from missing to late.
3. The producer's stamp shows, within one report period plus one stamp refresh, the counters the consumer reported.
4. Interrupted events and the dead state appear in both stamps after a producer stop.
5. Each C-159 counter is exercised by an injected case (other-epoch slices, a too-early slice, a duplicate, an incomplete multi-packet reassembly, a demoted round from core) and equals core's count.
6. The stamp carries the 50th, 99th, 99.9th percentiles and the maximum of lateness over the rolling 60 s window, matching core's figures for the connection (C-160, C-182), and the maximum since the connection went live beside them; reclaimable latency equals offset − (windowed maximum lateness + headroom) when positive and is absent otherwise (C-157).
7. No allocation and no lock on the data thread for publication.
8. A single large late slice raises the windowed maximum and lowers the reclaimable latency; more than 60 s later the windowed figures no longer include it, while the maximum since live still shows it (C-182).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — report decoding is core's (epic 014); "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — 015b-10.
## Out of scope
- Gap classification, lateness statistics and Demoted state accounting (C-109) (epic 009).
- Observed DSCP (015a-14).
