# 003-05 — Measure post-fix sync rate and lock
**Epic:** 003-POC-smoke-fixes · **Effort:** XS · **Depends on:** 003-02, 003-04

## Context
- Epic 003 fixes clock-sync pacing in the frozen proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`). Pre-fix the clock thread waited a fixed 5 ms receive timeout; on the measuring machine (`CONFIG_HZ=250`, P-03) `--sync-hz 50` achieved 37 exchanges/s and `--sync-hz 1000` achieved 77 (P-02) `[measured]`. 003-04 replaces this with a wait bounded by the next request deadline and a 100 ms ceiling.
- Epic criterion 003-AC3: after the fix the achieved sync rate is within 2 % of the configured rate at `--sync-hz 50` (the default rate C-36, inside the accepted range 10 to 200/s of C-112). The figure at `--sync-hz 1000` is recorded whatever it is: 1000 is outside the C-112 range, so whether it is reachable unprivileged is no longer an open question and nothing asserts it.
- Open sub-question owned by this ticket: "Does deadline pacing change the servo's lock behaviour enough to alter the baseline?" Falsifier: lock is no longer reached within the 25 s smoke run. "Locked" is the POC's own rule (P-04): smoothed error below max(20 µs, path delay / 4) after at least 8 windows `[code]`. The runtime's acquisition target C-63 does not apply to the POC.
- Measurement conditions (epic background): unprivileged, as for P-02, so before and after are comparable. Strict mode (C-127) is a runtime mode the POC lacks; C-147 settles that POC runs do not run in strict mode and their figures go only in the POC reference rows (P-nn).

## Work
1. On the measuring machine and with exactly the method recorded in 003-02, run unprivileged, headless: `hadropoc loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --sync-hz 50`, and the same with `--sync-hz 1000`, on the build with 003-04 applied.
2. Record achieved exchanges/s for both, tagged `[measured]`.
3. In each run, record whether and when the clock reached the locked state.
4. Answer the lock sub-question in writing (for 003-08 and 003-10).

## Acceptance criteria
1. Achieved rate at `--sync-hz 50` is between 49.0 and 51.0 exchanges/s (within 2 %).
2. Achieved rate at `--sync-hz 1000` is recorded as a number, whatever it is.
3. The clock reaches the locked state within 25 s at the smoke flags; if not, the falsifier has fired and this is recorded and escalated to the design owner before 003-09.
4. Figures carry machine identity, kernel `CONFIG_HZ`, commit and method.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Writing `BASELINE.md` (003-08) and the design-set update (003-10).
- Any assertion on sync rate in `tools/smoke.sh` (forbidden by the epic).
- Rates other than 50 and 1000; system criterion AC-7 over the C-112 range belongs to the runtime's servo (task 008).
