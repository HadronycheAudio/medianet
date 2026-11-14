# 021-02 — Mach host clock
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-01, epic 015
## Context
The macOS backend must provide the host clock H used by loop 1 of the shared two-loop timebase (built in epic 018): loop 1 maps H to the domain line through sync exchanges. A platform supplies a host clock with a declared jitter.

- Mechanism [memory]: mach absolute time. Ticks are **not** nanoseconds on Apple Silicon; convert with the mach timebase ratio (numer/denom). Mach absolute time **stops during sleep**.
- Sleep is a departure [decision]: both timebase loops are reset on wake and the edge returns through normal admission (C-124). The clock therefore needs no compensation for the time lost in sleep; it only has to stay monotonic.
- No design value exists for `declared_jitter_ns`; it is set from the measurement made in this ticket.
- Check given by the epic: unit test against a second clock.
- The media deadline clock exists in core only as an abstract trait; this backend provides it from mach host time [decision].
## Work
- `clock.rs`: `pub struct MachClock`, `impl HostClock for MachClock { now_ns, declared_jitter_ns }` (trait from epic 015).
- Read the timebase ratio once at construction; convert ticks to ns without overflow (128-bit intermediate).
- Implement core's abstract media deadline clock trait on the same mach time source.
- Choose and document `declared_jitter_ns` from a measurement made in this ticket (record value with `[measured]` tag in a doc comment).
- Unit tests: monotonicity; rate against a second clock (e.g. `CLOCK_UPTIME_RAW` / `std::time::Instant`) over at least 1 s; conversion with a synthetic non-unit ratio (e.g. 125/3).
## Acceptance criteria
1. `now_ns` is non-decreasing over 10^6 consecutive calls.
2. Over a 1 s interval, `now_ns` deltas agree with the second clock to within 1 µs plus the declared jitter.
3. Conversion is correct for ratios 1/1 and 125/3 (unit test with injected ratio), no overflow at 2^63 ticks-equivalent of 100 years uptime.
4. The documented behaviour "stops during sleep" is stated in the doc comment as `[memory]` until confirmed by 021-25, with a pointer to C-124 (loops reset on wake).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Sleep/wake handling (021-18). Device clock S (021-10). Receive timestamps (021-05).
