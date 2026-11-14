# 027-21 — Decide hosted edge sinks in L3
**Epic:** 027-L3-cross-sink-rig · **Effort:** XS · **Depends on:** 027-20, epic 021, epic 022

## Context
- Open sub-question owned by epic 027, to be answered after epics 021 and 022 (macOS and Windows reference sinks): "Should L3 also include a macOS or Windows sink against the device?" Falsifier: a hosted pair passes its L2 cross-check but a hosted-to-device pair fails.
- Product frame: Linux hosts are the backbone, macOS and Windows machines are edges, embedded devices are followers.
- L3 compares two certified sinks on one capture clock: pass if the 99.9th percentile of |skew| ≤ E_sink1 + E_sink2 for 1 hour and across restarts of either, max reported (C-79). The L3 tool (`hadronyche-l3`) and rig (027-09) were built for Linux-vs-device but the analysis is sink-agnostic.
- Figures for the decision: E targets macOS 100 µs (C-71), Windows WASAPI exclusive 1 ms (C-72); restart spread of D macOS 50 µs, Windows 0.5 ms (C-74); L2 cross-sink skew against the Linux sink at most 150 µs (macOS) and 1.05 ms (Windows) (C-82). All except C-82 are pending L2 measurement; C-82 is derived from them. Proposed E budget splits: macOS 40 + 40 + 20 µs, Windows 100 + 100 + 800 µs (clock + tracking + uncertainty of D, C-150). Pass limits use the platform target E, never the E measured in the same run (C-149). For a hosted edge sink the no-step tolerance is that platform's restart spread (C-166), and runs are strict (C-147).
- A laptop edge that sleeps departs and returns through normal admission (C-124); a hosted-edge L3 run must keep the edge awake or treat sleep as a restart.
- This ticket is a decision, not an implementation.

## Work
1. Once epics 021 and 022 have L2-certified sinks, review their stamps (timestamp source, D and method, E) and the L2 cross-check results against C-82.
2. Decide yes/no per platform, with reasoning, and record it as a `[decision]` in `doc/design/reports/l3-results.md` (C-130) and the open-items list.
3. If yes: open follow-up ticket(s) to run `steady` and the restart scenarios with a macOS or Windows sink on channel 1 instead of Linux; each with owner and falsifier.

## Acceptance criteria
1. A `[decision]` entry exists for macOS and for Windows, each with its reason.
2. If the decision is yes, follow-up tickets exist with owner and falsifier; if no, the sub-question is closed with the falsifier that would reopen it.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Tests green in CI, unprivileged" — no code; "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — reads stamps, changes none; "The frozen POC is not modified" — writes only `l3-results.md` in the design set (C-130); "The skew analysis runs in CI on recorded two-channel files" — no analysis; "The rig is photographed or diagrammed" — no rig change; "`l3-results.md` is committed with a `[measured]` skew figure" — 027-20.

## Out of scope
- Running macOS or Windows L3 measurements (follow-up tickets, if decided).
- The macOS and Windows sinks themselves (epics 021, 022).
