# 014-10 — Probe vectors and fuzzing
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-08, epic 007
## Context
System criterion AC-1: every message passes its test vectors in both directions (encoding the vector values gives the vector bytes; parsing the vector bytes gives the vector values). Epic 007 provides the vector format and runner; the vectors crate sits at the workspace root and the fuzzing tool is the implementing team's choice (C-192). Epic 004 supplies the wire layouts and the reject reasons the vectors expect (C-193). Every parser is fuzzed with zero panics, with the budget of C-95: 60 s on every change and 1 hour nightly, the corpus kept. Probe messages: request carries `t1`; reply carries `t1`, `t2`, `t3`; the initiator closes with `t4`. Functions: `encode_probe_request`, `answer_probe`, `close_probe` (014-08).

## Work
1. Add probe request and reply vectors in the epic 007 format: zero timestamps, negative timestamps, `i64::MIN` / `i64::MAX`, typical values.
2. Add negative vectors, each with the reject reason epic 004 specifies for it (C-193): truncated request, truncated reply, request passed to `close_probe`, reply passed to `answer_probe`.
3. Add fuzz targets (with the team's chosen fuzzing tool, C-192) `probe_answer` (fuzzes `answer_probe` input) and `probe_close` (fuzzes `close_probe` input), each with an encode/parse round-trip property on success.
4. Run each target 60 s on every change and 1 hour nightly, unprivileged, and keep the corpora in the repository (C-95).

## Acceptance criteria
1. All probe vectors pass in both directions (epic AC-1 / system AC-1).
2. Both fuzz targets run 60 s per change and 1 hour nightly with zero panics and zero round-trip mismatches; corpora are checked in (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Receiver report vectors (014-05); lineage vectors (014-14); classifier fuzzing (014-12).
