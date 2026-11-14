# 007a-02 — Vector file formats and version stamp
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** M · **Depends on:** 007a-01

## Context
- Crate `hadronyche-vectors` holds the conformance vectors of the protocol. The vector file formats are a deliverable of epic 007 itself; every later ticket writes files in the formats defined here, so they must be fixed first.
- The vector kinds and their shape, as fixed by the epic `[decision]`:
  - **Wire valid** (`vectors/wire/valid/*.bin` + `*.json`): one encoding in `.bin`, the decoded structure in the `.json`. Per message type at least one minimal, one typical and one maximal encoding.
  - **Wire invalid** (`vectors/wire/invalid/*.bin` + `*.json`): one violating input in `.bin`, the required rejection reason in `.json`.
  - **Control scenarios** (`vectors/control/*.json`): ordered lists of `(time, input) → expected (outputs, events, resulting state)` for a named state machine.
  - **Timeline** (`vectors/timeline/*.json`): position/time conversions and epoch-switch scenarios.
  - **Transport** (`vectors/transport/*.json`): a scripted transport scenario and the expected result.
  - **Reference** (`vectors/reference/*`): scripted inputs and expected output hashes for the reference echo and mixer, in f32 and Q1.31.
- Every vector file must carry the version of the specification it was generated from (Definition of done of the epic). The format of that version value is an open sub-question owned by epic 004; until it is answered, store it as an opaque string copied from the wire tables, so a later format change needs no schema change. It is not the regime stamp version number carried in the hello (C-18).
- Bad-input vectors must be able to express the required outcomes: no crash, no state change, bounded time, a drop counted per input and per reason, and for a version mismatch a named event. The time bound is C-164: linear in packet length and at most 10 µs per packet. It is a proposal pending L1, so the field holds the bound per file (the C-164 values by default) rather than a crate-wide constant, and a later L1 revision needs no schema change.
- Control scenarios also cover behaviour that is not a single message exchange: the per-connection counters published in the regime stamp, which C-159 completes as received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-47, C-159; other-epoch has its own counter, C-135), the lateness percentiles (50th, 99th, 99.9th and maximum, C-160), the health state Demoted (C-109) and connection alarms (C-173). The expected state must be able to carry such counters, and the expected events such alarms.

## Work
1. Write a JSON schema (or equivalent typed definitions in the crate, checked by test) for each of the six kinds above.
2. Each schema has a mandatory `spec_version` field (opaque string) and a mandatory `id` field unique across the crate.
3. The wire invalid schema has a `reason` field whose value must be a member of the rejection reason enumeration of the wire specification (epic 004), plus optional fields for expected counter increments, expected named events and the per-input time bound.
4. The control schema has a `machine` field and a `steps` list; each step has `time_ns` (i64), `input`, and `expected` with `outputs`, `events` and `state`; `state` may hold any of the named counters of C-159 and a health state (C-109).
5. The reference schema has fields for sample format (`f32` or `q1.31`), input script, expected output hash, and the gap-related counts (excluded windows, degraded slices).
6. Add a test that validates every file under `vectors/` against its schema and fails on a missing `spec_version`.
7. Add one small hand-written example of each kind under a test fixtures directory to prove the schemas.

## Acceptance criteria
1. 6 schemas exist, one per kind listed in Context.
2. The validation test fails on a file with no `spec_version`, on a duplicate `id`, and on an invalid-vector `reason` not in the enumeration (3 negative fixtures).
3. The validation test passes on the 6 example fixtures.
4. The control example fixture carries at least one C-159 counter and one health state in its expected state.
5. The test runs in CI without privileges.
6. The invalid-vector example fixture carries a per-input time bound equal to the C-164 values.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Reading vectors from `no_std` code (007a-03).
- Producing actual vectors (007a-05 onward).
- Choice of hash algorithm for reference vectors beyond reserving the field (007b-18).
