# 028b-11 — `propose-a` command
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** M · **Depends on:** 028a-03, 028a-10, 028a-11, 028b-01, 028b-08, epic 008a, epic 012, epic 015, epic 019

## Context
`hadronyche-commission propose-a <group>` observes the group's sinks in aligning state, collects lateness and D, and proposes the minimum feasible A with its breakdown per sink. A is operator configuration with no default (C-55) and changing it is a domain-wide discontinuity. `--write` writes the proposal to the configuration file; `--apply` applies it to the running group.
- **Read-only (C-107).** Without `--apply` the command drives no traffic, so it needs no commissioning window: it reads stamps and writes the configuration file. The operator starts the sinks.
- **Applying a commissioned change of A (C-224).** A commissioned change of a sink's own A reaches the sink through the tool, inside a declared commissioning window (C-153): the tool validates buffer memory first (C-211) and then announces the new A to the group's members. `--apply` therefore checks the window with the guard (028b-08), runs the check of 028a-10, writes the file as `--write` does, and sends the announcement; each module takes the value through its admission engine (epic 012, ticket 012-30). The tool picks no switch position: each sink switches at its own position one lead (C-43) ahead (C-214) and each fan-in at its own announced position (C-208).
- **Provisional A.** Sinks cannot run without an A, and alignment rings are sized to A (C-96). The operator therefore configures a generous provisional A for the group before measuring. A is the ceiling of every fan-in's offset ratchet (step one slice period, C-53, when persistently late: at least one late slice in each of 10 consecutive 1 s windows, C-54, C-101); when the gap to the ceiling is less than one slice period the last step lands on A exactly (C-163), so "reached the provisional A" is an exact equality; an input persistently late at the ceiling is Demoted (C-109). If any input of the group reaches the provisional A or is Demoted during the measurement, the measurement is truncated and the command asks for a larger provisional A.
- Lateness per consumer from the aligning state: 4 s observation (C-50), max plus headroom (C-51, C-52); the maximum is read from the lateness figures in the stamps (50th, 99th, 99.9th, maximum, C-160), via 028b-01. Those figures cover a rolling 60 s window, with the maximum since the connection went live beside them (C-182); the proposal uses the maximum since the connection went live, never the rolling 60 s figure (C-148). The clock bound in the headroom is the one the servo core publishes (epic 008a, C-187).
- **Changing the provisional A while inputs are live.** This is a commissioned change of A, applied with `--apply` inside a window (C-224). Each sink switches at its own position one lead ahead (C-214); the new A travels upstream in the receiver reports, one hop per report (C-198); every fan-in that inherited the old A re-enters aligning with resized rings and ceiling (C-186), at its own announced position one lead ahead (C-208). Lateness collected before every member and fan-in has re-aligned belongs to the old A, and lateness collected while the window is open is unqualified (C-178), so the measurement starts after the window closes.
- **Buffer memory (C-211, C-222).** The tool checks declared buffer memory before applying a change of A and refuses one that does not fit; the check covers every consumer that buffers to meet A, sinks included (C-222). `--write` and `--apply` both run the check of 028a-10 first. If an oversized A reaches a consumer anyway, its receiver report carries the A it was given, not the clamped value, and its cannot-meet-A alarm stays raised until a later change of A that fits (C-216), which `watch` reports (028b-18).
- D per sink: declared plumbing delay from the stamp: driver buffer, resampler and converter (C-170).
- Late slices are read from the per-connection summary counters in the stamps (C-47); a slice counted missing and later reclassified as late (C-108) counts as late.
- Formula: the commissioning rule, maximum lateness + D + headroom (C-148), implemented in 028a-11.
- The lateness and alignment figures this command reads from stamps are production measurements, unqualified while a commissioning window is open (C-178); the output records whether the measurement was taken inside one and, if so, marks the proposal unqualified.
- The L1 two-bundle rig (C-92): one Linux consumer on one GbE port aligns 2 × 96 channels (32 streams, 64 000 packets/s, 676 Mbit/s) for 1 hour with zero late slices at the commissioned A.

## Work
- Command reads the configuration (028a-03), resolves the group's members, checks that the group has a provisional A, verifies from stamps that the members are running in aligning state, collects at least 4 s of alignment data (configurable, default 60 s for stability, one rolling window, C-182), takes as each sink's maximum lateness the maximum since the connection went live (C-148), shows the largest rolling-window maximum seen over the collection beside it for information, and calls `propose_a`. After a change of the provisional A, restart the collection once the stamps show every member re-aligned and the window closed (C-186, C-208, C-214, C-178).
- Detect a truncated measurement (offset at the provisional A, or an input Demoted) and report it by sink.
- Output: table per sink — lateness, D, headroom, total — and the proposed A, with whether the run was inside a commissioning window, a run inside one being marked unqualified (C-178); `--write` stores `a_ms` in the configuration file (only the file; no live push), after the buffer-memory check of 028a-10 (C-211, C-222).
- `--apply`: check the window with the guard (028b-08), refusing outside one and naming the modules that do not stamp it; run the buffer-memory check; write `a_ms` as `--write` does; announce the new A to the group's members on the control group (C-224), encoded with the types epic 013 provides (as for the window announcement, 028b-09); then report from fresh stamps which members show the new A.
- Validation run on the C-92 rig (epic 019): configure the proposed A, run 1 hour, count late slices from stamp summary counters.

## Acceptance criteria
1. On the C-92 rig, the proposed A, once configured, yields **zero late slices for 1 hour** (epic AC-4, runtime AC-14).
2. The output shows lateness, D and headroom per sink (epic AC-4, C-148).
3. `--write` changes only the group's `a_ms` in the file; nothing is sent to the runtime except stamp fetch requests (epic AC-10).
4. A group with a member not running in aligning state, or with no provisional A, is reported by name and no A is proposed.
5. A fixture where one input's offset reaches the provisional A yields a truncation report and no A.
6. A fixture whose stamps show a commissioning window open during the measurement yields a proposal marked unqualified (C-178).
7. `--write` of an A that gives an aligning module, a sink included, rings larger than its declared buffer memory is refused, naming the module, and the file is unchanged (epic AC-11, C-211, C-222).
8. `--apply` outside a commissioning window refuses and sends nothing; inside one, an A failing the buffer-memory check is refused with nothing sent; otherwise the announcement is sent once, and on a lab rig every member's stamp then shows the new A (epic AC-12, C-224).
9. A fixture in which the stamps show only the rolling-window maximum changing while the since-live maximum is larger yields a proposal computed from the since-live maximum (C-148).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Certification of E (028b-12, 028b-14). Starting or restarting sinks. Picking switch positions (runtime epics, C-208, C-214). Defining the announcement encoding (epic 013, with 004).
