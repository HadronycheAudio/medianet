# 005-23 — Regime stamp coupling
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-06, 005-13, 005-15, epic 004

## Context
- Epic 005 specifies four pure control machines in `doc/design/spec/control.md` and `doc/design/spec/control.toml`. Its definition-of-done line "Regime stamp fields touched are complete and documented" applies to four couplings with the regime stamp, settled by the constants table:
  - **Transport (C-18):** the regime stamp is a dedicated control message in sections that each fit 1 252 bytes of UDP payload (C-17); it is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only a stamp version number. The discovery map (machine 1) records each peer's stamp version and fetches by unicast request when a hello names a newer version than the one held.
  - **Admission inputs:** admission (machine 3) reads from the peer's stamp the sink's E, which is numeric only for a commissioned sink, or "unknown" for an uncertified or self-monitored sink (C-103, C-140), and the target's calibration factor (time per abstract operation) used to convert the DSL cost bound (C-106).
  - **Stream health counters (C-47, completed by C-159):** producer and consumer each publish per-connection summary counters in their own regime stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead, plus the observed DSCP. Receiver reports are not multicast and there is no query message. Of these, the connection machine (machine 4) produces interrupted events (each entry into Interrupted), dead (entry into Dead) and demoted rounds (each entry into Demoted, C-109); the other counters and DSCP come from the data plane (late versus missing per C-108; other-epoch per C-135).
  - **Peer DSCP verification (C-152):** the holder (machine 2) reports the DSCP observed on each follower's sync requests, recorded on the clock thread where it answers them (C-188), and, per follower, the count of sync requests dropped beyond 200 per second (C-204); every module reports the DSCP observed on each peer's hellos (machine 1).
- The stamp layout itself belongs to epic 004; this ticket defines what the control machines read and contribute.

## Work
1. Write the "Regime stamp coupling" section of `control.md` covering the four couplings above, each with its C-id.
2. Add to machine `discovery` in `control.toml`: input "regime stamp message" (kind `message`), the stamp-version field of the hello as a guard, and output "regime stamp request" (unicast).
3. Add to machine `admission`: local input "peer regime stamp received" carrying E (or unknown) and the calibration factor.
4. Add to machines `connection_consumer` and `connection_producer` the C-159 counter names usable in the `counters` column (005-02), and list which counters the machine increments and which the data plane owns.
5. Record that machine `discovery` holds the observed hello DSCP per entry and machine `election` the observed sync-request DSCP per follower (C-152) and the per-follower dropped sync request count (C-204).
6. Send epic 004 the list of fields this specification reads or feeds, and record it in `control.md`.

## Acceptance criteria
1. The section names C-18, C-47, C-103, C-106, C-140, C-152, C-159 and C-204 and states each coupling in at most one paragraph.
2. `control.toml` declares the stamp message and request inputs and outputs for `discovery`, and the stamp input for `admission`; the checker of 005-03 passes.
3. The counter list has exactly the 11 C-159 counters plus the observed DSCP, each marked "connection machine" (interrupted events, dead, demoted rounds) or "data plane" (received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, observed DSCP).
4. No input is a receiver-report multicast or a stamp query beyond the C-18 unicast request.

## Definition of done
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The stamp's wire layout and sectioning (epic 004).
- The transition rows that use these inputs (005-07, 005-14, 005-16).
- Data-plane counting (other epics).
