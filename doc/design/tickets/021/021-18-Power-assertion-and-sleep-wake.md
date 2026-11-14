# 021-18 — Power assertion and sleep/wake handling
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-15, 021-16
## Context
`power.rs`. Porting trap [memory]: the machine **sleeps on lid close**. A live edge must hold a power assertion. Mach absolute time stops during sleep [memory].

- **Sleep is a departure** [decision, C-124]: both timebase loops are reset and the edge returns through normal admission (Locking, then Aligning). No clock-loop state is carried across sleep, so the stopped mach clock needs no compensation.
- Liveness as peers see it: interrupted after 8 consecutive missed slices (C-38), counting only slices absent at the deadline (C-102); dead 2 s after the last slice received with content or a marked gap (C-39, C-111).
- Time back: Locked within the acquisition time of C-63 (at most 10 s from the first exchange; pending simulation), then Aligning for 4 s (C-50).
- A capture edge is a source: after wake its streams carry new session numbers (C-19).
- Certification across sleep [decision, C-180]: a *commissioned* sink stays commissioned while its clock is unlocked; its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning. The lock state is the servo core's, read through the published time view (C-187).
- Epic criterion 021-AC4: lid close with a live stream: peers classify the edge interrupted then dead; on wake the edge resets both loops and re-admits through Locking and Aligning without process restart. With the power assertion held, idle sleep does not occur.
## Work
- Hold an idle-sleep-preventing power assertion while any connection (sink or capture) is live; release when the last one ends.
- Subscribe to system sleep/wake notifications: on will-sleep, end all streams cleanly; on did-wake, reset loop 1 and loop 2 (epic 018 reset entry point) and start re-admission through core's normal admission path, without process restart.
## Acceptance criteria
1. Manual hardware test: with a live stream and the assertion held, the machine does not idle-sleep over 2× the configured idle-sleep timeout.
2. Manual hardware test: lid close with a live stream; a Linux peer logs the edge interrupted (after 8 missed slices) then dead (2 s after the last slice with content or a marked gap).
3. On wake, the edge passes through Locking and Aligning and the stream resumes without restarting the process; the log shows both loops reset, Locked reached within C-63 of the first exchange, and Aligning lasting C-50.
4. Unit test: assertion count follows live-connection count 0→1→2→1→0.
5. Unit test with a fake notifier: did-wake calls the loop reset before any new exchange is accepted.
6. A sink *commissioned* before sleep stamps *commissioned* after wake without re-commissioning; its alignment claim is suspended from wake until Locked and then resumes (C-180).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
Admission protocol itself (core). Clock loops and their reset implementation (epic 018).
