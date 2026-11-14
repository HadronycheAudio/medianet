# 025b-04 — `hadronyche-replay` CLI and user documentation
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** S · **Depends on:** 025b-02, 025b-03
## Context
`hadronyche-replay` (crate `hadronyche-tools`) reads a recording of the slices delivered to a module (with gap flags) and the hashes of what it emitted, reruns the module's code offline, and reports the first differing slice. Windows of 1 024 samples touched by loss or by a degraded slice are excluded from exact comparison and counted `[decision]`. The library is 025b-03 and its outcome carries the excluded-window count; recordings are produced by `hadronyche-std/src/record.rs` (025b-02). The binary lives in `hadronyche-tools` (C-133). Fuzz budget for any custom argument parser: 60 s on every change and 1 hour nightly, corpus kept (C-95).

Definition-of-done item: **Probe and replay tools have user documentation with one worked example each.**

Presentation details (C-206): the human-readable output line format and the documentation file name are decided by this ticket and recorded in the tool's user documentation, not in a specification. Only what an operator's script or another epic depends on, the JSON output schema and the exit statuses, is specified in `doc/design/spec/` (C-130).
## Work
1. Binary `hadronyche-replay <recording> [--module ...]` printing slices compared, excluded window count, and either "identical" or the first differing slice and output; JSON output option.
2. Exit status 0 on identical, non-zero on differs or on unreadable recording.
3. User documentation (file name chosen here, C-206, e.g. `hadronyche-tools/doc/replay.md`) with one worked example: record a module with `record.rs`, replay it, then corrupt it and replay again; it records the human-readable output format.
4. Specify the JSON output schema and exit statuses in `doc/design/spec/` (C-206).
## Acceptance criteria
1. The worked example reproduces the documented output exactly.
2. Exit status is 0, non-zero, non-zero for identical, differs, and corrupt-header recordings respectively.
3. The JSON output schema and exit statuses are in `doc/design/spec/`; the line format is documented in the user documentation only (C-206).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Probe and replay tools have user documentation with one worked example each
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Probe docs (025a-14); acceptance run with the DSL echo (025b-05).
