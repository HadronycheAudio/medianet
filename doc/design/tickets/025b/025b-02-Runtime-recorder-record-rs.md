# 025b-02 — Runtime recorder `record.rs`
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** S · **Depends on:** 025b-01, epic 015
## Context
Deliverable: crate `hadronyche-std`, file `record.rs` — the runtime-side recorder of **input slices and output hashes for one module**, written through the **single-writer file service**.

The process function `process(state, inputs at position p with per-input gap flags, position) -> outputs` never blocks and performs no allocation, so the recorder must not block or allocate on the processing thread: it hands data to the file service, which does the writing.

Recording format and `Recording` type: ticket 025b-01.

The single-writer file service is provided by the hosted backend, epic 015 (C-144). The recorder hands its data to that service through a non-blocking hand-off and never writes files itself.
## Work
1. `hadronyche-std/src/record.rs`: attach to one module instance; per slice capture position, inputs with gap flags, per-output hash and degraded mark; hand off to the single-writer file service.
2. Hash outputs on the processing thread with the 025b-01 hash, or copy outputs and hash off-thread — document the choice.
3. Enable/disable at runtime for one named module.
4. Test: record a module for N slices in an L0 run and read the file back with the 025b-01 reader.
## Acceptance criteria
1. A one-minute (120 000-slice) recording of a gain module reads back with the 025b-01 reader, with every slice present and positions contiguous.
2. Zero allocations on the processing thread while recording (counting allocator in test).
3. Gap flags in the recording match the gaps injected in the test.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Replay (025b-03); the single-writer file service itself (epic 015, C-144).
