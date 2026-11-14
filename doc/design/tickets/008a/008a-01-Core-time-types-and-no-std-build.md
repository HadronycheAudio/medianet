# 008a-01 — Core time types and no_std build
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** epic 004, epic 006
## Context
Epic 008 builds a new clock servo and its seqlock publication in the `hadronyche-core` crate, validated by a deterministic simulation. This ticket lays down the shared data types every later ticket uses, and the build check that keeps the servo and seqlock usable on embedded followers.

- The holder's monotonic clock defines network time. A follower sends sync requests (50 per second by default, C-36; accepted range 10 to 200 per second, C-112) to the holder; each reply echoes the request's send time plus the holder's receive and send times, so each reply is a self-contained four-timestamp exchange `[decision]`. Every reply carries the timeline epoch, 16 bits wide (C-10) `[decision]`.
- Exchanges reach the servo already built by the platform backend, which has discarded any sample corrupted by a wall-clock step (C-121). The servo never parses wire messages.
- The sync exchange fields come from epic 004; the `TimelineEpoch` type comes from epic 006.
- The servo and seqlock must compile for a `no_std` target with no allocator (epic criterion A9). The servo is "fixed size, no allocation". Any bare-metal target without an allocator proves A9; this ticket picks one and records it.

## Work
In `hadronyche-core` (names may be refined, shape fixed by the epic):

```rust
pub struct Exchange { pub t1: i64, pub t2: i64, pub t3: i64, pub t4: i64, pub epoch: TimelineEpoch }

pub struct TimeView {
    pub offset_ns: i64, pub at_local_ns: i64, pub rate_ppb: f64,
    pub state: LockState, pub bound_ns: i64, pub epoch: TimelineEpoch,
}
pub enum LockState { Unlocked, Acquiring, Locked, Holdover }

pub struct ServoConfig { /* fields added by 008a-10..008a-14 */ }
```

1. Add a `time` (or `servo`) module with the types above; derive `Clone`, `Copy`, `Debug`, `PartialEq` where possible. `t1` = follower send, `t2` = holder receive, `t3` = holder send, `t4` = follower receive, all in ns; document this on the type, with the formulas from the epic: round trip = (t4 − t1) − (t3 − t2), offset = ((t2 − t1) + (t3 − t4)) / 2.
2. Import `TimelineEpoch` from the epic 006 deliverable (do not redefine it).
3. Make the module `#![no_std]`-compatible: no `std`, no `alloc`, no heap types. Gate any `std` use behind a feature that is off by default.
4. Declare `ServoConfig` with the field list of the epic, as placeholders filled by later tickets: measurement window (C-66), slew limit (C-60), rate-of-change limit (C-61), lock bound (C-62), E for the lapse (the Linux reference sink target C-70, never a measured E, C-149; optional, and when unknown the lapse uses 2 × the lock bound, C-169), holdover growth (C-64), step threshold (C-67), holdover entry delay (at most C-37, a hard limit, C-195; the value within it pending simulation), worst-case asymmetry bound, loop gains.
5. Add a CI job that builds `hadronyche-core` (servo and seqlock modules) for one bare-metal target with no allocator, and record the chosen target triple in the crate README.

## Acceptance criteria
1. `Exchange`, `TimeView`, `LockState` (exactly the four variants `Unlocked`, `Acquiring`, `Locked`, `Holdover`) and `ServoConfig` exist and are public.
2. `cargo build -p hadronyche-core --no-default-features --target <bare-metal target>` succeeds in CI with no `alloc` crate linked.
3. A unit test constructs an `Exchange` and a `TimeView` and round-trips them through `Copy`.
4. The CI job fails if any `alloc` or `std` dependency is introduced into the servo or seqlock modules (verified once by a deliberate local break).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Target triple of the no_std build documented in the crate README

## Out of scope
- Seqlock (008a-02), pacer (008a-03), servo behaviour (008a-10 to 008a-14).
- Parsing sync messages off the wire (epic 004) and building exchanges from socket stamps (platform backend, C-121).
