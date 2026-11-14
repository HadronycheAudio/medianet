# Epic 006 — Timeline epoch specification: tickets

## Summary
Epic 006 (phase P1, depends on epics 004 and 005, blocks 007, 008, 009, 010) specifies the domain timeline and its epochs: one timeline for the whole domain at the only wire rate, 96 kHz (C-01), with `position = (network_time − epoch_origin) × 96 000 Hz`, affine per 16-bit timeline epoch and never extrapolated across epochs; the four causes that bump the epoch (partition merge for the losing side, unlocked holder takeover per C-113, holder stall, step > 2 ms); how modules learn a new epoch from sync replies and how their sources announce a switch 64 slices ahead (flat, C-43), re-issued every 16 slices, with a bumped session number (C-19); how single-input modules, fan-ins and sinks handle inputs on different epochs, including their health accounting (C-102, C-122, C-135, C-159); and the epoch field of the regime stamp (C-18; message defined by epic 013, C-144, a cross-reference rather than a dependency, C-196). The output is the normative document `doc/design/spec/timeline.md`, three mermaid sequence diagrams, three prose scenarios for conformance vectors (epic 007), and the Rust types `TimelineEpoch`, `Line` and `Discontinuity` in `hadronyche-core` (C-133).

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 006-01 | Timeline spec skeleton and constants table | XS | none |
| 006-02 | Position map arithmetic | S | 006-01 |
| 006-03 | Timeline epoch width and wrap analysis | XS | 006-01, epic 004 |
| 006-04 | Sync reply contents and choice of epoch origin | S | 006-02, 006-03, epic 004 |
| 006-05 | Epoch-bump causes: detector, publisher, event | S | 006-04, epic 005 |
| 006-06 | Timebase-discontinuity event | XS | 006-05 |
| 006-07 | Handover without an epoch bump | XS | 006-05 |
| 006-08 | Timeline Rust types and inverse tests | S | 006-02, 006-03, 006-06 |
| 006-09 | Source-side switch procedure and session numbering | S | 006-05, 006-06, epic 004 |
| 006-10 | New epoch learned during Locking or holdover | S | 006-09, epic 005 |
| 006-11 | Consumer-side procedures: single input, fan-in, sink | S | 006-09 |
| 006-12 | Announcement ordering and loss survival | S | 006-11 |
| 006-13 | Partition merge sequence diagram | XS | 006-05, 006-06, 006-09 |
| 006-14 | Unlocked holder takeover sequence diagram | XS | 006-05, 006-06, 006-09 |
| 006-15 | Holder stall sequence diagram | XS | 006-05, 006-06, 006-09 |
| 006-16 | Scenario: two sources switch 10 slices apart into one fan-in | XS | 006-11, 006-12, 006-21 |
| 006-17 | Scenario: announcement lost three times, received on the fourth | XS | 006-12 |
| 006-18 | Scenario: a source that never switches | XS | 006-11, 006-12, 006-21 |
| 006-19 | Epoch field of the regime stamp | XS | 006-03, 006-09 (epic 013 is a cross-reference, C-196) |
| 006-20 | Timeline spec close-out: open items, tags, rendering | XS | 006-01 … 006-19, 006-21 |
| 006-21 | Health accounting of an input on another epoch (C-38, C-102, C-108, C-111, C-122, C-47, C-135, C-159) | XS | 006-11 |

No ticket was deleted and none was added in the final refinement. Rulings removed work inside tickets without removing a whole ticket: the 48 kHz and 44.1 kHz arithmetic, cross-rate tests and the `rate_hz` field (C-01, C-80) left 006-02, 006-04 and 006-08; the deep-graph decision left 006-12 (C-43). The withdrawn rational-L rule (C-105 revised) had produced no ticket and no rational arithmetic in this epic; only the statement "L is an exact rational number" in 006-01 and 006-11 was replaced by "L is always a whole number of samples". The new rules C-127 to C-176 add work inside existing tickets only (C-136 in 006-05 and 006-15, C-135 and C-159 in 006-21, 006-18 and 006-19, C-137 in 006-12, C-162 in 006-10), so no ticket was added. C-196 turns the link to epic 013 into a cross-reference inside 006-19 and the epic's header; C-135 and C-136 were already applied. No ticket was added. The last sweep (C-177 to C-212 and the late revisions) also adds work inside existing tickets only (006-01, 006-02, 006-04, 006-05, 006-07, 006-09, 006-10, 006-11, 006-14, 006-15, 006-19, 006-20); no ticket was added or deleted.

## Effort tally
- XS: 13 (3.25 days)
- S: 8 (8 days)
- M: 0
- Total: 21 tickets, about 11 days (11.25); epic scope M (C-126)

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| 1. `doc/design/spec/timeline.md` (file, constants table) | 006-01, 006-20 |
| 1a. Position ↔ network-time arithmetic at 96 kHz, rounding off a slice boundary, before-origin behaviour, `epoch_origin` representation | 006-02 |
| 1b. Sync reply contents for line reconstruction (epoch, origin, holder identity) | 006-04 |
| 1c. Complete list of bump causes with detector and publication | 006-05 |
| 1d. Source-side procedure (learn → +64 slices → announce every 16 → switch with bumped session) | 006-09 |
| 1e. Consumer-side procedure for single-input modules, fan-ins, sinks | 006-11, 006-12 |
| 1e. Health accounting of an other-epoch input (C-38, C-102, C-108, C-111, C-122, C-47, C-135, C-159) | 006-21 |
| 1e. Announcements omitted on a fallback path (C-137) | 006-12 |
| 1f. Module learning a new epoch while in Locking or holdover | 006-10 |
| 1g. Discontinuity event and payload (old epoch, new epoch, step size) | 006-06 |
| 1h. Session numbering across an epoch switch (C-19) | 006-09 |
| 1i. Epoch field of the regime stamp | 006-19 |
| 2. Sequence diagram: partition merge | 006-13 |
| 2. Sequence diagram: unlocked holder takeover | 006-14 |
| 2. Sequence diagram: holder stall | 006-15 |
| 3. Scenario: two sources into one fan-in switch 10 slices apart | 006-16 |
| 3. Scenario: announcement lost three times, received on the fourth | 006-17 |
| 3. Scenario: a source that never switches | 006-18 |
| 4. Types `DOMAIN_RATE_HZ`, `TimelineEpoch`, `Line` (`position_at`, `time_of`), `Discontinuity` | 006-08 |
| Q-1 rounding rule, Q-2 before origin | 006-02 |
| Q-3 choice of `epoch_origin` | 006-04 |
| Q-4 `step_ns` sign and unknown value | 006-06 |
| Q-5 source in Locking or holdover | 006-10 |
| Q-6 losing side (cited, answered by epic 005) | 006-05, 006-13 |
| Former Q-7, stall detection (C-136) | 006-05, 006-15 |
| Former Q-8, other-epoch counter and dead clock (C-135, C-159) | 006-21, 006-18, 006-16, 006-19 |
| Former Q-9, crate `hadronyche-core` (C-133) | 006-08 |
| Clock state after the holdover claim lapses (C-162); unknown E in holdover (C-169) | 006-10, 006-15 |
| Takeover from holdover is a continuous handover (C-195, C-212) | 006-07, 006-05, 006-14, 006-15 |
| Holder publishes the epoch from its clock thread (C-188) | 006-04, 006-05 |
| Every stream of a source switches, controller member and controller stream included (C-181, C-205, C-209) | 006-09 |
| Gap rules for transport state on an other-epoch input (C-141, C-184, C-200) | 006-11 |
| Resampler delay in D, never L; kit builds both resamplers (C-134, C-189, C-197, C-207) | 006-01, 006-02 |
| C-10 falsifier check (16-bit wrap) | 006-03 |
| C-43 deep-graph argument (L0 confirmation in task 017) | 006-12 |
| Dependency to 008: published view's epoch field; handover/bump distinction | 006-19, 006-07 |
| DoD: regime stamp epoch field | 006-19 |
| DoD: whole-epic checklist, not-applicable line (C-125) | 006-20 |

### Acceptance criteria

| Epic acceptance criterion | Tickets |
|---|---|
| 006-AC-1 `position_at`/`time_of` exact inverses at slice boundaries at 96 kHz; rounding off a slice boundary stated and tested by example | 006-02, 006-08 |
| 006-AC-2 every bump cause has a detector, a publisher and a resulting event | 006-05, 006-06 |
| 006-AC-3 announcement processed before its named position at every descendant, no gossip assumption | 006-12 |
| 006-AC-4 survives loss of 3 consecutive carrying slices; residual 4-lost case and outcome stated | 006-12, 006-17 |
| 006-AC-5 (system AC-18) fan-in never aligns across epochs; other-epoch input is a gap | 006-11, 006-21, 006-16, 006-18 |
| 006-AC-6 (system AC-36) one holder within 1.4 s after a healed partition; losing side changes epoch, streams restart at announced position; no silent step | 006-05, 006-13 |
| 006-AC-7 (system AC-5) 50 ppm handover (C-78) does not bump the epoch | 006-07, 006-14 (contrast note) |

## Remaining gaps
1. **Losing side of a partition merge (Q-6).** The epic says the losing side bumps but neither the epic nor the constants table says how "losing" is decided. Affects 006-05, 006-13. Answer from: epic 005 (control state machines).
2. **Definitions of Locking and holdover (part of Q-5).** C-124, C-65 and C-162 use the states (C-162 fixes the state after the holdover claim lapses: Unlocked, time unusable), C-195 fixes holdover entry relative to the election (at or below C-37), and C-187 says the servo core publishes the lock state; none of them defines Locking and holdover. 006-10 uses a provisional reading. Affects 006-10, 006-15. Answer from: epic 005.
6. **C-195 pending.** The holdover entry delay itself (expected at or below 1.5 s). Affects 006-07, 006-10, 006-14, 006-15, which use only the bound. Answer from: the simulation (epic 008b).
3. **Regime stamp layout for the epoch field.** C-18 settles the transport, C-47 and C-159 the counters, and C-144 says epic 013 defines the message; which section carries the epoch field and how a record set is delimited on the wire is a layout question. It no longer blocks: epic 013 is a cross-reference (C-196), and 006-19 states the field as a requirement for epic 013 to place. Affects 006-19. Answer from: epic 013.
4. **Rounding rule, before-origin behaviour, origin choice, `step_ns` convention (Q-1 to Q-4).** Chartered decisions of this epic, made inside its tickets and recorded as `[decision]`. Affects 006-02, 006-04, 006-06, 006-08. Answer from: this epic.
5. **C-61 pending.** Handover relaxation at most 1 ppm/s. Affects 006-07. Answer from: the simulation (epic 008).

None of the remaining gaps needs the design owner.

## Settled since the split
Former gaps (numbered as in the original "Gaps in the epic") and former open sub-questions that a constant or an epic change now closes:

1. *Gap 1, C-61 missing from the constants table* — epic change: C-61 is in the epic's Related constants and in 006-01.
2. *Gap 4 (rate part), non-dividing rates* — C-01, C-80, C-134: the wire carries 96 kHz only and a resampler node may appear only inside an edge module, so the 44.1 kHz and 48 kHz arithmetic and the `rate_hz` field are gone. C-105 (revised) makes L always a whole number of samples; the earlier rational-L rule is withdrawn. The rounding direction off a slice boundary stays open as Q-1 (remaining gap 4).
3. *Gap 5, "flat" in C-43 and slice length at other rates* — C-43 (64 slices, flat, not scaled with depth) and C-01 (one wire rate).
4. *Gap 8 (in part), Locking and holdover* — C-124 (an edge that sleeps is a departure and returns through Locking, Aligning), C-65 (alignment claim lapses after about 25 s in holdover) and C-162 (after the lapse: Unlocked, time unusable). The state definitions stay with epic 005 (remaining gap 2).
5. *Gap 9 (in part), regime stamp not defined* — C-18 (sectioned control message, multicast on regime change, 10 s refresh, version in the hello), C-47 and C-159 (per-connection counters). The field's place in the layout stays with epic 013, which defines the message (C-144; remaining gap 3).
6. *Gap 11, "Every parser touched is fuzzed" in a specification epic* — C-125: marked not applicable with its reason in the epic, 006-08 and 006-20; C-95 gives the budget if a parser is ever added.
7. *Open sub-question, 16-bit epoch wrap* — C-10 decided; 006-03 checks its falsifier.
8. *Open sub-question, 64-slice lead in a deep graph* — C-43 decided and flat; L0 task 017 confirms it.

Also settled, though not listed as gaps before: session numbering across a switch (C-19); the winner of an unlocked takeover (C-113); the health of an other-epoch input for the interrupted count and the degraded flag (C-102, C-122; new ticket 006-21); the location of the specification file (C-130); the `[standard]` evidence tag (C-132). Epic changes: acceptance criteria now have ids 006-AC-1 to 006-AC-7, so they no longer clash with the system AC numbers; epic 005 added to the dependencies; scope recomputed per C-126.

Settled in the final refinement (rules C-127 to C-176 and the revised C-104, C-105):
- *Q-7 / gap 6, holder stall threshold* — C-136: the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above 2 ms (C-67) bumps. 006-05, 006-15.
- *Q-8, other-epoch slice in health accounting* — C-135 (restarts the dead clock, not a miss, own counter) and C-159 (the counter is "other-epoch"). 006-21, 006-18, 006-16, 006-19.
- *Q-9 / gap 2, crate of the timeline types* — C-133: all protocol types live in `hadronyche-core`. 006-08.
- *Structural latency L* — C-105 revised: always a whole number of samples; C-134: resampler nodes only inside edge modules. 006-01, 006-02, 006-08, 006-11.
- *Announcements on a fallback path* — C-137: they fit the 100-byte header cap or are omitted; the outcome is the residual case. 006-12.
- *Clock state after the holdover lapse* — C-162: Unlocked, time unusable. 006-10, 006-15.
- *Owner of the regime stamp message* — C-144: defined by epic 013, sent by epic 015. 006-19 now depends on epic 013 instead of 004; the epic's dependencies list 013.
- *L2 measurement of a handover without a step* — C-166 (proposal): ±1 sample on Linux. Cited in 006-07.

Settled after the final refinement:
- *Dependency on epic 013* — C-196: a specification epic never blocks on an epic of a later phase, so epic 013 is a cross-reference, not a dependency. The epic header and Dependencies section, and 006-19's header, Work and acceptance criteria, now treat it so; this supersedes the earlier "006-19 now depends on epic 013". 006-19.
- *C-135 and C-136* — rows re-checked; the epic, 006-05, 006-06, 006-15, 006-16, 006-18 and 006-21 already state them, with no provisional reading left to correct.

Settled in the last sweep (rules C-177 to C-212 and the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184):
- *Resamplers and L* — C-134 revised, C-207: an edge's resampler delay is in D, never in L; C-189: the edge also has a steerable rate actuator, its delay in D; C-197: the kit (epic 024a) builds both, so 006-02's "edge epics" out-of-scope line is corrected. Epic, 006-01, 006-02.
- *Holder side of the sync reply* — C-188: the holder answers sync requests, and so publishes the epoch, on its clock thread. Epic, 006-04, 006-05.
- *Handover through holdover* — C-195 (holdover entry at or below C-37) and C-212 (the L2 handover is a holder loss with about 2 s of holdover and frequency continuity at takeover; C-202, emission error within the target E during holdover): a candidate locked to the old holder that takes over from holdover is a continuous handover, not an unlocked takeover. Epic (Background and 006-AC-7), 006-05, 006-07, 006-14, 006-15.
- *Lock state publisher* — C-187: the servo core (epic 008a) publishes the lock state and the published time view, a cross-reference. Epic, 006-10, 006-19.
- *Unknown E in holdover* — C-169: same about 25 s lapse. Epic, 006-10.
- *Controller streams* — C-181, C-205, C-209 (C-08 revised): the controller member counts as a bundle member and announces and switches; a single-stream source's overflow controller stream switches too. Epic, 006-09.
- *Transport state on an other-epoch input* — C-141, C-184, C-200: the gap rules apply. Epic, 006-11.
- *Conformance vectors* — C-193: epic 006 supplies the timeline and epoch scenarios. Epic Dependencies.
- No subject in this epic: C-153, C-177 to C-180, C-182, C-183, C-185, C-186, C-190 to C-192, C-194, C-198, C-201, C-203, C-204, C-206 (no tool presentation detail and no design-owner item here), C-208, C-210, C-211.

Not settled: gap 10 (losing side, remaining gap 1). Gaps 3 and 7 (before-origin behaviour, `step_ns` convention) are owned open sub-questions Q-2 and Q-4 of this epic (remaining gap 4).
