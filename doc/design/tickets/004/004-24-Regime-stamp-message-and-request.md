# 004-24 — Regime stamp message: sections, version and fetch request
**Epic:** 004-Wire-specification · **Effort:** M · **Depends on:** 004-16
## Context
Every module publishes a *regime stamp*: the declared configuration and health facts that peers and tools read to interpret its behaviour. The design settled how it travels `[decision]`:
- C-18: the regime stamp is a dedicated control message split into sections that each fit C-17 (at most 1 252 bytes of UDP payload). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number (field defined in 004-16).
- Control layer: UDP over IPv4, multicast with TTL 1, and unicast (C-100); DSCP CS5 (40) (C-13). Common control header and authentication reservation from 004-16 (C-12, C-115: scheme and tag zero, counter live).
- C-47: receiver reports are not multicast and there is no query message for stream health; the stamp request fetches a whole stamp, not a health query.
- C-34: at most 64 active modules, so a peer may hold up to 64 stamps; the stamp must have a fixed maximum size for a small device.
- Adding a section type is a minor change (004-04); other epics add section types as they stamp new state. The known section types are laid out in 004-25 and 004-28.
- C-144: the hosted backend (epic 015) sends the regime stamp message; this ticket lays out its bytes only.

## Work
1. Section layout in `wire.md`: common control header, stamp version number (same width and comparison rule as the hello field, 004-16), section index, section count, section type, section length, and a typed body. State how a receiver knows it holds a complete stamp of one version.
2. Fix the maximum section count and hence the maximum stamp size; state the per-peer storage a receiver needs.
3. Stamp request message (unicast): fields identifying the requested module and, if needed, a version the requester already holds. Specify how the stamp is returned (unicast to the requester or multicast) and tag the choice `[decision]`.
4. Sender rules: multicast every section on any regime change and every 10 s (C-18); bump the version on any change.
5. Receiver rules: skip unknown section types of the same major version by their length (004-04); discard an incomplete stamp when a newer version arrives.
6. Validity rules with outcomes: generic control violations, section index ≥ section count, section count above maximum, section count differing between sections of one version, section length overrun (`BadLength`), duplicate section, oversize section (`OversizeControl`).
7. Add the tables to the machine-readable file using the 004-02 "section of" attribute.

## Acceptance criteria
1. Every section type and the request have a maximum size ≤ 1 252 bytes (C-17, epic AC-3).
2. The maximum stamp size and per-peer storage are stated as numbers.
3. The multicast-on-change, 10 s refresh and unicast fetch of C-18 appear as sender and receiver rules.
4. At least 7 invalid forms with outcomes.
5. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The content of each section type (004-25, 004-28).
- Implementing and sending the message (epics 013 and 015, C-144).
- Which further state other epics stamp (each such epic adds its section type as a minor change).
