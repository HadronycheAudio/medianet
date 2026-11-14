# 006 — Timeline epoch specification

**Phase:** P1 · **Scope:** M (21 tickets, about 11 days) · **Depends on:** 004, 005 · **Cross-reference:** 013 (C-196) · **Blocks:** 007, 008, 009, 010

## Goal

Specify the domain timeline and its epochs precisely: how a position maps to network time, what bumps the timeline epoch, how each party learns of a new epoch, how sources switch at an announced position, and what a fan-in does with inputs on different epochs. The outcome is a normative document plus scenario descriptions that become conformance vectors.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated IPv4 LANs (C-100) whose only traffic is this runtime's. Modules discover each other, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone and the only clock holders in v1; macOS and Windows machines are edges; embedded devices are followers.

**Evidence tags.** `[code]` read in the POC · `[decision]` design choice · `[standard]` a figure cited from a named, published standard with its clause, if the specification needs one (C-132).

### The domain line `[decision]`

There is **one timeline for the whole domain**, at the domain sample rate. The wire carries one sample rate only, 96 000 Hz (C-01): every stream on a medianet is at the domain rate. Position is network time counted in domain-rate samples from a domain epoch origin:

`position = (network_time − epoch_origin) × 96 000 Hz`

- Streams have no private origins. Any two sources share one position space, so a mixer has a single timeline to stamp its output on.
- A capture edge stamps each sample with the position of its acoustic instant.
- There is no position space at any other rate. An edge whose hardware runs at another rate (typically 48 kHz) converts 2:1 through the kit's resampler node; the wire stays at the domain rate (C-01, C-80). A resampler node may appear only inside an edge module, and the compiler rejects it anywhere else (C-134); its conversion delay is part of the edge's plumbing delay D (C-170) and is never counted in L (C-134, C-207). An edge also has a separate, steerable rate actuator that tracks its hardware crystal, whose delay is likewise part of D (C-189); the kit (epic 024a) builds both resamplers and every edge uses them (C-197). The rate is therefore a constant of the map, not a field of it.
- Positions are 64-bit. A slice is 48 frames, 0.5 ms, 2 000 slices/s at defaults (C-02). A slice boundary is exactly 500 000 ns after the previous one; one sample is 10 416⅔ ns, so the map needs a rounding rule only for positions that are not on a slice boundary (open sub-question Q-1).
- The map is **affine per timeline epoch and never extrapolated across epochs**.
- Sources pace position from network time, not from their local crystal.

`[code]` In the POC the media clock is the sender's local crystal: the data-plane ticker runs on local monotonic time (`module.rs:1630-1634`), and each stream carries its own `sample_pos` origin. Both are retired.

### Timeline epochs `[decision]`

- A **timeline epoch** is a 16-bit counter (C-10). It is distinct from the *election* epoch: a continuous clock handover does not change it; a discontinuity does. C-10 is decided; its falsifier ("spec drafting shows a 16-bit epoch wraps within a plausible uptime") is checked by the wrap analysis of this epic, which compares epochs by 16-bit serial-number arithmetic.
- The epoch is carried in **every sync reply** and in **every slice header**.
- **What bumps it:**
  1. a partition merge, for the losing side (the two timebases have diverged);
  2. a new holder that was not locked when it took the clock (no locked candidate existed). When no candidate is locked, the holder-eligible candidate with the lowest module id wins the election and bumps the timeline epoch (C-113);
  3. a holder that detects its own stall, for example its host was suspended. The holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch (C-136);
  4. a step larger than the servo's step threshold (2 ms, C-67), which is never slewed. A step of the local wall clock is not this cause: it is detected and filtered in the platform backend that builds each exchange from receive stamps, and the servo only receives exchanges that passed that check (C-121).
- **Who announces:** the holder is a clock server, not a stream source, so it cannot announce in-band; it answers sync requests, and so publishes the epoch, on its clock thread, never on housekeeping (C-188). Each module's clock thread learns the new epoch (and the new line) from sync replies. Each of that module's **sources** then announces in-band, in its own streams, the future position at which it switches, and restarts its session there. A bundle source's controllers ride in their own member stream, which counts among the members (C-205, C-209); a single-stream source whose controllers overflow the header cap emits them in a second stream (C-181). Every such stream announces and switches like the others.
- **Sessions:** the session number is assigned by the source, one per stream; it is bumped on restart and on every timeline epoch change, and every consumer of a stream sees the same value (C-19).
- **Announcement mechanics:** an announcement names a future position, with a lead of 64 slices (32 ms at defaults, C-43), and is re-issued every 16 slices until the named position passes, in every bundle member (C-44). The lead is flat: the same 64 slices at every hop, not scaled with graph depth (C-43). Each module learns the new epoch from its own clock thread and announces its own switch with the full lead, so the lead does not shrink along a chain. Consumers process positions in order, so an announcement is always processed before the position it names; the lead exists only to survive loss (four transmissions). In-band announcements fit inside the header cap of a default slice (at most 100 bytes for all headers and the checksum) or are omitted on a fallback path (C-137); a consumer on a path where they are omitted sees the input on another epoch, treated as a gap until it is observed to switch. C-43 is decided with the falsifier "a descendant misses an announcement despite four re-issues"; L0 (task 017) confirms it in a deep graph.
- **At a fan-in:** an input that is still on another epoch is treated as a **gap** until it switches. A fan-in never combines samples from two epochs. An output whose input at `p − L` was gapped is flagged degraded (C-122), so an output fed by an other-epoch input carries the degraded flag. Every other gap rule applies too: transport state is held, a stopped transport staying where it is and a playing one moving on at its tempo (C-141, C-200), and before any has arrived it is stopped at zero, 120 beats per minute (C-184). L is always a whole number of domain-rate samples (C-105).
- **Health of an other-epoch input:** a slice on another timeline epoch proves the upstream is alive and resets the interrupted count (C-38, C-102); it is never counted as a miss. It restarts the dead clock (C-111) and has its own per-connection counter (C-135), the other-epoch counter of the stamp (C-159).
- **At a sink:** the audition offset A applies per epoch; `emit(p) = T(p) + A` uses the line of the epoch the slice carries. Alignment rings are sized to A (C-96).
- **Regime stamp:** the regime stamp is a dedicated control message in sections that each fit one fallback-MTU packet, multicast to the control group on any regime change and refreshed every 10 s; the hello carries only a stamp version number (C-18). An epoch change starts a new record set, so it is a regime change: the stamp is re-multicast with a new version. Producers and consumers publish per-connection summary counters in their own stamp (C-47); the full counter list includes an other-epoch counter (C-159). The stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144).
- **Clock states:** "Locking" and "holdover" are clock states defined by the control state machines (epic 005); the lock state and bound are published by the servo core (epic 008a) through the published time view (C-187). The holdover entry delay is pending simulation (epic 008b) and expected at or below the election silence threshold (C-37), so a follower is already in holdover when it calls an election (C-195). A follower in holdover grows its bound at 1 µs per second (C-64) and its alignment claim lapses after about 25 s (C-65; with E unknown the reference is 2 × C-62 = 50 µs, giving the same about 25 s, C-169); it is then Unlocked, with time unusable: one-way figures and alignment claims stop (C-162). A laptop edge that sleeps is a departure: both timebase loops are reset and it returns through normal admission (Locking, Aligning) (C-124), so it never resumes on a stale line.
- Continuity across a merge is **not** guaranteed. What is guaranteed is that the step is never slewed silently: every affected module gets a timebase-discontinuity event carrying the step size, and its streams restart with a new session at an announced position.

### Handover without a bump `[decision]`

A normal handover is frequency-continuous: the new holder starts at the old holder's rate and relaxes to its own crystal at no more than 1 ppm per second (C-61, pending simulation). Phase is carried on. The timeline epoch does not change. The L2 handover test uses a 50 ppm crystal difference between old and new holder (C-78) and is triggered as a holder loss: the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over with frequency continuity (C-212). A candidate locked to the old holder is still in holdover when it takes the clock (C-195), so a takeover from holdover, before the claim lapses (C-65, C-162), is a continuous handover and does not bump.

### Related constants

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s |
| C-10 | Stream number / session / timeline epoch widths | 16 / 32 / 16 bits |
| C-18 | Regime stamp transport | dedicated sectioned control message; multicast on any regime change, refreshed every 10 s; hello carries a stamp version |
| C-19 | Session number ownership | assigned by the source, one per stream; bumped on restart and on timeline epoch change |
| C-30 | Hello period | 1 s |
| C-33 | Election window | 400 ms |
| C-37 | Election on reply silence | 1.5 s |
| C-38 | Interrupted threshold | 8 consecutive missed slices (4 ms at defaults) |
| C-43 | Announcement lead | 64 slices (32 ms at defaults), flat |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member |
| C-47 | Third-party observability of stream health | per-connection summary counters per gap class in each party's regime stamp |
| C-61 | Rate of change of rate (handover relaxation) | at most 1 ppm per second, pending simulation |
| C-64 | Holdover bound growth | 1 µs per second |
| C-65 | Alignment claim lapse in holdover | about 25 s |
| C-67 | Step threshold | 2 ms |
| C-78 | L2 handover test | 50 ppm crystal difference |
| C-80 | Edge hardware at 48 kHz | converts 2:1 at the edge; the wire stays at C-01 |
| C-102 | Miss for the interrupted threshold | only absent slices; a slice on another timeline epoch resets the count |
| C-105 | Structural latency L | always a whole number of domain-rate samples; the resampler node is offered only for 2:1 and 1:2; the earlier rational-L rule is withdrawn |
| C-111 | Start of the dead threshold | 2 s from the last slice received with content or a marked gap |
| C-113 | Election when no candidate is locked | lowest module id among holder-eligible candidates wins and bumps the timeline epoch |
| C-120 | Test-only injection hooks | include forcing a stream onto a given timeline epoch |
| C-121 | Home of wall-clock step detection | the platform backend; the servo only receives exchanges that passed it |
| C-122 | Degraded flag on an output with latency L | output at p is degraded if its input at p − L was gapped |
| C-124 | Laptop edge sleep | a departure; returns through Locking, Aligning |
| C-125 | Definition-of-done applicability | a line with no subject is marked not applicable with the reason |
| C-126 | Epic scope labels | recomputed from the ticket tally |
| C-130 | Where task outputs live | specifications in `doc/design/spec/`; nothing is written into the frozen POC tree except by task 003 |
| C-132 | Evidence tag for a published standard | `[standard]`, with the clause; replaces `[memory]` once checked |
| C-133 | Where code lives | probes in `probes/`, tools in the `hadronyche-tools` crate, all protocol types (timeline, slices, stamps, control messages) in `hadronyche-core` |
| C-134 | Where a resampler node may appear | only inside an edge module; its conversion delay is part of D and never counted in L (C-207) |
| C-135 | A slice on another timeline epoch | restarts the dead clock (C-111), is not a miss (C-102), has its own per-connection counter |
| C-136 | Holder stall | monotonic clock compared with a clock that runs through suspend; divergence above 2 ms (C-67) bumps the timeline epoch |
| C-137 | Header cap | at most 100 bytes for all headers and the checksum in a default slice; in-band announcements fit inside it or are omitted on a fallback path |
| C-144 | Owner of four orphan duties | the hosted backend (epic 015), including sending the regime stamp message that epic 013 defines |
| C-159 | Per-connection counters in the stamp | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-141 | Transport state on a gap | last known state held; playhead advanced as that state dictates; output marked degraded |
| C-162 | Clock state after the holdover claim lapses | Unlocked, with time unusable |
| C-169 | Alignment claim in holdover when E is unknown | unknown E taken as 2 × C-62 = 50 µs; lapse at about 25 s |
| C-170 | Definition of D | a sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter |
| C-181 | Where controllers travel | bundle source: own member stream; single-stream source: in the audio packet within the 100-byte cap, else a second stream |
| C-184 | Transport state before any has arrived | stopped, playhead at zero, 120 beats per minute |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view |
| C-188 | Where the holder answers sync requests | on the clock thread, never on housekeeping |
| C-189 | Two kinds of resampler | fixed-ratio DSL node (2:1, 1:2) and the edge's steerable rate actuator; the actuator's delay is part of D |
| C-193 | What epic 006 supplies to the conformance vectors | timeline and epoch scenarios |
| C-195 | Holdover entry delay | pending simulation (epic 008b); at or below C-37, so a follower is in holdover when it calls an election |
| C-196 | Dependency hygiene | a specification epic never blocks on an epic of a later phase; such a link is a cross-reference, not a dependency |
| C-197 | Who builds the resamplers | the arithmetic kit (epic 024a) builds both; every edge uses them |
| C-200 | Playhead through a gap while stopped | does not advance |
| C-202 | Sync loss at L2 | no separate scenario; the handover run asserts emission error within the target E while the sink is in holdover between holders |
| C-205 | Controllers of a bundle source | always in their own member stream |
| C-207 | Resampler delay and L | counted only in D, never in L |
| C-209 | Controller member and the bundle cap | the controller member counts towards the 32 of C-08 |
| C-212 | How the L2 handover is triggered | as a holder loss, through about 2 s of holdover (C-37 plus C-33); frequency continuity asserted at takeover |

## Deliverables

1. `doc/design/spec/timeline.md` (specifications live in `doc/design/spec/`, C-130) containing:
   - the exact arithmetic of the position ↔ network-time map at the domain rate (C-01), including the rounding rule for positions off a slice boundary, the behaviour for a network time before the origin, and the representation of `epoch_origin`;
   - what a sync reply must carry so that a follower can reconstruct the line (epoch number, origin, and the holder identity);
   - the complete list of epoch-bump causes with, for each, who detects it and what it publishes;
   - the source-side procedure: learn → choose switch position (current position + 64 slices) → announce in every member every 16 slices → at the switch position, end the old session and start the new one, with a bumped session number (C-19) and the new epoch;
   - the consumer-side procedure for single-input modules, fan-ins and sinks, including the health accounting of an input on another epoch (C-38, C-102, C-111, C-122, C-47, C-135, C-159);
   - the behaviour of a module that learns of a new epoch while it is itself in Locking or holdover;
   - the discontinuity event and its payload (old epoch, new epoch, step size);
   - session numbering rules across an epoch switch (C-19);
   - the epoch field of the regime stamp.
2. A sequence diagram (mermaid) for each of: partition merge; unlocked holder takeover; holder stall.
3. Scenario scripts (prose) for vectors: two sources feeding one fan-in switch epochs 10 slices apart; an announcement lost three times and received on the fourth re-issue; a source that never switches.
4. The types:

```rust
// shape, names may be refined; lives in `hadronyche-core` (C-133)
pub const DOMAIN_RATE_HZ: u32 = 96_000; // C-01, the only rate on the wire
pub struct TimelineEpoch(pub u16);
pub struct Line { pub epoch: TimelineEpoch, pub origin_ns: i64 }
impl Line {
    pub fn position_at(&self, network_ns: i64) -> u64; // behaviour before the origin: Q-2
    pub fn time_of(&self, position: u64) -> i64;
}
pub struct Discontinuity { pub from: TimelineEpoch, pub to: TimelineEpoch, pub step_ns: i64 }
```

## Dependencies

- Needs from 004: the slice header (epoch field), the announcement formats and the sync reply fields.
- Cross-reference to 013, not a dependency (C-196): epic 013 defines the regime stamp message (C-18, C-144) and places the epoch field in its layout. This epic specifies the field's width, meaning and record-set rule without waiting for that layout, and records the field for epic 013 to carry.
- Needs from 005: the definitions of the clock states Locking and holdover, and the rule that decides the losing side of a partition merge.
- Provides to 007: the timeline and epoch scenarios for the conformance vectors (C-193).
- Provides to 008: the published view's epoch field (the view itself is published by the servo core, epic 008a, C-187) and the handover/bump distinction the simulation must exercise, including takeover from holdover (C-195, C-212).
- Provides to 009: the fan-in rule for other-epoch inputs.
- Provides to 010: the source-side switch procedure.

## Acceptance criteria

1. **006-AC-1** `position_at` and `time_of` are exact inverses for every position on a slice boundary at the domain rate (96 kHz, C-01); the rounding rule for positions off a slice boundary is stated and tested by example.
2. **006-AC-2** Every bump cause has a detector, a publisher and a resulting event.
3. **006-AC-3** The specification shows that an announcement is processed before its named position at every descendant, without any assumption about gossip.
4. **006-AC-4** With a lead of 64 slices and re-issue every 16, an announcement survives the loss of three consecutive carrying slices; the document states the residual case (four lost) and its outcome (the input is on another epoch and is treated as a gap until it is observed to switch).
5. **006-AC-5** Contributes to system criterion AC-18: a fan-in never aligns across timeline epochs; an input on another epoch is treated as a gap until it switches.
6. **006-AC-6** Contributes to system criterion AC-36: after a healed partition, one holder within 400 ms + 1 s = 1.4 s; the losing side receives a timeline epoch change and its streams restart at an announced position; no silent step.
7. **006-AC-7** Contributes to system criterion AC-5: a handover with a 50 ppm crystal difference (C-78), triggered as a holder loss through holdover (C-212), does not bump the epoch.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The three sequence diagrams render
- [ ] The epoch field of the regime stamp is specified (which epoch a record set belongs to; an epoch change starts a new record set)

**Not applicable in this epic (C-125).** "Every parser touched is fuzzed": this epic adds no parser. The slice header, announcement and sync reply formats belong to epic 004 and the regime stamp format to epic 013 (C-144), and the Rust types here only compute; if a parser is ever added under this epic it is fuzzed to the budget of C-95.

## Open sub-questions

Decided since the ticket split, and no longer open: the 16-bit epoch width (C-10, decided; the wrap analysis checks its falsifier); the 64-slice lead in a deep graph (C-43, decided and flat; L0 task 017 confirms it); Q-7, the holder stall threshold (C-136: monotonic clock against a clock that runs through suspend, divergence above 2 ms); Q-8, the health of an other-epoch slice (C-135: restarts the dead clock, not a miss, own counter, named other-epoch in C-159); Q-9, the crate of the timeline types (C-133: `hadronyche-core`).

| Id | Question | Owner | Falsifier |
|---|---|---|---|
| Q-1 | Which rounding rule maps positions off a slice boundary to nanoseconds and back? | This epic (spec section on the position map) | `position_at(time_of(p)) ≠ p` for some position `p` |
| Q-2 | What does `position_at` return for a network time before `epoch_origin`? | This epic (spec section on the position map) | Two implementations return different results for the same time before the origin |
| Q-3 | How is `epoch_origin` chosen at cold start and after a bump? | This epic (spec section on the sync reply) | Two modules reconstruct different lines from the same sync replies |
| Q-4 | What is the sign convention of `step_ns`, and its value when the step cannot be measured (stall, unlocked takeover)? | This epic (spec section on the discontinuity event) | Two modules report different `step_ns` for the same bump |
| Q-5 | What does a source do if it is in Locking or holdover when the epoch changes? | This epic, with the clock state definitions from epic 005 | It announces a switch computed from a stale line |
| Q-6 | How is the losing side of a partition merge determined? | Epic 005 | Both sides of a healed partition keep their timeline epoch, or both bump |
