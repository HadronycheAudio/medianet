# Epic 026a — Embedded HAL and host integration: tickets

## Summary

Epic 026a (phase P6, scope L: 13 tickets, about 18 days) is the first lettered part of epic 026, split under C-143 (an epic above 40 days by ticket tally is split into lettered parts; a reference to epic 026 means both parts). It brings the Hadronyche runtime to a `no_std`, no-alloc microcontroller as far as can be built and verified on a host. It defines the hardware abstraction crate `hadronyche-embedded` (five traits of hardware surface: `Transport`, `DeviceTime`, `AudioOut`, `Identity` defined here, and `RateActuator` defined by the kit epic 024a and re-exported, C-210; a board supplies four and `Device<T, C, A, I>` holds the software-ratio `RateActuator`, an adapter over the arithmetic kit's steerable rate actuator, internally; both resamplers come from epic 024a, C-197), with host mocks, CI jobs and the no-alloc / no-libm symbol check. On top of it, it integrates the unchanged `hadronyche-core` state machines into the device's cooperative poll loop — identity derivation, discovery with a 63-entry peer map (C-217) and follower-only election, servo and holdover with the alignment claim, admission against declared buffer memory, the media receive path with receiver reports, the audio edge's two-loop timebase — and implements only transport, time and audio hardware `[decision]`. The embedded profile declares, field by field, the guarantees that weaken without an OS; the device publishes its regime stamp in sections over the network (C-18), including the commissioning-window state (C-107, C-153), and a hosted collector in `hadronyche-std` records it, with a record format and file names decided here (C-206).

Part 026b builds the reference device firmware on a named board, runs the on-device measurements and acceptance tests, adds the remaining stamp fields and writes the porting guide. 026b depends on this part; no ticket here depends on 026b.

The epic criteria 026-AC-1 to 026-AC-12 are completed in 026b, which keeps their ids. This part's criteria are 026a-AC-1 to 026a-AC-9, the host-verifiable share of them (026a-AC-9 covers the audio edge on the host).

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 026a-01 | Crate skeleton and CI | S | none |
| 026a-02 | HAL traits and host mocks | S | 026a-01, epics 018, 024a |
| 026a-03 | Identity derivation | S | 026a-02, epic 011 |
| 026a-04 | Embedded profile declaration | S | 026a-02, epic 013 |
| 026a-05 | Device poll loop and follower discovery | M | 026a-02, 026a-03, 026a-04, epic 011 |
| 026a-06 | Follower servo and holdover | M | 026a-05, epic 008 |
| 026a-07 | Admission with declared memory limit | S | 026a-05, epic 012 |
| 026a-08 | Media receive path and receiver reports | M | 026a-07, epics 009, 010, 014 |
| 026a-09 | Software-ratio rate actuator | S | 026a-02, epic 024a |
| 026a-10 | Audio edge two-loop timebase | M | 026a-06, 026a-08, 026a-09, epic 018 |
| 026a-11 | Stamp publication over network | S | 026a-04, 026a-05, epic 013 |
| 026a-12 | Hosted stamp collector | S | 026a-11, epic 013 |
| 026a-13 | Commissioning window in stamp (new, C-107) | XS | 026a-11, epic 005 |

The rows are in a valid execution order. Ticket 026a-01 uses the target triple of the board chosen in 026b-01 when it exists and a generic Cortex-M target otherwise, so it has no dependency on 026b.

## Effort tally

- XS: 1 (0.25 days)
- S: 8 (8 days)
- M: 4 (10 days)
- Total: 13 tickets, about 18 days (18.25) — scope L by C-126 (more than 15 and at most 40 days)

Together with 026b (22 tickets, 22.75 days) the epic still totals 35 tickets and 41.0 days.

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| Crate `hadronyche-embedded` (no_std, no alloc), CI jobs, C-95 fuzz job, no-alloc / no-libm symbol check | 026a-01 |
| `Transport`, `Rx`, `StampSource`, `DeviceTime`, `AudioOut`, `Identity` (with `domain_id`, C-173) traits, and the kit's `RateActuator` re-exported (C-210); `Endpoint` (IPv4), `DscpClass` (C-13), `DevicePair`; host mocks | 026a-02 (definitions and mocks), 026a-03 (identity logic), 026a-09 (software-ratio `RateActuator` over the kit's actuator, separate from the fixed 2:1 node, C-189, C-197); the stored domain id on the board is 026b-02 |
| `Device<T, C, A, I>` with `poll() -> PollStats` | 026a-05, extended by 026a-06, 026a-07, 026a-08, 026a-10, 026a-11 |
| `profile.rs` — embedded capability declaration | 026a-04 |
| `stamp_publish.rs` — sectioned publication on change, every 10 s and on request | 026a-11, with the commissioning-window field from 026a-13; further fields are added in 026b (026b-11, 026b-12, 026b-14, 026b-19, 026b-20) |
| Hosted stamp collector in `hadronyche-std`, record format and file names decided here (C-206) | 026a-12 |
| Open sub-question: collector record format | 026a-12 |

### Acceptance criteria

| Part criterion | Tickets |
|---|---|
| 026a-AC-1 (host share of 026-AC-1, AC-35): follower, 63-entry peer map (C-217), bounded work per `poll` with worst case reported, stamp published in sections and collected | 026a-04, 026a-05, 026a-11, 026a-12 |
| 026a-AC-2 (host share of 026-AC-3): follower only even with lowest id, holdover after the entry delay (C-195), 1 µs/s, claim dropped at declared E or at 2 × C-62 with E unknown (C-65, C-169), Unlocked after (C-162), resumed on relock without re-commissioning (C-180); takeover by a usable-time candidate keeps the epoch and the stream (C-213, C-221) | 026a-05, 026a-06 |
| 026a-AC-3 (host share of 026-AC-4): identity with true random disabled; id stable, nonce new; domain 0 ignored and counted (C-173) | 026a-03, 026a-05 |
| 026a-AC-4 (host share of 026-AC-7): liveness rules on the host | 026a-08 |
| 026a-AC-5 (host share of 026-AC-9): refusal of a second stream on buffer memory, named reason | 026a-07 |
| 026a-AC-6 (host share of 026-AC-10): D with components and method, E `Certified` or unknown, in the stamp | 026a-04, 026a-10, 026a-11 |
| 026a-AC-7 (host share of 026-AC-11): no allocator, no libm in the device build | 026a-01 (check), and every code ticket's device build |
| 026a-AC-8 (host share of 026-AC-12): C-129 states incl. C-147 and C-161, C-131 source, C-107/C-153 window, recorded by the collector | 026a-04, 026a-11, 026a-12, 026a-13 |
| 026a-AC-9: audio edge on the host, emission at `T(p) + A` within 1 frame through the kit's node and actuator | 026a-09, 026a-10 |

### Definition of done

| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all tickets (026a-01 to 026a-13) |
| Every parser touched is fuzzed (C-95 budget, CI job in 026a-01) | 026a-04, 026a-05, 026a-06, 026a-07, 026a-08, 026a-11, 026a-12 |
| Regime stamp fields touched are complete and documented | 026a-04, 026a-07, 026a-10, 026a-11, 026a-12, 026a-13 |
| Design documents updated where behaviour differs, constants table first | 026a-03; the epic-wide update is 026b-22 |
| No new open item without an owner and a falsifier | 026a-03, 026a-05, 026a-06 |
| The frozen POC is not modified | 026a-01, 026a-12 |
| Every new factual claim carries an evidence tag (`[standard]` added by C-132) | all tickets |
| Crate builds for a device target and for the host in CI | 026a-01 onward |
| Reference board, timestamp source and oscillator named in docs | not applicable in this part (C-125): no board here; 026b-01, 026b-02, 026b-03, 026b-07, 026b-21 |
| "What weakens on no_std" reflected field by field in the stamp | 026a-04, 026a-11 |

## Settled since the split

- Split under C-143 — with 026b-13 (formerly 026-35) epic 026 tallied 41 days, above the 40-day line, so C-143 called for lettered parts. Settled by this split: 026a (host-testable HAL, core integration, stamp publication and collector, 13 tickets, 18.25 days) and 026b (reference device firmware, on-device measurements and tests, remaining stamp fields, porting guide, 22 tickets, 22.75 days). No ticket content changed; tickets were renumbered and their references updated. Former gap 10 is removed.
- Gap "trait count inconsistent" — epic change: five traits, `RateActuator` held inside `Device` with the software-ratio default; porting guide (026b-21) covers five.
- Gap "undefined types" — C-100 (`Endpoint` is IPv4 address + port), C-13 (`DscpClass`), C-131 (`StampSource` values; `Task` renamed `Userspace`), epic change (`DevicePair`, `PollStats` described).
- Gap "identity mixing unspecified" — C-11, C-46: module id stable across reboots, nonce new per boot; the mixing function is an implementation detail.
- Gap "stamp publication details missing" — C-18: sectioned control message, multicast on change and every 10 s, unicast request, hello carries a version number (collector record format and file names: this epic, 026a-12, C-206; record location C-171).
- Gap "holdover margin undefined" — C-65 with C-73: the margin is the device's declared E.
- Gap "AC-7 timing ambiguous" — C-41 and C-111: each 2 s timer starts at its own last event; tests stay at 1 s and 3 s.
- Gap "AC-9 limit ambiguous" — C-04 and C-96: a 7th channel is a second stream, refused on declared memory sized to A; the port cannot refuse it (reason name remains, gap 2).
- Gap "skew-buffer rate ambiguous" — C-01, C-80, C-96: the ring holds wire slices at 96 kHz; 23 040 bytes at A = 10 ms.
- Gap "2-channel vs 6-channel" — C-04: the stream is the unit of subscription; the device buffers 6 channels and plays what its codec has.
- Gap "optional capture source" — epic change: stated as not a deliverable of this epic.
- Epic sub-question "device E: bound or percentile" — C-73 (with C-103 for E unknown, C-140 for which certification state gives a number).
- Former gap 10 "strict mode and the embedded profile" — C-147: strict mode is hosted-only; a device stamps real-time scheduling and memory lock as present by construction (026a-04).
- Former gap 11 "alignment-claim margin with E unknown" — C-169 (corrected): an unknown E is taken as the fixed reference 2 × C-62 (50 µs); the device stops claiming alignment when the bound it currently publishes exceeds it, about 25 s into holdover at C-64. 026a-06 Work 5 and acceptance criterion 6 implement the lapse (the provisional last-lock reading is withdrawn); 026b-15 and the epic restated. C-162: after the lapse the clock is Unlocked with time unusable.
- Holdover lapse for a device with E unknown (gap 11 raised after the first C-169 revision) — settled by the corrected C-169 above; the matching epic sub-question is removed.
- Resampler and rational L — C-105 (revised): L is a whole number of samples and the resampler node is offered only at 2:1 and 1:2, which covers the device's 2:1; C-134: the node appears only inside an edge module and its delay is part of D; C-170 defines D (026a-09, 026a-10, 026b-05, 026b-14). No rational-number handling existed in these tickets, so nothing was removed.
- Control isolation — C-161: on a device, absent or unverified unless the bound of 026b-06 is shown (026a-04, 026b-06).
- Commissioning-window declaration — C-153: announced on the control group with an expiry time, closed at expiry or explicitly (encoding remains, gap 5; 026a-13).
- Liveness details — C-135 (other-epoch slices restart the dead clock and have their own counter), C-16 with C-156 (drain bound per slice period) (026a-08, 026b-11).
- Two kinds of resampler — C-189: the DSL resampler node is fixed-ratio (2:1 or 1:2, whole-sample delay, C-105); the rate actuator is a separate steerable resampler varying by parts per million, its delay part of D. The epic's "same resampler, with a steered ratio" reading is withdrawn; 026a-09 Work and acceptance criteria corrected (fixed node plus `SoftwareRatio` at nominal ratio 1 after it), 026a-10, 026b-03, 026b-04, 026b-05 and 026b-14 restated. The ABI's "frequency-dependent delay" property is epic 024's and needs no work here.
- Who builds the resamplers — C-197: the arithmetic kit (epic 024a) builds both the fixed 2:1 node and the steerable rate actuator; the device uses the kit's and builds none. Epic 024a added to the epic's header and Dependencies and to 026a-09's. The provisional reading in 026a-09 (its own steerable resampler in `hadronyche-embedded`) is withdrawn: Work and acceptance criteria corrected so `SoftwareRatio` is an adapter over the kit's actuator, with a new criterion that the crate holds no resampling filter of its own. A shorter filter or other change to the actuator is filed with epic 024a (026b-05, epic sub-question, gap 4). 026a-02, 026a-10 and 026b-21 restated. No new ticket: the adapter replaces the former implementation inside 026a-09's S.
- Home of the rate-actuator trait — C-210: defined in the kit epic (024a) with its implementation, ratio range ±1 000 ppm; every edge consumes it. The reading in which epic 018 owned the trait and `hadronyche-embedded` defined it is withdrawn: 026a-02 re-exports the kit's trait (Work and criterion 1 corrected, epic 024a added to its dependencies), 026a-09 implements the kit's trait with a ±1 000 ppm range test (epic 018 dropped from its dependencies), 026b-05, 026b-21, the epic's Deliverables, Dependencies and sub-question, and gap 4 restated with owner epic 024a.
- What an open commissioning window disqualifies — C-178: production figures in stamps are unqualified while a window is open; the tool's own measurements (D, restart spread, certification) are valid commissioning results recorded with their window. 026a-13, 026b-12, 026b-14 and the epic restated.
- Certification of a sink whose clock is unlocked — C-180: the device stays commissioned, its claim is suspended and resumes on relock without re-commissioning. 026a-06 Work 5 and criterion 5, 026b-15 criterion 5, epic 026-AC-3 and Background restated.
- Refusal reasons completed — C-185: 18 in all, so no buffer-memory reason is added (026a-07, 026b-18, gap 2, epic sub-question).
- A changed while live, its carrier and A beyond buffer memory — C-186, C-198, C-211: handled on the device by 026b-13; 026a-07 and 026a-08 point to it.
- Publisher of the clock lock state and bound — C-187: the servo core (epic 008a) through the published time view; 026a-06 reads them from there (dependency on epic 008 covers its parts, C-143).
- Holdover entry delay — C-195: pending simulation, at or below C-37; 026a-06 Work 4 and criterion 4, 026b-15 criterion 2 corrected (they had tied holdover entry to the 1.5 s election silence); gap 3 extended.
- Presentation details of tools — C-206: the device-stamp collector is built here (026a-12), so its record format and file names are decided here and documented; former gap 7 (owner epic 013) is removed and the epic sub-question reassigned to this epic.
- Resampler delay and L — C-207: conversion delay counted only in D, reported in the manifest on the device-facing port (026a-10 Work 5 and criterion 4, epic Background).
- Rules checked and not applicable to this epic: C-177 and C-201 (seqlock and CPU pinning; the device runs one cooperative loop with no threads to pin), C-179 (redundancy decision), C-181, C-205, C-209 and revised C-08 (controller placement and the bundle cap; producer and compiler rules, the device subscribes to whole streams), C-183, C-190 (compiler rules), C-184, C-199, C-200 and revised C-141 (transport state; no subject in the device's tickets), C-188 and C-204 (holder-side; the device sends 50 requests/s, inside the accepted range), C-192, C-193 (vector provenance and crate location; 026b-08 consumes epic 007's harness as it stands), C-194, C-202, C-212 (L2 and L3 rig parameters; the L3 rig is task 027), C-196 (specification epics), C-208 (fan-ins with an output; the device has none, 026b-13 out of scope). Revised C-134, C-153 and C-169 were already applied.
- Earlier rules checked and not applicable to this epic: C-137 (header cap; met by the producer, the device only parses), C-139 (hosted restart policy; on the device any fault is a reset), C-163 (last ratchet step; applied by core), C-136 (holder stall; the device is never holder), C-138, C-141, C-142, C-144 (duties of epic 015 only), C-146 (privileged tests; no ticket here needs privilege), C-148, C-149, C-150, C-151, C-154, C-155, C-158, C-165, C-166, C-167, C-175. C-35's owner change does not affect this epic.
- Rules C-213 to C-225 — applied in place, no ticket added or deleted, efforts unchanged: C-217 (peer map of 63 entries; a module holds no entry for itself): 026a-05 Facts, Work 3 and criterion 4, epic Background and 026a-AC-1. C-213 and C-221 (holder takeover by a usable-time candidate, epoch kept, Locked preferred, lowest id among equals): 026a-06 Context, new Work 7 and criterion 8 (the device follows the new holder without restarting its stream), epic Role and 026a-AC-2. C-215 and C-225 (an upstream fan-in's ratchet step announced in its output; the device's own step at its own position one lead ahead): 026a-08 Context, Work 3 and 4, criteria 8 and 9, epic Liveness. C-222 and C-216 (clamp applies to sinks; alarm held, given A in reports), C-214 (sink switch position) and C-224 (announcement by the tool inside a window): 026a-07 Context and epic Capacity point to 026b-13. C-219 (controller-stream cost): the reference device subscribes to audio members only, so no ticket computes it; 026a-07 and the epic note that admission (epic 012) would apply it. The encoding of the tool's announcement of a changed A is added to gap 5.
- Rules C-213 to C-225 checked and not applicable to this part: C-208 (fan-ins with an output), C-209 (bundle cap; producer and compiler rule), C-212, C-220, C-223 (L2 and L3 handover rig assertions; task 027 and epic 020), C-218 (compile-time rejections; epic 023).

New work created by rulings in this part: 026a-13 (C-107, C-153).

## Remaining gaps

1. **Allowed methods for "D with method".** What D contains is settled (C-170); the list of methods is not. Affects 026a-10 (and 026b-14). Owner: epic 018 (shared edge core).
2. **Which refusal reason a buffer-memory refusal carries.** The refusal rule is settled and the list is complete at 18 reasons (C-173, C-185), so no new reason is added; the table does not say which of the 18 applies. Affects 026a-07 (and 026b-18). Owner: epic 012 (admission).
3. **C-62 lock bound and holdover entry delay pending simulation.** Tests read 25 µs from the constants table and the holdover entry delay from the core servo (C-195: proposed by epic 008b, at or below C-37). Affects 026a-06 (and 026b-07, 026b-15). Owner: the servo simulation.
4. **`f64` without libm and soft-float cost.** `RateActuator::set_ratio(f64)` on a part that may lack a double-precision FPU. Affects 026a-09 (and 026b-05). Owner: measurement in 026b-05; epic 024a if the trait signature or the kit's actuator must change (C-197, C-210).
5. **Commissioning-window and changed-A message encoding.** The window life cycle is settled (C-153), and who announces a commissioned change of A and when (C-224); the encoding of either announcement is not. Affects 026a-13 (and 026b-13). Owner: epic 005 (control state machines) with the wire specification epic 004.
6. **How a domain id reaches a device.** C-173 makes peers ignore hellos with domain 0, so a device must hold a non-zero domain id; how it is written to the device's storage is not settled. Affects 026a-03 (and 026b-02). Owner: commissioning tooling with epic 005.

None needs the design owner.

## Ticket numbers before the split

| Old ticket | New ticket |
|---|---|
| 026-02 | 026a-01 |
| 026-03 | 026a-02 |
| 026-04 | 026a-03 |
| 026-05 | 026a-04 |
| 026-06 | 026a-05 |
| 026-07 | 026a-06 |
| 026-08 | 026a-07 |
| 026-09 | 026a-08 |
| 026-10 | 026a-09 |
| 026-11 | 026a-10 |
| 026-12 | 026a-11 |
| 026-13 | 026a-12 |
| 026-32 | 026a-13 |

The other 22 former tickets of epic 026 are in 026b; its index maps them.
