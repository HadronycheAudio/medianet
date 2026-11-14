# 019-04 — Record ingestion and report
**Epic:** 019-L1-two-machine-harness · **Effort:** M · **Depends on:** 019-03, epic 008a, epic 015, epic 017

## Context
After each L1 run, `hadronyche-l1` collects both hosts' record files and stamps and "produces one report" (epic 019 deliverable). Every scenario's verdict is computed from this report, and criterion 019-A requires every result file to carry both hosts' complete stamps.

Quantities the report must expose, with the settled definitions:
- **Alignment.** A fan-in consumer measures origin-to-arrival lateness for 4 s (C-50), sets its offset to the maximum observed plus headroom (C-51), the headroom being the largest of 25 % of that maximum, one slice period, and twice the clock bound (C-52); then it fires at fixed time `T(p) + offset`. A persistently late input (at least one late slice in each of 10 consecutive 1 s windows, C-101) steps its offset up by one slice period (C-53), up to the ceiling A. A is the presentation group's audition offset: no default, proposed by commissioning (C-55), also the skew budget cap to which rings are sized (C-96); for L1 runs the harness computes it with the commissioning rule and records it (C-148, 019-26). An input persistently late with its offset at A becomes Demoted (C-109).
- **Gap classes and counters (C-47, C-108, C-159).** Received, late, missing, marked gap; the stamp also carries other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead (C-159), and lateness at the 50th, 99th, 99.9th percentile and the maximum (C-160) over a rolling 60 s window, with the maximum since the connection went live beside them (C-182). The report's run-wide lateness figures are computed from the records; the stamp's 60 s figures are shown as such and are not compared with them. At the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded.
- **Health.** Interrupted after 8 consecutive missing or late slices; a marked gap or a slice on another timeline epoch resets the count (C-38, C-102). Dead after 2 s without a slice with content or a marked gap (C-39, C-111).
- **Sessions (C-19).** The source assigns one session number per stream, bumped on restart and on timeline epoch change; statistics are kept per stream and session, so a restart is not counted as loss.
- **Slice period**: 48 frames at 96 kHz = 0.5 ms (C-01, C-02, C-05).
- **Clock**: lock bound at most 25 µs (C-62), slew at most 10 ppm (C-60), rate change at most 1 ppm/s (C-61). One-way latency figures are accurate to half the path asymmetry; the worst-case asymmetry bound is published with every one-way figure. The lock state and bound come from the servo core (epic 008a) through the published time view (C-187).
- **DSCP** as received (C-13): sync 48, media and probes 46, control 40.
- **Socket overflow counter** per receive socket.
- **Strict mode (C-147).** A run whose numbers enter the constants table must run in strict mode (C-127) on both hosts.
- **Commissioning window (C-153, C-178).** While a commissioning window is open, ordinary production measurements in stamps (latency, loss, alignment figures) are unqualified.
- Lost packets per stream per direction (sequence gaps), distinct from missing slices.
- Fuzzing (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.

The record file format is defined by the hosted runtime (epic 015) and the reference modules (epic 017); the receiver report layout by the wire specification (epic 004). Receiver reports are unicast to the producer (C-47), so the report reads them from the producer host's records.

## Work
- Module `hadronyche_tools::records`: parser for the runtime's record file format (task 015/017) into typed events; fuzz target `record_file`.
- Module `hadronyche_tools::report`:
  - `Report { stamp: RunStamp, per_stream: Vec<StreamStats>, servo: ServoStats, dscp: DscpTable, overflow: OverflowStats, scenario_section: serde_json::Value, verdict: Verdict }`.
  - `StreamStats { stream, session, direction, slices_expected, slices_received, slices_late, slices_missing, slices_marked_gap, slices_other_epoch, reclassified_late, packets_lost, interrupted_events, dead_events, ratchet_steps, demoted, lateness_p50/p99/p999/max, offset_used, a_used }`.
  - `Report.eligible_for_constants: bool`, true only if both hosts' stamps show strict mode in force for the whole run (C-147) and no commissioning window open at any time during it (C-153, C-178); otherwise the report states which capability was absent, unverified or not requested (C-129), or the window's open interval.
  - `ServoStats { offset_p50/p99/max, slew_max_ppm, rate_change_max_ppm_s }`.
  - One-way figures are always rendered with their asymmetry bound next to them (`x ± bound`); a one-way figure without a bound is a report error.
  - Cross-check: the harness's own counts per gap class are compared with the C-47 and C-159 counters in the end-of-run stamps; a mismatch is reported.
- Output: `<out>/report.json` (machine) and `<out>/report.md` (human), both embedding the full `RunStamp` of both hosts.
- `hadronyche-l1 report <dir>` regenerates both from a results directory.
- Scenario hook: each scenario adds its own section and verdict via `Scenario::evaluate`.
- Golden-file tests using synthetic record files produced by the reference modules in dry-run.

## Acceptance criteria
1. Given a synthetic record set with 10 late and 3 missing slices on one stream, the report shows `slices_late = 10` and `slices_missing = 3` on that stream and 0/0 on the others.
2. A synthetic slice counted missing at its deadline that then arrives within the ring span ends as `slices_late + 1`, `slices_missing − 1`, `reclassified_late = 1` (C-108).
3. A synthetic stream restart (new session number) produces two `StreamStats` rows and no lost packets attributed to the restart (C-19).
4. `report.json` and `report.md` each contain both hosts' complete stamps (the 7 categories from 019-03); a test fails if either host's stamp is missing.
5. A one-way latency figure without an asymmetry bound makes report generation fail with a named error.
6. The `record_file` fuzz target runs 60 s on every change and 1 hour nightly without a crash, with its corpus kept (C-95); malformed records produce an error, never a panic.
7. `hadronyche-l1 report <dir>` on a saved dry-run directory reproduces byte-identical `report.json`.
8. A synthetic run whose stamp shows strict mode off on either host yields `eligible_for_constants = false` with the reason (C-147).
9. A synthetic run whose stamp shows a commissioning window open during the run yields `eligible_for_constants = false`, naming the window's interval (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Scenario-specific sections (added by each scenario ticket).
- The `l1-results.md` design document (019-06, 019-22).
- Changes to the runtime's record format (task 015/017) or the receiver report layout (epic 004).
