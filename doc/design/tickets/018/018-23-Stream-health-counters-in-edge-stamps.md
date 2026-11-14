# 018-23 — Stream health counters in edge stamps
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-16, 018-18, 018-22, epic 013

## Context
- **C-47:** producer and consumer each publish **per-connection summary counters in their own regime stamp**, plus the observed DSCP. Receiver reports are not multicast and there is no query message.
- **C-159** completes the set: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead.
- `AlsaSink` is a consumer; its counters come from 018-22. `AlsaCapture` is a producer; its counters are per outgoing connection (one per stream, or per bundle member, with its session number, C-19).
- DSCP classes: media EF (46) (C-13). The observed DSCP is what the module sees on its media packets.
- **C-178:** loss figures in the stamp are ordinary production measurements, unqualified while a commissioning window is open (C-153).
- The stamp travels in sections of at most 1 252 bytes of UDP payload (C-17, C-18) and is refreshed every 10 s.

## Work
1. Add the C-159 counter set per connection to the sink and capture stamps, using the epic 013 format.
2. Fill the sink's counters from 018-22, including demoted rounds (C-109, C-159) and other-epoch (C-135) as separate counters.
3. Fill the capture's producer-side counters for each outgoing stream.
4. Record the observed DSCP per connection.
5. Flag the counters accumulated while a commissioning window is open as unqualified (C-178).
6. Document each counter (meaning, unit, source) in the stamp documentation.

## Acceptance criteria
1. **Epic criterion 018-A9 (stamp part):** a fake-backend sink with injected loss and late arrivals publishes all eleven C-159 counters and the observed DSCP, matching the injected pattern (including slices on another epoch and duplicates).
2. A 24-channel capture publishes one counter set per member stream (4 streams).
3. A stamp with 32 connections' counters fits in sections of at most 1 252 bytes each.
4. Round-trip through the epic 013 serializer is lossless.
5. Counters published while a commissioning window is open are flagged unqualified (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every parser touched is fuzzed

## Out of scope
- Receiver reports (not multicast, C-47).
- The stamp format and sectioning (epic 013); sending the stamp message (epic 015, C-144).
- Lateness percentiles (018-24).
