# 004-17 — Election message layouts
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-16
## Context
Modules agree on one clock holder through an election on the discovery and control layer (UDP over IPv4, multicast with TTL 1, and unicast, C-100; DSCP CS5 (40), C-13). `wire.md` must contain a field table for each election message. Epic 005 builds the state machine on these definitions.

Facts `[decision]`:
- The hello already carries election epoch, believed holder, and capability bits including *holder-eligible* (004-16).
- C-213: a holder-eligible module may take over as holder if its time is usable: Locked, or in holdover with its alignment claim not lapsed (C-65, C-169: about 25 s into holdover, an unknown E taken as 2 × C-62). Such a candidate takes over with frequency continuity and the timeline epoch is kept.
- C-113: only when no candidate has usable time (C-213), the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch. Election messages must therefore make each candidate's usable time (lock state including holdover, and whether its alignment claim has lapsed), holder eligibility and module id visible, and the winner's timeline epoch, kept or bumped, must reach followers.
- Election timers (behaviour, not fields): listen-before-claim 1.5 s (C-32), election window 400 ms (C-33), election on reply silence 1.5 s (C-37).
- C-11: module id 64 bits, instance nonce 64 bits, domain 16 bits. C-12 with C-115: authentication reservation (scheme 8 + counter 64 + tag 128 bits) in every control message, scheme and tag zero in v1, counter live. C-17: each control message ≤ 1 252 bytes. C-34: at most 64 active modules.
- Binary format, fixed maximum sizes.
- No constant or design rule names the election message set. This ticket proposes it (open item owned by epic 004, confirmed by epic 005; falsifier: epic 005's election state machine needs a message or field the set lacks).

## Work
1. Enumerate the election messages (e.g. candidacy/claim, holder announcement, yield) needed beyond the hello; propose the minimum set and record it as a proposal to be confirmed by epic 005's owner.
2. Field table for each: common control header from 004-16 (magic, version, domain, message type, authentication reservation), module id, instance nonce, election epoch, time state sufficient to decide usable time (Locked, holdover with alignment claim held, holdover with alignment claim lapsed, or not locked; C-213), holder eligibility, believed holder, and the timeline epoch announced by a winner, kept when it has usable time (C-213) and bumped otherwise (C-113).
3. State maximum size of each (≤ 1 252 bytes).
4. Validity rules with outcomes: as for any control message (too short, bad magic, version, foreign domain, oversize, reserved non-zero) plus election-specific ones (election epoch out of range, sender not holder-eligible claiming holdership, time-state value invalid).
5. Add tables to the machine-readable file.

## Acceptance criteria
1. Each election message has a type code, field table and maximum size ≤ 1 252 bytes.
2. Each carries the C-12 reservation with v1 values per C-115.
3. The fields needed to apply C-213 and C-113 (usable time: Locked or holdover with the alignment claim not lapsed; holder eligibility; module id; the winner's timeline epoch, kept or bumped) are present.
4. At least 6 invalid forms per message with outcomes.
5. The proposed message set is recorded as an open item with epic 005 named as confirming owner and a falsifier.
6. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Election algorithm and timers (epic 005).
- Hello (004-16), admission (004-18).
