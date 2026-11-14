# 004-20 — Receiver report and one-way probe layouts
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-03, 004-04, 004-05
## Context
The measurement layer runs over UDP unicast (IPv4, C-100) on the reverse port given in the admission handshake. Reports use DSCP CS5 (40), probes EF (46) (C-13). The v1 message set is exactly two `[decision]`:

| Message | Sender | Rate | Content |
|---|---|---|---|
| Receiver report | consumer's data thread | one per bundle per 100 ms (C-40) | per-member counters of C-159; socket and interface drop counters; observed DSCP; the consumer's current A (C-198) |
| One-way probe request / reply | either end | on demand | timestamps closing a round trip |

Settled rulings `[decision]`:
- C-47, C-159: receiver reports are not multicast and there is no query message. Third parties see stream health in the regime stamps, where producer and consumer each publish per-connection counters: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), plus the observed DSCP (004-25). The producer builds its counters from receiver reports, so the report carries every C-159 counter. A slice on another timeline epoch has its own counter (C-135).
- C-108: at the deadline an absent slice is counted missing; if it arrives later while still within the ring span it is reclassified as late (missing −1, late +1). Counters in the report are therefore not monotonic per class; the report must say how a reclassification is reflected.
- C-102: only missing or late slices count toward the interrupted threshold (C-38: 8 consecutive missed slices); a marked gap or a slice on another timeline epoch resets the count. The interrupted count is a count of interrupted events.
- C-198: the receiver report carries the consumer's current A, its own or the one it inherited (C-142). A change of A propagates upstream one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186, behaviour, not a field). This is the only carrier of a changed A after admission.
- C-41: a producer times out after 2 s without receiver reports (behaviour, not a field).
- The Demoted health state (C-109: an input persistently late at the ratchet ceiling) is carried as the demoted-rounds counter of C-159, in the report and in the regime stamp; a third party tells a demoted input from a gapped one by that counter.
- C-12 names control messages only. Whether measurement messages carry the authentication reservation is not settled; this ticket decides it (owner epic 004; falsifier: a later scheme cannot be added without a major change).
- The probe's name says "one-way" while its reply closes a round trip. No constant reconciles this; this ticket states what the probe measures (owner epic 004; falsifier: a measurement consumer needs a one-way delay the probe's timestamps cannot yield).

Other facts: C-08 at most 32 members per bundle; forward media packets carry no per-receiver state, round-trip closure uses this measurement path (rule 8); domain 16 bits (C-11).

## Work
1. Receiver report field table: header (magic, version, domain, type), bundle identity, member count (≤ 32), per member: the eleven C-159 counters (width, wrap or saturate rule, treatment of the C-108 reclassification); socket drop and interface drop counters; observed DSCP (6 bits); the consumer's current A (C-198), with the same width and unit as the downstream A field of 004-18 (C-142). State the maximum size with 32 members.
2. One-way probe request/reply field table: identifiers and timestamps closing a round trip (width and unit consistent with 004-08/004-19). State which delays the timestamps yield (one-way in each direction under network time, round trip, or both) and tag `[decision]`.
3. Decide whether both messages carry the C-12 reservation (v1 values per C-115 if so).
4. Validity rules with outcomes: too short, version, foreign domain, member count > 32, length inconsistent with member count, unknown bundle, DSCP > 63, A out of its valid range, reply without matching fields.
5. Add tables to the machine-readable file.

## Acceptance criteria
1. Exactly two v1 measurement message kinds are defined (report; probe request/reply), and the report is unicast only (C-47).
2. Receiver report maximum size with 32 members is stated in bytes.
3. Every content item of the table above has a field, with the counters matching C-159 and the late/missing rule matching C-108; the consumer's current A (C-198) has a field with the width and unit of the C-142 field in 004-18.
4. The probe's measured quantities and the authentication-reservation choice are stated with falsifiers; Demoted is carried by the demoted-rounds counter (C-159).
5. At least 6 invalid forms with outcomes across both.
6. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Media measurement extension (004-10).
- Reverse port negotiation (004-18).
- Per-connection counters in the regime stamp (004-25).
