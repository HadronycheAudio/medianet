# 025a-14 — `hadronyche-probe` CLI and user documentation
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-13
## Context
`hadronyche-probe` in crate `hadronyche-tools` (C-133) is the **latency probe**: instantiates a module in test mode, feeds a marker (impulse or pseudo-random code) at a known position, locates it in each output by cross-correlation, reports measured structural delay per output as a whole number of domain-rate samples (C-105), or non-constant. Works on generated and hand-written modules. The library (025a-12, 025a-13) exists; this ticket adds the binary and docs.

Definition-of-done item of the epic: **Probe and replay tools have user documentation with one worked example each.**

Presentation details of the tool (the human table layout, the JSON field names, file names) are decided by this ticket and recorded in its user documentation; only what another epic or an operator's script depends on goes into a specification (C-206).

Fuzz budget for the argument parser, if it is custom code: 60 s on every change and 1 hour nightly, corpus kept (C-95).
## Work
1. Binary `hadronyche-probe` with options: module to probe (a compiled DSL program or a registered hand-written module), marker (`impulse` | `prn --seed N --len N`), number of slices, multi-spectrum mode, output format (human table and JSON).
2. Exit status non-zero when confidence is below threshold on any output.
3. User documentation (e.g. `hadronyche-tools/doc/probe.md`) with one worked example: probing a corpus delay program, showing the command, output and interpretation, including what a non-constant result looks like.
## Acceptance criteria
1. Running the documented worked example reproduces the documented output exactly.
2. JSON output contains, per output: output index, measured delay as a whole number of domain-rate samples (or non-constant with min and max), confidence.
3. Exit status is 0 on a clean delay module and non-zero on a silent module.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Probe and replay tools have user documentation with one worked example each
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Replay tool docs (025b-04); corpus-wide comparison (025a-15).
