# 022a-09 — Power: stay awake while live; sleep is a departure
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-01, epic 005, epic 018

## Context
- Part of epic 022a (Windows edge backend), `power.rs`: stay-awake request and power-throttling opt-out while live; suspend is a departure, resume returns through normal admission.
- Porting trap `[memory]`: sleep, modern standby and background power throttling end or degrade a live edge; request to stay awake while live and mark the process as audio work.
- Ownership is settled in the epic: `power.rs` owns stay-awake, the power-throttling opt-out and suspend/resume; `rt.rs` (022a-07) owns timer resolution and MMCSS.
- **Sleep is a departure** (C-124): both timebase loops (epic 018) are reset and the edge returns through normal admission, Locking then Aligning. There is no separate re-admission target: the return takes the acquisition time to Locked (C-63, pending simulation) plus the aligning duration (4 s, C-50).
- The edge returns in the same process, so its module id and instance nonce are unchanged and third parties have no colliding nonce to quarantine (C-46).
- A capture edge bumps the session number of each of its streams on return, as on any restart (C-19).
- Rules that apply unchanged: a stream is interrupted after 8 consecutive missed slices (C-38) and dead after 2 s (C-39, counted from the last slice with content or a marked gap, C-111); a producer stops after 2 s without receiver reports (C-41).
- Whether a departing edge announces its departure (so peers see an end rather than dead after 2 s) is an open sub-question owned by epic 005. Until it is answered, this ticket ends streams through the edge's normal end path when the OS gives notice.

## Work
1. `power.rs`: `StayAwake` guard — while any stream is live, request system stay-awake (execution-required, display-independent) and opt the process out of background power throttling; released when the last stream ends.
2. Subscribe to suspend / resume notifications.
3. On suspend notification: end every live stream through the edge's normal end path (and the departure message if epic 005 defines one) before the system sleeps.
4. On resume: reset both timebase loops (epic 018), re-open sockets and devices, bump each capture stream's session number (C-19), and go through admission again (Locking, Aligning) without restarting the process.
5. Tests: guard reference counting (unit); resume path with a mocked notification (unit: loops reset, sessions bumped); manual suspend/resume on reference hardware.

## Acceptance criteria
1. While a stream is live, the system idle sleep timer does not suspend the machine (manual: idle timeout set to 1 min, live for 5 min, no sleep).
2. When no stream is live, the stay-awake request and the throttling opt-out are released (unit test on the guard; manual check with the OS power-request listing).
3. On a forced suspend with OS notice, the edge ends its streams through the normal end path; the behaviour peers observe is recorded `[measured]`.
4. **(022-AC-11.)** After resume, both loops have been reset, the edge passes through Locking and Aligning and is playing again without a process restart; capture streams carry a new session number; the time to return is recorded `[measured]` against C-63 plus C-50.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The power fact is confirmed `[measured]` or corrected, for 022b-08

## Out of scope
- MMCSS and timer resolution (022a-07).
- The departure message and how peers treat it (epic 005).
