# 021-28 — Sync exchange building and step filter
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-05, epic 015, epic 016, epic 018
## Context
Loop 1 of the shared two-loop timebase (epic 018) maps the host clock H to the domain line through sync exchanges, 50 requests per second (C-36), within the accepted range of 10 to 200 per second (C-112).

- Home of step detection [decision, C-121]: the platform backend that builds each exchange from receive stamps is the home of wall-clock step detection; the servo only ever receives exchanges that passed it. On macOS that backend is this crate.
- On macOS both stamp sources are on the mach-time basis (kernel monotonic stamps or a userspace mach read, 021-05) [memory], so a wall-clock step cannot reach them directly. The filter's job here is to guarantee that: reject any exchange whose stamps are not all on the H basis, whose source changed mid-exchange (kernel to userspace fallback), or that straddles a sleep (mach time stops in sleep [memory]; on wake both loops are reset, C-124).
- The exchange type and the hand-off to loop 1 come from the hosted trait surface (epics 015, 016) and epic 018.
## Work
- In `net.rs`, build each sync exchange from the request send time and the reply receive stamp (021-05), tagging each stamp with its source.
- Filter: drop an exchange with mixed or unknown basis, with a source change inside it, with a negative or implausibly large round trip, or begun before the last wake; count drops by reason.
- Pass only surviving exchanges to loop 1.
## Acceptance criteria
1. Unit test: exchanges with consistent H-basis stamps pass unchanged.
2. Unit test: an exchange with one stamp from another basis, one crossing a forced kernel-to-userspace fallback, and one begun before a simulated wake are each dropped and counted under their reason.
3. Changing the system wall clock during a live run produces no step in loop 1's input (privileged test, run in the privileged nightly job, C-146).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
## Out of scope
The servo and loop 1 (epic 018). Receive stamping (021-05). Sleep handling (021-18).
