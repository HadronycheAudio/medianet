# 003-02 — Measure pre-fix sync rate
**Epic:** 003-POC-smoke-fixes · **Effort:** XS · **Depends on:** 003-01

## Context
- Epic 003 fixes the clock-sync pacing of the proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`) and must record the achieved sync rate **before and after** the fix at `--sync-hz 50` and `--sync-hz 1000` in `BASELINE.md`.
- Bug 2 `[measured]` `[code]`: the clock thread paces requests with a 5 ms socket receive timeout (`clock_sock.set_read_timeout(5 ms)`, `module.rs:1126`; loop at `module.rs:1504-1600` sends when `now >= next`, then blocks in `net::recv_ts`). After a reply it blocks again for a full timeout. Earlier measurement (constants table P-02): `--sync-hz 50` gave 37 completed exchanges/s, `--sync-hz 1000` gave 77 `[measured]`. One effective timeout of about 13 ms explains both.
- Why these two rates: 50/s is the default sync request rate (C-36) and lies in the accepted range 10 to 200/s (C-112). 1000/s is outside that range; it is measured only so that before and after can be compared with P-02.
- The measuring machine is the machine of P-02, with a `CONFIG_HZ=250` kernel (P-03). This ticket records its identity; every later figure in epic 003 is taken on it.
- Measurement conditions (epic background): unprivileged, as for P-02, so before and after are comparable. Strict mode (C-127) is a runtime mode the POC lacks; C-147 settles that POC runs do not run in strict mode and their figures go only in the POC reference rows (P-nn).
- This ticket owns the epic's open sub-question "How is completed exchanges per second observed on the untouched POC without modifying it?" Falsifier: the method cannot be reproduced by a second engineer within 5 %.

## Work
1. Check out and build `poc-untouched` (`cargo build --release`).
2. Choose the measurement method by reading the POC at `poc-untouched`: an existing output or counter if one reports completed exchanges; otherwise an external count that needs no code change (for example counting clock request/reply pairs on the loopback interface). Do not add instrumentation to the POC.
3. Run `hadropoc loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --sync-hz 50` and the same with `--sync-hz 1000`, unprivileged, headless (no terminal).
4. Derive achieved completed exchanges per second for each run with the chosen method.
5. Record: machine identity (host name, CPU, kernel version), kernel `CONFIG_HZ`, commit, command lines, method, results. Keep these notes for 003-05 (same method) and 003-08 (`BASELINE.md`). Do not commit them yet.

## Acceptance criteria
1. Two figures exist, one for `--sync-hz 50` and one for `--sync-hz 1000`, each in exchanges/s, tagged `[measured]`, measured at `poc-untouched`.
2. The measurement method is written down precisely enough that a second engineer reproduces each figure within 5 %; this answers the epic sub-question.
3. Machine identity and kernel `CONFIG_HZ` are recorded with the figures, and `CONFIG_HZ` is 250 (P-03); if it is not, the machine is not the measuring machine and this is reported before continuing.
4. No file under the POC working tree is changed by this ticket (`git status` clean apart from ignored paths).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The pacing fix itself (003-04) and post-fix measurement (003-05).
- Writing `BASELINE.md` (003-08).
- Measuring at other rates of the C-112 range; system criterion AC-7 over that range is checked against the runtime's servo (task 008).
