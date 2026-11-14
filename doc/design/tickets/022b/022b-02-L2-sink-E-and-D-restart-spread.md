# 022b-02 — L2 Windows: sink E over one hour and D restart spread
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** M · **Depends on:** 022a-13, 022a-17, 022b-01

## Context
- Part of epic 022b (Windows edge L2 certification), first L2 certification run.
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum reported beside it (C-73). A percentile claim, not a bound.
- Edge contract: `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`.
- Windows targets (pending measurement): E = **1 ms** (C-72); restart spread of D at most **0.5 ms** over **20 restarts** (C-74); typical D **6 to 20 ms** (C-81).
- D on Windows is **measurement only**; reported stream latency is unreliable across drivers `[memory]`.
- Exclusive-mode device period about 3 to 10 ms `[memory]`.
- A combination becomes certified with its E only when it passes here and enters the reference list (022b-07); until then the sink declares E *unknown* (C-103).
- Measuring D drives test traffic, so it runs inside a declared commissioning window (C-107). D and its restart spread measured there are valid commissioning results, each recorded with the window it was taken in; ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while a window is open (C-178).
- Raw results go to `doc/design/reports/` (C-130).
- Test parameters (C-166, proposals pending first runs): restart spread of D is max − min; the measurement window after a restart is 60 s; sample tolerances are measured against the median of the first 60 s.
- The Windows E budget is proposed as 100 µs clock + 100 µs tracking + 800 µs uncertainty of D (C-150, pending this run). Pass is against the target E of 1 ms (C-149), never the E measured in the same run.
- The run is in strict mode (C-147), and the harness computes and records A (C-148).
- Open sub-question owned here: is restart spread of D within 0.5 ms with 3 to 10 ms periods? Falsifier: spread above 0.5 ms → Windows sinks are certified with a larger E or as *unaligned*.

## Work
1. For each candidate combination (022b-01), inside a commissioning window: measure D by the L2 harness; restart the sink 20 times, measuring D over a 60 s window after each restart; compute spread as max − min (C-166); record each D and the spread with the window it was taken in (C-178).
2. After the window has closed, run the sink for one hour in strict mode (C-147) at the A computed and recorded by the harness (C-148); compute E (99.9th percentile) and maximum.
3. Repeat at the smallest and largest device period available on the rig (target 3 and 10 ms).
4. Record results and the sub-question answer in `doc/design/reports/` for 022b-07.
5. Report the measured clock, tracking and D-uncertainty shares of E against the C-150 proposal, so that 022b-08 can confirm or correct it.

## Acceptance criteria
1. **(022-AC-1 / AC-28, part.)** E ≤ 1 ms (99.9th percentile over one hour), maximum reported, on each combination that will be certified.
2. **(022-AC-1 / AC-28, part.)** Restart spread of D ≤ 0.5 ms over 20 restarts; D and the spread are recorded with the commissioning window they were taken in (C-178).
3. Measured D falls in, or is reported against, the typical 6 to 20 ms range.
4. The sub-question "restart spread within 0.5 ms with 3 to 10 ms periods?" is answered `[measured]`; if falsified, the decision (larger E or *unaligned*) is recorded.
5. No stamp figure used for E or its budget shares was taken while a commissioning window was open (C-178).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Cross-sink skew (022b-03), capture batching (022b-04), zero slip (022b-05).
