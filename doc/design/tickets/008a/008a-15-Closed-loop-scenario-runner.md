# 008a-15 — Closed-loop scenario runner
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** M · **Depends on:** 008a-02, 008a-03, 008a-05, 008a-06, 008a-08, 008a-09, 008a-11
## Context
Epic 008 drives the servo "entirely by a deterministic simulation" in the hosted, dev-only `sim` module of the `hadronyche-tools` crate. The models (two clocks, link, wall clock and simulated backend), the recorder, the pacer and the servo exist as separate pieces; this ticket wires them into one closed loop that every simulation case (008b-01 to 008b-16, 008b-19) runs.

Sync protocol `[decision]`: the follower sends requests (50 per second by default, C-36; accepted range 10 to 200, C-112) to the holder's sync port; the reply echoes the request's send timestamp t1 plus holder receive t2 and send t3; the follower stamps t4 on receipt. Every reply carries the timeline epoch. The platform backend builds the exchange and discards samples hit by a wall-clock step before the servo (C-121). Published view goes through the `PublishedView` seqlock.

## Work
1. Event loop: `RequestPacer::next_deadline` (follower local time) schedules requests; link model delivers request to holder; holder stamps t2/t3 from its clock model; link delivers reply; the simulated backend (008a-08) yields an `Exchange` or discards the sample; `Servo::feed`; `Servo::tick` at a configured tick period; `PublishedView::write`; `Recorder` samples via `PublishedView::read` plus the servo's correction term.
2. Scheduled scenario events: holder clock step, epoch change, link parameter change (bursts on/off), wall-clock step, link outage (for holdover).
3. `run(Scenario) -> (Trace, Summary)` with every parameter in plain data and a seed.
4. A smoke test scenario: ideal link, +10 ppm drift, 60 s.

## Acceptance criteria
1. Smoke scenario reaches `Locked` and its summary shows 0 honest-lock violations and 0 ns phase steps while Locked.
2. Running the smoke scenario twice with the same seed gives identical trace digests on one machine.
3. A 60 s scenario at 50 req/s runs in under 5 s wall time in CI (document the actual figure).
4. Every scheduled event type has a unit test showing it fires at the scheduled simulated instant.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Multi-holder scenarios (008a-16); the named cases (008b-01 onward); cross-machine determinism (008b-17).
