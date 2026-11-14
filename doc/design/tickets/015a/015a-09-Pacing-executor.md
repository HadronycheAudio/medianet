# 015a-09 — Pacing executor (pacing.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-01, 015a-05, epic 010
## Context
- File: `hadronyche-std/src/pacing.rs`: executes a core pacing schedule against the host clock.
- Rule `[decision]`: userspace pacing on the deadline clock. Packets of one slice period are spread over at most 25 % of the period (125 µs at defaults, C-14; default slice 48 frames = 0.5 ms, C-02), plus a per-source phase offset derived from the module id. The schedule is computed by core (task 010); this task executes it. Per-output override 0 disables spreading.
- Pacing fit (C-155): phase offset plus spread stay inside the slice period; the phase offset is in [0, 75 %) of the period, so with the 25 % spread every packet of a slice leaves before the next slice starts. Core computes a schedule that fits; the executor checks it.
- The per-output pacing override is operator policy, set in the commissioning configuration (C-165).
- Load at the upper end: a 96-channel source is 16 streams, 16 packets per slice, 32 000 packets/s (C-22).
- Sends use the non-blocking `send_to` (015a-05); a full queue is a counted drop.
## Work
1. A `Pacer` that takes a core schedule (list of send offsets relative to the slice deadline, including the phase offset) and a `HostClock`, and sends each packet at or after its offset without busy-waiting longer than a documented spin threshold.
2. Support override 0 (all packets of the slice sent back to back at the slice start + phase).
3. Expose per-slice lateness (actual − scheduled send time) for metrics.
4. No allocation per slice.
5. Refuse at construction a schedule whose phase offset is outside [0, 75 %) of the period or whose last send offset falls outside the slice period (C-155), with a named error.
## Acceptance criteria
1. With a 0.5 ms slice and a schedule of 16 packets, all packets of a slice are sent within 125 µs + phase offset of the slice start on the development machine, measured over 10 000 slices (report p50/p99 lateness, tagged `[measured]`).
2. With override 0, all packets are issued consecutively without inter-packet waits.
3. The sequence of send times follows the core schedule order exactly.
4. No heap allocation per slice (counting allocator test).
5. A schedule with a phase offset of 75 % of the period or more is refused (C-155); with a phase offset just below 75 % and override off, every packet is sent before the end of the slice period.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — send path only.
## Out of scope
- Computing the schedule or phase offset (epic 010).
- Integration into the data thread (015a-12).
