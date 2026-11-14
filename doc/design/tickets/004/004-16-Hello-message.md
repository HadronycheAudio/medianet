# 004-16 — Hello message layout and 32-member size bound
**Epic:** 004-Wire-specification · **Effort:** M · **Depends on:** 004-03, 004-04, 004-05
## Context
The hello belongs to the discovery and control layer: UDP over IPv4 (C-100), multicast with TTL 1, and unicast; DSCP CS5 (40) (C-13). It is sent every 1 s (C-30). Control rules `[decision]`:
- Binary format, not text (the POC's text gossip `"HADRO/1"` is retired `[code]`). Fixed maximum sizes so a 64-entry peer map (C-34: at most 64 active modules per medianet) and every parser fit a small device.
- Hello carries: module id, instance nonce, domain, protocol version, name, capability bits (including *holder-eligible*), ports with content types, bundles (group id, member list, channel map), election epoch, believed holder, timeline epoch, transport role, alarm field, data-plane liveness counter, regime stamp version number, the authentication reservation.
- C-18: the regime stamp travels in its own sectioned control message (004-24); the hello carries only the stamp version number, so a peer knows when to fetch.
- The 64-bit message counter of the authentication reservation doubles as the liveness counter. C-115: in v1 the scheme is 0 and the tag is zero; the counter is live.
- C-11: module id 64 bits, instance nonce 64 bits, domain 16 bits (0 = unconfigured). C-10: timeline epoch 16 bits. C-12: authentication reservation = scheme 8 + counter 64 + tag 128 bits. C-08: at most 32 members per bundle, the controller member included (C-209: a source with controllers has at most 31 audio members). C-09: at most 256 streams per source.
- C-17: every control message is at most 1 252 bytes of UDP payload (one packet at the 1 280-byte fallback MTU over IPv4, C-100).
- Rule 12: the version is carried in the hello so a mismatch is detected before a connection exists; a mismatch is a named event.
- C-173: a hello with domain 0 is ignored and counted, not refused; `ForeignDomain` is for a different non-zero domain. A connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A (C-174) and control-degraded (C-139: housekeeping restarts stopped after 5 panics within 60 s); the alarm field must express each of these six.
- C-46: two hellos with the same module id and different instance nonces are a collision that third parties quarantine until only one nonce has been seen for 3 s (C-31). This is state-machine behaviour (epic 005), not a parse rejection.

Open sub-questions owned by epic 004:

| Question | Owner | Falsifier |
|---|---|---|
| Does a hello for a 32-member bundle fit 1 252 bytes, and what are its maxima (name length and encoding, ports, content types per port, capability bits)? | This task | It does not; hello must be split or bundle descriptions moved to the handshake |
| Values of the transport role, and encoding of the alarm field (alarm set fixed by C-173) | This task proposes; epic 005 confirms the transport role | Epic 005 needs a role the encoding cannot express |

## Work
1. Write the hello field table in `wire.md` with every field listed above: widths, types, valid ranges; magic/version per 004-04; authentication reservation per 004-05 (C-115). Define the common control header (magic, version, domain, message type, authentication reservation) here; 004-17, 004-18, 004-24 and 004-27 reuse it.
2. Fix maximum counts: name length (bytes, encoding), ports, content types per port, bundles per hello, members per bundle (≤ 32), channel map entries; capability bit assignments (holder-eligible at a fixed bit); transport role values; alarm field encoding with one value or bit for each of the six connection alarms of C-173. Record the transport role values as a proposal for epic 005 to confirm.
3. Define the regime stamp version number field (C-18): width and the comparison rule a receiver uses to decide that its copy is stale.
4. Compute the maximum hello size, including a source with a 32-member bundle (C-08), and compare with 1 252 bytes. If it does not fit, report C-17 or the hello content as a falsified constant and specify the fallback (split hello, or move bundle descriptions to the admission handshake), updating the field table accordingly.
5. Validity rules with outcomes: too short, bad magic, version mismatch (named event), foreign domain (different non-zero domain, `ForeignDomain`), domain 0 (ignored and counted, C-173), oversize (`OversizeControl`), counts above maximum, length inconsistent (`BadLength`), name not valid in the chosen encoding, reserved non-zero, liveness counter going backwards for the same instance nonce. State that a module-id collision (C-46) is not a parse rejection.
6. Add the table and the computed maximum size to the machine-readable file.

## Acceptance criteria
1. Every one of the 17 hello contents listed in Context has a field (or field group) in the table.
2. The maximum hello size is stated in bytes, computed with a 32-member bundle, and is ≤ 1 252 bytes; otherwise a falsified constant is reported with the chosen fallback (epic AC-3).
3. Both open sub-questions are answered with a verdict against their falsifiers, the transport role recorded as a proposal with epic 005 as confirming owner; the alarm field expresses the six alarms of C-173.
4. At least 10 invalid forms with outcomes, including version mismatch as a named event.
5. The common control header is defined once and referenced by the other control-message tickets.
6. Passes the 004-02 checker, including its maximum-size computation.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Election messages (004-17), the admission handshake (004-18) and the regime stamp message (004-24).
- Discovery, election and collision-quarantine state machines (epic 005).
