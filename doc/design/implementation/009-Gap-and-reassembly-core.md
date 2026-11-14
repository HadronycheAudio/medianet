# 009 — Gap and reassembly core

**Phase:** P3 · **Scope:** L (29 tickets, about 29 days) · **Depends on:** 004, 006, 007, 008 · **Blocks:** 012, 014, 015, 017, 024, 026, 028, 010, 018

## Goal

Implement the receive side of the data plane in `hadronyche-core`, as pure `no_std` code with no sockets and no allocation: packet validation and source filtering, copy-index dedup, multi-packet reassembly, the per-input alignment ring, fixed-time fan-in, the three gap classes (marked gap, late, missing), the degraded flag (including its delay through a structural latency L), the input health states (Live, Interrupted, Demoted, Dead) with their thresholds, the offset estimator and ratchet, the per-connection stamp counters and alignment figures, the cannot-meet-A test, and default concealment.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated IPv4 LANs whose only traffic is this runtime's. Modules (sources, processors, sinks) exchange *slices*: the content of one stream for one stretch of media time, addressed by *position* on a domain-wide timeline (position = network time counted in samples at the domain rate from a domain epoch). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. The runtime owns *when and whether* content moves; module code owns *what* it is.

**Evidence tags.** `[code]` read in the POC · `[kernel-doc]` documented · `[decision]` design choice · `[standard]` cited from a named published standard with its clause (C-132; none is cited in this epic).

### Constants `[decision]`

The constants table (02-constants.md) is canonical; these rows are repeated from it.

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire; an edge whose hardware runs at another rate converts at the edge |
| C-02 | Slice length | 48 frames = 0.5 ms = 2 000 slices/s; consecutive slice positions differ by 48 |
| C-03 | Wire sample format | 4 bytes per sample (f32 or Q1.31) |
| C-04 | Channels per stream at defaults | 6; 192 bytes per channel per slice (48 × 4 B), 1 152 bytes of audio per slice; no per-channel id byte (the channel map is in the manifest) |
| C-05 | Default slice | one packet; multi-packet slices are an explicit, stamped regime |
| C-07 | Copy index | 2 bits; dedup by stream, position and packet index, first valid copy wins |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-10 | Stream number / session / timeline epoch widths | 16 / 32 / 16 bits |
| C-15 | Reassembly deadline, arrival-driven multi-packet input | 2 slice periods |
| C-16 | Bounded drain per input per cycle | 2 × the admitted packets per slice for that input; the cycle is one slice period (C-156) |
| C-19 | Session number | assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value |
| C-27 | Alignment buffer per 16-member bundle | about 370 KB at a 10 ms skew budget (16 × 20 slices × 1 152 B = 368 640 B of audio) |
| C-38 | Interrupted | 8 consecutive missed slices (4 ms) |
| C-39 | Dead | 2 s without data |
| C-40 | Receiver report | one per bundle every 100 ms |
| C-43 | In-band announcement lead | 64 slices (32 ms at defaults), flat |
| C-47 | Third-party observability | producer and consumer each publish per-connection summary counters in their own regime stamp: one counter per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP; the full list is C-159 |
| C-50 | Aligning duration | 4 s |
| C-51 | Offset estimator | maximum observed lateness plus headroom |
| C-52 | Headroom | largest of 25 % of the observed maximum, one slice period, twice the published clock bound |
| C-53 | Ratchet step | one slice period; ceiling is the presentation group's audition offset A |
| C-54 | Dwell before automatic re-alignment | 10 s |
| C-55 | Audition offset A | no default; one per presentation group; proposed by commissioning |
| C-94 | Wired jitter profile for simulation (provisional) | base one-way delay 100 µs; exponential jitter, 20 µs mean; 1 % of packets in bursts delayed by 1 ms |
| C-95 | Fuzz budget | each parser 60 s on every change and 1 hour nightly; the corpus is kept |
| C-96 | Skew budget cap | the presentation group's A; alignment rings are sized to A |
| C-100 | IP version | IPv4 only in v1 |
| C-101 | Persistently late | at least one late slice in each of 10 consecutive 1 s windows (the dwell C-54 as a test) |
| C-102 | Miss, for the interrupted threshold | only a slice absent at the deadline (missing, or arriving late). A marked gap or a slice on another timeline epoch resets the count |
| C-104 | Paths of different latency at a join | the compiler pads the shorter path with a whole-sample delay line; the module's L is the longest path |
| C-105 | Structural latency L | always a whole number of domain-rate samples; the earlier rational-L rule is withdrawn |
| C-108 | Late versus missing | at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded |
| C-109 | Demotion at the ratchet ceiling | an input persistently late with the offset already at A enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission |
| C-110 | Reassembly deadline, aligned multi-packet input | the consumer's own deadline for that position |
| C-111 | Start of the dead threshold | 2 s counted from the last slice received with content or a marked gap |
| C-122 | Degraded flag through structural latency L | follows the content: while processing input p the module emits the output stamped p − L, whose content came from input p − L; that output is degraded if the input at p − L was gapped |
| C-134 | Resampler node | only inside an edge module; an edge's conversion delay is part of its plumbing delay D, not of L |
| C-135 | Slice on another timeline epoch | restarts the dead clock (C-111), is not a miss (C-102), has its own per-connection counter |
| C-141 | Transport state on a gap | last known state held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; output marked degraded |
| C-142 | Fan-in outside any presentation group | inherits the A of the downstream group it feeds (largest if several), learned at admission; with no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused |
| C-144 | Owner of orphan duties | the hosted backend (epic 015) emits the gap slice when a module misses its own deadline |
| C-148 | A for ladder runs (L1, L2, L3) and for the commissioning proposal | computed with the commissioning rule (largest over sinks of maximum lateness + D + headroom), using the maximum lateness since the connection went live, never the rolling 60 s figure of C-182; recorded with the results |
| C-156 | Cycle of the drain bound C-16 | one slice period |
| C-157 | Reclaimable latency | offset − (current maximum lateness + headroom), when positive; published in the stamp's alignment group |
| C-158 | Dense controller with no prior value | the controller's initial value declared in the manifest (default 0) |
| C-159 | Per-connection counters in the stamp | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-160 | Lateness percentiles in the stamp | 50th, 99th, 99.9th and the maximum |
| C-163 | Last ratchet step | when A − offset is less than one slice period, the offset steps to A exactly |
| C-164 | Bound on processing one bad input (proposal, pending L1) | linear in packet length, at most 10 µs per packet |
| C-165 | Operator policy | the commissioning configuration sets the per-input alignment opt-out |
| C-173 | Connection alarms | raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded |
| C-174 | "Cannot meet A" | maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows |
| C-182 | Window of the stamp's lateness figures | percentiles (C-160), reclaimable latency (C-157) and the cannot-meet-A alarm (C-174) use a rolling 60 s window; the maximum since the connection went live is reported beside them |
| C-178 | What an open commissioning window disqualifies | ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while a window is open |
| C-181 | Where controllers travel | a bundle source's controllers always ride in their own member stream (C-205); a single-stream source's controllers share the audio packet while headers plus controllers stay within the 100-byte cap (C-137), beyond that in a second stream |
| C-184 | Transport state before any has arrived | stopped, playhead at zero, tempo 120 beats per minute (C-199) |
| C-185 | Refusal reasons, completed | adds NoSkewBudget: a fan-in with no downstream group and no configured budget (C-142) |
| C-186 | A changed while inputs are live | every fan-in that inherited the old A (C-142) re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view; every epic that reports one-way figures depends on it |
| C-192 | Smaller settled points (the part used here) | the vectors crate sits at the workspace root; the fuzzing tool is the implementing team's choice |
| C-198 | Carrier of a changed A upstream | the receiver report (C-40) carries the consumer's current A, its own or the one it inherited; a change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186) |
| C-199 | Default tempo | 120 beats per minute |
| C-200 | Playhead through a gap while stopped | does not advance (see C-141) |
| C-208 | Who picks the re-alignment position after a changed A | each fan-in that learns of the change picks its own position, one lead (C-43) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment; fan-ins at different depths switch at different, announced positions |
| C-209 | Controller member and the bundle cap | the controller member counts towards the 32 of C-08: a source with controllers has at most 31 audio members |
| C-211 | A larger than a fan-in's buffer memory | the commissioning tool checks declared buffer memory before applying a change of A and refuses one that does not fit; if one reaches a fan-in anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running |
| C-214 | Where a sink switches to a changed A | at its own position, one lead (C-43) ahead, as a fan-in does (C-208); it has nobody to announce it to |
| C-215 | A ratchet step | is a re-alignment like any other: it takes effect at a position the fan-in announces one lead (C-43) ahead in its own output, never silently on the next round |
| C-216 | A clamped A (C-211), in detail | the cannot-meet-A alarm stays raised until a later change of A that fits; the fan-in's receiver report (C-198) carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need |
| C-222 | Who the clamp of C-211 applies to | any consumer that buffers to meet A, sinks included; C-216 applies to a sink in the same way |
| C-225 | A ratchet step in a sink | a sink has no output to announce in, so it takes the step at its own position one lead (C-43) ahead, as it does for a changed A (C-214) |

### Position invariant `[decision]`

A module's output for position p is derived from its inputs at position p. The runtime stamps output positions; module code cannot change them. A module with declared structural delay L has its output stamped p − L while it processes input p. Stamps denote content position and are never restamped per hop. L is always a whole number of domain-rate samples (C-105); a resampler exists only inside an edge module and its delay is part of the edge's D, not of L (C-134). Paths of different latency are padded by the compiler with whole-sample delay lines (C-104).

### Fan-in alignment `[decision]`

- **Align only where needed.** Single-input modules are arrival-driven and forward immediately. Deadlines exist only at fan-in points and at sinks.
- A fan-in holds each aligned input in a ring indexed by position. The ring spans the presentation group's A (C-96); a fan-in outside any presentation group uses the A inherited from the downstream group it feeds (the largest if several) or, with no downstream group, an explicit budget from configuration, and without either its aligned inputs are refused with reason NoSkewBudget (C-142, C-185). When A changes while inputs are live, every fan-in that inherited the old A re-enters aligning at an announced position, with its rings and ratchet ceiling resized to the new A, as the sinks do (C-186; a sink switches at its own position one lead ahead and has nobody to announce it to, C-214); the fan-in picks that position itself, one lead (64 slices, C-43) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment (C-208). A new A deeper than the compiled ring is refused beforehand by the commissioning tool (C-211, through `check_depth`); if one reaches a fan-in anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm and keeps its streams running (C-211); the alarm stays raised until a later change of A that fits (C-216). The clamp applies to any consumer that buffers to meet A, sinks included, and C-216 applies to a sink in the same way (C-222); a sink epic uses this core's `check_depth` and `set_ceiling_short` for a change of the sink's own A. The change reaches it in the receiver reports (C-40) of the consumers it feeds, each of which carries that consumer's current A, its own or inherited; the fan-in in turn carries its own current A in the receiver reports it sends upstream, so a change propagates one hop per report on the data plane (C-198); after a clamped change it carries the A it was given, not the clamped value (C-216). A slice too far ahead for the ring is refused as too early and counted.
- It processes position p at the fixed time `T(p) + offset`, with whatever has arrived. Anything absent is a gap.
- **It always fires at the fixed time, never early** when all inputs happen to be present; early firing would make output timing data-dependent.
- Alignment is sample-exact by position whatever the clock error.
- A per-input "unaligned" opt-out exists (mix on arrival); a return edge in a cycle is declared unaligned. The opt-out is set per input before aligning (`FanIn::set_unaligned`), by the operator policy in the commissioning configuration (C-165) or for a return edge; an unaligned input has no ring, no lateness sample, no ratchet participation and is never late.
- All aligned inputs of a consumer, including all members of a bundle, share that consumer's one offset. Bundle members have independent gaps. A bundle has at most 32 members, the controller member included (C-08, C-209); a bundle source's controllers arrive in their own member stream (C-181, C-205), a single-stream source's controllers may share the audio packet (C-181).
- An input whose slices carry another timeline epoch is treated as a gap until it switches. Such a slice proves the upstream alive: it is not a miss (C-102), it restarts the dead clock and it has its own counter (C-135).
- A multi-packet slice on an aligned input is reassembled until the consumer's own deadline for its position (C-110); on an arrival-driven input, until 2 slice periods after its first packet (C-15).

### Offset `[decision]`

During the 4 s aligning state the consumer measures origin-to-arrival lateness per input (arrival time minus `T(p)`), which already includes every upstream hop. Offset = maximum observed plus headroom (C-51, C-52). Frozen once live. Ratchet: grows by one slice when an input is persistently late, that is late at least once in each of 10 consecutive 1 s windows (C-101, the dwell C-54); the step is a re-alignment like any other: it takes effect at a position the consumer picks one lead (C-43) ahead and announces in its own output, never silently on the next round (C-215, C-208); a sink has no output to announce in, so it takes the step at its own position one lead ahead, unannounced, as it does for a changed A (C-225, C-214); never shrinks automatically; capped by A; when A − offset is less than one slice the last step goes to A exactly (C-163). An input persistently late with the offset already at A enters the health state **Demoted** (C-109): every round treats it as gapped, its rounds are counted separately, and only re-admission clears it; there is no automatic retry. A shorter path leaves the offset safe; reclaimable latency, offset − (current maximum lateness + headroom) when positive, is published in the stamp's alignment group (C-157) together with the lateness percentiles 50th, 99th, 99.9th and the maximum (C-160). The consumer also tests "cannot meet A": maximum lateness + D + headroom above A in each of 10 consecutive 1 s windows (C-174). The percentiles, the current maximum lateness behind reclaimable latency and the maximum used by the cannot-meet-A test are taken over a rolling 60 s window; the maximum since the connection went live is reported beside them (C-182), and it is the figure the commissioning proposal of A and the ladder runs use, never the rolling one (C-148); the alarm itself is raised by the connection state (C-173). While a commissioning window is open these production figures and the C-159 counters are unqualified (C-178); this epic produces them unchanged and the window is stamped beside them (C-153). (Parameters validated by simulation in task 008.)

### Gap contract `[decision]`

Timing is always the consumer's deadline. Markers exist for classification. At the deadline for position p on an input:

| Observation | Class | Consequence |
|---|---|---|
| Slice p present with content | — (received) | Use it |
| Slice p present as a gap slice, or with the degraded flag | **Marked gap** | Upstream alive, loss further up; resets the miss count (C-102) |
| Slice p on another timeline epoch | Gap (not a miss) | Upstream alive; resets the miss count (C-102); no round combines two epochs |
| Slice p absent | **Missing** | Unknown cause; counts as a miss (C-102) |
| Slice p arrives after the deadline, while p is within the ring span | **Late** | Reclassified: missing −1, late +1; discarded (C-108); the remedy is budget, not network |
| Input in health state Demoted | Gap (counted as demoted) | Treated as gapped until re-admission (C-109) |
| 8 consecutive misses | Interrupted (health) | Module told; conceal; resume is immediate at the frozen offset when slices return on the same session (same C-19 session number) |
| 2 s since the last slice received with content, a marked gap or another timeline epoch | **Dead** (health) | End-of-stream event; resources released; re-admission needed (C-39, C-111, C-135) |

- Marked gap, late and missing are the three gap classes; Interrupted, Demoted and Dead are input health states, not gap classes.
- A consumer that processed a gap marks its own output slice **degraded**. Through a structural latency L the flag follows the content (C-122): the output stamped p − L is degraded if the input round at p − L, whose content it carries, was gapped. L is a whole number of samples (C-105); when it is not a multiple of 48 the output's samples come from two input rounds, and it is degraded if either was.
- If a module misses its own deadline, the runtime emits a gap slice for it. That emission (C-144), and stamping outputs at p − L, belong to the hosted backend (epic 015); this epic provides the round and the degraded delay line it uses.
- **Default concealment** (per content type, per-connection override; a slice whose packet carries both audio and a single-stream source's controllers, C-181, is concealed section by section with each section's content type): audio zero-fill; dense controllers hold last value, or the initial value declared in the manifest (default 0) when no value has been received (C-158); sparse keyed controllers rely on state refresh; transport state holds its last known state and the playhead advances as that state dictates: a stopped transport stays where it is (C-200), a playing one moves on at its tempo over the positions elapsed, and the output is marked degraded (C-141); before any transport state has arrived it is stopped, with the playhead at zero and a tempo of 120 beats per minute (C-184, C-199). The override is a per-input `Conceal` implementor held by the consumer, chosen without allocation.
- Reference mixer rule: a missing input contributes zero and the output slice is marked degraded.
- Counters published in the consumer's regime stamp (C-47, completed by C-159): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead. Rejected packets per reason and reassembly missing packets are kept for the receiver report. The observed DSCP is read by the platform backend, not this epic.

### Receive-side enforcement `[decision]`

Bounded time on the data thread is made real by a bounded drain per input per cycle, the cycle being one slice period (C-16, C-156), a source filter (only admitted peers are parsed, by the IPv4 source address and port pinned at admission plus stream number, C-100), and a socket buffer sized to the declared rate times the skew budget with the kernel's per-socket overflow counter read on every receive `[kernel-doc]`. Packets beyond the drain budget stay in the socket buffer for the next cycle; if the buffer overflows, the kernel's overflow counter counts them. Processing one bad packet is linear in packet length and at most 10 µs (C-164, a proposal that L1 measures). This task implements the drain budget and the source filter; the socket and its overflow counter are the backend's. Arrival times (`arrival_ns`) are supplied by the backend, which stamps their source (kernel, userspace, mac-hardware or interrupt) with a C-129 capability state (C-131); this epic takes them as given. Strict-mode capabilities (C-127, C-128) are the backend's as well.

Where outputs go (C-130, C-133): code in `hadronyche-core` (tests and `fuzz/` beside it); design changes in the design set's repository, constants table first; no probe or tool code in this epic; nothing written into the frozen POC tree.

`[code]` POC behaviour retired: the mixer keeps only the *latest* block per input and mixes on arrival with no alignment (`mixer.rs:101-124`); the data thread drains every input socket completely before looking at its deadline (`module.rs:1669`); a kernel-reassembled 9.6 KB datagram per slice.

## Deliverables

All in `hadronyche-core`, `no_std`, no allocation; capacities are const generics.

```rust
// shape, names may be refined
pub struct SourceFilter<const N: usize> { /* pinned (IPv4 addr + port, stream) pairs */ }
impl<const N: usize> SourceFilter<N> {
    pub fn admit(&self, from: &Addr, pkt: &[u8]) -> Result<InputIndex, Reject>;
}

pub struct Reassembler<const SLOTS: usize, const MAX_SLICE: usize> { /* per source */ }
impl<..> Reassembler<..> {
    pub fn push(&mut self, pkt: &MediaPacket, arrival_ns: i64) -> Option<SliceRef<'_>>; // dedup by (stream, position, index)
    pub fn expire(&mut self, now_ns: i64) -> Expired;          // arrival-driven input: C-15
    pub fn expire_before(&mut self, position: u64) -> Expired; // aligned input: the consumer's deadline, C-110
}

pub struct AlignRing<const DEPTH: usize, const MAX_SLICE: usize> { /* per aligned input, DEPTH sized to A */ }
impl<..> AlignRing<..> {
    pub fn insert(&mut self, slice: SliceRef<'_>, arrival_ns: i64) -> Inserted; // Stored | Late | OtherEpoch | Duplicate | TooEarly
    pub fn take(&mut self, position: u64) -> InputAt<'_>;                       // Content | MarkedGap | OtherEpoch | Missing
}

pub enum GapClass { MarkedGap, Late, Missing }
pub enum InputHealth { Live, Interrupted, Demoted, Dead }

pub struct FanIn<const INPUTS: usize, const DEPTH: usize, const MAX_SLICE: usize> {
    /* offset, estimator, ratchet, rings, per-input health, session and counters */ }
impl<..> FanIn<..> {
    pub fn set_unaligned(&mut self, input: InputIndex, unaligned: bool); // before aligning
    pub fn deadline_ns(&self, position: u64, line: &Line) -> i64;     // T(p) + offset, never earlier
    pub fn observe_lateness(&mut self, input: InputIndex, lateness_ns: i64);
    pub fn finish_aligning(&mut self, clock_bound_ns: i64, slice_ns: i64) -> i64; // max + headroom
    pub fn on_deadline(&mut self, position: u64) -> Round<'_>;        // per-input InputAt + health events; sets degraded
    pub fn ratchet(&mut self, now_ns: i64, ceiling_ns: i64) -> RatchetOutcome; // Stepped { at_position } (one lead ahead, C-43; announced by a fan-in, C-215; taken unannounced by a sink, C-225) | AtCeilingDemoted(input) | None
    pub fn readmit(&mut self, input: InputIndex);                     // clears Demoted and Dead
    pub fn change_inherited_a(&mut self, new_a_ns: i64) -> Realign;    // C-186, C-208: picks and announces its own position one lead ahead; C-211: clamps the ceiling to the compiled ring
    pub fn current_a_ns(&self) -> Option<i64>;                        // C-198: own or inherited A, carried in the receiver report sent upstream
    pub fn on_downstream_a(&mut self, downstream: usize, a_ns: i64) -> Option<i64>; // C-198, C-142: A from a downstream receiver report; Some(new inherited A) when the largest changes
}

pub struct DegradedDelay<const ROUNDS: usize> { /* gapped flag of past rounds, for a whole-sample structural latency L (C-105, C-122) */ }

pub trait Conceal { fn fill(&mut self, content: ContentType, out: &mut [u8], last: Option<&[u8]>,
                            initial: &[u8], positions_elapsed: u64); }
pub struct DefaultConceal; // audio zero-fill, dense hold-last or manifest initial value (C-158), sparse no-op, transport hold, stopped stays and playing advances at its tempo (C-141, C-200), or stopped at zero at 120 beats per minute before any state (C-184, C-199)

pub struct InputCounters { pub received: u64, pub late: u64, pub missing: u64, pub marked: u64,
                           pub other_epoch: u64, pub demoted: u64, pub too_early: u64,
                           pub duplicate: u64, pub incomplete_reassemblies: u64, pub reassembly_missing_packets: u64,
                           pub interrupted: u32, pub dead: u32, pub rejected: [u32; REJECT_REASONS] }
```

Plus: a `DrainBudget` helper giving the per-cycle packet limit for an input; const functions reporting ring depth and alignment memory for a given A (C-96), and `check_depth` for commissioning (task 028), which also refuses aligned inputs with no skew budget (C-142, reason NoSkewBudget, C-185) and is the check the commissioning tool applies before a change of A (C-211); the C-159 stamp summary and the alignment-group figures (C-157, C-160) over a rolling 60 s window with the maximum since live beside them (C-182); a cannot-meet-A test (C-174); unit tests; conformance-vector tests (vectors from the crate at the workspace root, C-192); fuzz targets (tool of the implementing team's choice, C-192) for packet validation and reassembly; a no-alloc test allocator and a lint that rejects lock types in the receive path.

## Dependencies

- Needs from 004: packet and slice layouts (including the session number, C-19), rejection reasons, content types, and the layout of the regime stamp sections that carry the C-47 counters. From 006: the `Line` type and the other-epoch rule. From 007: vectors. From 008: `TimeView` (clock bound, published by the servo core 008a, C-187) and the validated estimator and ratchet parameters.
- Provides to 012: connection-state inputs (Interrupted, Resumed, Demoted, Dead, session change, ratchet events). To 014: the counters a receiver report carries, the consumer's current A it carries upstream (C-198), and the C-47 / C-159 stamp summary. To 012 also: the cannot-meet-A event (C-173, C-174). To 015 and 017: the receive pipeline the hosted data thread and the L0 harness run, including the round and the degraded delay line used when stamping p − L and emitting gap slices. To 024: the input-set shape the DSL process function receives. To 026: the same code on the device. To 028: `check_depth` and the ring sizing functions, called with a commissioned A. Needs from 014: the A carried by each downstream consumer's receiver report (C-198), from which an inherited A change is detected. The position at which a fan-in re-aligns after a change of inherited A is picked by the fan-in itself (C-208); its round exposes the pending announced position, which the output path (epic 015) carries in the fan-in's output.

## Acceptance criteria

Each criterion has an epic id (EC-n); system criteria keep their AC-n ids.

1. **EC-1** (system AC-1): conformance vectors for media packets pass 100 %; zero parser panics or hangs under fuzzing at the C-95 budget.
2. **EC-2** (system AC-10): in a diamond (source feeds a fan-in directly and through one processor with injected per-hop delay), the fan-in output equals the offline reference bit for bit. The offline reference is the same bit-exact test processor and the reference mixer applied to the undelayed source slices, position by position.
3. **EC-3** (system AC-15): injected delay beyond the offset is counted as late only (through the C-108 reclassification); injected drop as missing only.
4. **EC-4** (system AC-16): marked gap propagates with the degraded flag; interrupted after exactly 8 consecutive misses as defined by C-102 (a marked gap or an other-epoch slice resets the count); dead 2 s after the last slice with content, a marked gap or another timeline epoch (C-111, C-135); resume from interrupted is immediate at the frozen offset with no re-alignment when slices return on the same session (C-19).
5. **EC-5** (system AC-18): an input on another timeline epoch is treated as a gap; no round combines two epochs.
6. **EC-6** (system AC-11, core part): with one input presenting 10 × its admitted packets, the per-cycle work on that input is bounded by C-16 and the fan-in deadline is computed and met in the simulated schedule.
7. **EC-7** Fixed-time firing: with all inputs present early, `on_deadline` output timing is identical to the case with one input arriving at the last moment.
8. **EC-8** Dedup: a duplicate copy (same stream, position, packet index, different copy index) is counted and discarded; the first valid copy wins.
9. **EC-9** Multi-packet: an incomplete slice is dropped at its deadline (C-15 arrival-driven, C-110 aligned) and the number of missing packets is counted.
10. **EC-10** Compiles for a `no_std` target without an allocator; memory use for a 16-member bundle at a 10 ms skew budget is about 370 KB (C-27) and is reported by a const function; the ring depth for a given A is computed by a const function and `check_depth` refuses an A deeper than the compiled capacity.
11. **EC-11** Ratchet and demotion: an input late in each of 10 consecutive 1 s windows (C-101) steps the offset by one slice, or to A exactly when less than one slice remains (C-163); with the offset at A it enters Demoted (C-109), is gapped and counted as demoted in every later round, and leaves Demoted only through `readmit`.
12. **EC-12** Degraded through latency (C-122): for a whole-sample structural latency L (C-105), the output stamped p − L is degraded exactly when the input round at p − L, whose content it carries, was gapped.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Late, missing, marked and rejected counters are all exposed for the receiver report and the stamp

**Not applicable in this epic (C-125):** none. Every line has a subject here: the media packet parser is touched (fuzzed), the consumer's C-47 summary counters are regime stamp fields, and the receive path is the code the allocation and lock checks cover.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is the C-16 drain bound large enough for legitimate bursts (for example Windows batched capture emission)? | L1 / L2 Windows (tasks 019, 022) | Legitimate bursts exceed it and overflow the socket buffer (kernel overflow counter rises) |
| What is the "cycle" of C-16? | Decided (C-156): one slice period | — |
| Maximum skew budget a ring must support | Decided (C-96, C-142): the presentation group's A, or for a fan-in outside any group the inherited A or a configured budget; rings are sized to it | A commissioned A needs a deeper ring than the compiled capacity |
| Is maximum-plus-headroom over 4 s sufficient? | Task 008, then L1 | Late slices after alignment on a clean network |
| Should a demoted input be re-tried automatically? | Decided (C-109): no; Demoted is cleared only by re-admission | — |
| What does the reassembler do when all SLOTS are busy (assumed: evict the oldest incomplete slot, counted)? | This epic (ticket 009-06), then L1 | `slot_overflow` rises on a clean network with SLOTS sized for the admitted multi-packet streams |
| How is reclaimable latency defined and where is it reported? | Decided (C-157): offset − (current maximum lateness + headroom), when positive, in the stamp's alignment group | — |
| Over which window are the "current maximum lateness" of C-157 and the percentiles of C-160 taken? | Decided (C-182): a rolling 60 s window, also used by the cannot-meet-A test (C-174); the maximum since the connection went live is reported beside them | — |
| What happens when an aligned input yields no lateness sample during the 4 s aligning state (assumed: the consumer stays Aligning)? | Control state machines (epic 005) | A consumer never leaves Aligning while one of its inputs is silent |
| Dense-controller hold-last when no last value exists | Decided (C-158): the initial value declared in the manifest, default 0 | — |
| Under which stamp class are the extra counters published? | Decided (C-159): each has its own per-connection counter; layout by epic 004 | — |
| What does a consumer do when an interrupted input returns on a different session (C-19)? Assumed: no resume, a session-change event to the connection state machine | Control state machines (epic 005) | A restarted source is resumed at an offset measured for its previous session |
| Transport state concealed before any state has arrived | Decided (C-184, C-199): stopped, playhead at zero, tempo 120 beats per minute | — |
| Does the playhead of a held transport state advance through a gap whatever the state? | Decided (C-141 as revised, C-200): only as the state dictates; a stopped transport does not advance, a playing one moves on at its tempo | — |
| What does a fan-in that inherited A do when A changes while its inputs are live? | Decided (C-186): it re-enters aligning at an announced position, rings and ratchet ceiling resized | — |
| How does a changed A reach a fan-in that inherited it? | Decided (C-198): in the receiver report (C-40), which carries each consumer's current A, own or inherited; one hop per report, on the data plane | — |
| Who chooses and carries the announced position of a re-alignment triggered by a C-198 receiver report? | Decided (C-208): the fan-in itself, one lead (C-43) ahead, announced in its own output as for a ratchet-triggered re-alignment | — |
| What does a fan-in do when a changed A exceeds its compiled ring? | Decided (C-211): the commissioning tool refuses such a change beforehand; if one arrives anyway, the fan-in keeps the deepest ring it can hold as ceiling, raises cannot-meet-A and keeps its streams running | — |
| Does a ratchet step take effect on the next round or at an announced position? | Decided (C-215): it is a re-alignment like any other, at a position the fan-in announces one lead (C-43) ahead in its own output, never silently on the next round | — |
| After a clamped change of A (C-211), how long does cannot-meet-A stay raised, and which A does the fan-in carry upstream? | Decided (C-216): raised until a later change of A that fits; the receiver report carries the A it was given, not the clamped value | — |
| Where does a sink switch to a changed A (C-186 "as the sinks do")? | Decided (C-214): at its own position, one lead (C-43) ahead, with nothing to announce; implemented by the sink epics | — |
| Does the clamp of C-211, with C-216, apply to sinks? | Decided (C-222): to any consumer that buffers to meet A, sinks included; C-216 applies to a sink in the same way | — |
| Where does a sink take a ratchet step, having no output to announce it in? | Decided (C-225): at its own position, one lead (C-43) ahead, as for a changed A (C-214) | — |
| C-122 wording versus the position invariant | Decided (C-122 as revised): the output stamped p − L carries input p − L and is degraded if that input was gapped | — |
