# 014-05 — Receiver report vectors and fuzzing
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-04, epic 007
## Context
System criterion AC-1 requires every message to pass its test vectors in both directions (encoding the vector values gives the vector bytes; parsing the vector bytes gives the vector values). Epic 007 provides the vector format and runner; the vectors crate sits at the workspace root and the fuzzing tool is the implementing team's choice (C-192). Epic 004 supplies the wire layouts and the reject reasons the vectors expect (C-193). Every parser is fuzzed with zero panics, with the budget of C-95: 60 s on every change and 1 hour nightly, the corpus kept.

The receiver report: `at_position: u64`, up to 32 members (C-08) with `stream`, `session` (per member, C-19), `received`, `late`, `missing`, `marked_gap`, `other_epoch`, `too_early`, `duplicates`, `incomplete_reassemblies`, `demoted_rounds` (C-159), `interrupted`, `health` (including Demoted, C-109), drop counters (`Option<u32>` each), observed DSCP (`Option<u8>`). At most 1 252 bytes (C-17, C-100). Encoder `encode_report`, parser `ReceiverReport::parse`.

## Work
1. Add receiver-report vectors in the epic 007 format, at least: 1 member with all options present; 16 members; 32 members with all options present (the size worst case); members with different sessions; a member in health Demoted with a non-zero demoted-rounds count; a member with distinct non-zero values in each C-159 count; drops absent (macOS/Windows case); observed DSCP absent; counter values at their maximum width.
2. Add negative vectors (bytes that must be rejected), each with the reject reason epic 004 specifies for it (C-193): truncated, 33 members, wrong type, unknown health value.
3. Add a fuzz target `receiver_report_parse` (with the team's chosen fuzzing tool, C-192) calling `ReceiverReport::parse` and, on success, iterating all accessors; plus a round-trip property (parsed then re-encoded bytes equal the input).
4. Run the fuzz target 60 s on every change and 1 hour nightly, unprivileged, and keep the corpus in the repository (C-95).

## Acceptance criteria
1. All report vectors pass in both directions (epic AC-1 / system AC-1).
2. The fuzz target runs 60 s per change and 1 hour nightly with zero panics and zero round-trip mismatches; the corpus is checked in (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Probe vectors and fuzzing (014-10); lineage vectors (014-14); classifier fuzzing (014-12).
