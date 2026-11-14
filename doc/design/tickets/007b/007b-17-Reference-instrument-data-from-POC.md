# 007b-17 — Reference instrument data from the POC
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-02, epic 002, epic 003

## Context
- The POC test instrument (`/home/dorian/workspace/hadropoc`, tag `poc-baseline`) contributes exact-tier reference data `[code]`:
  - the add/multiply-only `sine()` (`music.rs:263`);
  - the fixed-point voice renderer (`music.rs:382`);
  - the mix functions (`audio.rs:217`, `audio.rs:237`).
- Known hole: the POC's fixed-point sine table is built with libm on each machine (`music.rs:311`) `[code]`; the vectors must ship the table as data.
- Open sub-question owned by task 002: is the POC voice comparison bit-level, so its outputs can be exact-tier reference data? Falsifier: it is tolerance-only; voice reference vectors must then be regenerated from the new reference echo instead.
- Nothing may be extracted before task 002 has cleared the relevant instrument. The POC must not be modified.

## Work
1. Confirm task 002 clearance for `sine()`, the voice renderer and the mix functions; record the references.
2. Export the fixed-point sine table, as computed at tag `poc-baseline`, to `vectors/reference/sine-table.*` as data, with `spec_version` and its source.
3. Define scripted inputs (voices, levels, durations) for the instrument, and store the resulting renderer and mix outputs (or their hashes) under `vectors/reference/`.
4. If task 002 answers "tolerance-only" for voice comparison, do not ship POC voice outputs as exact data; note that 007b-18 regenerates them from the new reference echo.

## Acceptance criteria
1. The sine table file exists and a test shows that loading it needs no libm.
2. Recomputing the outputs from the shipped table and the scripted inputs gives identical bytes on the host.
3. Each file names its POC source line and the task 002 clearance.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No vector is extracted from the POC before task 002 has cleared the relevant instrument
- [ ] Every new factual claim carries an evidence tag
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- Expected hashes for the reference echo and mixer, gap runs and both sample formats (007b-18).
- Cross-machine comparison (007b-19).
