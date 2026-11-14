# 003-04 — Deadline-paced clock sync wait
**Epic:** 003-POC-smoke-fixes · **Effort:** S · **Depends on:** 003-01, 003-02

## Context
- Epic 003 applies three minimal fixes to the frozen proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust). This is fix 2. Fixes must be minimal and must not change wire behaviour, the servo, the analyzer, or any measured quantity other than the achieved sync rate.
- Bug 2 `[measured]` `[code]`: in `spawn_clock`, the clock thread paces its requests with a 5 ms socket receive timeout (`clock_sock.set_read_timeout(5 ms)`, `module.rs:1126`). The loop at `module.rs:1504-1600` sends a request when `now >= next`, then blocks in `net::recv_ts`. After the reply arrives it blocks again for a full timeout. On the measuring machine (`CONFIG_HZ=250`, P-03), `--sync-hz 50` achieved 37 completed exchanges/s and `--sync-hz 1000` achieved 77 (P-02) `[measured]`; one effective timeout of about 13 ms explains both.
- The data thread already waits correctly with `ppoll` and a computed timeout (`module.rs:1663-1667`) `[code]`.
- Target: within 2 % of the configured rate at `--sync-hz 50`, the default rate (C-36), which lies in the accepted range 10 to 200/s (C-112). 1000/s is outside that range and has no target.
- Depends on 003-02 for the measurement method used in criterion 3.

## Work
1. In `spawn_clock`, replace the fixed 5 ms receive timeout with a wait bounded by `min(next request deadline − now, 100 ms)`, e.g. `ppoll` on the clock socket with a timeout computed from `next − now`, following the data thread pattern at `module.rs:1663-1667`.
2. Keep a **100 ms ceiling** on any single wait so the stop flag is still noticed.
3. When the wait returns readable, receive with `net::recv_ts` without blocking for an extra timeout; when it times out at the deadline, send the next request.
4. Do not change the request/reply message format, timestamps taken, or servo code.

## Acceptance criteria
1. No fixed 5 ms read timeout remains on the clock socket; every wait in the clock loop is bounded by the next request deadline and by 100 ms.
2. Setting the stop flag ends the clock thread within 100 ms + one loop iteration (checked by a run with `--duration` that exits promptly).
3. With `--sync-hz 50`, the achieved rate is between 49.0 and 51.0 exchanges/s (epic 003-AC3), measured with the method recorded in 003-02 (the formal before/after record is 003-05).
4. `git diff` of this ticket touches only the clock thread wait code; wire format, servo and analyzer files are unchanged.
5. The two existing tests in `transport.rs` still pass.

## Definition of done
- [ ] Tests green in CI, unprivileged (no CI exists: `cargo test`, unprivileged)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No change to the wire format, the servo algorithm or the analyzer
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Recording the before/after figures and the lock check (003-02, 003-05).
- Any servo redesign (the runtime's servo is a redesign tested in task 008, which also checks system criterion AC-7 over the C-112 range).
- Asserting sync rate in `tools/smoke.sh`: smoke assertions must never depend on sync-rate timing (003-07).
