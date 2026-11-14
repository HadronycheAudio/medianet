# 007a-09 — Valid vectors: regime stamp message and request
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** S · **Depends on:** 007a-05, epic 004

## Context
- Epic criterion 007-AC1 requires valid vectors in both directions for every message type of the wire specification.
- The regime stamp travels in a dedicated control message split into sections that each fit the control message bound: at most 1 252 bytes of UDP payload (C-17, C-18). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number (C-18).
- Content the stamp is known to carry, with field layouts from the wire tables of epic 004 `[decision]`:
  - per-connection counters published by producer and consumer: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-47 completed by C-159; other-epoch is its own counter, C-135), and the observed DSCP (C-47);
  - lateness percentiles: 50th, 99th, 99.9th and the maximum (C-160);
  - reclaimable latency, offset − (current maximum lateness + headroom) when positive, in the stamp's alignment group (C-157);
  - the percentiles and reclaimable latency are taken over a rolling 60 s window, and the maximum since the connection went live is reported beside them (C-182);
  - peer DSCP verification: the holder reports the DSCP observed on each follower's sync requests, and every module reports the DSCP observed on each peer's hellos (C-152);
  - the holder's per-follower count of sync requests dropped for exceeding 200 per second (C-204, C-112);
  - the calibration factor of the target (time per abstract operation) (C-106);
  - the commissioning window while it is open, as announced by the tool with its expiry time (C-107, C-153);
  - multi-packet slicing when used, which is an explicit, stamped regime (C-05).
  - capability states, each one of four values: present, absent, unverified, not requested (C-129), among them control isolation, which is present on hosted platforms by construction and absent or unverified on a device unless the implementer shows a bound (C-161); strict mode requires real-time scheduling on the data and clock threads and locked memory, CPU idle-state restriction stays best-effort and is stamped (C-127), and an unverified capability counts as absent for strict mode (C-128);
  - the receive-timestamp source, one of kernel, userspace, mac-hardware, interrupt, each reported with one of the four capability states (C-131).
- Like every control message, the stamp sections and the request carry the authentication reservation with scheme 0 and a zero tag, and a live liveness counter (C-12, C-115).
- Epic 007 adds no stamp field; it only encodes the fields epic 004 defines.

## Work
1. Extend `gen` to emit minimal, typical and maximal vectors for the regime stamp section message and for the unicast stamp request.
2. Emit at least one complete stamp that needs 2 or more sections, with the expected reassembled stamp in the `.json`.
3. Emit a stamp of a module carrying the largest per-connection counter set the tables allow, split so that no section exceeds 1 252 bytes.
4. Commit the files to `vectors/wire/valid/`.

## Acceptance criteria
1. The stamp section message and the stamp request each have at least 3 valid vectors (minimal, typical, maximal).
2. No stamp section vector exceeds 1 252 bytes of UDP payload.
3. At least 1 multi-section stamp exists, and its sections decode and reassemble to the recorded stamp.
4. At least 1 typical stamp carries all 11 per-connection counters of C-159 and the observed DSCP for one connection, the 4 lateness figures of C-160, the maximum since the connection went live (C-182) and a reclaimable latency (C-157); at least 1 holder stamp carries a sync-request DSCP per follower and a dropped-sync-request count per follower, non-zero for at least one (C-204), and at least 1 stamp carries a hello DSCP per peer (C-152).
5. Across the stamp vectors, every capability state of C-129 and every receive-timestamp source of C-131 appears at least once.
6. All files carry `spec_version`, pass the 007a-02 schema test and are reproduced byte for byte by `gen`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Every parser touched is fuzzed
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Regime stamp fields touched are complete and documented": this ticket encodes stamp fields defined by epic 004 and adds or changes none.

## Out of scope
- Distribution timing: multicast on change, 10 s refresh, answer to a request (007b-11).
- Over-size sections and other invalid stamp encodings (007a-11).
- Defining stamp fields (epic 004).
