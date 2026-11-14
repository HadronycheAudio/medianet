# 006-02 — Position map arithmetic
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-01
## Context
Part of `doc/design/spec/timeline.md` (epic 006). This ticket writes the "Domain line and position map" section: the exact arithmetic of the position ↔ network-time map at the domain rate, the rounding rule for positions off a slice boundary, the behaviour before the origin, and the representation of `epoch_origin`. It answers the epic's open sub-questions Q-1 and Q-2.

Facts to restate in the section (all `[decision]`):
- There is **one timeline for the whole domain**. The wire carries one sample rate only, 96 000 Hz (C-01); every stream on a medianet is at the domain rate. Position is network time counted in domain-rate samples from a domain epoch origin: `position = (network_time − epoch_origin) × 96 000 Hz`.
- There is no position space at any other rate. An edge whose hardware runs at 48 kHz converts 2:1 through the kit's resampler node (C-80); a resampler node may appear only inside an edge module (C-134), and its delay is part of D, never of L (C-207). The rate is a constant of the map, not a field of `Line`.
- Streams have no private origins. Any two sources share one position space, so a mixer has a single timeline to stamp its output on.
- A capture edge stamps each sample with the position of its acoustic instant.
- Positions are 64-bit. A slice is 48 frames, 0.5 ms, 2 000 slices/s at defaults (C-02).
- The map is **affine per timeline epoch and never extrapolated across epochs**.
- Sources pace position from network time, not from their local crystal.

Useful numbers: a slice boundary is every 48 samples = 500 000 ns exactly, so slice-boundary positions map to whole nanoseconds without rounding. One sample is 31 250 / 3 ns = 10 416⅔ ns, so a position off a slice boundary (a sample inside a slice, as a capture edge stamps) is not an integer number of nanoseconds and the rounding direction matters.

Open sub-questions answered here:

| Id | Question | Owner | Falsifier |
|---|---|---|---|
| Q-1 | Which rounding rule maps positions off a slice boundary to nanoseconds and back? | This ticket | `position_at(time_of(p)) ≠ p` for some position `p` |
| Q-2 | What does `position_at` return for a network time before `epoch_origin`? | This ticket | Two implementations return different results for the same time before the origin |

The Rust signatures that will implement this section (ticket 006-08) are:
```rust
pub const DOMAIN_RATE_HZ: u32 = 96_000; // C-01
pub struct Line { pub epoch: TimelineEpoch, pub origin_ns: i64 }
impl Line {
    pub fn position_at(&self, network_ns: i64) -> u64;
    pub fn time_of(&self, position: u64) -> i64;
}
```

## Work
1. Specify `epoch_origin` representation: signed 64-bit nanoseconds of network time (`origin_ns: i64`), the network-time scale being the holder's. State its range and why i64 is sufficient.
2. Specify the exact integer arithmetic, with a wide (128-bit) intermediate so that `(network_ns − origin_ns) × 96 000` cannot overflow. Proposed rule, to confirm and tag `[decision]`:
   - `position_at(t) = floor((t − origin_ns) × 96 000 / 10^9)` = `floor((t − origin_ns) × 12 / 125 000)`
   - `time_of(p) = origin_ns + ceil(p × 10^9 / 96 000)` = `origin_ns + ceil(p × 31 250 / 3)`
   - Consequence to state: `position_at(time_of(p)) == p` for every `p`, and `time_of(position_at(t)) ≤ t`; both are exact equalities on slice boundaries.
   If a different rule is chosen, it must still give exact inverses at every slice boundary.
3. Specify the behaviour of `position_at` for `t < origin_ns` (position is unsigned): either a defined error/`Option` or a clamp; choose one and record it as `[decision]` (closes Q-2).
4. State that the map is never extrapolated across epochs: a position computed on epoch N's line is meaningless on epoch N+1's line.
5. State that no other rate has a position space (C-01), and that mapping a device clock to domain positions is the edge's work, outside this section: its steerable rate actuator tracks the hardware crystal and, for 48 kHz hardware, the fixed 2:1 node converts the rate (C-80, C-189); the kit (epic 024a) builds both (C-197), and their delay is part of D, never L (C-207).
6. Add a table of worked examples (at least 6 rows at 96 kHz: three on slice boundaries, three off them, for example `p = 1` → `origin + 10 417 ns`, `p = 2` → `origin + 20 834 ns`, `p = 3` → `origin + 31 250 ns`, and the reverse direction for a time inside a sample) that 006-08 turns into unit tests.

## Acceptance criteria
1. The section gives formulas for both directions, the rounding direction of each, and the intermediate width.
2. The section proves (or shows by bound) that `position_at(time_of(p)) == p` for every position on a slice boundary at 96 kHz (006-AC-1), and states which other inverse identity holds.
3. The rounding rule for positions off a slice boundary is stated and illustrated with at least three examples (006-AC-1).
4. The behaviour before the origin is specified (Q-2 closed).
5. The worked-example table has at least 6 rows, each with input and exact output.
6. The section states that 96 kHz is the only rate with a position space (C-01, C-80, C-134).

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- How `origin_ns` is chosen at cold start and after a bump (006-04).
- The Rust implementation and its tests (006-08).
- Sync reply wire format (epic 004).
- Sample-rate conversion at edges: the kit (epic 024a) builds the resamplers and every edge uses them (C-80, C-189, C-197).
