# 007b-11 — Regime stamp distribution scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007a-04, 007a-09, epic 005

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- Regime stamp distribution (C-18) `[decision]`: the stamp is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello (every 1 s, C-30) carries only the stamp version number.
- Third-party observability of stream health is only through the stamp: producer and consumer each publish per-connection counters in their own stamp, received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-47 completed by C-159), with the observed DSCP (C-47) and the lateness percentiles 50th, 99th, 99.9th and maximum (C-160); receiver reports are not multicast and there is no query message (C-47).
- Window of the lateness figures (C-182): the percentiles (C-160) and the reclaimable latency (C-157: offset − (current maximum lateness + headroom), when positive, published in the stamp's alignment group) use a rolling 60 s window; the maximum since the connection went live is reported beside them.
- Peer DSCP verification (C-152): the holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos.
- The stamp message is defined by epic 013 and sent by the hosted backend (epic 015, C-144); these vectors judge what any implementation sends, whoever builds it.
- Which control state machine owns stamp distribution is still owned by epic 005; the `machine` field takes the name epic 005 gives.
- Message encodings are in 007a-09.

## Work
1. Write scenarios in `vectors/control/stamp-*.json`:
   - steady state: one multicast stamp every 10 s; none at 10 s − 1 ns after the previous one;
   - regime change: an immediate multicast of the new stamp, and the hello stamp version changes from the next hello on;
   - unicast request: an immediate unicast answer with the current stamp, and no extra multicast;
   - counter publication: after a scripted sequence of received, late, missing, marked-gap, other-epoch, too-early and duplicate slices, an incomplete reassembly, a demoted round, an interrupted event and a dead connection, the next stamp of producer and of consumer carries the matching counts and the lateness percentiles of the scripted lateness values;
   - lateness window: a scripted lateness series longer than 60 s whose largest value falls more than 60 s before the stamp: the percentiles, the window maximum and the reclaimable latency are computed over the last 60 s only, and the maximum since live still carries the older value (C-182, C-157); the scenario states the offset and the headroom inputs it uses;
   - DSCP verification: followers send sync requests and peers send hellos with scripted DSCP values; the holder's next stamp carries the sync-request DSCP per follower and every module's stamp the hello DSCP per peer.
2. Add one scenario per remaining transition row of stamp distribution in the tables.

## Acceptance criteria
1. The 10 s refresh is tested at 10 s and 10 s − 1 ns.
2. A regime change produces a multicast stamp within the same step and a changed version in the next hello.
3. A unicast request produces exactly one unicast answer and no multicast.
4. The counter scenario expects each of the 11 counters of C-159 at its scripted value in both the producer's and the consumer's stamp, and the 4 lateness figures of C-160.
5. No scenario expects a receiver report on a multicast address or a stream-health query message.
6. All files carry `spec_version`.
7. The DSCP scenario expects the observed value per follower in the holder's stamp and per peer in every module's stamp (C-152).
8. The lateness-window scenario expects the percentiles, window maximum and reclaimable latency over the last 60 s only, and the maximum since live beside them (C-182).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

Not applicable (C-125): "Regime stamp fields touched are complete and documented": the scenarios read stamp fields defined by epic 004 and add or change none.

## Out of scope
- Encoding of the stamp and the request (007a-09).
- Connection state transitions behind the counters (007b-07, 007b-08).
