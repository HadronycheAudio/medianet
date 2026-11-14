# 004-04 — Versioning rules, magic and version fields
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-01, 004-03
## Context
`doc/design/spec/wire.md` must contain versioning rules: what a minor and a major change are, and what a receiver does with each. Every media packet header starts with a magic and a version (media rule 1), and the hello carries the protocol version (control rules).

Facts this ticket needs `[decision]`:
- Rule 12: a version mismatch is a named event, never a silent drop. The version is also carried in the hello so it is detected before a connection exists.
- Rejection reasons available (004-03): `BadMagic`, `VersionMismatch` (also raises a named event).
- The POC used `MAGIC = 0x4844` and `VERSION = 11` with a silent drop on mismatch `[code]`; retired.
- Implementations are `no_std` with bounded parsing time.
- No constant fixes the magic or the version: the epic leaves both to this ticket (open sub-question "Magic value, version field width and scheme, v1 version value", owner epic 004; falsifier: a v1 receiver cannot tell a POC packet or a future major version from a v1 packet before reading any other field).
- Adding a regime stamp section type is a minor change: the regime stamp (C-18) is extended by other epics as they stamp new state `[decision]`.

## Work
1. Choose the magic value and width for v1 and justify it (distinguishable from the POC magic `0x4844`, tag `[decision]`).
2. Define the version field: width, split into major and minor (or another scheme), and the v1 value.
3. Define a **minor change** (e.g. new optional announcement type, new content message kind, new regime stamp section type, new reason value in a reserved range) and a **major change** (any change to an existing field's offset, width or meaning).
4. Define receiver behaviour: same major, higher minor: accept and skip unknown optional parts by their length; different major: drop, count `VersionMismatch`, raise the named event; for the hello, raise the event before any connection exists.
5. State the parse order: the magic and version are checked before any other field, so a future major version may change everything after them.
6. Add the magic and version fields and rules to the machine-readable file.

## Acceptance criteria
1. The magic value and width and the v1 version value are stated.
2. The section defines minor and major changes with at least 3 examples each; a new regime stamp section type is listed as minor.
3. Receiver behaviour is specified for: same version, same major/higher minor, same major/lower minor, different major, bad magic (5 cases), each with an outcome (accept, drop and count under a named reason, named event).
4. The magic and version are at fixed offsets that no future version may move.
5. The machine-readable file contains these fields and passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The remaining media packet header fields (004-06) and hello layout (004-16).
- Reserved-field rules (004-05).
