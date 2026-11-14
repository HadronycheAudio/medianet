# 015b-11 — Design documents update
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** XS · **Depends on:** 015a-14, 015a-15, 015a-16, 015a-18, 015a-19, 015a-20, 015a-21, 015b-01, 015b-02, 015b-10, 015b-12, 015b-13, 015b-14, 015b-15
## Context
- Definition of done items: design documents updated where behaviour differs, constants table first; no new open item without an owner and a falsifier; every new factual claim carries an evidence tag; regime stamp fields touched are complete and documented.
- Locations (C-130): constants table `doc/design/02-constants.md`; reports and results `doc/design/reports/`; specifications `doc/design/spec/`; evidence appendix `doc/design/06-evidence-appendix.md`. Nothing is written into the frozen POC tree. The architecture specification, which holds the thread model, is `doc/design/01-architecture.md` (C-171).
- Constants and rules used by this epic: C-01 (96 kHz only on the wire), C-04 (6 channels, 192 bytes per channel per slice, 1 152 bytes of audio), C-05 (one packet per slice), C-06 and C-100 (fallback path MTU 1 280 bytes, 1 252 bytes of UDP payload; IPv4 only, TTL 1), C-08 and C-209 (≤ 32 bundle members, the controller member included), C-13 (DSCP 48/46/40), C-14 (spread ≤ 25 % of period, 125 µs), C-16 (drain ≤ 2 × admitted packets per slice, also the batch size), C-17 (control messages ≤ 1 252 bytes), C-18 (stamp transport), C-20 to C-22 and C-92 (wire figures), C-36 and C-112 (sync 50/s default, 10 to 200 accepted), C-37 (election on 1.5 s reply silence), C-38, C-39, C-102, C-108, C-111, C-135, C-168 (liveness), C-40, C-41 (reports, producer stop), C-47, C-157, C-159, C-160, C-182 (stream health and alignment figures in the stamp; rolling 60 s window beside the maximum since live), C-93 (2–4 µs receive cost guess), C-95 (fuzz budget), C-96 (skew budget = A), C-116 (probe size), C-120 and C-145 (test-only hooks), C-121 (home of step detection), C-125 (DoD applicability), C-127 to C-129, C-147, C-161, C-165 (strict mode, capability states, operator policy), C-130, C-133, C-171 (where outputs and tools live), C-131 (receive-timestamp sources), C-136 (holder stall), C-137 (header cap), C-138 (supported OS versions), C-139 and C-185 (housekeeping restart policy, ControlDegraded refusal), C-144 (duties of this epic), C-146 (privileged nightly job), C-152 and C-188 (peer-verified sync and control DSCP, recorded on the holder's clock thread), C-177 and C-201 (seqlock safety and CPU pin), C-204 (over-rate sync requests dropped and counted), C-206 (presentation details of this epic's tools and reports), C-155 (pacing fit), C-156 (drain cycle), C-173 (alarms).
- Values this epic owns and must propose for the constants table: `declared_jitter_ns()` and the step-detector tolerance (measured in strict mode by 015b-15), housekeeping queue capacities (015a-15), measured receive cost (015b-10), achieved sync rates (015b-01). Measured values come from strict-mode runs (C-147).
## Work
1. Update the constants table first with every value chosen or measured above, each with status and evidence tag; measured values link to their report in `doc/design/reports/`.
2. Update the thread model in `doc/design/01-architecture.md` (C-171) and the receive/send path documents where implementation differs.
3. List new open items with owner and falsifier; confirm the epic's remaining open sub-questions are still listed with theirs.
## Acceptance criteria
1. Every value introduced by any ticket of epics 015a and 015b appears in the constants table, and every measured value cites a strict-mode run (C-147).
2. Every new open item has an owner and a falsifier.
3. Every new factual claim has an evidence tag, and each `[measured]` claim cites its report.
4. Every regime stamp field this epic touches (drop, DSCP, health and C-159 counters, lateness percentiles and reclaimable latency with their C-182 window, per-follower dropped sync requests (C-204), injection, buffer clamp, capability states, processing overrun) is documented with name, unit and source.
## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented
- Not applicable (C-125): "Tests green in CI, unprivileged" — documentation only, no code; "Every parser touched is fuzzed" — no parser; "The frozen POC is not modified" — no code; "Trait surface compiles with a stub backend" — no code; "Per-packet receive cost measured" — measured in 015b-10, recorded here.
## Out of scope
- Code changes.
