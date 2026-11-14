# 028a — Commissioning configuration and offline validation

**Phase:** P7 · **Scope:** M (11 tickets, about 8.75 days) · **Depends on:** 012 · **Blocks:** 028b

## Goal

Build the offline half of the operator-facing tool (`hadronyche-commission`): the crate and command-line skeleton, the commissioning configuration format, the capacity arithmetic, the `validate` command that checks a configuration with no network, and the pure A-proposal computation. Everything here runs on a desk with no medianet attached. Epic 028b builds on it the tools that need a running medianet (survey, commissioning window, `propose-a`, certification, burst test and soak, `watch`) and the operator checklists and guide. Together the two parts form the commissioning tool of epic 028, split under C-143 (a reference to epic 028 means both parts).

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline. Linux hosts are the backbone (clock holder, elections, conformance, **commissioning**); macOS and Windows machines are edges; embedded devices are followers. IP is **IPv4 only** in v1, multicast with TTL 1 (C-100).

**Boundary `[decision]`.** The runtime owns *when and whether*; module code owns *what*; the **operator owns how much** latency and capacity to spend. Operator policy is: the audition offset A per presentation group; the required E per group; strict mode; per-input alignment opt-out; pacing override; trunk capacities; the domain id. This tool is how the operator sets those knowingly. Strict mode now has a defined meaning: it requires real-time scheduling on the data and clock threads and locked memory; restricting CPU idle states stays best-effort and is stamped (C-127); a capability that is unverified is treated as absent and refused (C-128). Strict mode is hosted-only: a device stamps real-time scheduling and memory lock as present by construction (C-147). The commissioning configuration sets strict mode, per-input alignment opt-out and pacing override (C-165).

**Recommended configuration.** Wired, isolated LAN with no internet traffic. A single non-blocking switch is the simple case: every link is an endpoint link, so the runtime's own online endpoint admission is complete. With several switches, **trunk** links are invisible to endpoints.

**Where the code and outputs live.** The tool is a binary in the `hadronyche-tools` crate in the design set's repository (C-133). Any constant goes into `doc/design/02-constants.md` first (C-130); measured results are reports in `doc/design/reports/` (C-130). Nothing is written into the frozen POC tree.

**The split.** This part delivers the binary with a stub for every subcommand, so that epic 028b only fills stubs in. The hard rule for the whole tool (C-107) is fixed in the skeleton: outside a declared commissioning window that every module stamps, the tool only reads stamps and writes configuration (and sends the window announcement or close itself, C-153); inside the window it may drive test traffic and announce a commissioned change of A (C-224). Nothing in this part reads stamps or sends anything: `validate` runs with no network. The window, the stamps and every online subcommand belong to epic 028b.

**What this part does, and the rule behind each part.**

1. **Audition offset A — configured per group, proposed by a fixed rule.**
   - Every sink in a presentation group emits sample p at `emit(p) = T(p) + A` and submits it to hardware at `emit(p) − D` (D = that sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter, C-170).
   - A is **operator configuration**, one per presentation group (default group: the whole domain). **It has no default value** (C-55). Changing A is a domain-wide discontinuity.
   - A is also the skew budget cap: alignment rings are sized to A (C-96). A larger A therefore costs ring memory (about 370 KB per 16-member bundle at a 10 ms budget, C-27).
   - The **minimum feasible A** follows the commissioning rule: the largest, over the group's sinks, of (maximum lateness + D + headroom), the headroom being the consumer rule C-52 (C-148; the ladder harnesses use the same rule). This part implements the rule as a pure function; epic 028b collects the measurements and runs it (`propose-a`).
   - Lateness is what each consumer measures in its aligning state: 4 s of observation (C-50), offset = maximum observed plus headroom (C-51), headroom = the largest of 25 % of that maximum, one slice period (0.5 ms, C-02), twice the published clock bound (C-52). The clock bound is published by the servo core (epic 008a, C-187); here it is an input value. The stamp publishes the maximum over a rolling 60 s window, with the maximum since the connection went live beside it (C-182); the proposal uses the maximum since the connection went live, never the rolling 60 s figure (C-148).
   - Measurements are taken under a provisional A; a measurement in which any input reached that ceiling or was Demoted (C-109) is truncated and must not be used.
   - A fan-in outside any presentation group inherits the A of the downstream group it feeds (the largest if several), learned at admission; with no downstream group it needs an explicit budget in the configuration, or its aligned inputs are refused with reason NoSkewBudget (C-142, C-185).
   - **A against buffer memory (C-211, C-222).** Before applying a change of A (writing it to the configuration, `validate`, `propose-a --write`) the tool checks the declared buffer memory of every consumer that buffers to meet A, sinks included (C-222), against rings sized to the A it will hold (C-96; about 370 KB per 16-member bundle at 10 ms, C-27, that is channels × 96 000 × 4 bytes × A) and refuses a change that does not fit. A change of A reaches live fan-ins through the receiver reports, one hop per report (C-198), and each re-aligns with resized rings at its own announced position (C-186, C-208); each sink switches at its own position, one lead (C-43) ahead, with nobody to announce it to (C-214). The check therefore covers every aligning module the change reaches. If such an A reaches a consumer anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running (C-211, C-222); the alarm stays raised until a later change of A that fits, and its receiver report carries the A it was given, not the clamped value (C-216).
   - **Applying a commissioned change of A (C-224).** A commissioned change of a sink's own A reaches the sink through the tool, inside a declared commissioning window (C-153): the tool validates buffer memory first (C-211) and then announces the new A to the group's members. The check is this part's (028a-10); the window and the announcement are epic 028b's (028b-09, 028b-11).
   - Typical D: Linux about 1 ms; macOS a few ms; Windows 6 to 20 ms (C-81). A group containing a Windows edge needs an A of the order of 10 to 25 ms.

2. **Trunk validation — offline only.**
   - **The runtime never computes trunk feasibility.** Trunk capacities are **declared** per link in the commissioning configuration, and a declared capacity is **per direction** (C-117).
   - The tool validates the requested connection graph against the declared trunks offline, using the wire rates of item 3, per direction, and refuses a configuration that exceeds the ceiling.
   - The post-hoc alarm from the regime stamps (observed rates per link and direction versus declared) is `watch`, in epic 028b; it reuses this part's trunk walk.
   - Endpoint port admission stays online and dynamic in the runtime.

3. **Capacity arithmetic** the tool uses:
   - One sample rate on the wire: 96 kHz, the domain rate (C-01). Edge hardware at 48 kHz converts 2:1 at the edge (C-80); a resampler node exists only inside an edge module, only for 2:1 and 1:2 (C-105, C-134), and its delay is part of that edge's D (C-134, C-170). The arithmetic never sees another rate.
   - Default stream: 6 channels (C-04), 48 frames = 0.5 ms (C-02), 4-byte samples (C-03): 192 bytes per channel per slice, 1 152 bytes of audio per packet, no per-channel id byte (the channel map is in the manifest, C-04). About 1 320 bytes per packet on the wire including all Ethernet overhead (C-20), 2 000 packets/s, about **21.1 Mbit/s per stream, about 3.5 Mbit/s per channel** (C-21).
   - Per source: 24 channels = 4 streams = 84 Mbit/s = 8 000 packets/s; 48 = 8 streams = 169 Mbit/s = 16 000 packets/s; 96 = 16 streams = 338 Mbit/s = 32 000 packets/s (C-22).
   - **Admission ceiling 70 % of the link rate** (C-23).
   - **GbE port wall: about 198 channel-copies** (8 × 24, 4 × 48 or 2 × 96 channels) (C-24). **Any port carrying more runs 10 GbE** (C-25). Multicast fan-out is not available in v1.
   - A 100 Mbit device port holds about 19 channels (C-26).
   - The L1 two-bundle rig (C-92): one Linux consumer on one GbE port aligns 2 × 96 channels (32 streams, 64 000 packets/s, 676 Mbit/s).
   - Controllers: a bundle source's controllers always ride in their own member stream (C-181, C-205), which counts towards the 32 members of C-08 (C-209); a single-stream source's controllers share the audio packet within the 100-byte header cap, beyond which the source emits a second stream (C-181, C-137). The rates above are audio streams only. A source with controllers therefore has at most 31 audio members (186 channels) (C-209). The wire cost of a controller stream is computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s (C-219); admission and the tool use this same figure, so offline port and trunk sums include it.

4. **Scale refusal.** At most **64 active modules** per medianet (C-34) and **64 followers per holder** (C-35). Every module other than the holder follows it, so with at most 64 modules a holder has at most 63 real followers (C-114): the module limit enforces both, and the tool refuses configurations above it. The same limit sizes every module's peer map at 63 entries, since a module holds no entry for itself (C-217). The holder is elected at run time (C-113), so the configuration does not name it; at least one holder-capable (Linux) node must exist.

5. **Group E, offline.**
   - Each sink publishes E = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum beside it (C-73). Targets: Linux reference sink 50 µs (C-70); macOS 100 µs (C-71); Windows WASAPI exclusive 1 ms (C-72).
   - A presentation group may carry a **required E**; the runtime refuses a sink whose E exceeds it. Only the commissioned state gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140). A sink with unknown E is refused with reason RequiredEExceeded from any group that sets a required E and admitted to groups that set none (C-103). `validate` applies this to the declared E values before deployment; measuring and certifying E (`certify-sink`, `certify-group`) is epic 028b.

6. **Operator policy, offline.** Strict mode refuses to start with domain id 0 (C-11) and is hosted-only (C-147). Pacing override: the intra-slice egress spread is at most 25 % of the slice period (C-14), and phase offset plus spread stay inside the slice period, so the phase offset is in [0, 75 %) of the period (C-155). Alignment opt-out is per input, that is per connection (C-165).

7. **Domain.** Domain id is 16 bits, operator-configured; 0 means unconfigured (C-11) and strict mode refuses to start with it. The tool assigns and checks it.

8. **Tests without privilege.** A test that needs a privilege (for example a network sandbox proving `validate` opens no socket) runs in the separate privileged nightly job; per-change CI runs unprivileged (C-146).

**Configuration shape.** `validate` must check module count, holder-capable nodes, port sums, trunk directions and declared E offline, so the configuration declares, besides the domain, trunks, groups and connections: nodes (name, OS, endpoint port rate, the switch they attach to), modules (name, node, output channel counts, and for sinks the declared E or none), and for each trunk the two switches it joins (needed for per-direction sums, C-117). A group's `required_e_us` is optional (C-103); its `a_ms` is mandatory (C-55). Operator policy (C-165): strict mode per hosted node (C-147), per-input alignment opt-out per connection, pacing override per module (checked against the pacing fit, C-14, C-155); and an explicit skew budget for a fan-in that feeds no presentation group (C-142). An aligning module, a sink included, may declare its buffer memory (`buffer_kb`), which the tool checks against the A it will hold (C-211, C-222). An output with a controller stream declares its controller bytes per slice and its header bytes (at most the 100-byte cap), taken from the module's manifest, so that the tool computes the stream's wire cost by C-219. The default group (the whole domain) is written as a `[[group]]` named `default` with no `members` list: it holds every sink not listed in another group, and its `a_ms` is required like any other.

## Deliverables

Crate `hadronyche-tools`, binary `hadronyche-commission`, with a stub for every subcommand of the tool (`validate`, `survey`, `propose-a`, `certify-sink`, `certify-group`, `burst-test`, `soak`, `watch`, `window`), a common exit-code convention and CI, running unprivileged. This part implements:

| Subcommand or module | Does |
|---|---|
| `validate <config>` | Offline: domain id, module count ≤ 64 (which bounds followers per holder at 63, C-114), a holder-capable node exists, per-port sums ≤ 70 %, ports above about 198 channel-copies flagged for 10 GbE, trunks per direction against declared capacities, presentation groups, A and required E consistent (including unknown E, C-103), operator policy (strict mode, alignment opt-out, pacing fit, C-155, C-165), fan-in budgets (C-142), declared buffer memory against A for every consumer that buffers to meet A, sinks included (C-211, C-222), and controller streams in port and trunk sums at the C-219 cost, with the controller member counted in the bundle cap (C-209) |
| `hadronyche_tools::capacity` | The capacity arithmetic of item 3, including the controller-stream cost of C-219, shared by `validate` and, in epic 028b, by `watch`, `burst-test` and `soak` |
| `hadronyche_tools::config` | Parser and writer for the configuration file below, fuzzed |
| A proposal computation | Pure function implementing the commissioning rule (C-148) with its per-sink breakdown and truncation check; called by `propose-a` in epic 028b |
| Buffer-memory check | Exposed as a function that `propose-a --write` (epic 028b) calls with the candidate A (C-211) |

The other subcommands (`survey`, `propose-a`, `certify-sink`, `certify-group`, `burst-test`, `soak`, `watch`, `window open` / `window close`) are stubs here and are implemented by epic 028b.

Configuration file format (declarative; shape, names may be refined):

```toml
domain = 17
[[node]]
name = "desk"; os = "linux"; port_mbit = 1000; switch = "sw1"; strict = true   # strict mode, hosted only (C-147, C-165)
[[node]]
name = "stage-left"; os = "macos"; port_mbit = 1000; switch = "sw2"
[[module]]
name = "desk"; node = "desk"; outputs = { out = 24 }   # an output with controllers adds controller_bytes and header_bytes (C-219)
[[module]]
name = "stage-left"; node = "stage-left"; sink = true; e_us = 100   # omit e_us: E unknown (C-103); buffer_kb = ... declares buffer memory (C-211)
[[link]]            # trunks only; endpoint links are discovered
name = "sw1-sw2"; a = "sw1"; b = "sw2"; rate_mbit = 10000   # per direction (C-117)
[[group]]
name = "main"; a_ms = 6.0; required_e_us = 100   # required_e_us optional (C-103)
members = ["stage-left", "stage-right"]
[[group]]
name = "default"; a_ms = 20.0   # no members: every sink not in another group
[[connection]]
from = "desk.out"; to = "stage-left.in"; path = ["sw1-sw2"]   # align = false opts this input out of alignment (C-165)
# a module may carry pacing = { ... } (override, C-155) and, for a fan-in feeding no group, budget_ms (C-142)
```

## Dependencies

- **Needs:** admission units and the manifest (012), for the comparison of offline port sums with the runtime's own admission (runtime AC-12); both sides cost a controller stream by the same formula (C-219).
- **Provides:** the crate, binary skeleton, configuration parser and writer, capacity arithmetic, `validate`, the A proposal computation and the buffer-memory check, to epic 028b.
- **Between the parts:** epic 028b depends on this part (its stamp reader, certificate type and `propose-a`, `burst-test`, `watch` build on 028a-01, 028a-02, 028a-03, 028a-07, 028a-10 and 028a-11). This part depends on nothing in epic 028b.

## Acceptance criteria

Acceptance criterion ids are those of epic 028; the others (AC-4, AC-6 to AC-10, AC-12) belong to epic 028b.

1. **AC-1 Trunk validation.** A graph that puts 800 Mbit/s in one direction on a declared 1 000 Mbit/s trunk is refused offline (ceiling 700 Mbit/s per direction, C-117), naming the link, the direction and the connections; 400 Mbit/s in each direction passes.
2. **AC-2 Port wall.** A 96-channel source fanned out to three consumers on GbE is refused with "port would carry 1 014 Mbit/s; use 10 GbE" (3 × 337.9 Mbit/s, C-20, C-22), and the same configuration is refused by the runtime's own admission (runtime AC-12).
3. **AC-3 Scale.** A 65-module configuration is refused; a configuration with no holder-capable node is refused.
4. **AC-5 Group E.** `validate` refuses, before deployment, a Windows sink declaring 1 ms in a group with a required E of 100 µs, and a sink with unknown E in any group that sets a required E (C-103); a sink with unknown E in a group with no required E passes. (The runtime's own refusal at admission is runtime AC-30, verified by the runtime epics, not here.)
5. **AC-11 A against buffer memory (validate half).** A configuration whose A gives an aligning module rings larger than its declared buffer memory (channels × 96 000 × 4 bytes × A, C-27, C-96) is refused by `validate`, naming the module, the A and both sizes (C-211); a sink is checked like a fan-in (C-222). The `propose-a --write` half is verified in epic 028b, using this part's check.

The per-sink breakdown of the A proposal (lateness, D, headroom, C-148), part of AC-4, is computed here (028a-11); AC-4 itself is verified on the rig by epic 028b.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] `validate` runs with no network and has tests for every refusal
- [ ] Each checklist item states how to verify it and what the stamp shows when it is done
- [ ] The commissioning guide walks through one complete medianet from empty switch to certificate

Fuzzing follows the fuzz budget: each parser 60 s on every change and 1 hour nightly, corpus kept (C-95).

**Not applicable in this part (C-125).** "Regime stamp fields touched are complete and documented": this part reads and defines no stamp field; stamps are read only by epic 028b, and stamp fields are owned by epic 013. "Each checklist item states how to verify it and what the stamp shows when it is done" and "The commissioning guide walks through one complete medianet from empty switch to certificate": the checklists and the guide are delivered by epic 028b, since they depend on `survey`, `burst-test` and `soak`.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Does the runtime's own port-wall refusal (runtime AC-12) quote 1 014 Mbit/s, the figure from C-20 and C-22, for AC-2? | 012 | The runtime refusal quotes a different figure for the same configuration |
