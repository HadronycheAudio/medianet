# 014-09 — Probe one-way figures
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-08, epic 008a
## Context
A probe exchange gives four timestamps `t1..t4` (see 014-08). One-way figures derived from them `[decision]`:
- are valid **only while the clock's time is usable**, and are withheld otherwise. Time is usable while Locked and in holdover until the alignment claim lapses (about 25 s, C-65; the published bound grows by 1 µs per second meanwhile, C-64). For a device whose E is unknown, the lapse is when its published bound exceeds 50 µs (twice the lock bound), also about 25 s into holdover (C-169); the time view of epic 008a applies it, and the caller's `ClockStatus` reflects it. After the lapse the clock is Unlocked with time unusable, and one-way figures stop (C-162). `ClockStatus::locked` means "time usable" in this sense;
- carry the published clock bound;
- carry the worst-case asymmetry bound. No in-band method can observe path asymmetry; the forward/return split is accurate to half the difference between the two directions. Each one-way delay lies between 0 and the round trip, so the worst case is **half the round trip**; the epic settles this as the bound carried.

Forward one-way = `t2 - t1`, return one-way = `t4 - t3` (meaningful only when both clocks share the domain timeline, that is, while time is usable). These figures feed the ladder (epics 019, 020). The receive stamps `t2` and `t4` come from the platform backend; its receive-timestamp source (kernel, userspace, mac-hardware or interrupt) is reported in the regime stamp with a capability state (C-129, C-131), not by this ticket.

Settled (C-187, closing epic open sub-question 9): the servo core, epic 008a, publishes the lock state and the clock bound through the published time view. The caller fills the `ClockStatus` input below from that view; this ticket does not read the clock itself.

Shapes from the epic:
```rust
pub struct ClockStatus { pub locked: bool, pub bound_ns: u32 }
pub struct OneWay { pub forward_ns: i64, pub return_ns: i64, pub clock_bound_ns: u32, pub asymmetry_bound_ns: i64 }
impl Probe { pub fn one_way(&self, clock: ClockStatus) -> Option<OneWay>; }
```

## Work
1. Implement `ClockStatus`, `OneWay` and `Probe::one_way`.
2. Return `None` when `locked` is false.
3. Fill `asymmetry_bound_ns` with `round_trip_ns() / 2` and copy `bound_ns` into `clock_bound_ns`.
4. No allocation, no panics; constant time.

## Acceptance criteria
1. With `locked = false` (Unlocked, including after the holdover claim lapses, C-162), `one_way` returns `None` for any probe (epic AC-9, withholding part).
2. With `locked = true` and injected delays `d_fwd = 137 000` ns, `d_ret = 211 000` ns, it returns `forward_ns = 137 000`, `return_ns = 211 000` exactly, with the given clock bound copied through and `asymmetry_bound_ns = 174 000` (round trip 348 000 / 2).
3. Negative computed one-way values (clock error larger than the path delay) are returned as computed, not clamped, and a test documents that case.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The clock servo, lock detection and the published time view (servo core, epic 008a, C-187).
- Use of the figures by the ladder (epics 019, 020).
