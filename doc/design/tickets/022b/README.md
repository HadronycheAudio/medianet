# Epic 022b — Windows edge L2 certification: tickets

## Summary
Epic 022 was split under C-143 into 022a (Windows backend and WASAPI edge) and 022b (this part). 022b (phase P4, second half; depends on epic 022a and on epics 001, 020, 025; blocks 028) certifies the Windows edge at L2 on a documented reference rig: sink E over one hour and the restart spread of D, cross-sink skew against the Linux reference sink, batched capture against a Linux consumer at the harness-computed A, zero slip, the drift ramp, and DSCP without administrator rights with peer verification of all three classes (C-47, C-152). Every run is in strict mode (C-147), records the A computed by the harness (C-148), is judged against the target E (C-149) and uses the C-166 parameters as completed by C-191, with the handover triggered as a holder loss, E asserted through its holdover, frequency continuity at takeover within C-61 and the timeline epoch kept (C-202, C-212, C-213); D and its restart spread are valid commissioning results recorded with their commissioning window, and stamp-read production figures are taken with no window open (C-178). It loads the calibration factor recorded by epic 025 (C-176), publishes `doc/design/reports/l2-results-windows.md` with the reference list approved per C-172, hands checklist inputs to epic 028, and confirms or corrects every `[memory]` Windows fact in 022a and 022b.

**Scope:** M (10 tickets, about 12.25 days), computed per C-126.

## Tickets

| Ticket | Title | Effort | Depends on | Former id |
|---|---|---|---|---|
| 022b-01 | Documented manual hardware job and reference rig | S | 022a-15, epic 020 | 022-19 |
| 022b-02 | L2 Windows: sink E over one hour and D restart spread | M | 022a-13, 022a-17, 022b-01 | 022-20 |
| 022b-03 | L2 Windows: cross-sink skew against the Linux reference sink | S | 022b-02 | 022-21 |
| 022b-04 | L2 Windows: batched capture emission vs Linux consumer margins | M | 022a-14, 022a-17, 022b-01 | 022-22 |
| 022b-05 | L2 Windows: zero slip after loss, handover and step | S | 022b-02, epic 020 | 022-23 |
| 022b-06 | DSCP without administrator rights, and AC-34 peer check | S | 022a-05, 022a-14, 022a-17, 022a-21, 022b-01 | 022-24 |
| 022b-07 | `l2-results-windows.md` and certified reference list | S | 022a-08, 022a-09, 022a-10, 022a-18, 022a-19, 022a-20, 022a-21, 022b-02, 022b-03, 022b-04, 022b-05, 022b-06, 022b-09, 022b-10 | 022-25 |
| 022b-08 | Confirm `[memory]` facts and update design documents | S | 022b-07, epic 001 | 022-26 |
| 022b-09 | Cost calibration factor for the Windows target (loads the epic-025 value, C-176) | XS | 022a-17, 022b-01, epic 025 | 022-29 |
| 022b-10 | L2 Windows: drift ramp (C-119) | S | 022b-02, epic 020 | 022-30 |

No ticket was deleted: no ruling removed work from this part, and none of these tickets handled rational latency.

## Effort tally
- XS: 1 (0.25 day)
- S: 7 (7 days)
- M: 2 (5 days)
- Total: 10 tickets, about 12.25 days → scope M (C-126: XS 0.25 d, S 1 d, M 2.5 d; up to 5 = S, 15 = M, 40 = L, more = XL)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| Manual hardware job, reference rig, runner in `hadronyche-tools`, reference hardware proposal (C-172) | 022b-01 |
| L2: E and D restart spread (C-72, C-74, C-166), E budget split reported (C-150) | 022b-02 |
| L2: cross-sink skew (C-82, C-79) | 022b-03 |
| L2: batched capture margins at the harness-computed A (C-148) | 022b-04 |
| L2: zero slip (C-76, C-166, C-191), handover as a holder loss (C-202, C-212, C-213) | 022b-05 |
| L2: DSCP without administrator rights; peer verification of media, sync and control (C-47, C-152) | 022b-06 |
| L2: drift ramp (C-119) | 022b-10 |
| Calibration factor in the stamp data file (C-106, C-176) | 022b-09 |
| Document `doc/design/reports/l2-results-windows.md`, reference list, epic-028 inputs | 022b-07 |
| `[memory]` facts confirmed, evidence appendix, constants table | 022b-08 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 022-AC-1 (AC-28): E ≤ 1 ms | 022b-02 |
| 022-AC-1 (AC-28): D restart spread ≤ 0.5 ms over 20 restarts | 022b-02 |
| 022-AC-1 (AC-28): cross-sink skew ≤ 1.05 ms (C-82, C-79 statistic) | 022b-03 |
| 022-AC-1 (AC-28): batched capture causes no gaps at commissioned A (C-108, C-101, C-109) | 022b-04 |
| 022-AC-2: zero slip, including E during holdover, continuity at takeover and epoch kept in the holder-loss handover (C-202, C-212, C-213) | 022b-05 |
| 022-AC-9 (AC-34 Windows part): observed DSCP per class (C-47, C-152) | 022b-06 |
| 022-AC-10: receive-timestamp claim replaced by `[measured]` from 001 | 022b-08 |
| 022-AC-15: drift ramp (C-119) | 022b-10 |
| 022b-AC-R: certification record and reference list | 022b-07 |

### Definition of done

| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | 022b-01 (runner), 022b-09 (data file) |
| Every parser touched is fuzzed | Not applicable in this part (C-125): no parser is written |
| Regime stamp fields touched are complete and documented | 022b-07 (reference-list data), 022b-09 |
| Design documents updated, constants table first | 022b-08; approved reference hardware recorded in the constants table by 022b-07 (C-172) |
| No new open item without owner and falsifier | 022b-02, 022b-04, 022b-06, 022b-07, 022b-08, 022b-10 |
| Frozen POC not modified | Not applicable in this part (C-125): no subject in the POC tree; 022b-08 still checks it |
| Every new factual claim carries an evidence tag | all |
| Every `[memory]` fact confirmed or corrected | 022b-02, 022b-04, 022b-06 (measurement), 022b-08 (text, both parts) |
| Reference list published; everything else *unverified* | 022b-07 |
| CI builds on Windows; hardware tests a documented manual job | 022b-01 (with 022a-01 for the CI build) |

### Open sub-questions

| Question | Handling |
|---|---|
| D restart spread within 0.5 ms at 3–10 ms periods? | 022b-02 |
| Batched capture breaks consumer margins? | 022b-04 |
| DSCP without administrator rights? | 022b-06 (feeds epic 028) |
| Announced step size in zero slip | Settled by C-191 (5 ms): 022b-05 |
| Meaning of "unqualified" measurements in a commissioning window | Settled by C-178: 022b-01, 022b-02, 022b-04 |
| D measurement in the field | Epic 028; see Remaining gaps |
| Receive-timestamping control | Task 001; closed in 022b-08 |

## Remaining gaps
1. **Field measurement of D and D_capture** — who measures them and how (D is defined by C-170). Affects 022b-02, 022b-07. Owner: epic 028.

## Settled since the split
- Former gap 5, zero-slip disturbance sizes: the loss size, handover tolerance, comparison windows and reference median are set by C-166; the step size and timing by C-191 (below).
- Former gap 7, peer verification of sync and control DSCP: C-152; 022b-06 now checks all three classes against the holder's and peers' stamps.
- Former gap 9, calibration benchmark: C-176. Epic 025 runs the reference workload once per listed target; 022b-09 only loads the recorded value into the stamp data file.
- Strict mode for measurement runs (C-147), A for ladder runs computed by the harness (C-148), target E in pass criteria (C-149): 022b-01 to 022b-05, 022b-07, 022b-10.
- Windows E budget split (C-150, proposal): reported in 022b-02 and confirmed or corrected in 022b-08.
- L2 parameters (C-166): 022b-02 (60 s window, max − min, median of first 60 s), 022b-05.
- Numeric E only for *commissioned* (C-140): 022b-07.
- Supported OS versions after task 001 (C-138); reference hardware approved by the product owner and recorded in the constants table (C-172); operator checklists in `doc/operations/` (C-171): 022b-01, 022b-07.
- Commissioning-window life cycle (C-153): 022b-01.
- Former remaining gap 1, announced step size in zero slip, and the rest of the L2 timing: C-191 (proposals pending first runs) — the announced step is 5 ms; loss bursts are spaced 1 s apart on the media path only; the 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run. Applied in 022b-05 and the epic's 022-AC-2.
- Former remaining gap 2, what an open commissioning window disqualifies: C-178. D and its restart spread measured inside the window are valid commissioning results, each recorded with its window; ordinary production measurements in stamps are unqualified while a window is open. 022b-01 (job document), 022b-02 (D recorded with its window, one-hour E run after the window closes) and 022b-04 (counters read with no window open) corrected accordingly.
- Per-connection counters (C-159) recorded in the batched-capture run: 022b-04.
- Handover as a holder loss (C-212) and sync loss folded into the handover run (C-202, which withdraws C-191's "sync loss is its own scenario"): the old holder stops answering, the sink passes through about 2 s of holdover and the new holder takes over; the run asserts E (1 ms) during holdover and frequency continuity at takeover, and a run without observed holdover fails. Applied in 022b-05 (Context, Work, new criterion 4), the epic's parameters and 022-AC-2, 022b-07 (target list) and 022b-01 (second holder-eligible Linux node on the rig).
- Frequency continuity at takeover (C-212 as revised): rate change within C-61 (at most 1 ppm per second) and no phase step. Applied in the epic's parameters and 022-AC-2, 022b-05 (Context, Work, criterion 4) and 022b-07 (target list).
- Who may take over as holder (C-213): a holder-eligible module with usable time (Locked, or in holdover with its alignment claim not lapsed, C-65, C-169) takes over with frequency continuity and keeps the timeline epoch; the epoch is bumped (C-113) only when no candidate has usable time. The handover run now records the new holder's state and asserts the epoch unchanged: the epic's parameters and 022-AC-2, 022b-05 (Context, Work, new criterion 5), 022b-01 (second node Locked at the trigger), 022b-07 (target list). No new ticket: the work fits 022b-05 (S).
- Handover rulings (C-220, C-221, C-223): the 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report (epic handover paragraph and 022-AC-2, 022b-01 job document, 022b-05 Context, Work and new criterion 6, 022b-07 target list); after the loss the lowest-id candidate wins (C-221), so the second Linux node holds the lowest id, set by the epic 020 harness (epic, 022b-01, 022b-05); per-sink emission error in holdover stays an L2 assertion (C-220). C-212 rechecked: 022b-01 already named the second holder-eligible Linux node; the epic's rig deliverable now names it too. No new ticket; the tally is unchanged.
- Window of the stamp's lateness figures (C-182): rolling 60 s with the since-live maximum; 022b-04 collects the one-hour distribution in the harness.
- "Both stamps" of a certification record (C-191): the regime stamps of the Windows sink and of the holder it followed; 022b-07 and 022b-AC-R.
- Tolerance for a pair of different sinks (C-203): 022b-03 reports both sinks' figures beside the verdict.
- Two kinds of resampler (C-189, C-197, C-210, C-207): the edge's rate actuator, steerable with a ±1 000 ppm range and built by the kit, is what the drift ramp exercises; its delay is in D, never in L. Epic background and 022b-10.
- Presentation details of tools (C-206): the runner's output format and result file names are decided in 022b-01.
- Earlier settlements kept: skew definition (C-82 with the C-79 statistic), reference-list location and format (C-130), scope label from the tally (C-126), and C-100 to C-133 as applied before.
