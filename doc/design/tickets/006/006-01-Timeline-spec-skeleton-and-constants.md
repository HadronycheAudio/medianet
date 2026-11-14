# 006-01 — Timeline spec skeleton and constants table
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** none
## Context
Epic 006 produces a normative document, `doc/design/spec/timeline.md`, that specifies the domain timeline and its epochs: how a position maps to network time, what bumps the timeline epoch, how each party learns of a new epoch, how sources switch at an announced position, and what a fan-in does with inputs on different epochs. Later tickets of this epic each fill one section. This ticket creates the file with its outline and, first of all, its constants table, so that every later section refers to constants by id.

Product frame (repeat in the document's introduction): Hadronyche Medianets are wired, isolated IPv4 LANs (C-100) whose only traffic is this runtime's. Modules discover each other, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone and the only clock holders in v1; macOS and Windows machines are edges; embedded devices are followers. The wire carries one sample rate only, 96 kHz (C-01); an edge whose hardware runs at another rate converts at the edge (C-80), and a resampler node may appear only inside an edge module (C-134); its delay is counted in D, never in L (C-207).

Evidence tags used in the document: `[code]` read in the POC, `[decision]` design choice (plus `[measured]`, `[kernel-doc]`, `[memory]` if a later section needs them, and `[standard]` for a figure cited from a named, published standard with its clause, which replaces `[memory]` once the citation is checked, C-132). The file lives in `doc/design/spec/`, beside the design documents (C-130).

Constants the document uses, copied from the design's constants table `doc/design/02-constants.md`, which wins on any difference:

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s |
| C-10 | Stream number / session / timeline epoch widths | 16 / 32 / 16 bits |
| C-18 | Regime stamp transport | dedicated sectioned control message; multicast on any regime change, refreshed every 10 s; the hello carries a stamp version |
| C-19 | Session number ownership | assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer sees the same value |
| C-30 | Hello period | 1 s |
| C-33 | Election window | 400 ms |
| C-37 | Election on reply silence | 1.5 s |
| C-38 | Interrupted threshold | 8 consecutive missed slices (4 ms at defaults) |
| C-43 | Announcement lead | 64 slices (32 ms at defaults), flat |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member |
| C-47 | Third-party observability of stream health | per-connection summary counters per gap class (received, late, missing, marked gap) in each party's regime stamp |
| C-61 | Rate of change of rate (handover relaxation) | at most 1 ppm per second (pending simulation, epic 008) |
| C-64 | Holdover bound growth | 1 µs per second |
| C-65 | Alignment claim lapse in holdover | about 25 s |
| C-67 | Step threshold | 2 ms |
| C-78 | L2 handover test | 50 ppm crystal difference |
| C-80 | Edge hardware at 48 kHz | converts 2:1 at the edge; the wire stays at C-01 |
| C-102 | Miss for the interrupted threshold | only absent slices; a slice on another timeline epoch resets the count |
| C-105 | Structural latency L | always a whole number of domain-rate samples; the earlier rational-L rule is withdrawn |
| C-111 | Start of the dead threshold | 2 s from the last slice received with content or a marked gap |
| C-113 | Election when no candidate is locked | lowest module id among holder-eligible candidates wins and bumps the timeline epoch |
| C-120 | Test-only injection hooks | include forcing a stream onto a given timeline epoch |
| C-121 | Home of wall-clock step detection | the platform backend; the servo only receives exchanges that passed it |
| C-122 | Degraded flag on an output with latency L | output at p is degraded if its input at p − L was gapped |
| C-124 | Laptop edge sleep | a departure; returns through Locking, Aligning |
| C-134 | Where a resampler node may appear | only inside an edge module; its conversion delay is part of D and never counted in L (C-207) |
| C-135 | A slice on another timeline epoch | restarts the dead clock (C-111), is not a miss (C-102), has its own per-connection counter |
| C-136 | Holder stall | monotonic clock compared with a clock that runs through suspend; divergence above 2 ms (C-67) bumps the timeline epoch |
| C-137 | Header cap | at most 100 bytes for all headers and the checksum in a default slice; in-band announcements fit inside it or are omitted on a fallback path |
| C-159 | Per-connection counters in the stamp | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-141 | Transport state on a gap | last known state held; playhead advanced as that state dictates; output marked degraded |
| C-162 | Clock state after the holdover claim lapses | Unlocked, with time unusable |
| C-169 | Alignment claim in holdover when E is unknown | unknown E taken as 2 × C-62 = 50 µs; lapse at about 25 s |
| C-181 | Where controllers travel | bundle source: own member stream; single-stream source: in the audio packet within the 100-byte cap, else a second stream |
| C-184 | Transport state before any has arrived | stopped, playhead at zero, 120 beats per minute |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view |
| C-188 | Where the holder answers sync requests | on the clock thread, never on housekeeping |
| C-189 | Two kinds of resampler | fixed-ratio DSL node (2:1, 1:2) and the edge's steerable rate actuator; the actuator's delay is part of D |
| C-195 | Holdover entry delay | pending simulation (epic 008b); at or below C-37, so a follower is in holdover when it calls an election |
| C-197 | Who builds the resamplers | the arithmetic kit (epic 024a) builds both; every edge uses them |
| C-200 | Playhead through a gap while stopped | does not advance |
| C-202 | Sync loss at L2 | no separate scenario; the handover run asserts emission error within the target E while the sink is in holdover between holders |
| C-205 | Controllers of a bundle source | always in their own member stream |
| C-207 | Resampler delay and L | counted only in D, never in L |
| C-209 | Controller member and the bundle cap | the controller member counts towards the 32 of C-08 |
| C-212 | How the L2 handover is triggered | as a holder loss, through about 2 s of holdover (C-37 plus C-33); frequency continuity asserted at takeover |

## Work
1. Create `doc/design/spec/timeline.md` with:
   - a title, a status line ("normative, epic 006"), and the product-frame paragraph above;
   - the constants table above, with a status column copied from `doc/design/02-constants.md` (C-61 and C-195 pending simulation; the others decided or derived), as the first section after the introduction, and one sentence stating that the constants table wins on any difference;
   - empty, titled sections (one line "to be written in 006-TT" each) for: Domain line and position map (006-02); Epoch width and wrap (006-03); Sync reply and line reconstruction (006-04); Epoch-bump causes (006-05); Discontinuity event (006-06); Handover without a bump (006-07); Source-side switch procedure and session numbering (006-09); Modules in Locking or holdover (006-10); Consumer-side procedures (006-11); Health of an input on another epoch (006-21); Announcement ordering and loss (006-12); Sequence diagrams (006-13..15); Scenarios (006-16..18); Regime stamp epoch field (006-19); Open items (006-20).
2. Add a short "Retired POC behaviour" note: `[code]` in the POC the media clock is the sender's local crystal: the data-plane ticker runs on local monotonic time (`module.rs:1630-1634`), and each stream carries its own `sample_pos` origin. Both are retired.
3. State the evidence-tag convention in the document.

## Acceptance criteria
1. `doc/design/spec/timeline.md` exists and its first section after the introduction is the constants table with exactly the 47 rows above, whose values match `doc/design/02-constants.md`.
2. Every section listed in Work step 1 is present as a heading.
3. The retired POC behaviour note carries the `[code]` tag and cites `module.rs:1630-1634`.
4. The file renders as Markdown without errors.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No placeholder section lacks the ticket id that fills it

## Out of scope
- The content of each section (006-02 to 006-21).
- Changing any constant value; values come from the design's constants table.
- Wire formats (slice header, announcement, sync reply): epic 004; the regime stamp message: epic 013 (C-144).
