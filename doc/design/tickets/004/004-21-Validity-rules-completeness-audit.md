# 004-21 — Validity rules completeness audit
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-06, 004-08, 004-09, 004-10, 004-11, 004-12, 004-13, 004-16, 004-17, 004-18, 004-19, 004-20, 004-24, 004-25, 004-26, 004-27, 004-28, 004-29
## Context
Epic 004 acceptance criterion 4: every message type has an exhaustive list of invalid forms with a rejection reason. Each message ticket added its own validity rules; this ticket consolidates them and closes holes.

The epic contributes to two system criteria:
- **AC-1**: 100 % of conformance vectors pass; zero parser panics or hangs under fuzzing.
- **AC-23**: malformed, wrong-version and foreign-domain traffic causes no crash and no state change; each rejection is counted per input and per reason; version mismatch produces a named event.

Starting rejection set (closed enumeration, refined in 004-03): `TooShort, BadMagic, VersionMismatch, ForeignDomain, UnknownStream, UnadmittedSource, BadLength, BadChecksum, BadPacketIndex, WrongEpoch, OversizeControl, ReservedNonZero`. Rule 12: version mismatch is a named event, never a silent drop. Rule 13: foreign-domain packets are dropped and counted. C-17: control messages ≤ 1 252 bytes, each regime stamp section included (C-18); the path-MTU probe pair is exempt (C-116). C-95: every wire parser is fuzzed 60 s on every change and 1 hour nightly, with the corpus kept, by the implementing tasks. C-164 (proposal, pending L1): processing one bad input is linear in packet length and takes at most 10 µs per packet. C-173: `ForeignDomain` is for a different non-zero domain; a hello with domain 0 is ignored and counted.

Message types in scope: media packet header, slice header, 4 announcement types, measurement extension, content messages (audio, dense controllers, sparse keyed controllers, transport state), checksum; hello; each election message; each admission message; the manifest; the path-MTU probe pair; each regime stamp section type and the stamp request; the commissioning window announcement and close; sync request and reply; receiver report; one-way probe.

## Work
1. Build a "Validity rules" matrix in `wire.md`: rows = message types, columns = generic invalid-form classes (truncated, bad magic, wrong version, foreign/zero domain, length inconsistent, count above maximum, reserved non-zero, unknown type code, value out of range, oversize, wrong epoch, unknown sender/stream). Each cell: rejection reason, named event, or "not applicable" with a reason.
2. Fill any empty cell by adding the rule to the message section; add `Reject` variants via 004-03's process if needed.
3. State the required outcome uniformly: drop, count under (input, reason), no state change; plus named event for version mismatch.
4. State the bounded-parsing property per message: maximum bytes examined, no loops over unbounded counts, and work linear in packet length so that the C-164 bound can hold (supports AC-1 fuzzing).
5. Ensure each message's reasons are listed in the machine-readable file so task 007 can generate one negative vector per (message, reason).

## Acceptance criteria
1. The matrix has no empty cell for any of the message types listed in Context.
2. Every rejection reason used exists in the closed enumeration and every enumeration value is used at least once.
3. `VersionMismatch` is a named event for every message carrying a version.
4. Every message states a maximum parse length; the probe exemption (C-116) is the only one above 1 252 bytes among control messages.
5. The machine-readable file lists the reasons per message and passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] Not applicable (C-125): this epic writes no wire parser. The line is discharged by handing task 007 and tasks 009 to 014 the bounds and reason lists they need to fuzz each parser under C-95

## Out of scope
- Writing fuzzers or conformance vectors (task 007 and implementation tasks 009 to 014).
