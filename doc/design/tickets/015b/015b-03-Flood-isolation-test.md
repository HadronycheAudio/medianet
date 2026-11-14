# 015b-03 — Flood isolation test (AC-11)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-14, 015a-17, 015b-12
## Context
- AC-11 (015b-A1): one input flooded at 10 × its admitted rate: zero late slices on other inputs, no missed deadline on the module; the excess appears in the socket overflow counter.
- Mechanism: bounded drain (C-16) — per input per slice period (C-156), at most 2 × the admitted packets per slice; one socket per bundle; socket buffer sized to declared rate × A (C-96); `SO_RXQ_OVFL` cumulative drop counter `[kernel-doc]`.
- The flood must come from the pinned source, or the source filter discards it before the drain is exercised. The flood is therefore sent with the C-120 raw-packet hook on the producer's media socket (015b-12); each use is stamped as an injection.
- POC defect `[code]`: `src/module.rs:1669` drained every input completely before checking its deadline.
## Work
1. Integration test: one consumer module with ≥ 2 inputs on loopback, a commissioned A for the test group; the producer of one input floods it at 10× its admitted packet rate for ≥ 30 s through the raw-packet hook.
2. Collect late-slice counts per input, missed-deadline count of the module, the overflow counter of the flooded socket, and the stamp.
## Acceptance criteria
1. Late slices on non-flooded inputs = 0.
2. Missed deadlines on the module = 0.
3. Overflow counter of the flooded input's socket > 0 and reported in the stamp.
4. The producer's stamp records the injection.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — read only; "Design documents updated" — no behaviour differs; "No new open item" — none raised; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — 015b-10.
## Out of scope
- Real-time priority effects (task 016); hardware NIC tests (task 019).
