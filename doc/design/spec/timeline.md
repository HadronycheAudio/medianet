# Domain timeline and epochs

Status: normative, epic 006.

This document specifies the domain timeline and its epochs: how a position maps to network time, what bumps the timeline epoch, how each party learns of a new epoch, how sources switch at an announced position, and what a fan-in does with inputs on different epochs. Later tickets of epic 006 each fill one section of this document. The file lives in `doc/design/spec/`, beside the design documents (C-130).

Hadronyche Medianets are wired, isolated IPv4 LANs (C-100) whose only traffic is this runtime's. Modules discover each other, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone and the only clock holders in v1; macOS and Windows machines are edges; embedded devices are followers. The wire carries one sample rate only, 96 kHz (C-01); an edge whose hardware runs at another rate converts at the edge (C-80), and a resampler node may appear only inside an edge module (C-134); its delay is counted in D, never in L (C-207).

Evidence tags used in this document: `[code]` read in the POC, `[decision]` design choice (plus `[measured]`, `[kernel-doc]`, `[memory]` if a later section needs them, and `[standard]` for a figure cited from a named, published standard with its clause, which replaces `[memory]` once the citation is checked, C-132).

## Constants used by this document

Every later section refers to constants by id. The values and statuses below are copied from the design's constants table, [02-constants.md](../02-constants.md). If a value in this document differs from that table, the constants table wins.

| ID | Constant | Value | Status |
|---|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, and it is the only rate on the wire: every stream on a medianet is at the domain rate. An edge whose hardware runs at another rate (typically 48 kHz) converts at the edge | decided |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s | inherited |
| C-10 | Stream number / session / timeline epoch widths | 16 / 32 / 16 bits | decided |
| C-18 | Regime stamp transport | a dedicated control message split into sections that each fit C-17. Multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number | decided |
| C-19 | Session number ownership | assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value | decided |
| C-30 | Hello period | 1 s | inherited |
| C-33 | Election window | 400 ms | inherited |
| C-37 | Election on reply silence | 1.5 s | decided |
| C-38 | Interrupted threshold | 8 consecutive missed slices (4 ms at defaults) | decided |
| C-43 | In-band announcement lead | 64 slices (32 ms at defaults), flat | decided |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member | decided |
| C-47 | Third-party observability of stream health | producer and consumer each publish per-connection summary counters in their own regime stamp, one counter per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP; receiver reports are not multicast and there is no query message | decided |
| C-61 | Rate of change of rate | at most 1 ppm per second | pending-simulation |
| C-64 | Holdover bound growth | 1 µs per second (1 ppm floor) | decided |
| C-65 | Alignment claim lapse in holdover | about 25 s: (C-70 − C-62) / C-64 | derived |
| C-67 | Step threshold | 2 ms | inherited |
| C-78 | L2 handover test | crystal difference of 50 ppm between old and new holder | decided |
| C-80 | Edge hardware at 48 kHz | the edge converts 2:1 through the kit's resampler node; the wire stays at the domain rate (C-01) | decided |
| C-102 | What counts as a miss for the interrupted threshold (C-38) | only slices absent at the deadline: missing, or arriving late. A marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count | decided |
| C-105 | Representation of structural latency L | always a whole number of domain-rate samples. The kit's resampler node is offered only in designs whose delay is a whole number of samples (2:1 and 1:2); other ratios are out of v1. The earlier rational-L rule is withdrawn | decided |
| C-111 | Start of the dead threshold (C-39) | 2 s counted from the last slice received with content or a marked gap | decided |
| C-113 | Election when no candidate has usable time | the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch. "Usable time" is defined in C-213 | decided |
| C-120 | Test-only injection hooks, beyond the public injection API | force a module id and nonce; force a stream onto a given timeline epoch; send a raw packet on a media or control socket. Stamped like any injection | decided |
| C-121 | Home of wall-clock step detection | the platform backend that builds each exchange from receive stamps; the servo only ever receives exchanges that passed it | decided |
| C-122 | Degraded flag on an output with latency L | follows the content. Stamps denote content position: while a module processes input position p it emits the output stamped p − L, whose content came from input position p − L. That output is degraded if the input at p − L was gapped, so the flag is delayed by L together with the content | decided |
| C-124 | Laptop edge sleep | sleep is a departure: both timebase loops are reset and the edge returns through normal admission (Locking, Aligning) | decided |
| C-134 | Where a resampler node may appear | only inside an edge module; the compiler rejects it anywhere else. An edge's conversion delay is part of its plumbing delay D and is never counted in L (C-207) | decided |
| C-135 | A slice on another timeline epoch | restarts the dead clock (C-111), does not count as a miss (C-102), and has its own per-connection counter | decided |
| C-136 | Holder stall | the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch | decided |
| C-137 | Header cap | a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path | decided |
| C-159 | Per-connection counters in the stamp (completes C-47) | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead | decided |
| C-141 | Transport state on a gap | the last known state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo. The output is marked degraded | decided |
| C-162 | Clock state after the holdover claim lapses (C-65) | Unlocked, with time unusable: one-way figures and alignment claims stop | decided |
| C-169 | Alignment claim in holdover when E is unknown | the lapse rule C-65 needs a fixed reference, so an unknown E is taken as twice the lock bound (2 × C-62, that is 50 µs): the device stops claiming alignment when the bound it currently publishes exceeds that value, about 25 s into holdover at the growth of C-64 | decided |
| C-181 | Where controllers travel | a bundle source's controllers always ride in their own member stream (C-205). A single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The compiler checks this from the manifest | decided |
| C-184 | Transport state before any has arrived | stopped, playhead at zero, tempo 120 beats per minute (C-199) | decided |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view; every epic that reports one-way figures depends on it | decided |
| C-188 | Where the holder answers sync requests | on the clock thread, never on housekeeping; this is also where it records the DSCP observed on each follower's requests (C-152) | decided |
| C-189 | Two kinds of resampler | the DSL resampler node is fixed-ratio, 2:1 or 1:2, with whole-sample delay (C-105). An edge's rate actuator is a separate, steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal; its delay is part of D. The ABI keeps the "frequency-dependent delay" property on outputs, for recursive filters | decided |
| C-195 | Holdover entry delay | at most the election silence threshold (1.5 s, C-37): a hard limit, so a follower is always in holdover with a growing bound when it calls an election. The value within that limit is pending simulation (epic 008b proposes it; the POC used 2 s). If no value at or below 1.5 s works, that is reported as a finding | pending-simulation |
| C-197 | Who builds the resamplers | the arithmetic kit (epic 024a) builds both the fixed 2:1 / 1:2 node and the steerable rate actuator, once. Epic 024a moves to phase P4, ahead of the Linux edge; every edge (Linux, macOS, Windows, embedded) uses the kit's and builds none of its own | decided |
| C-200 | Playhead through a gap while stopped | does not advance (see C-141) | decided |
| C-202 | Sync loss at L2 | no separate scenario. The handover run gains one assertion: emission error stays within the target E while the sink's clock is in holdover between holders. The loss scenario stays on the media path | decided |
| C-205 | Controllers of a bundle source | always in their own member stream | decided |
| C-207 | Resampler delay and L | an edge's resampler delay is counted only in D and never in L. L is the module's structural delay in the domain-rate timeline; the manifest reports the conversion delay separately, on the device-facing port | decided |
| C-209 | Controller member and the bundle cap | the controller member counts towards the 32 of C-08: a source with controllers has at most 31 audio members (186 channels). A wider source with controllers is rejected at compile time | decided |
| C-212 | How the L2 and L3 handover is triggered | as a holder loss: the old holder stops answering, the sinks pass through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. At L2 the run asserts each sink's emission error within the target E during holdover (C-202) and frequency continuity at takeover (rate change within C-61, no phase step). At L3 the rig sees only the difference between the sinks, so it asserts what C-220 states. A run in which holdover was not observed is failed, not passed. The rig therefore needs a second holder-eligible Linux node | decided |

## Retired POC behaviour

`[code]` In the POC the media clock is the sender's local crystal: the data-plane ticker runs on local monotonic time (`module.rs:1630-1634`), and each stream carries its own `sample_pos` origin. Both are retired.

## Domain line and position map

This section is written in 006-02. It closes the epic's open sub-questions Q-1 (the rounding rule off a slice boundary) and Q-2 (behaviour before the origin). Notation: `t` is a network time in nanoseconds, `p` a position, `origin_ns` the epoch origin of the line in question.

### One timeline, one position space

There is one timeline for the whole domain `[decision]`. The wire carries one sample rate only, 96 000 Hz, and every stream on a medianet is at the domain rate (C-01); the rate is a constant of the map, not a field of `Line` `[decision]`. There is no position space at any other rate: 96 kHz is the only rate with a position space (C-01) `[decision]`. Position is network time counted in domain-rate samples from a domain epoch origin: `position = (network_time - epoch_origin) × 96 000 Hz` `[decision]`.

An edge whose hardware runs at 48 kHz converts 2:1 through the kit's resampler node (C-80) `[decision]`, and a resampler node may appear only inside an edge module (C-134) `[decision]`. Mapping a device clock to domain positions is the edge's work and is outside this section: the edge's steerable rate actuator tracks the hardware crystal (C-189), and the kit (epic 024a) builds both the fixed 2:1 / 1:2 node and the actuator, so every edge uses the kit's and builds none of its own (C-197); their delay is part of D, never of L (C-207) `[decision]`.

Streams have no private origins: any two sources share one position space, so a mixer has a single timeline to stamp its output on `[decision]`. A capture edge stamps each sample with the position of its acoustic instant `[decision]`. Sources pace position from network time, never from their local crystal `[decision]`; the POC's local-crystal pacing is retired (`[code]`, the Retired POC behaviour section of this document). Positions are 64-bit `[decision]`; a slice is 48 frames, 0.5 ms, 2 000 slices/s at defaults (C-02).

### The epoch origin

`epoch_origin` is held as `origin_ns: i64`, nanoseconds of network time on the holder's scale `[decision]`. Its range is the full span of an i64 nanosecond count. i64 is sufficient because network time itself is a signed 64-bit nanosecond count on the holder platforms of v1 — a Linux host, the only kind of clock holder, keeps time as `ktime_t` `[kernel-doc]` — so the origin can name any instant network time can name and no wider representation is needed `[decision]`. How the value is chosen at cold start and after a bump is 006-04's work, not this section's.

The map is affine per timeline epoch and never extrapolated across epochs `[decision]`: a position computed on epoch N's line is meaningless on epoch N+1's line, because each epoch has its own origin. What bumps the epoch is 006-05's work.

### The map in both directions

The map and its rounding rule (Q-1) `[decision]`. Both directions are a single integer division of a product, and the product is computed in a 128-bit signed intermediate (`i128`) before the division `[decision]`:

- `position_at(t) = floor((t - origin_ns) × 96 000 / 10^9) = floor((t - origin_ns) × 12 / 125 000)` — round down (floor): 96 000 / 10^9 reduces to 12 / 125 000, and the quotient is floored to a whole position.
- `time_of(p) = origin_ns + ceil(p × 10^9 / 96 000) = origin_ns + ceil(p × 31 250 / 3)` — round up (ceil): 10^9 / 96 000 reduces to 31 250 / 3, and the offset is ceiled to a whole nanosecond.

The reduced fractions equal the unreduced ones, so either form may be implemented; the width argument is stated for the unreduced products. The 128-bit intermediate is what keeps `(network_ns - origin_ns) × 96 000` from overflowing: the difference of two i64 values is below 2^64, so that product is below 2^64 × 96 000 < 2^81, and `p × 10^9` with `p` in the full u64 range is below 2^64 × 10^9 < 2^94; both are far inside the i128 bound of 2^127, so neither product can overflow `[decision]`. 006-08 tests the full i64 `origin_ns` range combined with `p < 2^40`.

### The inverse identities (006-AC-1)

Both identities are exact properties of the rule above and hold everywhere on the line, not only on slice boundaries `[decision]`:

- For every position `p >= 0`: `position_at(time_of(p)) == p`.
- For every network time `t >= origin_ns`: `time_of(position_at(t)) <= t`, with equality whenever `t` is a time the map itself names, in particular on every slice boundary.

By bound: for a whole `n`, `ceil(x) <= n` exactly when `x <= n`, so `time_of(p) <= t` exactly when `p <= (t - origin_ns) × 12 / 125 000`; `position_at(t)` is therefore the largest position whose `time_of` is at most `t`. `time_of` is strictly increasing (one sample is 10 416⅔ ns, far more than a nanosecond), so the largest position whose `time_of` is at most `time_of(p)` is `p` itself — that is the first identity — and the second identity is the definition of largest, read at `t`. A round trip cannot overflow the results either: `position_at` returns below (2^64 - 1) × 12 / 125 000 < 2^51 positions, and the time of a position `position_at` produced is at most the original `t`.

Slice boundaries are exact without rounding, which is what 006-AC-1 asserts. A slice is 48 frames (C-02); 48 × 31 250 = 1 500 000 is divisible by 3, so a boundary `p = 48·k` has `p × 31 250 / 3 = 500 000·k`, a whole nanosecond — 48 samples = 500 000 ns exactly. On a boundary both directions are exact with no rounding: `time_of(48·k) = origin_ns + 500 000·k` and `position_at(origin_ns + 500 000·k) = 48·k`; 006-08 tests this exhaustively for every k from 0 to 1 000 000.

### Off a slice boundary (Q-1)

One sample is `31 250 / 3 ns = 10 416⅔ ns`, so a position off a slice boundary — a sample inside a slice, the positions a capture edge stamps — maps to a fraction of a nanosecond. The rounding rule `[decision]`: `time_of` rounds such a position up, with ceil, to the next whole nanosecond, and `position_at` in the other direction rounds down, with floor, to the position of the sample that contains the time. What forces the rounding is divisibility by 3, not the boundary: a position `p` is a whole number of nanoseconds exactly when `p` is divisible by 3 (31 250 shares no factor with 3). A position not divisible by 3 leaves a fractional nanosecond — ⅔ ns when `p ≡ 1 (mod 3)`, ⅓ ns when `p ≡ 2 (mod 3)` — and the ceil lifts it to the next whole nanosecond: `p = 1` gives 10 416⅔ rounded up to 10 417, `p = 2` gives 20 833⅓ rounded up to 20 834, `p = 4` gives 41 666⅔ rounded up to 41 667. A position off a slice boundary but divisible by 3 is not rounded at all: `p = 3` is exactly 31 250 ns. And every slice boundary is divisible by 3 because 48 is, so a boundary position is never rounded: 48 samples = 500 000 ns exactly. This closes Q-1: the falsifier, a `p` whose round trip differs from `p`, is excluded by the first identity, which holds for every `p >= 0` and not only at slice boundaries.

### Before the origin (Q-2)

`position_at` may be asked for a network time before the origin, `t < origin_ns`, where the position would be negative and positions are unsigned. Such a time has no position on this line: the map is not extrapolated backwards, exactly as it is never extrapolated across epochs, and the times before a line's origin belong to earlier lines where a position is meaningless. Clamping to 0 would misreport the instant as the origin's own. `position_at` therefore returns `None` — the Option branch, not a clamp `[decision]`. Two implementations of this section return the same `None` for the same time before the origin, which closes Q-2 (falsifier: two implementations disagreeing on a pre-origin time).

The Rust shape that ticket 006-08 implements is reconciled with this rule: `position_at` returns an `Option<u64>`, and 006-08 implements the reconciled signature.

```rust
pub const DOMAIN_RATE_HZ: u32 = 96_000; // C-01: the only rate on the wire
pub struct Line { pub epoch: TimelineEpoch, pub origin_ns: i64 }
impl Line {
    // Reconciled with the before-origin rule: Option, not a clamped u64.
    pub fn position_at(&self, network_ns: i64) -> Option<u64>;
    pub fn time_of(&self, position: u64) -> i64;
}
```

### Worked examples

Exact for every line; written against `origin_ns = 0`, where `origin + N ns` means `origin_ns + N`:

| Map | Input | Output |
|---|---|---|
| time_of | p = 1 | origin + 10 417 ns |
| time_of | p = 2 | origin + 20 834 ns |
| time_of | p = 3 | origin + 31 250 ns |
| time_of | p = 4 | origin + 41 667 ns |
| time_of | p = 48 | origin + 500 000 ns |
| time_of | p = 96 | origin + 1 000 000 ns |
| time_of | p = 49 | origin + 510 417 ns |
| time_of | p = 1 000 | origin + 10 416 667 ns |
| position_at | origin + 15 000 ns | p = 1 |
| position_at | origin + 31 249 ns | p = 2 |
| position_at | origin + 500 000 ns | p = 48 |
| position_at | origin + 510 417 ns | p = 49 |

`p = 1`, `2`, `4`, `49` and `1 000` are off a slice boundary and show the ceil: 10 416⅔, 20 833⅓, 41 666⅔, 510 416⅔ and 10 416 666⅔ ns are rounded up to whole nanoseconds. `p = 3` is off a slice boundary too but exact, 31 250 ns, because 3 divides it. `p = 48` and `p = 96` are slice boundaries, exact both ways: 48 samples = 500 000 ns and 96 samples = 1 000 000 ns; `p = 49` is the first sample of the second slice. The last four rows are the reverse direction: 15 000 ns lies inside sample 1 and floors down to `p = 1`; 31 249 ns, one nanosecond short of position 3's exact time, floors down to `p = 2`; the boundary 500 000 ns maps exactly to `p = 48`; and 510 417 ns maps to `p = 49`. 006-08 turns every row into a unit test.

## Epoch width and wrap

To be written in 006-03.

## Sync reply and line reconstruction

To be written in 006-04.

## Epoch-bump causes

To be written in 006-05.

## Discontinuity event

To be written in 006-06.

## Handover without a bump

To be written in 006-07.

## Source-side switch procedure and session numbering

To be written in 006-09.

## Modules in Locking or holdover

To be written in 006-10.

## Consumer-side procedures

To be written in 006-11.

## Health of an input on another epoch

To be written in 006-21.

## Announcement ordering and loss

To be written in 006-12.

## Sequence diagrams

To be written in 006-13..15.

## Scenarios

To be written in 006-16..18.

## Regime stamp epoch field

To be written in 006-19.

## Open items

To be written in 006-20.
