# 020-15 — Restarts scenario and D repeatability
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-14

## Context
A sink emits sample p at `emit(p) = T(p) + A` and hands it to hardware at `submit(p) = emit(p) − D`, where D is the sink's declared plumbing delay: its total delay from hand-off to emission, driver buffer + resampler + converter (C-170). **D must be repeatable across restarts** `[decision]`. The L2 procedure restarts the edge 20 times for the repeatability figure.

Targets `[decision]` (pending measurement): Linux ALSA reference sink — restart spread of D at most **25 µs over 20 restarts (C-74)**, typical D about 1 ms (C-81). macOS at most 50 µs; Windows at most 0.5 ms (C-74). At 96 kHz (C-01) one sample is 10.4 µs.

The record (020-13) carries "D and its method" and the restart spread.

After a restart the edge goes through normal admission: Locking, then Aligning (4 s, C-50); acquisition to Locked takes at most 10 s from the first exchange (C-63, pending simulation).

Settled (C-166, C-170): after each restart D is measured over a 60 s window; the restart spread of D is max − min. Since D is the delay from hand-off to emission (C-170), the measured D of a cycle is the declared D plus that cycle's settled emission error.

## Work
- Scenario `restarts` in `run.rs`: 20 cycles (count overridable); each cycle closes and reopens the sink under test (and its device), waits for Locked (C-63) and the end of Aligning (C-50), measures the settled marker offset over a 60 s window (C-166), then restarts. The capture device stays open across cycles so the capture side is constant.
- Per restart: the effective D, computed as the declared D plus the measured settled emission error of that cycle (C-170), so the spread of effective D equals the spread of emission offset across restarts; record both declared and effective D and state this method in the record.
- Restart spread = max − min of effective D over the 20 restarts (C-166); keep all 20 values in the record.
- CI test against a simulated device with a per-restart random offset of known spread.

## Acceptance criteria
1. `hadronyche-l2 run restarts` performs 20 restarts and reports 20 values of D and their spread.
2. The spread on the simulated device equals the injected spread within 1 µs.
3. The record shows D, its method, and the spread, with a pass/fail against the platform figure (Linux 25 µs, C-74).
4. Any restart that does not reach Locked within 10 s of its first exchange (C-63) is reported as a failure, not silently dropped.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Running the real 20-restart certification (020-22).
- Changing the sink's D declaration (epic 018).
