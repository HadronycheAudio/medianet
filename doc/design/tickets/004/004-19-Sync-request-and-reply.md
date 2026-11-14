# 004-19 — Sync request and reply layouts
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-03, 004-04, 004-05
## Context
Clock sync exchanges run between a follower and the clock holder over UDP unicast on IPv4 (C-100), DSCP CS6 (48) (C-13). Rules `[decision]`:
- Stateless request/reply: the reply echoes the request's own send timestamp, so a stale or duplicate reply is a valid exchange with an honestly large round trip.
- Every reply carries the **timeline epoch** (16 bits, C-10) and the holder's identity (module id 64 bits, C-11). The epoch in the reply changes on an election in which no candidate has usable time (C-113, C-213) and on a holder stall (C-136); a takeover by a candidate with usable time keeps it (C-213).
- C-152: the holder reports in its regime stamp the DSCP observed on each follower's sync requests (laid out in 004-28); the request carries no DSCP field.
- Requests and replies are the same size.
- Domain 16 bits, 0 = unconfigured (C-11). Foreign-domain packets are dropped and counted (rule 13).
- Request rate is behaviour, not a field: followers send 50 per second (C-36) and the holder accepts 10 to 200 per second (C-112). Requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's regime stamp (C-204); the request carries no field for this, and the counter is laid out in 004-28.
- Wall-clock step detection lives in the platform backend that builds each exchange from receive stamps (C-121); the wire carries the timestamps it needs, nothing more.
- Epic 001 supplies a non-blocking report on receive timestamps per OS; it affects only notes on timestamp fields.
- C-12 names control messages only. Whether sync messages carry the authentication reservation is not settled; this ticket decides it (open item owned by epic 004; falsifier: a later authentication scheme cannot be added to sync messages without a major change).

## Work
1. Field table for request and reply in `wire.md`: header (magic, version per 004-04; domain; message type), request send timestamp (echoed in reply), reply receive and send timestamps, timeline epoch, holder module id; padding in the request so both are equal size.
2. Timestamp width, unit and clock (consistent with or explicitly different from the slice-header send timestamp, 004-08).
3. Decide whether sync messages carry the C-12 authentication reservation (with v1 values per C-115 if they do) and tag the choice `[decision]`, with its falsifier.
4. Validity rules with outcomes: size not equal to the fixed size (`BadLength`), bad magic, version, foreign domain, request padding non-zero (`ReservedNonZero`), reply from a non-holder, reply with wrong epoch (`WrongEpoch`).
5. Leave a note slot for per-OS receive-timestamp remarks from epic 001; the receive-timestamp source (C-131) is declared in the regime stamp (004-25), not in sync messages.
6. Add tables to the machine-readable file.

## Acceptance criteria
1. Request and reply have identical byte size, stated as a number.
2. The reply contains the echoed request send timestamp, the timeline epoch (16 bits) and the holder module id (64 bits).
3. No field requires the holder to keep per-request state.
4. The authentication-reservation decision is stated with its falsifier.
5. At least 6 invalid forms with outcomes.
6. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Clock filtering, servo and step detection (other epics).
- Rate limiting of requests (behaviour, C-112, C-204); the per-follower drop counter in the holder's stamp (004-28).
