# 022b — Windows edge L2 certification

**Phase:** P4 (second half; does not block P5) · **Scope:** M (10 tickets, about 12.25 days) · **Depends on:** 022a, 001, 020, 025b (cost calibration factor only, the last ticket; see the roadmap) · **Blocks:** 028

Epic 022 is split under C-143 into two parts: **022a** builds the Windows backend and the WASAPI edge; **022b** (this file) certifies it at L2 and publishes the certification record. This part depends on 022a: it needs the working sink and capture, the `windows-std` stamp, strict mode and `WasapiRawCapture`. A reference elsewhere to "epic 022" means both parts. Acceptance criteria keep the original `022-AC-n` numbering in both parts.

## Goal

Certify the Windows **edge** built in epic 022a at L2: measure E, the restart spread of D, cross-sink skew, batched-capture margins, zero slip, the drift ramp and peer-verified DSCP on a documented reference rig; publish `doc/design/reports/l2-results-windows.md` with a short reference list of certified OS build / device / driver combinations; and confirm or correct every `[memory]` Windows fact. Its certified numbers are deliberately looser than Linux's, and the regime stamp says exactly how.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; write `T(p)` for the network time of position p. Linux hosts are the backbone (clock holder, elections, conformance, commissioning). macOS and Windows machines join as **edges** — typically a musician's laptop. The Windows edge never holds the clock and never stands for election.

**Evidence warning.** Every Windows platform fact in epics 022a and 022b is `[memory]` — recalled, not verified — until task 001 (receive-stamp probes) and this part's L2 run confirm it. A figure from a named, published standard may be cited with its clause and tagged `[standard]` once the citation is checked (C-132); vendor platform behaviour is not a standard and must be measured. Supported OS versions are set after task 001 reports; its probes test the current release and the oldest obtainable one (C-138).

**What 022a delivers `[decision]`.** A Windows edge on WASAPI in exclusive mode only (never shared mode); capture of physical device inputs only; if another process has the device open the edge reports "interface held" and certifies nothing (C-175); no spinning ticker: sinks consume, capture emits one batch of slices per engine period, stamped *pacing: batch per engine period*; one rate on the wire, 96 kHz (C-01), with a 48 kHz device converted through the kit's resampler node (C-80), which exists only inside an edge module and only at 2:1 or 1:2 (C-105, C-134); IPv4 only, multicast TTL 1 (C-100); strict mode (C-127, C-128) set by operator policy (C-165).

**The edge contract `[decision]`.**

- `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. A is the presentation group's audition offset (C-55). D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter; for capture, the mirror, from acoustic instant to the sample being available (C-170). On Windows D is **measurement only**: reported stream latency is unreliable across drivers `[memory]`.
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum reported beside it (C-73). A percentile claim, not a bound.
- Only the certification state *commissioned* gives a numeric E; uncertified and self-monitored sinks declare E *unknown* (C-140) and are refused with RequiredEExceeded by any group that sets a required E (C-103).
- **Two-loop timebase** (task 018): loop 1 maps QPC to the domain line through sync exchanges; loop 2 maps the device sample counter to QPC; the ratio of the edge's rate actuator is the quotient of the slopes; rate-loop bandwidth well below 1 Hz (C-68). The rate actuator is the steerable resampler, distinct from the fixed 2:1 / 1:2 node, built with its trait by the kit (epic 024a) with a ratio range of ±1 000 ppm; its delay is part of D, never of L (C-189, C-197, C-210, C-207).

**Windows targets (pending measurement here).**

| Quantity | Target | Rule |
|---|---|---|
| E | 1 ms | C-72, defined by C-73 |
| E budget split (proposal) | 100 µs clock + 100 µs tracking + 800 µs uncertainty of D | C-150 |
| Restart spread of D | at most 0.5 ms over 20 restarts; spread is max − min | C-74, C-166 |
| Typical D | 6 to 20 ms | C-81 |
| Cross-sink skew against the Linux reference sink | at most 1.05 ms (50 µs Linux E, C-70, plus 1 ms); 99.9th percentile of absolute skew over one hour and across restarts of either sink, maximum reported | C-82, C-79 |
| Zero slip | marker offset returns within the platform restart spread | C-76 |
| Drift ramp | ±100 ppm at no more than 0.5 ppm/s; E held throughout | C-119 |

E is dominated by D uncertainty and scheduling spikes, not by the clock.

**How the L2 runs are conducted.**

- **Strict mode** (C-147): every run whose numbers enter the constants table runs in strict mode — MMCSS registration granted on the data and clock threads and memory locked (C-127). L0 does not.
- **A** (C-148): the harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results; that is the "commissioned A" of the criteria.
- **Which E** (C-149): pass criteria use the platform's target E (1 ms), never the E measured in the same run.
- **Parameters** (C-166, proposals pending first runs): no-step tolerance at handover is the platform restart spread; comparison windows of 10 s; injected loss 1 % random plus bursts of 8 slices; measurement window after a restart 60 s; restart spread of D is max − min; sample tolerances are measured against the median of the first 60 s. Completed by C-191 (proposals pending first runs): loss bursts are spaced 1 s apart and hit the media path only; the announced step is 5 ms; a 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run. Sync loss has no separate scenario (C-202, superseding C-191's "its own scenario"): it is covered by the handover run.
- **Handover trigger** (C-212, C-202): the handover is triggered as a holder loss — the old holder stops answering, the Windows sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. The run asserts emission error within the target E (1 ms) while the sink is in holdover between holders, and frequency continuity at takeover: rate change within C-61 (at most 1 ppm per second) and no phase step; the sink's clock state is read from its published time view (C-187). A run in which holdover was not observed is failed, not passed. The new holder is the second holder-eligible Linux node, whose time is usable at the trigger (Locked, or in holdover with its alignment claim not lapsed, C-213, C-65, C-169), so it takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time (C-213). After the loss every candidate is in holdover, so the lowest module id wins (C-221); the epic 020 harness gives the second Linux node the lowest id among the rig's holder-eligible modules (020-17), and the Windows edge is never a candidate. The 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, totalling 50 ppm; both parts are recorded in the run report (C-223). Per-sink emission error in holdover is an L2 assertion only; the L3 rig cannot observe it (C-212, C-220).
- **Commissioning window** (C-107, C-153): measuring D drives test traffic, so it runs inside a declared window that every module stamps; the tool announces it on the control group with an expiry time and it closes at expiry or on explicit close. What an open window disqualifies (C-178): ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while it is open; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in. So D and its restart spread measured inside the window are valid certification evidence, and production figures read from stamps (lateness and loss counters, alignment figures) count only when taken with no window open.
- **Locations** (C-130, C-133, C-171): results and the certification record in `doc/design/reports/`; the evidence appendix is `doc/design/06-evidence-appendix.md`; the constants table is `doc/design/02-constants.md`; operator checklists in `doc/operations/` (written by epic 028); the runner and the L2 harness binaries in `hadronyche-tools`.
- **Reference hardware** (C-172): proposed by the implementing team, approved by the product owner, recorded in the constants table.

**Batched capture and consumer margins.** Exclusive-mode hand-off is quantised to the device period, about 3 to 10 ms `[memory]`, so a capture edge emits 6 to 20 slices (0.5 ms each, C-02) per callback. Lateness at consumers includes the period, so A must cover it. A consumer measures arrival lateness for 4 s (C-50) and sets its offset to the observed maximum plus headroom (C-51, C-52). An absent slice is counted missing at the deadline and reclassified late if it arrives within the ring span (C-108); an input with at least one late slice in each of 10 consecutive 1 s windows is persistently late and ratchets (C-101, C-53); persistently late at the ceiling A, it is Demoted (C-109). The consumer publishes its per-connection counters (C-159) in its regime stamp.

**DSCP peer verification.** DSCP sync 48, media and one-way probes 46, control 40 (C-13). Plain `IP_TOS` is typically ignored on Windows; marking goes through the QoS flow API and may need administrator rights or policy `[memory]`. Peers publish what they observe: the consumer per media connection (C-47); the holder for each follower's sync requests and every module for each peer's hellos (C-152). All three classes are therefore peer-verifiable.

**Cost calibration** (C-106, C-176). The DSL conformance suite (epic 025) runs a reference workload once per listed target and records time per operation; this part loads the value for each listed Windows target into the stamp's data file.

**Reference list.** A table in the certification record and a data file in the crate read by the stamp builder (022a). Candidates are the combinations available on the reference rig within the supported OS versions (C-138); the list holds at least one combination. Everything else stamps *unverified* and declares E unknown.

**Uncertifiable in v1 regardless of engineering.** Any bound on scheduling latency; a device whose driver misreports latency, without the commissioning measurement; any OS build, driver and device combination outside the short reference list; isolation of the laptop (dual-homed, emits discovery chatter); a device that runs at neither 96 kHz nor 48 kHz (C-105).

**Trust statement.** "Trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment." Commissioning baseline: dedicated medianet NIC, discovery chatter suppressed, firewall scoped to medianet ports (checklist delivered by task 028).

## Deliverables

- The documented manual hardware job and reference rig (a Linux backbone holder plus the second holder-eligible Linux node for the holder-loss handover, C-212), with the runner in `hadronyche-tools` (C-133).
- L2 Windows runs: sink E and D restart spread; cross-sink skew; batched capture against a Linux consumer; zero slip; drift ramp; DSCP without administrator rights and peer verification of all three classes.
- The cost calibration factor per listed Windows target in the stamp's data file (C-176).
- Document `doc/design/reports/l2-results-windows.md` (C-130): the certification record, including the reference list, approved per C-172 and recorded in the constants table.
- Checklist inputs for epic 028 (firewall, DSCP policy route, power, dedicated NIC, D configuration key).
- Every `[memory]` Windows fact in epics 022a and 022b confirmed `[measured]` or corrected, with entries in the evidence appendix.

## Dependencies

- **Needs:** the complete Windows edge, its stamp, strict mode and `WasapiRawCapture` (022a); the stamp-probe result for Windows (001); the L2 harness and its scenario set (020); the calibration factors recorded by the DSL conformance suite (025, C-176).
- **Provides:** a certified platform profile and per-OS checklist inputs to the commissioning tool (028), including the edge configuration key that holds the commissioned D.

## Acceptance criteria

1. **022-AC-1 (AC-28).** E at most 1 ms (99.9th percentile over one hour, maximum reported); restart spread of D at most 0.5 ms over 20 restarts; cross-sink skew against the Linux reference sink at most 1.05 ms (C-82, 99.9th percentile over one hour and across restarts of either sink, maximum reported); **batched capture emission causes no gaps at the commissioned A** on a Linux consumer over one hour: zero slices counted missing or late (C-108), no ratchet step (C-101) and no Demoted input (C-109). All runs in strict mode (C-147), with A computed and recorded by the harness (C-148), judged against the target E (C-149).
2. **022-AC-2. Zero slip.** After injected loss (1 % random plus bursts of 8 slices spaced 1 s apart on the media path only, C-166, C-191), a 50 ppm handover (C-78) triggered as a holder loss 60 s into the run (C-191, C-212) and an announced step of 5 ms (C-191), the marker offset returns to its prior value within the platform's restart spread (C-76), compared over a 10 s window that starts when the injected event ends (C-191). In the handover run, holdover between holders is observed, emission error stays within the target E (1 ms) throughout it, and frequency is continuous at takeover (rate change within C-61, no phase step); a run without observed holdover fails (C-202, C-212). The new holder had usable time at the trigger and the timeline epoch is unchanged across the handover (C-213). The run report records the measured natural crystal difference and the injected drift that make up the 50 ppm (C-223).
3. **022-AC-9. AC-34 (Windows part).** The Linux consumer's regime stamp shows, for its connection from the Windows capture edge, the DSCP value it actually received (C-47); the Windows stamp's media class agrees. The holder's and peers' stamps show the sync and control DSCP observed from the edge (C-152), and the Windows stamp's sync and control classes agree.
4. **022-AC-10.** The receive-timestamp claim in the design text is replaced by a `[measured]` statement from task 001.
5. **022-AC-15. Drift ramp (C-119).** With injected crystal drift sweeping ±100 ppm at no more than 0.5 ppm/s, the Windows sink holds E (1 ms) throughout.
6. **022b-AC-R. Certification record.** `l2-results-windows.md` lists a measured value or test result and a verdict for every part of 022-AC-1 and for 022-AC-2 to 022-AC-16 (including those delivered by 022a); the reference list holds at least one certified combination, approved per C-172; the record carries both stamps, the regime stamps of the Windows sink and of the holder it followed (C-191).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Every `[memory]` fact in this file is either confirmed (`[measured]`) or corrected
- [ ] A short reference list of certified OS build / device / driver combinations is published; everything else stamps *unverified*
- [ ] CI builds the crate on Windows; hardware tests are a documented manual job

**Not applicable in this part** (C-125): "Every parser touched is fuzzed" — this part writes no parser (the runner and the calibration data file use no new parser of untrusted input); "The frozen POC is not modified" — this part writes only `hadronyche-tools`, the crate's data files and documents under `doc/design/` (C-130) and has no subject in the POC tree. Every other line has a subject here: "Tests green in CI, unprivileged" covers the runner in `hadronyche-tools` and the data-file loading tests; the stamp fields touched are the calibration factor and the reference-list data; the `[memory]` line covers the facts of both 022a and 022b (022b-08); the CI line is met by 022b-01 (manual job) together with the Windows CI build of 022a-01.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is restart spread of D within 0.5 ms with 3 to 10 ms periods? | This part (022b-02) | Spread above 0.5 ms: Windows sinks are certified with a larger E or as *unaligned* |
| Does batched capture emission break consumer margins? | This part (022b-04) | Consumers see late or missing slices, a ratchet step or a Demoted input at the commissioned A |
| Can DSCP be set without administrator rights on a standalone laptop? | This part (022b-06), feeding task 028 | No: marking stays *unverified* and the checklist documents the policy route |
| Who measures D and D_capture in the field and how | Task 028 (inside a commissioning window, C-107, C-153) | A commissioned D differs from the L2-measured D by more than the restart spread |
| Is the UDP receive-timestamping control present, and on which builds? | Task 001 (C-138) | Absent: revisit the 1 ms target only if the measured clock share exceeds the 100 µs allotment of C-150 |
