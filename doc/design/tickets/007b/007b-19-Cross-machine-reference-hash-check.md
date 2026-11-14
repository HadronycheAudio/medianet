# 007b-19 — Cross-machine reference hash check
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007b-18

## Context
- Epic criterion 007-AC6: reference vectors pass on at least two different machines with identical hashes, in both sample formats (f32 and Q1.31).
- The reference vectors (echo and mixer, with and without gaps) are in `vectors/reference/` (007b-18).
- The machine pair for cross-machine bit-exactness is settled: x86-64 and 64-bit ARM (C-151).
- Open sub-question owned by task 025 (fingerprint suite): do f32 reference hashes agree across the listed targets? Falsifier: a target passes the fingerprint and differs on a vector. This ticket does not answer it; it only proves two machines agree.

## Work
1. Add a command that recomputes all reference hashes and compares them with the committed values.
2. Run it on an x86-64 machine and a 64-bit ARM machine (C-151) and record the machine descriptions and results in the CI log and in a results file under `doc/design/reports/` (C-130).
3. If the two machines are both CI runners, make the job run on both.

## Acceptance criteria
1. 100 % of reference vectors match on the x86-64 machine and on the 64-bit ARM machine, for f32 and for Q1.31.
2. The machine descriptions (CPU architecture, operating system) are recorded with the result.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Cross-machine results are tagged `[measured]`
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Embedded targets and the fingerprint suite (epics 025, 026).
