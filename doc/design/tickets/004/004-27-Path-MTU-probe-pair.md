# 004-27 — Path-MTU probe pair
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-18
## Context
The admission handshake (004-18) includes a path-MTU probe pair, so a consumer and producer learn whether the default one-packet slice (C-05) crosses the path. Settled rulings `[decision]`:
- C-116: the probe pair is exempt from the control message bound C-17; the probe is sent at the intended media packet size.
- C-100: IPv4 only; rule 11: every packet is sent with don't-fragment set and IP fragmentation is forbidden.
- C-06: the path-MTU fallback is 1 280 bytes, 1 252 bytes of UDP payload.
- C-173: the refusal `PathMtuTooSmall` applies when the probe at the 1 280-byte fallback also fails.
- C-137: a default slice fits 1 252 bytes of UDP payload, so a path that passes the fallback probe carries the default slice.
- Control layer, DSCP CS5 (40) (C-13); common control header and authentication reservation from 004-16 (C-12, C-115).

## Work
1. Field tables for the probe and its reply: common control header, probe identifier, intended media packet size, padding (content and required value).
2. State the probe size rule (the intended media packet size, C-116), the DF requirement, how the reply identifies the probe, the reply's size, and the outcome when no reply arrives: probe again at the 1 280-byte fallback (C-06); if that probe also fails, refuse with `PathMtuTooSmall` (C-173).
3. Mark the probe as exempt in the machine-readable file so the 004-02 checker accepts its size.
4. Validity rules with outcomes: probe size different from its declared intended size, padding non-zero (`ReservedNonZero`), reply not matching any outstanding probe, generic control violations.

## Acceptance criteria
1. The probe pair is specified with sizes, DF requirement and matching rule, and the fallback sequence ends in `PathMtuTooSmall` when the 1 280-byte probe fails (C-173).
2. The exemption from C-17 cites C-116 and is the only one in the control layer.
3. At least 4 invalid forms with outcomes.
4. Passes the 004-02 checker with the exemption attribute.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The admission messages (004-18) and the manifest (004-26).
- Probe retry timing (state machine, epic 005).
