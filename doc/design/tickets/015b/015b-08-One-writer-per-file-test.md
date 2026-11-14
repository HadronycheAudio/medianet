# 015b-08 — One writer per file test (AC-31)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** XS · **Depends on:** 015a-17
## Context
- AC-31 (015b-A5): four modules in one process writing metrics produce one parseable file; no line fusion.
- Rule: a file has one writer, owned by the process, never by a module; writes go through a dedicated writer thread (015a-08). Framing is one record per line (epic); the record schema is owned by its consumer, the L1 harness (epic 019), and is not needed here.
- POC defect `[code]`: `src/module.rs:1354` — each module called `File::create` on the same metrics path; in loopback four handles overwrote each other and the file was corrupt `[measured]`.
## Work
1. Integration test: `Runtime` with 4 modules, all writing metrics to the same path for ≥ 30 s, each record tagged with its module and a sequence number.
2. Parse the file line by line.
## Acceptance criteria
1. Exactly one file exists at the path and every line is one whole record.
2. Each module's line count equals its emitted count minus its counted drops; sequence numbers per module are increasing.
3. No line contains fields from two records (no fusion).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — the test's line reader is test code, not shipped; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "No new open item" — none raised; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- Metrics record schema (epic 019).
