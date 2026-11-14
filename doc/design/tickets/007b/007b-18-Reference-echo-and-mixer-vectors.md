# 007b-18 — Reference echo and mixer vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007b-17

## Context
- Reference modules (sender, echo, mixer) are kept permanently and are bit-exact by construction `[decision]`. The exact tier records input slices and output hashes, replays the same code offline and compares.
- Deliverable: `vectors/reference/*`: scripted inputs and expected output hashes for the reference echo and mixer, in both f32 and Q1.31 (4 bytes per sample, no 3-byte variant, C-03), including runs with gaps. The hash algorithm and the input script format are chosen by epic 007 itself, in this ticket, and recorded in `conformance.md` (007b-23).
- Exact-tier gap baseline to encode `[decision]`:
  - windows of 1 024 samples touched by loss or by a degraded slice are excluded from comparison and counted;
  - the reference mixer treats a missing input as zero and marks its output slice degraded.
- The degraded flag follows the content (C-122): stamps denote content position; while a module processes input position p it emits the output stamped p − L, whose content came from input position p − L, and that output is degraded if the input at p − L was gapped. The flag is therefore delayed by L together with the content. L is always a whole number of domain-rate samples (C-105).
- Default slice is 6 channels × 48 frames at 96 000 Hz (C-01, C-02, C-04).
- Source data: sine table and instrument outputs from 007b-17. If task 002 rules POC voice comparison tolerance-only, regenerate voice reference data from the new reference echo instead.

## Work
1. Choose and document the output hash algorithm and the input script format, tagged `[decision]`.
2. Produce vectors for echo and mixer, each in f32 and Q1.31 (4 combinations), without gaps.
3. Produce gap runs: a lost slice, a degraded slice, a missing mixer input; expected values include the hash over non-excluded windows, the number of excluded 1 024-sample windows, and the degraded flag on output slices: for an input gap at p, the output stamped p carries the flag, emitted L samples later while input p + L is processed (C-122). Gap runs use L = 0 and at least one non-zero whole-sample L (C-105).
4. Commit under `vectors/reference/` with `spec_version`.

## Acceptance criteria
1. At least 4 no-gap vectors (echo/mixer × f32/Q1.31) and at least 3 gap runs per sample format.
2. In the missing-input run, the expected mixer output equals the mix with that input set to zero and every affected output slice has the degraded flag.
3. In every gap run of a module with non-zero L, the degraded flag is on the output stamped p for an input gap at p, and that output is emitted while input p + L is processed, not while input p is processed (C-122).
4. Expected excluded-window counts match the number of 1 024-sample windows touched by loss or degradation.
5. Re-running the reference generation on the same host gives identical hashes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No vector is extracted from the POC before task 002 has cleared the relevant instrument
- [ ] Every new factual claim carries an evidence tag
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- Running on two machines (007b-19).
- f32 agreement across embedded targets (open sub-question owned by task 025).
- Tolerance-tier vectors.
