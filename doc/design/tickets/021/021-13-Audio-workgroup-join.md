# 021-13 — Audio device workgroup join
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** XS · **Depends on:** 021-09, 021-10
## Context
Mechanism [memory]: real-time threads use the time-constraint policy and **join the audio device's workgroup** so the scheduler coordinates them with the device I/O thread. Part of `rt.rs` (`OsRealtime`, epic 015).
## Work
- Obtain the open device's I/O thread workgroup and join every edge real-time worker thread to it in `setup_thread` when a device is given; leave it on thread exit.
- Record "workgroup joined: yes/no (+error)" in the capability report of 021-09.
## Acceptance criteria
1. Manual hardware test: with a device open, worker threads report joined; the capability report shows "workgroup joined: yes".
2. If no device is open or the join fails, the thread still runs with time-constraint policy and the report shows "no" with the error.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
Time-constraint policy and wake-latency measurement (021-09).
