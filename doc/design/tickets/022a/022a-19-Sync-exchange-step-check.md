# 022a-19 — Sync-exchange building and wall-clock step check
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-02, 022a-04, epic 016, epic 018

## Context
- Part of epic 022a (Windows edge backend), `net.rs`.
- Ruling C-121: wall-clock step detection lives in the platform backend that builds each sync exchange from receive stamps; the servo (epic 018) only ever receives exchanges that passed it.
- On Windows the host clock H is QPC (022a-02) and receive stamps are QPC too, either from the kernel timestamping control or from a userspace QPC read just after the receive returns (022a-04, C-131). The check therefore guards against stamps that are not in QPC units or that are inconsistent with H, rather than against a stepped wall clock.
- The epic states the check: reject a stamp that is not in QPC units, that is later than the userspace QPC read taken just after the receive returned, or that runs backwards against the previous stamp on the same socket.
- Sync requests run at 50/s (C-36), within the holder's accepted 10 to 200 per second (C-112); DSCP sync 48 (C-13).
- The exact exchange type and trait signature come from epics 016 and 018 (open sub-question owned there).

## Work
1. Build each sync exchange from the send and receive stamps of the request and reply, all in QPC ns.
2. Step check per the rule above; a failing exchange is dropped, counted in a named diagnostic counter and never passed to the servo.
3. Expose the counter for the stamp diagnostics (022a-17).
4. Unit tests with synthetic stamps: in-order, kernel stamp after the userspace read, backwards stamp, non-QPC source.

## Acceptance criteria
1. **(022-AC-14.)** Each synthetic failing case is rejected and counted; no rejected exchange reaches a mock servo.
2. Over 10 000 exchanges on loopback with consistent stamps, none is rejected.
3. The exchange builder is the only path from receive stamps to the servo in the crate (code review).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The servo and its step threshold (C-67, epic 018).
- Receive-timestamp capture itself (022a-04).
