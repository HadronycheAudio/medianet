# 013-04 — Canonical configuration encoding
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02

## Context
- The stamp's Identity group contains: protocol version, build identity, **full configuration**, domain, profile. `[decision]`
- Open sub-question owned by this ticket: *How is "full configuration" encoded compactly and comparably?* Falsifier: **two identical configurations yield different stamp ids.**
- The `StampId` is content-derived over regime fields (013-05) and the configuration is a regime field, so the configuration encoding must be deterministic: equal configurations give equal bytes regardless of field order, defaults written or omitted, or the node that produced them.
- Transport (C-18): the stamp travels as sections, each fitting one control message of at most 1 252 bytes of UDP payload (C-17). The configuration need not fit with the rest of the stamp, but its section must fit one message or be split by the rule of 013-13.
- Constraints: `hadronyche-core` is `no_std` and does no allocation; a medianet has at most 64 active modules (C-34).

## Work
1. Decide and document (as a `[decision]`) the compact form of "full configuration": either a canonical byte encoding carried in the stamp, or a fixed-size digest of the canonical encoding plus a way to fetch the full form, or both.
2. Implement a canonicalisation function in `hadronyche-core` that writes the canonical encoding (or digest) into a caller buffer without allocation: fixed field order, explicit defaults, normalised numbers and strings.
3. If the canonical form is decoded anywhere (for example to fetch and compare), add its decoder to the fuzz harness of 013-03.
4. Write the answer to the open sub-question into the module docs for `doc/design/spec/stamp.md`, with its falsifier.

## Acceptance criteria
1. Two configurations that differ only in source field order or in writing a default explicitly vs omitting it produce identical canonical bytes (property test, at least 1 000 generated cases).
2. Changing any single configuration value changes the canonical bytes (property test, at least 1 000 cases).
3. The canonical form of a configuration with 64 modules (C-34) has a stated maximum size, and the test asserts it; if that size exceeds one control message (1 252 bytes), the section is marked for splitting by 013-13.
4. No allocation (`no_std` build).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Open sub-question "How is full configuration encoded compactly and comparably?" answered in writing.
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (the Identity section that carries the field is 013-06); "Design documents updated where behaviour differs, constants table first" (no constant affected); "The stamp format is versioned independently of the media format" (no framing change).

## Out of scope
- The Identity section itself (013-06), `StampId` hashing (013-05), the overall size budget and splitting (013-13).
- The configuration schema itself belongs to other epics; this ticket only canonicalises it.
